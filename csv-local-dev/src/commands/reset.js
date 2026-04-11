#!/usr/bin/env node
'use strict';

const chalk = require('chalk');
const ora = require('ora');
const path = require('path');
const fs = require('fs');

const SRC_DIR = path.dirname(__dirname);
const STATE_DIR = path.join(SRC_DIR, 'state');

async function parseChainOptions(chainsOption) {
  if (chainsOption === 'all') {
    return ['bitcoin', 'ethereum', 'sui', 'aptos'];
  }
  return chainsOption.split(',').map(c => c.trim().toLowerCase());
}

async function stopRunningInstance() {
  const stateFile = path.join(STATE_DIR, 'server.json');
  if (fs.existsSync(stateFile)) {
    const state = JSON.parse(fs.readFileSync(stateFile, 'utf-8'));
    if (state.pid) {
      try {
        process.kill(state.pid, 'SIGTERM');
        // Wait for process to die
        await new Promise(resolve => setTimeout(resolve, 1000));
      } catch {
        // Process already dead
      }
    }
    // Also try HTTP shutdown
    if (state.port) {
      try {
        await fetch(`http://localhost:${state.port}/shutdown`, {
          method: 'POST',
          signal: AbortSignal.timeout(2000)
        });
      } catch {
        // ignore
      }
    }
  }
}

function clearAllState() {
  if (fs.existsSync(STATE_DIR)) {
    const files = fs.readdirSync(STATE_DIR);
    for (const file of files) {
      const filePath = path.join(STATE_DIR, file);
      if (fs.statSync(filePath).isFile()) {
        fs.unlinkSync(filePath);
      }
    }
  }
}

async function execute(options) {
  const chains = await parseChainOptions(options.chains);
  const spinner = ora();

  console.log(chalk.bold.cyan('\n  CSV Local Dev - Reset to Clean State'));
  console.log(chalk.gray('  ' + '='.repeat(50)));

  // Step 1: Stop any running instance
  spinner.start('Stopping running instance...');
  await stopRunningInstance();
  spinner.succeed(chalk.green('Running instance stopped'));

  // Step 2: Clear all state
  spinner.start('Clearing state files...');
  clearAllState();
  spinner.succeed(chalk.green('State cleared'));

  // Step 3: Clear simulator state directories
  spinner.start('Clearing simulator state...');
  const simStateDirs = [
    path.join(SRC_DIR, 'state', 'bitcoin'),
    path.join(SRC_DIR, 'state', 'ethereum'),
    path.join(SRC_DIR, 'state', 'sui'),
    path.join(SRC_DIR, 'state', 'aptos'),
    path.join(SRC_DIR, 'state', 'registry')
  ];

  for (const dir of simStateDirs) {
    if (fs.existsSync(dir)) {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }
  spinner.succeed(chalk.green('Simulator state cleared'));

  // Step 4: Restart with clean state
  spinner.start('Restarting with clean state...');

  // Clear require cache for simulators to get fresh state
  const simFiles = [
    'bitcoin.js', 'ethereum.js', 'sui.js', 'aptos.js',
    'registry.js', 'wallet.js', 'rpc-proxy.js'
  ];
  for (const file of simFiles) {
    const filePath = path.join(SRC_DIR, 'simulator', file);
    if (require.cache[filePath]) {
      delete require.cache[filePath];
    }
  }

  // Re-import and start
  const startCommand = require(path.join(SRC_DIR, 'commands', 'start'));
  await startCommand.execute({
    chains: options.chains,
    fastMode: options.fastMode,
    port: options.port,
    dashboard: false
  });

  console.log(chalk.gray('\n  ' + '='.repeat(50)));
  console.log(chalk.green('\n  Reset complete - clean state restored.'));
  console.log('');
}

module.exports = { execute };
