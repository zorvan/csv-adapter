#!/usr/bin/env node
'use strict';

const { Command } = require('commander');
const chalk = require('chalk');
const path = require('path');

const program = new Command();

// Resolve paths relative to this file
const SRC_DIR = path.dirname(__filename);
const COMMANDS_DIR = path.join(SRC_DIR, 'commands');

program
  .name('csv-local')
  .description('CSV Adapter Local Chain Simulator - Start, stop, and manage local blockchain environments')
  .version('1.0.0');

program
  .command('start')
  .description('Start the local dev environment with simulated chains')
  .option('--chains <chains>', 'Comma-separated list of chains to start (default: all)', 'all')
  .option('--fast-mode', 'Generate blocks every 1s instead of default intervals', false)
  .option('--port <port>', 'Base port for RPC endpoints (default: 8545)', '8545')
  .option('--background', 'Run chains in background (default: foreground dashboard)', false)
  .option('--no-dashboard', 'Disable the TUI dashboard even in foreground mode', false)
  .action(async (options) => {
    try {
      const startCommand = require(path.join(COMMANDS_DIR, 'start'));
      await startCommand.execute(options);
    } catch (error) {
      console.error(chalk.red('Error starting local dev environment:'), error.message);
      process.exit(1);
    }
  });

program
  .command('status')
  .description('Show chain health dashboard')
  .option('--json', 'Output status as JSON', false)
  .action(async (options) => {
    try {
      const statusCommand = require(path.join(COMMANDS_DIR, 'status'));
      await statusCommand.execute(options);
    } catch (error) {
      console.error(chalk.red('Error getting status:'), error.message);
      process.exit(1);
    }
  });

program
  .command('stop')
  .description('Stop all simulated chains and clean up')
  .option('--force', 'Force stop without graceful shutdown', false)
  .action(async (options) => {
    try {
      const stopCommand = require(path.join(COMMANDS_DIR, 'stop'));
      await stopCommand.execute(options);
    } catch (error) {
      console.error(chalk.red('Error stopping chains:'), error.message);
      process.exit(1);
    }
  });

program
  .command('reset')
  .description('Reset to initial clean state (stop, clear, restart)')
  .option('--chains <chains>', 'Comma-separated list of chains to reset (default: all)', 'all')
  .option('--fast-mode', 'Generate blocks every 1s after reset', false)
  .option('--port <port>', 'Base port for RPC endpoints (default: 8545)', '8545')
  .action(async (options) => {
    try {
      const resetCommand = require(path.join(COMMANDS_DIR, 'reset'));
      await resetCommand.execute(options);
    } catch (error) {
      console.error(chalk.red('Error resetting chains:'), error.message);
      process.exit(1);
    }
  });

program
  .command('wallet')
  .description('Show dev wallet information')
  .option('--chain <chain>', 'Show wallet for specific chain')
  .option('--export', 'Export wallet keys (WARNING: dev only)', false)
  .action(async (options) => {
    try {
      const { WalletManager } = require(path.join(SRC_DIR, 'simulator', 'wallet'));
      const walletManager = new WalletManager();
      await walletManager.initialize();
      
      if (options.chain) {
        const wallet = walletManager.getWallet(options.chain);
        if (wallet) {
          console.log(chalk.cyan(`\n${options.chain.toUpperCase()} Dev Wallet:`));
          console.log(chalk.gray('  Address:'), chalk.white(wallet.address));
          console.log(chalk.gray('  Balance:'), chalk.green(wallet.balance));
          if (options.export) {
            console.log(chalk.yellow('\n  WARNING: Private key exposure!'));
            console.log(chalk.gray('  Private Key:'), chalk.red(wallet.privateKey));
          }
        } else {
          console.error(chalk.red(`No wallet found for chain: ${options.chain}`));
        }
      } else {
        console.log(chalk.cyan('\nDev Wallets:'));
        const chains = ['bitcoin', 'ethereum', 'sui', 'aptos'];
        for (const chain of chains) {
          const wallet = walletManager.getWallet(chain);
          console.log(chalk.gray(`  ${chain.padEnd(12)}`), chalk.white(wallet.address.substring(0, 20) + '...'));
        }
      }
    } catch (error) {
      console.error(chalk.red('Error showing wallet info:'), error.message);
      process.exit(1);
    }
  });

program
  .command('scenario')
  .description('Run a pre-built scenario')
  .argument('<name>', 'Scenario name (basic-transfer, double-spend)')
  .action(async (name) => {
    try {
      const scenarioPath = path.join(SRC_DIR, 'scenarios', `${name}.js`);
      try {
        const scenario = require(scenarioPath);
        console.log(chalk.cyan(`\nRunning scenario: ${name}`));
        await scenario.execute();
        console.log(chalk.green(`\nScenario '${name}' completed successfully.`));
      } catch (err) {
        console.error(chalk.red(`Scenario '${name}' not found or failed: ${err.message}`));
        process.exit(1);
      }
    } catch (error) {
      console.error(chalk.red('Error running scenario:'), error.message);
      process.exit(1);
    }
  });

// Parse arguments
program.parse(process.argv);

// Show help if no command provided
if (!process.argv.slice(2).length) {
  program.outputHelp();
  console.log('\n' + chalk.gray('Quick start:'));
  console.log(chalk.cyan('  csv-local start              ') + chalk.gray('# Start all chains with dashboard'));
  console.log(chalk.cyan('  csv-local start --fast-mode  ') + chalk.gray('# Start with 1s block times'));
  console.log(chalk.cyan('  csv-local status             ') + chalk.gray('# Show chain health'));
  console.log(chalk.cyan('  csv-local reset              ') + chalk.gray('# Reset to clean state'));
  console.log(chalk.cyan('  csv-local stop               ') + chalk.gray('# Stop all chains'));
  console.log('');
}
