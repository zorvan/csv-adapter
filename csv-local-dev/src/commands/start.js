#!/usr/bin/env node
'use strict';

const express = require('express');
const cors = require('cors');
const chalk = require('chalk');
const ora = require('ora');
const path = require('path');
const fs = require('fs');

const SRC_DIR = path.dirname(__dirname);
const SIMULATOR_DIR = path.join(SRC_DIR, 'simulator');
const STATE_DIR = path.join(SRC_DIR, 'state');

const { BitcoinSimulator } = require(path.join(SIMULATOR_DIR, 'bitcoin'));
const { EthereumSimulator } = require(path.join(SIMULATOR_DIR, 'ethereum'));
const { SuiSimulator } = require(path.join(SIMULATOR_DIR, 'sui'));
const { AptosSimulator } = require(path.join(SIMULATOR_DIR, 'aptos'));
const { CrossChainRegistry } = require(path.join(SIMULATOR_DIR, 'registry'));
const { WalletManager } = require(path.join(SIMULATOR_DIR, 'wallet'));
const { RpcProxy } = require(path.join(SIMULATOR_DIR, 'rpc-proxy'));

const CHAIN_PORTS = {
  bitcoin: 0,
  ethereum: 0,
  sui: 0,
  aptos: 0,
  proxy: 0
};

let server = null;

async function parseChainOptions(chainsOption) {
  if (chainsOption === 'all') {
    return ['bitcoin', 'ethereum', 'sui', 'aptos'];
  }
  return chainsOption.split(',').map(c => c.trim().toLowerCase());
}

function ensureStateDir() {
  if (!fs.existsSync(STATE_DIR)) {
    fs.mkdirSync(STATE_DIR, { recursive: true });
  }
}

function saveState(state) {
  ensureStateDir();
  fs.writeFileSync(
    path.join(STATE_DIR, 'server.json'),
    JSON.stringify({
      pid: process.pid,
      port: state.port,
      chains: state.chains,
      startedAt: new Date().toISOString()
    }, null, 2)
  );
}

function clearState() {
  const stateFile = path.join(STATE_DIR, 'server.json');
  if (fs.existsSync(stateFile)) {
    fs.unlinkSync(stateFile);
  }
}

