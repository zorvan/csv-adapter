#!/usr/bin/env node
'use strict';

/**
 * Terminal Dashboard
 * 
 * Real-time display of chain status, wallet balances, and recent transfers.
 * Uses simple ANSI escape codes for a lightweight TUI (no external dependencies).
 */

const chalk = require('chalk');

// ANSI escape codes
const ESC = {
  clear: '\x1B[2J',
  home: '\x1B[H',
  cursorHide: '\x1B[?25l',
  cursorShow: '\x1B[?25h',
  up: (n) => `\x1B[${n}A`,
  down: (n) => `\x1B[${n}B`,
  eraseLine: '\x1B[2K'
};

let refreshInterval = null;
let currentData = null;

function getHealthIndicator(status) {
  if (!status) return chalk.red('●');
  if (status.running) return chalk.green('●');
  return chalk.red('●');
}

function getHealthText(status) {
  if (!status) return chalk.red('Offline');
  if (status.running) return chalk.green('Running');
  return chalk.red('Stopped');
}

function formatBlockHeight(height) {
  return height ? height.toLocaleString().padStart(8) : '0'.padStart(8);
}

function formatLatency(latency) {
  if (!latency) return 'N/A';
  if (latency < 10) return chalk.green(`${latency.toFixed(1)}ms`);
  if (latency < 50) return chalk.yellow(`${latency.toFixed(1)}ms`);
  return chalk.red(`${latency.toFixed(1)}ms`);
}

function formatBalance(balance) {
  if (!balance) return '0';
  if (balance.length > 20) return balance.substring(0, 20);
  return balance;
}

function renderDashboard(data) {
  const lines = [];
  const width = 80;

  // Header
  lines.push(chalk.bold.cyan('  CSV Local Dev Environment'));
  lines.push(chalk.gray('  ' + '='.repeat(width - 2)));
  lines.push('');

  if (!data || !data.chains) {
    lines.push(chalk.yellow('  No data available. Chains may not be running.'));
    lines.push('');
    return lines.join('\n');
  }

  // Chain Status Section
  lines.push(chalk.bold.cyan('  Chain Status'));
  lines.push(chalk.gray('  ' + '-'.repeat(width - 2)));
  
  const headerRow = [
    chalk.gray('Chain'.padEnd(14)),
    chalk.gray('Status'),
    chalk.gray('Block Height'),
    chalk.gray('Mempool'),
    chalk.gray('Latency'),
    chalk.gray('TPS')
  ].join('  ');
  lines.push('  ' + headerRow);
  lines.push(chalk.gray('  ' + '-'.repeat(width - 2)));

  const chainOrder = ['bitcoin', 'ethereum', 'sui', 'aptos'];
  for (const chain of chainOrder) {
    const chainData = data.chains[chain];
    if (!chainData) continue;

    const indicator = getHealthIndicator(chainData);
    const status = getHealthText(chainData);
    const blockHeight = formatBlockHeight(chainData.blockHeight || 0);
    const mempool = (chainData.mempoolSize || 0).toString().padStart(4);
    const latency = formatLatency(chainData.latency);
    const tps = (chainData.tps || 0).toFixed(1);

    const chainNames = {
      bitcoin: chalk.gray('Bitcoin'.padEnd(14)),
      ethereum: chalk.gray('Ethereum'.padEnd(14)),
      sui: chalk.gray('Sui'.padEnd(14)),
      aptos: chalk.gray('Aptos'.padEnd(14))
    };

    lines.push(
      '  ' +
      (chainNames[chain] || chalk.gray(chain.padEnd(14))) +
      '  ' + indicator +
      '  ' + chalk.white(blockHeight) +
      '  ' + chalk.white(mempool) +
      '  ' + latency +
      '  ' + chalk.white(tps)
    );
  }

  lines.push('');

  // Wallet Balances Section
  if (data.wallets && Object.keys(data.wallets).length > 0) {
    lines.push(chalk.bold.cyan('  Dev Wallet Balances'));
    lines.push(chalk.gray('  ' + '-'.repeat(width - 2)));

    for (const [chain, wallet] of Object.entries(data.wallets)) {
      const chainLabels = {
        bitcoin: chalk.gray('Bitcoin'.padEnd(12)),
        ethereum: chalk.gray('Ethereum'.padEnd(12)),
        sui: chalk.gray('Sui'.padEnd(12)),
        aptos: chalk.gray('Aptos'.padEnd(12))
      };

      const label = chainLabels[chain] || chalk.gray(chain.padEnd(12));
      const address = wallet.address ? wallet.address.substring(0, 18) + '...' : 'N/A';
      const balance = formatBalance(wallet.balance || '0');

      lines.push('  ' + label + '  ' + chalk.white(address) + '  ' + chalk.green(balance));
    }

    lines.push('');
  }

  // Registry Stats
  if (data.registry) {
    lines.push(chalk.bold.cyan('  Cross-Chain Registry'));
    lines.push(chalk.gray('  ' + '-'.repeat(width - 2)));
    lines.push(chalk.gray('    Total Rights:       ') + chalk.white(data.registry.totalRights || 0));
    lines.push(chalk.gray('    Pending Transfers:  ') + chalk.yellow(data.registry.activeTransfers || 0));
    lines.push(chalk.gray('    Completed:          ') + chalk.green(data.registry.completedTransfers || 0));
    lines.push('');
  }

  // RPC Stats
  if (data.stats) {
    lines.push(chalk.bold.cyan('  RPC Proxy Stats'));
    lines.push(chalk.gray('  ' + '-'.repeat(width - 2)));
    lines.push(chalk.gray('    Total Requests:     ') + chalk.white(data.stats.totalRequests || 0));
    lines.push(chalk.gray('    Avg Latency:        ') + chalk.white(`${data.stats.avgLatency || 0}ms`));
    
    if (data.stats.requestsByChain) {
      for (const [chain, count] of Object.entries(data.stats.requestsByChain)) {
        lines.push(chalk.gray(`    ${chain}:`.padEnd(24)) + chalk.white(count));
      }
    }
    lines.push('');
  }

  // Footer
  lines.push(chalk.gray('  ' + '='.repeat(width - 2)));
  lines.push(chalk.gray('  Press ') + chalk.cyan('Ctrl+C') + chalk.gray(' to stop all chains'));
  lines.push('');

  return lines.join('\n');
}

function startDashboard(options) {
  const { simulators, walletManager, registry, port, onExit } = options;

  // Hide cursor
  process.stdout.write(ESC.cursorHide);

  // Handle exit
  process.on('SIGINT', () => {
    process.stdout.write(ESC.cursorShow);
    if (onExit) onExit();
  });

  // Refresh function
  const refresh = () => {
    const data = {
      chains: {},
      wallets: walletManager ? walletManager.getAllWallets() : {},
      registry: registry ? registry.getStats() : null,
      stats: null
    };

    for (const [name, sim] of Object.entries(simulators || {})) {
      data.chains[name] = sim.getStatus();
    }

    currentData = data;
    const output = renderDashboard(data);
    
    // Clear screen and render
    process.stdout.write(ESC.home + ESC.clear + output);
  };

  // Initial render
  refresh();

  // Update every 2 seconds
  refreshInterval = setInterval(refresh, 2000);
}

function stopDashboard() {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
  process.stdout.write(ESC.cursorShow);
}

// Export for use by start command
module.exports = { startDashboard, stopDashboard, renderDashboard };
