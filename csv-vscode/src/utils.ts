import * as vscode from "vscode";
import * as path from "path";

/**
 * Chain color mapping for visual identification.
 * Returns a ThemeColor compatible with both light and dark themes.
 */
const CHAIN_COLORS: Record<string, vscode.ThemeColor> = {
  ethereum: new vscode.ThemeColor("charts.blue"),
  bitcoin: new vscode.ThemeColor("charts.orange"),
  aptos: new vscode.ThemeColor("charts.purple"),
  sui: new vscode.ThemeColor("charts.red"),
  solana: new vscode.ThemeColor("charts.green"),
};

const DEFAULT_CHAIN_COLOR = new vscode.ThemeColor("foreground");

/**
 * Regular expression for validating Right IDs.
 * Format: 0x followed by 64 hexadecimal characters.
 */
const RIGHT_ID_REGEX = /^0x[a-fA-F0-9]{64}$/;

/**
 * Reads configuration values from the workspace.
 * Falls back to user settings if workspace settings are not available.
 */
export function getWorkspaceConfig(): vscode.WorkspaceConfiguration {
  return vscode.workspace.getConfiguration("csv");
}

/**
 * Truncates a Right ID for display purposes.
 * Shows first 10 and last 4 characters with ellipsis.
 *
 * @param rightId - Full Right ID string
 * @param maxLength - Maximum display length (default: 18)
 * @returns Truncated Right ID for display
 */
export function formatRightId(rightId: string, maxLength = 18): string {
  if (!rightId || rightId.length <= maxLength) {
    return rightId;
  }

  const prefixLength = Math.max(6, Math.floor(maxLength / 2) - 1);
  const suffixLength = Math.max(4, maxLength - prefixLength - 1);

  return `${rightId.substring(0, prefixLength)}...${rightId.substring(
    rightId.length - suffixLength
  )}`;
}

/**
 * Validates whether a string is a properly formatted Right ID.
 *
 * @param rightId - String to validate
 * @returns true if the string matches the Right ID format
 */
export function isValidRightId(rightId: string): boolean {
  if (!rightId || typeof rightId !== "string") {
    return false;
  }
  return RIGHT_ID_REGEX.test(rightId.trim());
}

/**
 * Returns the VS Code ThemeColor for a given chain.
 * Supports both light and dark themes automatically.
 *
 * @param chain - Chain name (case-insensitive)
 * @returns ThemeColor for the chain
 */
export function getChainColor(chain: string): vscode.ThemeColor {
  const normalizedChain = chain.toLowerCase();
  return CHAIN_COLORS[normalizedChain] || DEFAULT_CHAIN_COLOR;
}

/**
 * Opens a URL in the user's default browser.
 * Uses VS Code's built-in environment API.
 *
 * @param url - URL to open
 * @returns Promise that resolves when the URL is opened
 */
export async function openExternal(url: string): Promise<boolean> {
  try {
    const uri = vscode.Uri.parse(url);
    return await vscode.env.openExternal(uri);
  } catch (error) {
    vscode.window.showErrorMessage(
      `Failed to open URL: ${url}. Error: ${error}`
    );
    return false;
  }
}

/**
 * Formats a chain address for display with truncation.
 *
 * @param address - Full address string
 * @param chars - Number of characters to show on each side (default: 6)
 * @returns Formatted address string
 */
export function formatAddress(address: string, chars = 6): string {
  if (!address || address.length <= chars * 2 + 2) {
    return address;
  }

  return `${address.substring(0, chars + 2)}...${address.substring(
    address.length - chars
  )}`;
}

/**
 * Formats a timestamp into a human-readable relative time string.
 *
 * @param timestamp - Date or timestamp string
 * @returns Relative time string (e.g., "2 hours ago")
 */
export function formatRelativeTime(timestamp: Date | string): string {
  const date = typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) {
    return "just now";
  }
  if (minutes < 60) {
    return `${minutes} minute${minutes === 1 ? "" : "s"} ago`;
  }
  if (hours < 24) {
    return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  }
  if (days < 30) {
    return `${days} day${days === 1 ? "" : "s"} ago`;
  }

  return date.toLocaleDateString();
}

/**
 * Normalizes a chain name to lowercase for consistent comparisons.
 *
 * @param chain - Chain name to normalize
 * @returns Lowercase chain name
 */
export function normalizeChain(chain: string): string {
  return chain.toLowerCase().trim();
}

/**
 * Gets the file extension for a given chain's code output.
 *
 * @param chain - Chain name
 * @param language - Programming language preference (rust/typescript)
 * @returns Appropriate file extension
 */
export function getFileExtension(
  chain: string,
  language: "rust" | "typescript" = "typescript"
): string {
  if (language === "rust") {
    return ".rs";
  }
  return ".ts";
}

/**
 * Generates a nonce for Content Security Policy.
 *
 * @param length - Length of the nonce (default: 32)
 * @returns Random nonce string
 */
export function generateNonce(length = 32): string {
  let text = "";
  const possible =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < length; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}

/**
 * Resolves a path relative to the workspace root.
 * Falls back to the first workspace folder or undefined.
 *
 * @param relativePath - Path relative to workspace root
 * @returns Absolute path or undefined
 */
export function resolveWorkspacePath(relativePath: string): string | undefined {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  return path.join(workspaceFolder.uri.fsPath, relativePath);
}

/**
 * Creates a status bar item for CSV Adapter.
 *
 * @param text - Status bar text
 * @param tooltip - Status bar tooltip
 * @param command - Command to execute on click
 * @returns Disposable status bar item
 */
export function createStatusBarItem(
  text: string,
  tooltip: string,
  command?: string
): vscode.StatusBarItem {
  const item = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  item.text = text;
  item.tooltip = tooltip;
  item.command = command;
  item.show();
  return item;
}
