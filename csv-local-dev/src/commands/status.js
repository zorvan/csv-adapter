#!/usr/bin/env node
'use strict';

const chalk = require('chalk');
const path = require('path');
const fs = require('fs');

const SRC_DIR = path.dirname(__dirname);
const STATE_DIR = path.join(SRC_DIR, 'state');

function loadState() {
  const stateFile = path.join(STATE_DIR, 'server.json');
  if (fs.existsSync(stateFile)) {
    return JSON.parse(fs.readFileSync(stateFile, 'utf-8'));
  }
  return null;
}

async function fetchHealth(port) {
  try {
    const response = await fetch(`http://localhost:${port}/health`);
    return await response.json();
  } catch {
    return null;
  }
}

async function fetchWallets(port) {
  try {
    const response = await fetch(`http://localhost:${port}/dashboard`);
    if (response.ok) {
      return await response.json();
    }
  } catch {
    // ignore
  }
  return null;
}

function getHealthIndicator(status) {
  if (!status) return chalk.red('●');
  if (status.running) return chalk.green('●');
  if (status.degraded) return chalk.yellow('●');
  return chalk.red('●');
}

function formatUptime(startedAt) {
  if (!startedAt) return 'N/A';
  const start = new Date(startedAt);
  const now = new Date();
  const diff = now - start;
  const minutes = Math.floor(diff / 60000);
  const seconds = Math.floor((diff % 60000) / 1000);
  if (minutes > 0) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
}

function formatBlockHeight(height) {
  return height ? height.toLocaleString() : '0';
}

function formatLatency(latency) {
  if (!latency) return 'N/A';
  if (latency < 10) return chalk.green(`${latency.toFixed(1)}ms`);
  if (latency < 50) return chalk.yellow(`${latency.toFixed(1)}ms`);
  return chalk.red(`${latency.toFixed(1)}ms`);
}

async function execute(options) {
  const state = loadState();

  if (!state) {
    if (options.json) {
      console.log(JSON.stringify({ status: 'stopped', chains: {} }, null, 2));
    } else {
      console.log(chalk.yellow('\n  No local dev environment is currently running.'));
      console.log(chalk.gray('  Start it with: ') + chalk.cyan('csv-local start') + chalk.gray('\n'));
    }
    return;
  }

  const health = await fetchHealth(state.port);
  const dashboard = await fetchWallets(state.port);

  if (options.json) {
    console.log(JSON.stringify({
      status: health ? 'running' : 'unreachable',
      pid: state.pid,
      uptime: formatUptime(state.startedAt),
      chains: health?.chains || {},
      wallets: dashboard?.wallets || {}
    }, null, 2));
    return;
  }

  // Print dashboard
  console.log(chalk.bold.cyan('\n  CSV Local Dev - Chain Health Dashboard'));
  console.log(chalk.gray('  ' + '='.repeat(60)));

  console.log(chalk.gray('\n  Server Info:'));
  console.log(chalk.gray('    PID:      ') + chalk.white(state.pid));
  console.log(chalk.gray('    Port:     ') + chalk.white(state.port));
  console.log(chalk.gray('    Uptime:   ') + chalk.white(formatUptime(state.startedAt)));
  console.log(chalk.gray('    Status:   ') + (health ? chalk.green('Running') : chalk.red('Unreachable')));

  if (!health) {
    console.log(chalk.yellow('\n  Server is not responding. Try: csv-local reset'));
    console.log('');
    return;
  }

  // Chain status table
  console.log(chalk.cyan('\n  Chain Status:'));
  console.log(chalk.gray('  ' + '-'.repeat(60)));

  const chainHeaders = [
    chalk.gray('Chain'.padEnd(14)),
    chalk.gray('Status'),
    chalk.gray('Block'),
    chalk.gray('Pending'),
    chalk.gray('Latency'),
    chalk.gray('TPS')
  ].join('  ');
  console.log('  ' + chainHeaders);
  console.log(chalk.gray('  ' + '-'.repeat(60)));

  const chainNames = ['bitcoin', 'ethereum', 'sui', 'aptos'];
  for (const chain of chainNames) {
    const chainStatus = health.chains[chain];
    if (!chainStatus) continue;

    const indicator = getHealthIndicator(chainStatus);
    const blockHeight = formatBlockHeight(chainStatus.blockHeight || chainStatus.block_number || 0);
    const pending = (chainStatus.pendingTxs || chainStatus.pending_transactions || 0).toString().padStart(4);
    const latency = formatLatency(chainStatus.latency);
    const tps = (chainStatus.tps || 0).toFixed(1);

    const chainDisplay = {
      bitcoin: chalk.gray('Bitcoin'.padEnd(14)),
      ethereum: chalk.gray('Ethereum'.padEnd(14)),
      sui: chalk.gray('Sui'.padEnd(14)),
      aptos: chalk.gray('Aptos'.padEnd(14))
    };

    console.log(
      '  ' +
      chainDisplay[chain] +
      '  ' + indicator +
      '  ' + chalk.white(blockHeight.padStart(6)) +
      '  ' + chalk.white(pending) +
      '  ' + latency +
      '  ' + chalk.white(tps)
    );
  }

  // Wallet balances
  if (dashboard && dashboard.wallets) {
    console.log(chalk.cyan('\n  Dev Wallet Balances:'));
    console.log(chalk.gray('  ' + '-'.repeat(60)));

    const walletHeaders = [
      chalk.gray('Chain'.padEnd(14)),
      chalk.gray('Address'),
      chalk.gray('Balance')
    ].join('  ');
    console.log('  ' + walletHeaders);
    console.log(chalk.gray('  ' + '-'.repeat(60)));

    for (const [chain, wallet] of Object.entries(dashboard.wallets)) {
      const addr = wallet.address ? wallet.address.substring(0, 20) + '...' : 'N/A';
      const balance = wallet.balance || '0';
      const balanceDisplay = {
        bitcoin: chalk.gray('Bitcoin'.padEnd(14)),
        ethereum: chalk.gray('Ethereum'.padEnd(14)),
        sui: chalk.gray('Sui'.padEnd(14)),
        aptos: chalk.gray('Aptos'.padEnd(14))
      };

      console.log(
        '  ' +
        (balanceDisplay[chain] || chalk.gray(chain.padEnd(14))) +
        '  ' + chalk.white(addr) +
        '  ' + chalk.green(balance)
      );
    }
  }

  // Registry stats
  if (dashboard && dashboard.registry) {
    console.log(chalk.cyan('\n  Cross-Chain Registry:'));
    console.log(chalk.gray('  ' + '-'.repeat(60)));
    console.log(chalk.gray('    Total Rights:     ') + chalk.white(dashboard.registry.totalRights || 0));
    console.log(chalk.gray('    Active Transfers: ') + chalk.white(dashboard.registry.activeTransfers || 0));
    console.log(chalk.gray('    Completed:        ') + chalk.green(dashboard.registry.completedTransfers || 0));
  }

  console.log(chalk.gray('\n  ' + '='.repeat(60)) + '\n');
}

module.exports = { execute };
