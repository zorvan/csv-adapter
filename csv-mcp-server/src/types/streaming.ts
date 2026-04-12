/**
 * Streaming type definitions for CSV MCP Server v2
 *
 * Defines the interfaces and enums used for Server-Sent Events (SSE)
 * streaming of long-running operations such as cross-chain transfers.
 */

// ---------------------------------------------------------------------------
// Stream lifecycle
// ---------------------------------------------------------------------------

export enum StreamStatus {
  /** Stream has been created but not yet started */
  IDLE = "idle",
  /** Stream is actively emitting progress events */
  RUNNING = "running",
  /** Stream has been temporarily paused by the client */
  PAUSED = "paused",
  /** Stream finished successfully */
  COMPLETED = "completed",
  /** Stream terminated due to an error */
  ERROR = "error",
  /** Stream was cancelled by the client */
  CANCELLED = "cancelled",
}

// ---------------------------------------------------------------------------
// Progress event payload
// ---------------------------------------------------------------------------

export interface TransferProgressEvent {
  /** Monotonically increasing step number (1-based) */
  step: number;

  /** Human-readable action identifier */
  action:
    | "locking"
    | "generating_proof"
    | "submitting_proof"
    | "verifying"
    | "minting";

  /** Chain involved in this step (if applicable) */
  chain?: string;

  /** Current status of this step */
  status: "in_progress" | "completed" | "failed" | "skipped";

  /** Overall progress percentage (0-100) */
  progress_percent: number;

  /** Estimated seconds remaining (null when unknown) */
  estimated_remaining_seconds: number | null;

  /** Optional human-readable detail message */
  message?: string;

  /** ISO-8601 timestamp when the event was emitted */
  timestamp: string;

  /** Optional chain-specific metadata (e.g. confirmations, tx hash) */
  metadata?: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Streaming options
// ---------------------------------------------------------------------------

export interface StreamingOptions {
  /** Whether to enable streaming progress events (default: false) */
  stream?: boolean;

  /** Minimum interval between progress events in milliseconds (default: 500) */
  throttle_ms?: number;

  /** Total timeout for the entire operation in milliseconds (default: 600000 / 10 min) */
  timeout_ms?: number;
}

// ---------------------------------------------------------------------------
// Internal: stream state tracking
// ---------------------------------------------------------------------------

export interface StreamState {
  status: StreamStatus;
  startTime: number | null;
  elapsedTimeMs: number;
  lastEvent: TransferProgressEvent | null;
  error: Error | null;
}

// ---------------------------------------------------------------------------
// SSE-specific helpers
// ---------------------------------------------------------------------------

/** Shape of an SSE progress notification sent via MCP server.sendNotification() */
export interface ProgressNotification {
  method: "notifications/progress";
  params: {
    transfer_id: string;
    event: TransferProgressEvent;
  };
}
