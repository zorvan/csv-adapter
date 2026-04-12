/**
 * CSV Adapter MCP Server v2
 *
 * Enables AI agents (Claude, Cursor, Copilot, web-based agents) to operate
 * CSV cross-chain operations with optional SSE streaming for long-running tasks.
 *
 * Usage (stdio — Claude Desktop, Cursor, etc.):
 *   npx @csv-adapter/mcp-server
 *
 * Usage (SSE — web-based agents):
 *   npx @csv-adapter/mcp-server --transport sse --port 3000
 *
 * Claude Desktop config (~/.config/claude-desktop/config.json):
 *   {
 *     "mcpServers": {
 *       "csv": {
 *         "command": "csv-mcp-server",
 *         "args": ["--chains", "bitcoin,ethereum,sui,aptos"]
 *       }
 *     }
 *   }
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { SSEServerTransport } from "@modelcontextprotocol/sdk/server/sse.js";
import express from "express";
import { z } from "zod";
import { registerRightTools } from "./tools/right.js";
import { registerTransferTools } from "./tools/transfer.js";
import { registerProofTools } from "./tools/proof.js";
import { registerWalletTools } from "./tools/wallet.js";
import { registerStreamingTransferTools } from "./transfers.js";

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

interface ServerConfig {
  transport: "stdio" | "sse";
  port: number;
  chains: string[];
}

function parseArgs(args: string[]): ServerConfig {
  const config: ServerConfig = {
    transport: "stdio",
    port: 3000,
    chains: [],
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case "--transport":
        config.transport = (args[++i] as "stdio" | "sse") || "stdio";
        break;
      case "--port":
        config.port = parseInt(args[++i] || "3000", 10);
        break;
      case "--chains": {
        const raw = args[++i] || "";
        config.chains = raw.split(",").map((c) => c.trim()).filter(Boolean);
        break;
      }
    }
  }

  return config;
}

// ---------------------------------------------------------------------------
// MCP server setup
// ---------------------------------------------------------------------------

const server = new McpServer({
  name: "csv-adapter",
  version: "2.0.0",
  description: "Client-side validation system for cross-chain rights (v2 with SSE streaming)",
});

// Register all tool categories (v1 tools remain for backward compatibility)
registerRightTools(server);
registerTransferTools(server);       // v1 (non-streaming)
registerStreamingTransferTools(server); // v2 (streaming)
registerProofTools(server);
registerWalletTools(server);

// ---------------------------------------------------------------------------
// Transport: stdio (default — backward compatible with v1)
// ---------------------------------------------------------------------------

async function startStdio(): Promise<void> {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("CSV MCP Server v2 running on stdio (v1 compatible)");
}

// ---------------------------------------------------------------------------
// Transport: SSE (web-based agents with streaming progress)
// ---------------------------------------------------------------------------

async function startSSE(port: number): Promise<void> {
  const app = express();

  // Each SSE client gets its own transport instance
  // so that sendNotification() can route events to the correct connection.
  app.get("/sse", async (_req, res) => {
    const transport = new SSEServerTransport("/messages", res);
    await server.connect(transport);
    console.error(`[SSE] Client connected (transport endpoint: /messages)`);
  });

  // Message endpoint — the SDK handles POST requests here
  app.post("/messages", async (req, res) => {
    // The SSEServerTransport handles message routing internally.
    // We just need to ensure the route is registered so Express
    // does not 404.
    res.status(405).send("Use the /sse endpoint to establish an SSE connection");
  });

  // Health check
  app.get("/health", (_req, res) => {
    res.json({ status: "ok", version: "2.0.0", transport: "sse" });
  });

  app.listen(port, () => {
    console.error(`CSV MCP Server v2 running on SSE at http://localhost:${port}`);
    console.error(`  SSE endpoint:  http://localhost:${port}/sse`);
    console.error(`  Health check:  http://localhost:${port}/health`);
  });
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

async function main() {
  const config = parseArgs(process.argv.slice(2));

  switch (config.transport) {
    case "sse":
      await startSSE(config.port);
      break;
    case "stdio":
    default:
      await startStdio();
      break;
  }
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
