import * as vscode from "vscode";
import { registerCommands } from "./commands";
import { WalletExplorerProvider } from "./wallet-explorer";
import { ProofVisualizerPanel } from "./proof-visualizer";
import { CsvErrorLensProvider } from "./error-lens";
import { extensionConfiguration } from "./configuration";

const EXTENSION_KEY = "csv.adapter.activated";

/**
 * Extension entry point. Registers all features and shows a welcome message on first activation.
 */
export function activate(context: vscode.ExtensionContext): void {
  console.log("CSV Adapter extension is now active");

  // Initialize configuration
  extensionConfiguration.initialize(context);

  // Register commands
  registerCommands(context);

  // Register wallet explorer
  const walletExplorerProvider = new WalletExplorerProvider(context);
  vscode.window.registerTreeDataProvider(
    "csvWalletExplorer",
    walletExplorerProvider
  );

  // Register refresh command for wallet explorer
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "csv.refreshWalletExplorer",
      () => {
        walletExplorerProvider.refresh();
      }
    )
  );

  // Register proof visualizer panel
  context.subscriptions.push(
    vscode.commands.registerCommand("csv.inspectProof", () => {
      ProofVisualizerPanel.createOrShow(context.extensionUri);
    })
  );

  // Register inline error lens provider
  const errorLensProvider = new CsvErrorLensProvider();
  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider(
      [
        { language: "rust", scheme: "file" },
        { language: "typescript", scheme: "file" },
        { language: "javascript", scheme: "file" },
      ],
      errorLensProvider,
      {
        providedCodeActionKinds:
          CsvErrorLensProvider.providedCodeActionKinds,
      }
    )
  );

  // Show welcome message on first activation
  const hasActivatedBefore = context.globalState.get<boolean>(EXTENSION_KEY);
  if (!hasActivatedBefore) {
    context.globalState.update(EXTENSION_KEY, true);
    showWelcomeMessage(context);
  }

  // Listen for configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("csv")) {
        extensionConfiguration.onConfigurationChanged(context);
        walletExplorerProvider.refresh();
      }
    })
  );
}

/**
 * Displays a welcome notification with quick start options.
 */
async function showWelcomeMessage(context: vscode.ExtensionContext): Promise<void> {
  const selection = await vscode.window.showInformationMessage(
    "CSV Adapter is now installed! Get started by creating your first Right or exploring the documentation.",
    "Create Right",
    "Open Documentation",
    "Run Tutorial"
  );

  if (selection === "Create Right") {
    await vscode.commands.executeCommand("csv.createRight");
  } else if (selection === "Open Documentation") {
    await vscode.commands.executeCommand("csv.openDocumentation");
  } else if (selection === "Run Tutorial") {
    await vscode.commands.executeCommand("csv.runTutorial");
  }
}

/**
 * Extension cleanup on deactivation.
 */
export function deactivate(): void {
  // Dispose all panels and providers
  ProofVisualizerPanel.disposeAll();
}
