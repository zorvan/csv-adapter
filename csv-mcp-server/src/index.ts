#!/usr/bin/env node
/**
 * CSV MCP Server — AI Agent Integration
 *
 * Enables AI agents (Claude, GPT, etc.) to operate CSV sanads workflows
 * through the Model Context Protocol (MCP).
 *
 * High-value actions for MCP:
 * - create_seal(chain, value) — agent creates a seal
 * - transfer_sanad(sanad_id, destination) — agent transfers a sanad
 * - verify_proof(bundle_json) — agent verifies a proof bundle
 * - get_sanads(address) — agent lists sanads for an address
 * - monitor_transfer(transfer_id) — agent watches transfer status
 *
 * Usage:
 *   csv-mcp                    # Start MCP server (stdio transport)
 *   csv-mcp --stdio            # Explicit stdio transport
 *   csv-mcp --sse --port 3000  # SSE transport on port 3000
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';
import { z } from 'zod';
import { spawn, ChildProcess } from 'child_process';
import { promisify } from 'util';
import * as path from 'path';
import * as fs from 'fs';

// =========================================================================
// Input validation — SEC-07
// =========================================================================

const VALID_CHAINS = ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'] as const;
type ValidChain = (typeof VALID_CHAINS)[number];

/**
 * Return a validation error result object.
 */
function err(msg: string): { content: Array<{ type: 'text'; text: string }>; isError: true } {
  return { content: [{ type: 'text', text: `Validation error: ${msg}` }], isError: true };
}

/**
 * Validate a sanad ID: must be a 64-character hex string (32 bytes).
 * Returns null if valid, error message if invalid.
 */
function validateSanadId(id: unknown): string | null {
  if (typeof id !== 'string') return 'sanad_id must be a string';
  if (id.length === 0) return 'sanad_id must not be empty';
  if (id.startsWith('-') || id.startsWith('--')) return 'sanad_id must not start with a dash';
  if (!/^[0-9a-fA-F]{64}$/.test(id)) return 'sanad_id must be a 64-character hex string (32 bytes)';
  return null;
}

/**
 * Validate a transfer ID: same format as sanad ID.
 */
function validateTransferId(id: unknown): string | null {
  return validateSanadId(id);
}

/**
 * Validate a chain name against the allowed enum.
 * Returns the lowercased chain if valid, error message if invalid.
 */
function validateChain(chain: unknown): ValidChain | string {
  if (typeof chain !== 'string') return 'chain must be a string';
  if (chain.startsWith('-') || chain.startsWith('--')) return 'chain must not start with a dash';
  const lower = chain.toLowerCase();
  if ((VALID_CHAINS as readonly string[]).includes(lower)) return lower as ValidChain;
  return `chain must be one of: ${VALID_CHAINS.join(', ')}`;
}

/**
 * Validate a blockchain address: hex string with optional 0x prefix, 20-66 chars.
 */
function validateAddress(address: unknown): string | null {
  if (typeof address !== 'string') return 'address must be a string';
  if (address.length === 0) return 'address must not be empty';
  if (address.startsWith('-') || address.startsWith('--')) return 'address must not start with a dash';
  const checkStr = address.startsWith('0x') || address.startsWith('0X') ? address.slice(2) : address;
  if (checkStr.length < 20 || checkStr.length > 66) return 'address length must be 20-66 hex characters (with or without 0x prefix)';
  if (!/^[0-9a-fA-F]+$/.test(checkStr)) return 'address must contain only hex characters';
  return null;
}

/**
 * Validate a destination string (address or owner identifier).
 */
