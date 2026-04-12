import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";

/**
 * Panel that visualizes CSV proof structures using mermaid.js.
 * Renders Merkle trees as interactive diagrams and shows proof verification steps.
 */
export class ProofVisualizerPanel {
  public static readonly viewType = "csv.proofVisualizer";

  private static _currentPanel: ProofVisualizerPanel | undefined;

  private readonly _panel: vscode.WebviewPanel;
  private _disposables: vscode.Disposable[] = [];

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    this._panel.webview.onDidReceiveMessage(
      (message) => this._handleMessage(message),
      null,
      this._disposables
    );

    this._update(extensionUri);
  }

  /**
   * Creates a new panel or reveals the existing one.
   */
  public static createOrShow(extensionUri: vscode.Uri): void {
    const column = vscode.window.activeTextEditor
      ? vscode.window.activeTextEditor.viewColumn
      : undefined;

    if (ProofVisualizerPanel._currentPanel) {
      ProofVisualizerPanel._currentPanel._panel.reveal(column);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      ProofVisualizerPanel.viewType,
      "CSV Proof Visualizer",
      column || vscode.ViewColumn.One,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(extensionUri, "webview")],
      }
    );

    ProofVisualizerPanel._currentPanel = new ProofVisualizerPanel(
      panel,
      extensionUri
    );
  }

  /**
   * Disposes all active panels.
   */
  public static disposeAll(): void {
    if (ProofVisualizerPanel._currentPanel) {
      ProofVisualizerPanel._currentPanel.dispose();
    }
  }

  /**
   * Handles messages from the webview.
   */
  private _handleMessage(message: unknown): void {
    if (typeof message === "object" && message !== null) {
      const msg = message as { command?: string; data?: unknown };
      switch (msg.command) {
        case "refresh":
          vscode.window.showInformationMessage("Proof visualizer refreshed");
          break;
        case "error":
          vscode.window.showErrorMessage(
            `Visualizer error: ${JSON.stringify(msg.data)}`
          );
          break;
      }
    }
  }

  /**
   * Disposes the panel and cleans up resources.
   */
  public dispose(): void {
    ProofVisualizerPanel._currentPanel = undefined;

    this._panel.dispose();

    while (this._disposables.length) {
      const disposable = this._disposables.pop();
      if (disposable) {
        disposable.dispose();
      }
    }
  }

  /**
   * Updates the webview content.
   */
  private _update(extensionUri: vscode.Uri): void {
    const webview = this._panel.webview;

    this._panel.title = "CSV Proof Visualizer";
    this._panel.webview.html = this._getHtmlForWebview(webview, extensionUri);
  }

  /**
   * Generates the HTML for the webview.
   */
  private _getHtmlForWebview(
    webview: vscode.Webview,
    extensionUri: vscode.Uri
  ): string {
    const htmlPath = vscode.Uri.joinPath(extensionUri, "webview", "proof-visualizer.html");

    try {
      const html = fs.readFileSync(htmlPath.fsPath, "utf-8");

      // Convert local resource URIs for mermaid.js CDN
      const nonce = this._getNonce();

      return html
        .replace(
          /<script[^>]*src="[^"]*mermaid[^"]*"[^>]*><\/script>/gi,
          `<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" nonce="${nonce}"></script>`
        )
        .replace(
          /<link[^>]*href="[^"]*mermaid[^"]*"[^>]*>/gi,
          `<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.css" nonce="${nonce}">`
        );
    } catch {
      return this._getFallbackHtml(webview, nonce);
    }
  }

  /**
   * Returns a Content Security Policy nonce.
   */
  private _getNonce(): string {
    let text = "";
    const possible =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for (let i = 0; i < 32; i++) {
      text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
  }

  /**
   * Provides fallback HTML if the template file cannot be read.
   */
  private _getFallbackHtml(webview: vscode.Webview, nonce: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CSV Proof Visualizer</title>
  <style>
    body { padding: 20px; font-family: var(--vscode-font-family); color: var(--vscode-foreground); background: var(--vscode-editor-background); }
    .proof-step { padding: 10px; margin: 5px 0; border-radius: 4px; background: var(--vscode-editorWidget-background); }
    .valid { border-left: 3px solid var(--vscode-terminal-ansiGreen); }
    .invalid { border-left: 3px solid var(--vscode-terminal-ansiRed); }
  </style>
</head>
<body>
  <h1>CSV Proof Visualizer</h1>
  <p>Select a proof to visualize from the wallet explorer, or run "CSV: Inspect Proof" command.</p>
  <div id="proof-container"></div>
</body>
</html>`;
  }
}