async function execute(options) {
  const spinner = ora();
  const chains = await parseChainOptions(options.chains);
  const basePort = parseInt(options.port, 10);
  const fastMode = options.fastMode === true;
  const blockInterval = fastMode ? 1000 : 5000;

  console.log(chalk.bold.cyan('\n  CSV Local Dev Environment'));
  console.log(chalk.gray('  ' + '='.repeat(50)));

  // Initialize wallet manager
  spinner.start('Generating dev wallets...');
  const walletManager = new WalletManager();
  await walletManager.initialize();
  spinner.succeed(chalk.green('Dev wallets generated'));

  // Initialize cross-chain registry
  const registry = new CrossChainRegistry();

  // Initialize simulators
  const simulators = {};

  if (chains.includes('bitcoin')) {
    spinner.start('Starting Bitcoin simulator (regtest)...');
    simulators.bitcoin = new BitcoinSimulator({
      blockInterval,
      walletManager,
      registry
    });
    await simulators.bitcoin.initialize();
    spinner.succeed(chalk.green('Bitcoin simulator started'));
  }

  if (chains.includes('ethereum')) {
    spinner.start('Starting Ethereum simulator (anvil-like)...');
    simulators.ethereum = new EthereumSimulator({
      blockInterval,
      walletManager,
      registry
    });
    await simulators.ethereum.initialize();
    spinner.succeed(chalk.green('Ethereum simulator started'));
  }

  if (chains.includes('sui')) {
    spinner.start('Starting Sui simulator (devnet)...');
    simulators.sui = new SuiSimulator({
      blockInterval,
      walletManager,
      registry
    });
    await simulators.sui.initialize();
    spinner.succeed(chalk.green('Sui simulator started'));
  }

  if (chains.includes('aptos')) {
    spinner.start('Starting Aptos simulator (testnet)...');
    simulators.aptos = new AptosSimulator({
      blockInterval,
      walletManager,
      registry
    });
    await simulators.aptos.initialize();
    spinner.succeed(chalk.green('Aptos simulator started'));
  }

  // Start RPC server
  spinner.start('Starting RPC proxy server...');
  const app = express();
  app.use(cors());
  app.use(express.json({ limit: '50mb' }));

  const rpcProxy = new RpcProxy({ simulators, registry });
  rpcProxy.setupRoutes(app);

  // Health endpoint
  app.get('/health', (req, res) => {
    res.json({
      status: 'healthy',
      chains: Object.keys(simulators).reduce((acc, name) => {
        acc[name] = simulators[name].getStatus();
        return acc;
      }, {}),
      timestamp: new Date().toISOString()
    });
  });

  // Dashboard endpoint
  app.get('/dashboard', (req, res) => {
    res.json(rpcProxy.getDashboardData());
  });

  // Find available port
  const proxyPort = basePort;
  
  // Mount chain-specific RPC endpoints
  if (simulators.bitcoin) {
    app.post('/bitcoin', async (req, res) => {
      try {
        const result = await simulators.bitcoin.handleRpc(req.body);
        res.json(result);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });
  }

  if (simulators.ethereum) {
    app.post('/ethereum', async (req, res) => {
      try {
        const result = await simulators.ethereum.handleRpc(req.body);
        res.json(result);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });
    // Also support JSON-RPC path
    app.post('/eth-rpc', async (req, res) => {
      try {
        const result = await simulators.ethereum.handleRpc(req.body);
        res.json(result);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });
  }

  if (simulators.sui) {
    app.post('/sui', async (req, res) => {
      try {
        const result = await simulators.sui.handleRpc(req.body);
        res.json(result);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });
  }

  if (simulators.aptos) {
    app.post('/aptos', async (req, res) => {
      try {
        const result = await simulators.aptos.handleRpc(req.body);
        res.json(result);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });
  }

  // Registry endpoints
  app.get('/registry/rights', (req, res) => {
    res.json(registry.getAllRights());
  });

  app.get('/registry/rights/:id', (req, res) => {
    const right = registry.getRight(req.params.id);
    if (right) {
      res.json(right);
    } else {
      res.status(404).json({ error: 'Right not found' });
    }
  });

  app.post('/registry/transfer', async (req, res) => {
    try {
      const transfer = await registry.registerTransfer(req.body);
      res.json({ success: true, transfer });
    } catch (error) {
      res.status(400).json({ error: error.message });
    }
  });

  server = app.listen(proxyPort, () => {
    spinner.succeed(chalk.green('RPC proxy server started'));

    // Save state
    saveState({
      port: proxyPort,
      chains: chains
    });

    // Print summary
    console.log(chalk.gray('\n  ' + '-'.repeat(50)));
    console.log(chalk.bold.cyan('\n  Chain Endpoints:'));

    if (simulators.bitcoin) {
      console.log(chalk.gray('    Bitcoin:    ') + chalk.white(`http://localhost:${proxyPort}/bitcoin`));
    }
    if (simulators.ethereum) {
      console.log(chalk.gray('    Ethereum:   ') + chalk.white(`http://localhost:${proxyPort}/ethereum`));
    }
    if (simulators.sui) {
      console.log(chalk.gray('    Sui:        ') + chalk.white(`http://localhost:${proxyPort}/sui`));
    }
    if (simulators.aptos) {
      console.log(chalk.gray('    Aptos:      ') + chalk.white(`http://localhost:${proxyPort}/aptos`));
    }

    console.log(chalk.gray('\n  Management:'));
    console.log(chalk.gray('    Health:     ') + chalk.white(`http://localhost:${proxyPort}/health`));
    console.log(chalk.gray('    Registry:   ') + chalk.white(`http://localhost:${proxyPort}/registry/rights`));
    console.log(chalk.gray('    Dashboard:  ') + chalk.white(`http://localhost:${proxyPort}/dashboard`));

    // Print wallet info
    console.log(chalk.gray('\n  Dev Wallets:'));
    for (const chain of chains) {
      const wallet = walletManager.getWallet(chain);
      if (wallet) {
        console.log(chalk.gray(`    ${chain.padEnd(12)}`) + chalk.white(wallet.address.substring(0, 42)));
      }
    }

    console.log(chalk.gray('\n  ' + '='.repeat(50)));
    console.log(chalk.green('\n  Local environment is ready!'));
    console.log(chalk.gray('  Press Ctrl+C to stop all chains\n'));

    // Show dashboard if enabled
    if (options.dashboard !== false && !options.background) {
      const { startDashboard } = require(path.join(SRC_DIR, 'dashboard'));
      startDashboard({
        simulators,
        walletManager,
        registry,
        port: proxyPort,
        onExit: async () => {
          console.log(chalk.yellow('\n\nShutting down...'));
          for (const [name, sim] of Object.entries(simulators)) {
            await sim.stop();
          }
          if (server) {
            server.close();
          }
          clearState();
          process.exit(0);
        }
      });
    } else if (options.background) {
      console.log(chalk.gray('  Running in background. Use ') + chalk.cyan('csv-local status') + chalk.gray(' to check health.'));
      console.log(chalk.gray('  Use ') + chalk.cyan('csv-local stop') + chalk.gray(' to stop.\n'));
    }
  });

  // Handle graceful shutdown
  process.on('SIGINT', async () => {
    console.log(chalk.yellow('\n\nShutting down all chains...'));
    for (const [name, sim] of Object.entries(simulators)) {
      await sim.stop();
    }
    if (server) {
      server.close();
    }
    clearState();
    process.exit(0);
  });

  process.on('SIGTERM', async () => {
    for (const [name, sim] of Object.entries(simulators)) {
      await sim.stop();
    }
    if (server) {
      server.close();
    }
    clearState();
    process.exit(0);
  });

  // Store server reference for stop command
  global.__csv_local_server__ = server;
  global.__csv_local_simulators__ = simulators;
}

module.exports = { execute };
