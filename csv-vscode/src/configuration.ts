import * as vscode from "vscode";

/**
 * Configuration interface for CSV Adapter extension settings.
 */
export interface CsvConfig {
  chains: string[];
  network: "mainnet" | "testnet" | "devnet";
  rpcEndpoints: Record<string, string>;
  walletPath: string;
  logLevel: "debug" | "info" | "warn" | "error";
}

/**
 * Default configuration values.
 */
const DEFAULT_CONFIG: CsvConfig = {
  chains: ["ethereum", "aptos", "sui"],
  network: "testnet",
  rpcEndpoints: {},
  walletPath: "",
  logLevel: "info",
};

/**
 * Singleton manager for extension configuration.
 * Provides typed access to VS Code settings with validation.
 */
class ExtensionConfiguration {
  private _config: CsvConfig = { ...DEFAULT_CONFIG };

  /**
   * Initializes configuration from VS Code settings.
   */
  initialize(context: vscode.ExtensionContext): void {
    this._config = this.readConfig();
  }

  /**
   * Called when configuration changes.
   */
  onConfigurationChanged(_context: vscode.ExtensionContext): void {
    this._config = this.readConfig();
  }

  /**
   * Returns the current configuration.
   */
  getConfig(_context?: never): CsvConfig {
    return { ...this._config };
  }

  /**
   * Reads configuration from VS Code settings with validation.
   */
  private readConfig(): CsvConfig {
    const config = vscode.workspace.getConfiguration("csv");

    const chains = this.readChains(config);
    const network = this.readNetwork(config);
    const rpcEndpoints = this.readRpcEndpoints(config);
    const walletPath = this.readWalletPath(config);
    const logLevel = this.readLogLevel(config);

    return {
      chains,
      network,
      rpcEndpoints,
      walletPath,
      logLevel,
    };
  }

  /**
   * Reads and validates the chains configuration.
   */
  private readChains(config: vscode.WorkspaceConfiguration): string[] {
    const chains = config.get<string[]>("chains");
    const validChains = ["ethereum", "bitcoin", "aptos", "sui", "solana"];

    if (!Array.isArray(chains) || chains.length === 0) {
      return DEFAULT_CONFIG.chains;
    }

    return chains.filter((chain) => validChains.includes(chain));
  }

  /**
   * Reads and validates the network configuration.
   */
  private readNetwork(
    config: vscode.WorkspaceConfiguration
  ): "mainnet" | "testnet" | "devnet" {
    const network = config.get<string>("network");

    if (network === "mainnet" || network === "testnet" || network === "devnet") {
      return network;
    }

    return DEFAULT_CONFIG.network;
  }

  /**
   * Reads custom RPC endpoints for each chain.
   */
  private readRpcEndpoints(
    config: vscode.WorkspaceConfiguration
  ): Record<string, string> {
    const chains = ["ethereum", "bitcoin", "aptos", "sui", "solana"];
    const endpoints: Record<string, string> = {};

    for (const chain of chains) {
      const rpc = config.get<string>(`rpc.${chain}`);
      if (rpc && rpc.trim().length > 0) {
        endpoints[chain] = rpc.trim();
      }
    }

    return endpoints;
  }

  /**
   * Reads and validates the wallet path configuration.
   */
  private readWalletPath(config: vscode.WorkspaceConfiguration): string {
    const path = config.get<string>("wallet.path");
    return path || DEFAULT_CONFIG.walletPath;
  }

  /**
   * Reads and validates the log level configuration.
   */
  private readLogLevel(
    config: vscode.WorkspaceConfiguration
  ): "debug" | "info" | "warn" | "error" {
    const level = config.get<string>("logLevel");
    const validLevels = ["debug", "info", "warn", "error"];

    if (validLevels.includes(level || "")) {
      return level as "debug" | "info" | "warn" | "error";
    }

    return DEFAULT_CONFIG.logLevel;
  }

  /**
   * Updates a specific configuration value.
   */
  async updateConfig(
    key: string,
    value: unknown,
    target: vscode.ConfigurationTarget = vscode.ConfigurationTarget.Workspace
  ): Promise<void> {
    await vscode.workspace.getConfiguration("csv").update(key, value, target);
    this._config = this.readConfig();
  }

  /**
   * Returns the RPC endpoint for a specific chain.
   * Falls back to default endpoints if not configured.
   */
  getRpcEndpoint(chain: string): string | undefined {
    return this._config.rpcEndpoints[chain];
  }

  /**
   * Returns whether the extension is properly configured.
   */
  isConfigured(): boolean {
    return (
      this._config.chains.length > 0 &&
      this._config.network !== undefined
    );
  }

  /**
   * Returns configuration warnings if any settings are missing.
   */
  getWarnings(): string[] {
    const warnings: string[] = [];

    if (this._config.chains.length === 0) {
      warnings.push("No chains configured. Set csv.chains in settings.");
    }

    if (!this._config.walletPath) {
      warnings.push("No wallet path configured. Set csv.wallet.path in settings.");
    }

    return warnings;
  }
}

// Export singleton instance
export const extensionConfiguration = new ExtensionConfiguration();
