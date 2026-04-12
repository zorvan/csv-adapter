/**
 * Streaming cross-chain transfer tools for CSV MCP Server v2
 *
 * Tools:
 * - csv_transfer_cross_chain: Transfer a Right from one chain to another (with optional SSE streaming)
 *
 * When `wait_for_completion` is true and streaming is enabled, progress events
 * are emitted via server.sendNotification() at each phase of the transfer.
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import {
  TransferProgressStream,
  estimateRemaining,
} from "./streaming.js";
import { TransferProgressEvent, StreamStatus } from "./types/streaming.js";

// ---------------------------------------------------------------------------
// In-flight stream registry (keyed by transfer_id)
// ---------------------------------------------------------------------------

const activeStreams = new Map<string, TransferProgressStream>();

export function getActiveStream(transferId: string): TransferProgressStream | undefined {
  return activeStreams.get(transferId);
}

// ---------------------------------------------------------------------------
// Helper: parse a duration string like "10m", "1h", "30s" to milliseconds
// ---------------------------------------------------------------------------

function parseDuration(input: string): number {
  const match = input.match(/^(\d+)([smh])$/);
  if (!match) return 600_000;
  const value = parseInt(match[1], 10);
  const unit = match[2];
  switch (unit) {
    case "s":
      return value * 1_000;
    case "m":
      return value * 60_000;
    case "h":
      return value * 3_600_000;
    default:
      return 600_000;
  }
}

// ---------------------------------------------------------------------------
// Transfer step definitions
// ---------------------------------------------------------------------------

interface TransferStep {
  step: number;
  action: TransferProgressEvent["action"];
  chain?: string;
  durationMs: number;
  confirmations?: number;
  confirmationsRequired?: number;
}

const TRANSFER_STEPS: TransferStep[] = [
  { step: 1, action: "locking", chain: undefined, durationMs: 8000, confirmations: 0, confirmationsRequired: 6 },
  { step: 2, action: "generating_proof", durationMs: 12000 },
  { step: 3, action: "submitting_proof", chain: undefined, durationMs: 10000 },
  { step: 4, action: "verifying", durationMs: 5000 },
  { step: 5, action: "minting", chain: undefined, durationMs: 10000 },
];

function totalDuration(): number {
  return TRANSFER_STEPS.reduce((sum, s) => sum + s.durationMs, 0);
}

// ---------------------------------------------------------------------------
// Tool registration
// ---------------------------------------------------------------------------

const ChainEnum = z.enum(["bitcoin", "ethereum", "sui", "aptos"]);

export function registerStreamingTransferTools(server: McpServer) {
  server.tool(
    "csv_transfer_cross_chain",
    "Transfer a Right from one blockchain to another. This locks the Right on the source chain, generates a cryptographic proof, and mints it on the destination chain. When wait_for_completion is true, progress events are streamed via SSE.",
    {
      right_id: z.string().regex(/^0x[a-fA-F0-9]{64}$/).describe("The Right ID to transfer"),
      from_chain: ChainEnum.describe("Source chain where Right currently exists"),
      to_chain: ChainEnum.describe("Destination chain to transfer to"),
      destination_owner: z.string().describe("New owner address on destination chain"),
      wait_for_completion: z.boolean().optional().default(true).describe("If true, poll until transfer completes or fails. Enables streaming progress events."),
      timeout: z.string().regex(/^\d+[smh]$/).optional().default("10m").describe("Maximum time to wait (e.g., '10m', '1h')"),
    },
    async ({ right_id, from_chain, to_chain, destination_owner, wait_for_completion, timeout }, extra) => {
      const transfer_id = "0x" + "f".repeat(64);
      const timeoutMs = parseDuration(timeout);
      const steps: Array<Record<string, unknown>> = [];
      const startTime = Date.now();

      // ----- Streaming path -----
      if (wait_for_completion) {
        const stream = new TransferProgressStream({
          stream: true,
          timeout_ms: timeoutMs,
        });

        activeStreams.set(transfer_id, stream);

        // Register progress listener that forwards events via MCP notification
        stream.onProgress((event: TransferProgressEvent) => {
          if (extra && typeof extra.sendProgress === "function") {
            extra.sendProgress({
              transfer_id,
              event,
            }).catch((err: unknown) => {
              console.error("[csv_transfer_cross_chain] Failed to send progress notification:", err);
            });
          }
        });

        stream.start();

        try {
          let cumulativeMs = 0;

          for (const stepDef of TRANSFER_STEPS) {
            // Check if stream was cancelled
            if (stream.status === StreamStatus.CANCELLED) {
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    success: false,
                    transfer_id,
                    error_code: "CSV_TRANSFER_CANCELLED",
                    error_message: "Transfer was cancelled by the client",
                    steps_completed: steps.length,
                    steps_total: TRANSFER_STEPS.length,
                  }, null, 2),
                }],
                isError: true,
              };
            }

            const chainLabel = stepDef.chain === "from" ? from_chain : stepDef.chain === "to" ? to_chain : stepDef.chain;

            // Emit in_progress event
            const progressPercent = Math.round((cumulativeMs / totalDuration()) * 100);
            const remaining = estimateRemaining(progressPercent, cumulativeMs + stepDef.durationMs);

            stream.emit({
              step: stepDef.step,
              action: stepDef.action,
              chain: chainLabel,
              status: "in_progress",
              progress_percent: progressPercent,
              estimated_remaining_seconds: remaining,
              message: `Starting ${stepDef.action.replace(/_/g, " ")}${chainLabel ? ` on ${chainLabel}` : ""}`,
              metadata: stepDef.confirmationsRequired
                ? { confirmations: 0, confirmations_required: stepDef.confirmationsRequired }
                : undefined,
            });

            // Simulate the work for this step
            await simulateStep(stepDef, stream, cumulativeMs, from_chain, to_chain);

            cumulativeMs += stepDef.durationMs;

            // Emit completed event
            const completedPercent = Math.round((cumulativeMs / totalDuration()) * 100);
            stream.emit({
              step: stepDef.step,
              action: stepDef.action,
              chain: chainLabel,
              status: "completed",
              progress_percent: completedPercent,
              estimated_remaining_seconds: estimateRemaining(completedPercent, cumulativeMs),
              message: `Completed ${stepDef.action.replace(/_/g, " ")}`,
              metadata: {
                duration_ms: stepDef.durationMs,
                ...(stepDef.action === "locking" ? { lock_transaction: "0x" + "1".repeat(64) } : {}),
                ...(stepDef.action === "submitting_proof" ? { submit_transaction: "0x" + "3".repeat(64) } : {}),
                ...(stepDef.action === "minting" ? { mint_transaction: "0x" + "4".repeat(64) } : {}),
              },
            });

            steps.push({
              step: stepDef.step,
              action: stepDef.action,
              chain: chainLabel,
              status: "completed",
              duration_ms: stepDef.durationMs,
            });
          }

          stream.complete();
          const totalTimeMs = Date.now() - startTime;

          return {
            content: [{
              type: "text",
              text: JSON.stringify({
                success: true,
                transfer_id,
                right_id,
                from_chain,
                to_chain,
                destination_owner,
                status: "completed",
                steps,
                source_transaction: "0x" + "1".repeat(64),
                destination_transaction: "0x" + "4".repeat(64),
                total_time_ms: totalTimeMs,
                completed_at: new Date().toISOString(),
              }, null, 2),
            }],
          };
        } catch (error: unknown) {
          const err = error instanceof Error ? error : new Error(String(error));
          stream.fail(err);

          return {
            content: [{
              type: "text",
              text: JSON.stringify({
                success: false,
                transfer_id,
                error_code: "CSV_TRANSFER_FAILED",
                error_message: err.message,
                retryable: true,
                steps_completed: steps.length,
                steps_total: TRANSFER_STEPS.length,
                suggested_fix: "Check Right ownership and destination address format",
                docs_url: "https://docs.csv.dev/errors/transfer-failed",
              }, null, 2),
            }],
            isError: true,
          };
        } finally {
          activeStreams.delete(transfer_id);
        }
      }

      // ----- Non-streaming (fire-and-forget) path -----
      // Return immediately with a pending status
      return {
        content: [{
          type: "text",
          text: JSON.stringify({
            success: true,
            transfer_id,
            right_id,
            from_chain,
            to_chain,
            destination_owner,
            status: "pending",
            message: "Transfer initiated. Use csv_transfer_status to check progress.",
            polling_endpoint: "csv_transfer_status",
            estimated_completion_seconds: 45,
          }, null, 2),
        }],
      };
    }
  );
}

// ---------------------------------------------------------------------------
// Simulation helpers
// ---------------------------------------------------------------------------

async function simulateStep(
  stepDef: TransferStep,
  stream: TransferProgressStream,
  cumulativeMs: number,
  from_chain: string,
  to_chain: string
): Promise<void> {
  const chainLabel = stepDef.chain === "from" ? from_chain : stepDef.chain === "to" ? to_chain : stepDef.chain;

  if (stepDef.confirmationsRequired && stepDef.confirmations !== undefined) {
    // For locking: emit intermediate confirmation updates
    for (let c = 1; c <= stepDef.confirmationsRequired; c++) {
      if (stream.status !== StreamStatus.RUNNING) return;

      const progressPercent = Math.round(((cumulativeMs + (c / stepDef.confirmationsRequired) * stepDef.durationMs) / totalDuration()) * 100);

      stream.emit({
        step: stepDef.step,
        action: stepDef.action,
        chain: chainLabel,
        status: "in_progress",
        progress_percent: progressPercent,
        estimated_remaining_seconds: estimateRemaining(progressPercent, cumulativeMs + (c / stepDef.confirmationsRequired) * stepDef.durationMs),
        message: `Waiting for confirmations: ${c}/${stepDef.confirmationsRequired}`,
        metadata: { confirmations: c, confirmations_required: stepDef.confirmationsRequired },
      });

      await sleep(stepDef.durationMs / stepDef.confirmationsRequired);
    }
  } else {
    // For other steps: emit a progress bump halfway through
    await sleep(stepDef.durationMs / 2);

    if (stream.status !== StreamStatus.RUNNING) return;

    const midMs = cumulativeMs + stepDef.durationMs / 2;
    const progressPercent = Math.round((midMs / totalDuration()) * 100);

    stream.emit({
      step: stepDef.step,
      action: stepDef.action,
      chain: chainLabel,
      status: "in_progress",
      progress_percent: progressPercent,
      estimated_remaining_seconds: estimateRemaining(progressPercent, midMs),
      message: `${stepDef.action.replace(/_/g, " ")} in progress...`,
      metadata: { progress_percent: progressPercent },
    });

    await sleep(stepDef.durationMs / 2);
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
