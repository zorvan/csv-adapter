/**
 * CSV Adapter MCP Server
 * 
 * Enables AI agents (Claude, Cursor, Copilot) to operate CSV cross-chain operations.
 * 
 * Usage:
 *   npx @csv-adapter/mcp-server
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
import { z } from "zod";
import { registerRightTools } from "./tools/right.js";
import { registerTransferTools } from "./tools/transfer.js";
import { registerProofTools } from "./tools/proof.js";
import { registerWalletTools } from "./tools/wallet.js";

// Create MCP server instance
const server = new McpServer({
  name: "csv-adapter",
  version: "0.1.0",
  description: "Client-side validation system for cross-chain rights",
});

// Register all tool categories
registerRightTools(server);
registerTransferTools(server);
registerProofTools(server);
registerWalletTools(server);

// Start server using stdio transport (for Claude Desktop, Cursor, etc.)
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("CSV MCP Server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
