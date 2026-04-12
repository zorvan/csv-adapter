import * as vscode from "vscode";
import { getChainColor, formatRightId } from "./utils";
import { extensionConfiguration } from "./configuration";

/**
 * Types of tree items in the wallet explorer.
 */
export enum WalletItemType {
  Wallet = "wallet",
  Chain = "chain",
  Balance = "balance",
  Right = "right",
  RightsGroup = "rights-group",
  Transfer = "transfer",
  TransfersGroup = "transfers-group",
  Error = "error",
}

/**
 * Tree item representing a node in the wallet explorer.
 */
class WalletTreeItem extends vscode.TreeItem {
  public readonly itemType: WalletItemType;

  constructor(
    label: string,
    collapsibleState: vscode.TreeItemCollapsibleState,
    itemType: WalletItemType,
    options?: {
      iconPath?: vscode.ThemeIcon | vscode.Uri;
      description?: string;
      tooltip?: string;
      contextValue?: string;
      command?: vscode.Command;
    }
  ) {
    super(label, collapsibleState);

    this.itemType = itemType;
    this.iconPath = options?.iconPath;
    this.description = options?.description;
    this.tooltip = options?.tooltip || label;
    this.contextValue = options?.contextValue || itemType;

    if (options?.command) {
      this.command = options.command;
    }
  }
}

/**
 * Data provider for the wallet explorer tree view.
 * Displays wallet balances, Rights tree, and recent transfers.
 */
