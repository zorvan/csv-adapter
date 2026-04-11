#!/usr/bin/env node
'use strict';

const chalk = require('chalk');
const ora = require('ora');
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

function clearState() {
  if (fs.existsSync(STATE_DIR)) {
    const files = fs.readdirSync(STATE_DIR);
    for (const file of files) {
      fs.unlinkSync(path.join(STATE_DIR, file));
    }
  }
}

async function stopServer(port, force = false) {
  if (force) {
    // Force kill the process
    const state = loadState();
    if (state && state.pid) {
      try {
        process.kill(state.pid, 'SIGKILL');
        return { success: true, method: 'force' };
      } catch (err) {
        // Process already dead
        return { success: true, method: 'already_stopped' };
      }
    }
  }

  // Graceful shutdown via HTTP
  try {
    const response = await fetch(`http://localhost:${port}/shutdown`, {
      method: 'POST',
      signal: AbortSignal.timeout(3000)
    });
    if (response.ok) {
      return { success: true, method: 'graceful' };
    }
  } catch {
    // Server might not have shutdown endpoint, try process kill
  }

  // Try process kill as fallback
  const state = loadState();
  if (state && state.pid) {
    try {
      process.kill(state.pid, 'SIGTERM');
      return { success: true, method: 'sigterm' };
    } catch (err) {
      return { success: true, method: 'already_stopped' };
    }
  }

  return { success: false, method: 'none' };
}

async function execute(options) {
  const force = options.force === true;
  const spinner = ora();

  const state = loadState();
  if (!state) {
    console.log(chalk.yellow('\n  No local dev environment is currently running.'));
    console.log('');
    return;
  }

  console.log(chalk.bold.cyan('\n  Stopping CSV Local Dev Environment'));
  console.log(chalk.gray('  ' + '='.repeat(50)));

  // Stop server
  spinner.start('Stopping RPC proxy server...');
  const stopResult = await stopServer(state.port, force);
  if (stopResult.success) {
    spinner.succeed(chalk.green(`RPC proxy stopped (${stopResult.method})`));
  } else {
    spinner.fail(chalk.red('Failed to stop RPC proxy'));
  }

  // Show shutdown summary
  console.log(chalk.gray('\n  ' + '-'.repeat(50)));
  console.log(chalk.cyan('\n  Shutdown Summary:'));
  console.log(chalk.gray('    Server PID:     ') + chalk.white(state.pid || 'N/A'));
  console.log(chalk.gray('    Port:           ') + chalk.white(state.port));
  console.log(chalk.gray('    Chains Stopped: ') + chalk.white((state.chains || []).join(', ') || 'all'));
  console.log(chalk.gray('    Method:         ') + chalk.white(stopResult.method));
  console.log(chalk.gray('    State Cleared:  ') + chalk.green('Yes'));

  // Clear state
  clearState();

  console.log(chalk.gray('\n  ' + '='.repeat(50)));
  console.log(chalk.green('\n  All services stopped successfully.'));
  console.log(chalk.gray('  Restart with: ') + chalk.cyan('csv-local start') + chalk.gray('\n'));
}

module.exports = { execute };
