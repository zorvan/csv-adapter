import * as vscode from "vscode";

/**
 * Detects CSV-specific error patterns in code and provides quick fixes.
 * Implements CodeActionProvider to offer inline suggestions.
 */
export class CsvErrorLensProvider implements vscode.CodeActionProvider {
  /**
   * The kinds of code actions this provider offers.
   */
  public static readonly providedCodeActionKinds = [
    vscode.CodeActionKind.QuickFix,
  ];

  /**
   * CSV-specific error patterns to detect.
   */
  private readonly errorPatterns: Array<{
    pattern: RegExp;
    message: string;
    fix: string;
    kind: "dependency" | "typo" | "error-handling" | "import";
  }> = [
    {
      pattern: /import.*@csv-adapter.*not found|Cannot find module.*csv-adapter/i,
      message: "Missing @csv-adapter/sdk dependency",
      fix: "Add @csv-adapter/sdk dependency",
      kind: "dependency",
    },
    {
      pattern: /(chain\s*:\s*["'])(eth|btc|apt|sol)(["'])/i,
      message: "Chain name may be abbreviated. Use full name.",
      fix: "Fix chain name typo",
      kind: "typo",
    },
    {
      pattern: /\.transfer\(|\.createRight\(|\.verifyProof\(/g,
      message: "Consider adding error handling for this CSV operation",
      fix: "Add error handling",
      kind: "error-handling",
    },
    {
      pattern: /(CsvClient|CsvError|MerkleProof|Right|ChainConfig)(?!\s*(extends|implements|:|<|{|,))/g,
      message: "Type may be undefined. Import the missing type.",
      fix: "Import missing type",
      kind: "import",
    },
  ];

  /**
   * Provides code actions for the given range and document.
   */
  provideCodeActions(
    document: vscode.TextDocument,
    _range: vscode.Range | vscode.Selection,
    context: vscode.CodeActionContext,
    _token: vscode.CancellationToken
  ): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];
    const text = document.getText();
    const languageId = document.languageId;

    // Only apply to relevant languages
    if (!["rust", "typescript", "javascript", "typescriptreact", "javascriptreact"].includes(languageId)) {
      return actions;
    }

    // Check each error pattern
    for (const errorPattern of this.errorPatterns) {
      const match = text.match(errorPattern.pattern);
      if (match) {
        const diagnostic = this.createDiagnostic(document, text, errorPattern);
        if (diagnostic) {
          const action = this.createQuickFix(
            document,
            diagnostic,
            errorPattern
          );
          if (action) {
            actions.push(action);
          }
        }
      }
    }

    // Also check context diagnostics
    for (const diagnostic of context.diagnostics) {
      const csvActions = this.handleDiagnostic(document, diagnostic);
      actions.push(...csvActions);
    }

    return actions;
  }

  /**
   * Creates a diagnostic for the matched pattern.
   */
  private createDiagnostic(
    document: vscode.TextDocument,
    text: string,
    errorPattern: { pattern: RegExp; message: string }
  ): vscode.Diagnostic | null {
    const match = text.match(errorPattern.pattern);
    if (!match || !match.index) {
      return null;
    }

    const startPos = document.positionAt(match.index);
    const endPos = document.positionAt(match.index + match[0].length);
    const range = new vscode.Range(startPos, endPos);

    return new vscode.Diagnostic(
      range,
      `CSV Adapter: ${errorPattern.message}`,
      vscode.DiagnosticSeverity.Information
    );
  }

  /**
   * Creates a quick fix for the given error pattern.
   */
  private createQuickFix(
    document: vscode.TextDocument,
    diagnostic: vscode.Diagnostic,
    errorPattern: { fix: string; kind: string }
  ): vscode.CodeAction | null {
    const action = new vscode.CodeAction(
      `CSV: ${errorPattern.fix}`,
      vscode.CodeActionKind.QuickFix
    );
    action.diagnostics = [diagnostic];

    switch (errorPattern.kind) {
      case "dependency":
        action.edit = this.createDependencyFix(document, diagnostic);
        break;
      case "typo":
        action.edit = this.createTypoFix(document, diagnostic);
        break;
      case "error-handling":
        action.edit = this.createErrorHandlingFix(document, diagnostic);
        break;
      case "import":
        action.edit = this.createImportFix(document, diagnostic);
        break;
    }

    action.isPreferred = true;
    return action;
  }

  /**
   * Creates a workspace edit to add the SDK dependency.
   */
  private createDependencyFix(
    _document: vscode.TextDocument,
    _diagnostic: vscode.Diagnostic
  ): vscode.WorkspaceEdit {
    const edit = new vscode.WorkspaceEdit();

    // This would typically modify package.json or Cargo.toml
    // For now, provide a terminal command suggestion
    vscode.window.showInformationMessage(
      'Run "npm install @csv-adapter/sdk" or "cargo add csv-adapter" to add the dependency'
    );

    return edit;
  }

  /**
   * Creates a workspace edit to fix chain name typos.
   */
  private createTypoFix(
    document: vscode.TextDocument,
    diagnostic: vscode.Diagnostic
  ): vscode.WorkspaceEdit {
    const text = document.getText(diagnostic.range);
    const edit = new vscode.WorkspaceEdit();

    const chainMap: Record<string, string> = {
      eth: "ethereum",
      btc: "bitcoin",
      apt: "aptos",
      sol: "solana",
      Eth: "Ethereum",
      Btc: "Bitcoin",
      Apt: "Aptos",
      Sol: "Solana",
    };

    // Extract the abbreviated chain name
    const match = text.match(/["']?(eth|btc|apt|sol|Eth|Btc|Apt|Sol)["']?/i);
    if (match) {
      const fullName = chainMap[match[1]];
      if (fullName) {
        edit.replace(document.uri, diagnostic.range, `"${fullName}"`);
      }
    }

    return edit;
  }

  /**
   * Creates a workspace edit to wrap the call in error handling.
   */
  private createErrorHandlingFix(
    document: vscode.TextDocument,
    diagnostic: vscode.Diagnostic
  ): vscode.WorkspaceEdit {
    const edit = new vscode.WorkspaceEdit();
    const line = document.lineAt(diagnostic.range.start.line);
    const indent = line.text.match(/^(\s*)/)?.[1] || "";

    const isRust = document.languageId === "rust";

    if (isRust) {
      const wrapStart = `${indent}match `;
      const wrapEnd = ` {\n${indent}    Ok(result) => result,\n${indent}    Err(e) => {\n${indent}        eprintln!("CSV error: {}", e);\n${indent}        return Err(e.into());\n${indent}    }\n${indent}}`;

      edit.insert(document.uri, diagnostic.range.start, wrapStart);
      edit.insert(document.uri, diagnostic.range.end, wrapEnd);
    } else {
      const wrapStart = `${indent}try {\n${indent}  `;
      const wrapEnd = `\n${indent}} catch (error) {\n${indent}  if (error instanceof CsvError) {\n${indent}    console.error("CSV error:", error.message);\n${indent}    throw error;\n${indent}  }\n${indent}  throw error;\n${indent}}`;

      edit.insert(document.uri, diagnostic.range.start, wrapStart);
      edit.insert(document.uri, diagnostic.range.end, wrapEnd);
    }

    return edit;
  }

  /**
   * Creates a workspace edit to add missing imports.
   */
  private createImportFix(
    document: vscode.TextDocument,
    _diagnostic: vscode.Diagnostic
  ): vscode.WorkspaceEdit {
    const edit = new vscode.WorkspaceEdit();
    const isRust = document.languageId === "rust";

    // Find the first import line or add at the top
    const firstLine = document.lineAt(0);

    let importStatement: string;
    if (isRust) {
      importStatement = "use csv_adapter::{CsvClient, CsvError, MerkleProof, Right, ChainConfig};\n";
    } else {
      importStatement = 'import { CsvClient, CsvError, MerkleProof, Right, ChainConfig } from "@csv-adapter/sdk";\n';
    }

    edit.insert(document.uri, firstLine.range.start, importStatement);
    return edit;
  }

  /**
   * Handles existing diagnostics from VS Code (e.g., TypeScript compiler errors).
   */
  private handleDiagnostic(
    document: vscode.TextDocument,
    diagnostic: vscode.Diagnostic
  ): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];
    const message = diagnostic.message;

    // Handle missing module errors
    if (message.includes("csv-adapter") || message.includes("@csv-adapter")) {
      const action = new vscode.CodeAction(
        "CSV: Add @csv-adapter/sdk dependency",
        vscode.CodeActionKind.QuickFix
      );
      action.diagnostics = [diagnostic];
      action.isPreferred = true;
      vscode.window.showInformationMessage(
        'Run "npm install @csv-adapter/sdk" to install the missing package'
      );
      actions.push(action);
    }

    // Handle missing type errors
    if (
      message.includes("cannot find name") &&
      (message.includes("Csv") ||
        message.includes("Merkle") ||
        message.includes("Right") ||
        message.includes("Chain"))
    ) {
      const action = new vscode.CodeAction(
        "CSV: Import missing type",
        vscode.CodeActionKind.QuickFix
      );
      action.diagnostics = [diagnostic];
      action.isPreferred = true;
      actions.push(action);
    }

    return actions;
  }
}