export class WalletExplorerProvider
  implements vscode.TreeDataProvider<WalletTreeItem>
{
  private _onDidChangeTreeData = new vscode.EventEmitter<
    WalletTreeItem | undefined | null | void
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private readonly context: vscode.ExtensionContext;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  /**
   * Triggers a refresh of the tree view.
   */
  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  /**
   * Returns the tree item for display.
   */
  getTreeItem(element: WalletTreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * Returns child elements for a given parent, or root elements if no parent.
   */
  getChildren(element?: WalletTreeItem): Thenable<WalletTreeItem[]> {
    if (!element) {
      return this.getRootItems();
    }

    switch (element.itemType) {
      case WalletItemType.Chain:
        return this.getChainChildren(element);
      case WalletItemType.RightsGroup:
        return this.getRightsChildren(element);
      case WalletItemType.TransfersGroup:
        return this.getTransfersChildren(element);
      default:
        return Promise.resolve([]);
    }
  }

  /**
   * Returns the root-level items for the wallet explorer.
   */
  private async getRootItems(): Promise<WalletTreeItem[]> {
    const items: WalletTreeItem[] = [];
    const config = extensionConfiguration.getConfig(this.context);

    // Wallet status section
    const walletPath = config.walletPath;
    if (walletPath) {
      items.push(
        new WalletTreeItem(
          "Wallet",
          vscode.TreeItemCollapsibleState.Collapsed,
          WalletItemType.Wallet,
          {
            iconPath: new vscode.ThemeIcon("wallet"),
            description: walletPath,
          }
        )
      );
    } else {
      items.push(
        new WalletTreeItem(
          "No wallet configured",
          vscode.TreeItemCollapsibleState.None,
          WalletItemType.Error,
          {
            iconPath: new vscode.ThemeIcon("warning", new vscode.ThemeColor("editorWarning.foreground")),
            description: "Set csv.wallet.path in settings",
          }
        )
      );
    }

    // Chains section
    const chains = config.chains;
    if (chains.length > 0) {
      for (const chain of chains) {
        const chainColor = getChainColor(chain);
        items.push(
          new WalletTreeItem(
            chain.charAt(0).toUpperCase() + chain.slice(1),
            vscode.TreeItemCollapsibleState.Collapsed,
            WalletItemType.Chain,
            {
              iconPath: new vscode.ThemeIcon("globe"),
              description: config.network,
              tooltip: `${chain} (${config.network})`,
            }
          )
        );
      }
    }

    // Rights section
    items.push(
      new WalletTreeItem(
        "Rights",
        vscode.TreeItemCollapsibleState.Collapsed,
        WalletItemType.RightsGroup,
        {
          iconPath: new vscode.ThemeIcon("key"),
          description: "Grouped by chain",
        }
      )
    );

    // Recent transfers section
    items.push(
      new WalletTreeItem(
        "Recent Transfers",
        vscode.TreeItemCollapsibleState.Collapsed,
        WalletItemType.TransfersGroup,
        {
          iconPath: new vscode.ThemeIcon("arrow-swap"),
          description: "Last 10",
        }
      )
    );

    return items;
  }

  /**
   * Returns child items for a chain node (balances, rights count).
   */
  private async getChainChildren(
    chainItem: WalletTreeItem
  ): Promise<WalletTreeItem[]> {
    const chain = chainItem.label.toLowerCase() as string;
    const items: WalletTreeItem[] = [];

    // Simulated balance - in production, fetch from actual chain
    const balance = await this.getBalance(chain);
    items.push(
      new WalletTreeItem(
        "Balance",
        vscode.TreeItemCollapsibleState.None,
        WalletItemType.Balance,
        {
          iconPath: new vscode.ThemeIcon("credit-card"),
          description: balance,
        }
      )
    );

    return items;
  }

  /**
   * Returns Rights children grouped by chain.
   */
  private async getRightsChildren(
    _parent: WalletTreeItem
  ): Promise<WalletTreeItem[]> {
    const items: WalletTreeItem[] = [];

    // Simulated rights list - in production, fetch from CSV adapter
    const mockRights = [
      { id: "0x" + "a".repeat(64), chain: "ethereum", type: "ownership" },
      { id: "0x" + "b".repeat(64), chain: "aptos", type: "transfer" },
      { id: "0x" + "c".repeat(64), chain: "sui", type: "ownership" },
    ];

    if (mockRights.length === 0) {
      items.push(
        new WalletTreeItem(
          "No Rights found",
          vscode.TreeItemCollapsibleState.None,
          WalletItemType.Error,
          {
            iconPath: new vscode.ThemeIcon("info"),
          }
        )
      );
      return items;
    }

    for (const right of mockRights) {
      const chainColor = getChainColor(right.chain);
      items.push(
        new WalletTreeItem(
          formatRightId(right.id),
          vscode.TreeItemCollapsibleState.None,
          WalletItemType.Right,
          {
            iconPath: new vscode.ThemeIcon("key", chainColor),
            description: right.chain,
            tooltip: `Right ID: ${right.id}\nType: ${right.type}\nChain: ${right.chain}`,
            contextValue: "right-item",
            command: {
              command: "csv.copyRightId",
              title: "Copy Right ID",
              arguments: [right.id],
            },
          }
        )
      );
    }

    return items;
  }

  /**
   * Returns recent transfers children.
   */
  private async getTransfersChildren(
    _parent: WalletTreeItem
  ): Promise<WalletTreeItem[]> {
    const items: WalletTreeItem[] = [];

    // Simulated transfers - in production, fetch from chain history
    const mockTransfers = [
      { from: "ethereum", to: "aptos", amount: "100", status: "completed" },
      { from: "aptos", to: "sui", amount: "50", status: "pending" },
    ];

    if (mockTransfers.length === 0) {
      items.push(
        new WalletTreeItem(
          "No recent transfers",
          vscode.TreeItemCollapsibleState.None,
          WalletItemType.Error,
          {
            iconPath: new vscode.ThemeIcon("info"),
          }
        )
      );
      return items;
    }

    for (const transfer of mockTransfers) {
      const statusIcon =
        transfer.status === "completed"
          ? new vscode.ThemeIcon("check", new vscode.ThemeColor("terminal.ansiGreen"))
          : new vscode.ThemeIcon("clock", new vscode.ThemeColor("terminal.ansiYellow"));

      items.push(
        new WalletTreeItem(
          `${transfer.amount} ${transfer.from} -> ${transfer.to}`,
          vscode.TreeItemCollapsibleState.None,
          WalletItemType.Transfer,
          {
            iconPath: statusIcon,
            description: transfer.status,
          }
        )
      );
    }

    return items;
  }

  /**
   * Retrieves the balance for a given chain.
   * In production, this would query the actual chain RPC endpoint.
   */
  private async getBalance(chain: string): Promise<string> {
    // Simulated balance - replace with actual chain query
    return `${(Math.random() * 1000).toFixed(4)} ${chain.toUpperCase()}`;
  }
}