function validateDestination(destination: unknown): string | null {
  if (typeof destination !== 'string') return 'destination must be a string';
  if (destination.length === 0) return 'destination must not be empty';
  if (destination.startsWith('-') || destination.startsWith('--')) return 'destination must not start with a dash';
  if (/[;|&$`\\!#(){}[\]~\s]/.test(destination)) return 'destination contains characters that are not allowed';
  const checkStr = destination.startsWith('0x') || destination.startsWith('0X') ? destination.slice(2) : destination;
  if (/^[0-9a-fA-F]+$/.test(checkStr)) {
    if (checkStr.length < 20 || checkStr.length > 66) return 'destination address length must be 20-66 hex characters';
  }
  return null;
}

/**
 * Validate a value parameter: must be a positive number.
 * Returns the parsed number if valid, error message if invalid.
 */
function validatePositiveNumber(value: unknown): number | string {
  if (typeof value !== 'number' && typeof value !== 'string') return 'value must be a number';
  const num = typeof value === 'string' ? parseFloat(value) : value;
  if (isNaN(num) || num <= 0 || !isFinite(num)) return 'value must be a positive number';
  return num;
}

/**
 * Validate that a string is valid JSON.
 * Returns parsed value if valid, error message if invalid.
 */
function validateJsonString(json: unknown): { parsed: any } | string {
  if (typeof json !== 'string') return 'bundle must be a JSON string';
  if (json.length === 0) return 'bundle string must not be empty';
  if (json.startsWith('-') || json.startsWith('--')) return 'bundle must not start with a dash';
  try {
    const parsed = JSON.parse(json);
    return { parsed };
  } catch {
    return 'bundle must be valid JSON';
  }
}

/**
 * Validate a consignment JSON string: must parse as a JSON object.
 */
function validateConsignment(json: unknown): { parsed: any } | string {
  const result = validateJsonString(json);
  if (typeof result === 'string') return result;
  if (typeof result.parsed !== 'object' || result.parsed === null || Array.isArray(result.parsed)) {
    return 'consignment must be a JSON object';
  }
  return result;
}

/**
 * Validate a hex ID (generic): alphanumeric + optional 0x prefix.
 */
function validateHexId(id: unknown, label: string, expectedLength?: number): string | null {
  if (typeof id !== 'string') return `${label} must be a string`;
  if (id.length === 0) return `${label} must not be empty`;
  if (id.startsWith('-') || id.startsWith('--')) return `${label} must not start with a dash`;
  const checkStr = id.startsWith('0x') || id.startsWith('0X') ? id.slice(2) : id;
  if (!/^[0-9a-fA-F]+$/.test(checkStr)) return `${label} must be a hex string`;
  if (expectedLength && checkStr.length !== expectedLength) return `${label} must be exactly ${expectedLength} hex characters`;
  return null;
}

// =========================================================================
// CSV CLI wrapper
// =========================================================================

/**
 * Execute a csv-cli command and return the result.
 * Uses the actual csv-cli binary for real operations.
 */
async function executeCsvCommand(args: string[]): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  return new Promise((resolve) => {
    console.error(`[csv-mcp] Executing: csv ${args.join(' ')}`);
    
    // Try to find csv-cli binary in common locations
    const possiblePaths = [
      path.join(__dirname, '../../csv-cli/target/release/csv'),
      path.join(__dirname, '../../csv-cli/target/debug/csv'),
      'csv', // Assume it's in PATH
      './csv-cli/target/release/csv',
      './csv-cli/target/debug/csv',
    ];
    
    let csvPath: string | null = null;
    for (const possiblePath of possiblePaths) {
      if (fs.existsSync(possiblePath)) {
        csvPath = possiblePath;
        console.error(`[csv-mcp] Found CSV CLI at: ${csvPath}`);
        break;
      }
    }
    
    if (!csvPath) {
      // Fallback to 'csv' command and let the shell handle it
      csvPath = 'csv';
      console.error(`[csv-mcp] Using CSV CLI from PATH: ${csvPath}`);
    }
    
    const child: ChildProcess = spawn(csvPath, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        RUST_LOG: 'info', // Keep logging reasonable
      }
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout?.on('data', (data: Buffer) => {
      stdout += data.toString();
    });
    
    child.stderr?.on('data', (data: Buffer) => {
      stderr += data.toString();
    });
    
    child.on('close', (code: number | null) => {
      const result = {
        stdout: stdout.trim(),
        stderr: stderr.trim(),
        exitCode: code || 0,
      };
      
      // Log the result for debugging
      if (result.exitCode === 0) {
        console.error(`[csv-mcp] Command succeeded: ${result.stdout.substring(0, 200)}${result.stdout.length > 200 ? '...' : ''}`);
      } else {
        console.error(`[csv-mcp] Command failed (exit code ${result.exitCode}): ${result.stderr}`);
      }
      
      resolve(result);
    });
    
    child.on('error', (error: Error) => {
      console.error(`[csv-mcp] CLI execution error: ${error.message}`);
      resolve({
        stdout: '',
        stderr: `CLI execution failed: ${error.message}. Make sure csv-cli is installed and accessible. Tried paths: ${possiblePaths.join(', ')}`,
        exitCode: 1,
      });
    });
    
    // Timeout after 60 seconds (increased for long-running operations)
    const timeout = setTimeout(() => {
      child.kill('SIGTERM');
      resolve({
        stdout: '',
        stderr: 'Command timed out after 60 seconds. The operation may have failed or is taking too long.',
        exitCode: 124,
      });
    });
    
    child.on('close', () => {
      clearTimeout(timeout);
    });
  });
}

/**
 * Parse CLI output and handle JSON responses gracefully.
 */
function parseCliOutput(output: string): any {
  if (!output.trim()) {
    return null;
  }
  
  try {
    return JSON.parse(output);
  } catch {
    // If it's not JSON, return as-is
    return output;
  }
}

// =========================================================================
// Tool definitions
// =========================================================================

/**
 * Get a list of all MCP tools provided by this server.
 */
function getTools() {
  return [
    {
      name: 'create_seal',
      description:
        'Create a single-use seal on a blockchain. ' +
        'A seal is a chain-native lock that enforces the single-use property of a digital sanad. ' +
        'Each chain has its own seal format (Bitcoin: UTXO, Ethereum: storage slot, Sui: ObjectId, etc.).',
      inputSchema: {
        type: 'object',
        properties: {
          chain: {
            type: 'string',
            enum: ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'],
            description: 'The blockchain to create the seal on',
          },
          value: {
            type: 'number',
            description: 'Optional value to lock (chain-specific units: satoshis, wei, etc.)',
          },
        },
        required: ['chain'],
      },
    },
    {
      name: 'transfer_sanad',
      description:
        'Transfer a digital sanad to a new owner. ' +
        'This consumes the current seal and creates a new one for the destination. ' +
        'The transfer is recorded in the commitment chain for provenance.',
      inputSchema: {
        type: 'object',
        properties: {
          sanad_id: {
            type: 'string',
            description: 'The sanad ID to transfer (32-byte hex string)',
          },
          destination: {
            type: 'string',
            description: 'The destination address or owner identifier',
          },
          chain: {
            type: 'string',
            enum: ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'],
            description: 'The chain where the sanad exists',
          },
        },
        required: ['sanad_id', 'destination'],
      },
    },
    {
      name: 'verify_proof',
      description:
        'Verify a proof bundle offline. ' +
        'A proof bundle contains all cryptographic evidence needed to verify a sanad. ' +
        'This verification requires NO blockchain RPC calls — pure cryptography. ' +
        'This is the CSV competitive advantage over traditional bridges.',
      inputSchema: {
        type: 'object',
        properties: {
          bundle_json: {
            type: 'string',
            description: 'JSON string of a ProofBundle to verify',
          },
        },
        required: ['bundle_json'],
      },
    },
    {
      name: 'get_sanads',
      description:
        'List all sanads owned by an address on a specific chain. ' +
        'Returns sanad IDs, values, and current status.',
      inputSchema: {
        type: 'object',
        properties: {
          address: {
            type: 'string',
            description: 'The blockchain address to query',
          },
          chain: {
            type: 'string',
            enum: ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'],
            description: 'Optional chain filter',
          },
        },
        required: ['address'],
      },
    },
    {
      name: 'monitor_transfer',
      description:
        'Monitor the status of a cross-chain transfer. ' +
        'Returns the current state in the transfer lifecycle: ' +
        'Locked → AwaitingFinality → BuildingProof → ProofReady → Minting → Complete',
      inputSchema: {
        type: 'object',
        properties: {
          transfer_id: {
            type: 'string',
            description: 'The transfer ID to monitor',
          },
        },
        required: ['transfer_id'],
      },
    },
    {
      name: 'get_protocol_info',
      description:
        'Get CSV protocol information including version, capabilities, and supported chains.',
      inputSchema: {
        type: 'object',
        properties: {},
        required: [],
      },
    },
    {
      name: 'export_proof_bundle',
      description:
        'Export a proof bundle as a portable JSON file. ' +
        'The exported bundle can be shared with any counterparty for offline verification.',
      inputSchema: {
        type: 'object',
        properties: {
          sanad_id: {
            type: 'string',
            description: 'The sanad ID to generate a proof bundle for',
          },
          chain: {
            type: 'string',
            enum: ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'],
            description: 'Optional chain filter for the proof bundle',
          },
        },
        required: ['sanad_id'],
      },
    },
    {
      name: 'accept_consignment',
      description:
        'Accept a consignment (complete transfer artifact) into local state. ' +
        'The consignment is verified before acceptance: ' +
        '1. Structural validation 2. Commitment chain 3. Double-spend check 4. State transitions',
      inputSchema: {
        type: 'object',
        properties: {
          consignment_json: {
            type: 'string',
            description: 'JSON string of a Consignment to accept',
          },
        },
        required: ['consignment_json'],
      },
    },
    {
      name: 'health_check',
      description: 'Check if the CSV CLI is properly installed and accessible',
      inputSchema: {
        type: 'object',
        properties: {},
        required: [],
      },
    },
  ];
}

// =========================================================================
// Server setup
// =========================================================================

async function startServer(transportType: 'stdio' | 'sse' = 'stdio', port?: number) {
  const server = new McpServer({
    name: 'csv-mcp-server',
    version: '0.4.0',
  });

  // Register all tools
  const tools = getTools();

  // create_seal tool
  server.registerTool('create_seal', {
    description: tools.find((t) => t.name === 'create_seal')!.description,
    inputSchema: tools.find((t) => t.name === 'create_seal')!.inputSchema as any,
  }, async (args: any) => {
    const chainCheck = validateChain(args.chain);
    if (typeof chainCheck === 'string') return err(chainCheck);
    if (args.value !== undefined) {
      const valueCheck = validatePositiveNumber(args.value);
      if (typeof valueCheck === 'string') return err(valueCheck);
    }
    const result = await executeCsvCommand(['seal', 'create', '--chain', chainCheck, ...(args.value ? ['--value', String(args.value)] : [])]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // transfer_sanad tool
  server.registerTool('transfer_sanad', {
    description: tools.find((t) => t.name === 'transfer_sanad')!.description,
    inputSchema: tools.find((t) => t.name === 'transfer_sanad')!.inputSchema as any,
  }, async (args: any) => {
    const idErr = validateSanadId(args.sanad_id);
    if (idErr) return err(idErr);
    const destErr = validateDestination(args.destination);
    if (destErr) return err(destErr);
    if (args.chain) {
      const chainCheck = validateChain(args.chain);
      if (typeof chainCheck === 'string') return err(chainCheck);
      const result = await executeCsvCommand([
        'sanad', 'transfer',
        args.sanad_id,
        '--to', args.destination,
        '--chain', chainCheck,
      ]);
      return {
        content: [{ type: 'text' as const, text: result.stdout }],
        isError: result.exitCode !== 0,
      };
    }
    const result = await executeCsvCommand([
      'sanad', 'transfer',
      args.sanad_id,
      '--to', args.destination,
    ]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // verify_proof tool
  server.registerTool('verify_proof', {
    description: tools.find((t) => t.name === 'verify_proof')!.description,
    inputSchema: tools.find((t) => t.name === 'verify_proof')!.inputSchema as any,
  }, async (args: any) => {
    const bundleCheck = validateJsonString(args.bundle_json);
    if (typeof bundleCheck === 'string') return err(bundleCheck);
    const result = await executeCsvCommand(['proof', 'verify', '--proof', args.bundle_json]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // get_sanads tool
  server.registerTool('get_sanads', {
    description: tools.find((t) => t.name === 'get_sanads')!.description,
    inputSchema: tools.find((t) => t.name === 'get_sanads')!.inputSchema as any,
  }, async (args: any) => {
    const addrErr = validateAddress(args.address);
    if (addrErr) return err(addrErr);
    if (args.chain) {
      const chainCheck = validateChain(args.chain);
      if (typeof chainCheck === 'string') return err(chainCheck);
      const result = await executeCsvCommand([
        'sanad', 'list',
        args.address,
        '--chain', chainCheck,
      ]);
      return {
        content: [{ type: 'text' as const, text: result.stdout }],
        isError: result.exitCode !== 0,
      };
    }
    const result = await executeCsvCommand([
      'sanad', 'list',
      args.address,
    ]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // monitor_transfer tool
  server.registerTool('monitor_transfer', {
    description: tools.find((t) => t.name === 'monitor_transfer')!.description,
    inputSchema: tools.find((t) => t.name === 'monitor_transfer')!.inputSchema as any,
  }, async (args: any) => {
    const idErr = validateTransferId(args.transfer_id);
    if (idErr) return err(idErr);
    const result = await executeCsvCommand(['cross-chain', 'status', args.transfer_id]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // get_protocol_info tool
  server.registerTool('get_protocol_info', {
    description: tools.find((t) => t.name === 'get_protocol_info')!.description,
    inputSchema: tools.find((t) => t.name === 'get_protocol_info')!.inputSchema as any,
  }, async (_args: any) => {
    const info = {
      protocol: 'CSV (Client-Side Validation)',
      version: '0.4.0',
      supportedChains: ['bitcoin', 'ethereum', 'sui', 'aptos', 'solana'],
      features: {
        singleUseSeals: true,
        offlineVerification: true,
        crossChainTransfers: true,
        commitmentChain: true,
        mpcBatching: true,
        zkProofs: true,
      },
      competitiveAdvantages: [
        'No custody — sanads are off-chain state, seals are chain-enforced',
        'No trusted bridge — proof bundles are self-verifying',
        'Offline verification — anyone with the bundle can verify',
        'Cryptographic double-spend prevention',
        'Cross-chain provenance — tamper-evident audit log',
      ],
    };
    const text = JSON.stringify(info, null, 2);
    return {
      content: [{ type: 'text' as const, text }],
      isError: false,
    };
  });

  // export_proof_bundle tool
  server.registerTool('export_proof_bundle', {
    description: tools.find((t) => t.name === 'export_proof_bundle')!.description,
    inputSchema: tools.find((t) => t.name === 'export_proof_bundle')!.inputSchema as any,
  }, async (args: any) => {
    const idErr = validateSanadId(args.sanad_id);
    if (idErr) return err(idErr);
    if (args.chain) {
      const chainCheck = validateChain(args.chain);
      if (typeof chainCheck === 'string') return err(chainCheck);
      const result = await executeCsvCommand([
        'proof', 'generate',
        '--sanad-id', args.sanad_id,
        '--chain', chainCheck,
      ]);
      return {
        content: [{ type: 'text' as const, text: result.stdout }],
        isError: result.exitCode !== 0,
      };
    }
    const result = await executeCsvCommand([
      'proof', 'generate',
      '--sanad-id', args.sanad_id,
    ]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // accept_consignment tool
  server.registerTool('accept_consignment', {
    description: tools.find((t) => t.name === 'accept_consignment')!.description,
    inputSchema: tools.find((t) => t.name === 'accept_consignment')!.inputSchema as any,
  }, async (args: any) => {
    const consignmentCheck = validateConsignment(args.consignment_json);
    if (typeof consignmentCheck === 'string') return err(consignmentCheck);
    const result = await executeCsvCommand(['validate', 'accept', '--json', args.consignment_json]);
    return {
      content: [{ type: 'text' as const, text: result.stdout }],
      isError: result.exitCode !== 0,
    };
  });

  // health_check tool
  server.registerTool('health_check', {
    description: tools.find((t) => t.name === 'health_check')!.description,
    inputSchema: tools.find((t) => t.name === 'health_check')!.inputSchema as any,
  }, async (_args: any) => {
    try {
      // Try to run a simple CLI command to check if it's working
      const result = await executeCsvCommand(['--version']);
      
      if (result.exitCode === 0) {
        const healthInfo = {
          status: 'healthy',
          csv_cli_version: result.stdout,
          mcp_server_version: '0.4.0',
          timestamp: new Date().toISOString(),
        };
        return {
          content: [{ type: 'text' as const, text: JSON.stringify(healthInfo, null, 2) }],
          isError: false,
        };
      } else {
        const healthInfo = {
          status: 'unhealthy',
          error: result.stderr || 'Unknown error',
          mcp_server_version: '0.4.0',
          timestamp: new Date().toISOString(),
        };
        const text = JSON.stringify(healthInfo, null, 2);
        return {
          content: [{ type: 'text' as const, text }],
          isError: true,
        };
      }
    } catch (error) {
      const healthInfo = {
        status: 'unhealthy',
        error: error instanceof Error ? error.message : 'Unknown error',
        mcp_server_version: '0.4.0',
        timestamp: new Date().toISOString(),
      };
      const text = JSON.stringify(healthInfo, null, 2);
      return {
        content: [{ type: 'text' as const, text }],
        isError: true,
      };
    }
  });

  // Start the server
  if (transportType === 'stdio') {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error('CSV MCP Server running on stdio');
  } else if (transportType === 'sse' && port) {
    console.error('SSE transport requires an HTTP server (e.g., Express). Use stdio transport for now.');
    console.error('To enable SSE, integrate with an HTTP server and use SSEServerTransport with ServerResponse.');
    process.exit(1);
  }
}

// =========================================================================
// CLI entry point
// =========================================================================

const args = process.argv.slice(2);
const transportType: 'stdio' | 'sse' = args.includes('--sse') ? 'sse' : 'stdio';
const portMatch = args.find((a: string) => a.startsWith('--port='));
const port = portMatch ? parseInt(portMatch.split('=')[1], 10) : undefined;

startServer(transportType, port).catch((err) => {
  console.error('Failed to start MCP server:', err);
  process.exit(1);
});
