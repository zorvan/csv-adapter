#!/usr/bin/env node
'use strict';

/**
 * Scenario: Basic Cross-Chain Transfer
 * 
 * Demonstrates transferring a Right from Sui to Ethereum.
 * This scenario:
 * 1. Creates a Right on Sui
 * 2. Initiates cross-chain transfer to Ethereum
 * 3. Completes the transfer
 * 4. Verifies the Right now exists on Ethereum
 */

async function execute() {
  const chalk = require('chalk');
  const path = require('path');
  const { CrossChainRegistry } = require(path.join(__dirname, '..', 'simulator', 'registry'));

  const registry = new CrossChainRegistry();

  console.log(chalk.cyan('\n  Scenario: Transfer Right from Sui to Ethereum'));
  console.log(chalk.gray('  ' + '='.repeat(55)));

  // Step 1: Create a Right on Sui
  console.log(chalk.yellow('\n  Step 1: Creating Right on Sui...'));
  const right = registry.createRight({
    name: 'Digital Art #001',
    owner: '0xsui_owner_address_001',
    chain: 'sui',
    metadata: {
      type: 'NFT',
      collection: 'CSV Demo Collection',
      creator: 'demo@csv.local'
    },
    txHash: '0xsui_tx_hash_create'
  });

  console.log(chalk.gray('    Right ID:     ') + chalk.white(right.id));
  console.log(chalk.gray('    Name:         ') + chalk.white(right.name));
  console.log(chalk.gray('    Owner:        ') + chalk.white(right.owner));
  console.log(chalk.gray('    Chain:        ') + chalk.green(right.chain));
  console.log(chalk.gray('    Status:       ') + chalk.green(right.status));

  // Step 2: Initiate transfer to Ethereum
  console.log(chalk.yellow('\n  Step 2: Initiating cross-chain transfer...'));
  const transfer = await registry.registerTransfer({
    rightId: right.id,
    fromChain: 'sui',
    toChain: 'ethereum',
    fromOwner: '0xsui_owner_address_001',
    toOwner: '0xeth_new_owner_address_002',
    sourceTxHash: '0xsui_tx_hash_transfer_init'
  });

  console.log(chalk.gray('    Transfer ID:  ') + chalk.white(transfer.id));
  console.log(chalk.gray('    From:         ') + chalk.green('Sui'));
  console.log(chalk.gray('    To:           ') + chalk.blue('Ethereum'));
  console.log(chalk.gray('    From Owner:   ') + chalk.white(transfer.fromOwner));
  console.log(chalk.gray('    To Owner:     ') + chalk.white(transfer.toOwner));
  console.log(chalk.gray('    Status:       ') + chalk.yellow(transfer.status));

  // Step 3: Complete the transfer
  console.log(chalk.yellow('\n  Step 3: Completing transfer on Ethereum...'));
  const { right: updatedRight } = await registry.completeTransfer(
    transfer.id,
    '0xeth_tx_hash_transfer_complete'
  );

  console.log(chalk.gray('    Right Chain:  ') + chalk.green(updatedRight.chain));
  console.log(chalk.gray('    Right Owner:  ') + chalk.white(updatedRight.owner));
  console.log(chalk.gray('    Status:       ') + chalk.green(updatedRight.status));

  // Step 4: Verify the Right has moved
  console.log(chalk.yellow('\n  Step 4: Verification'));
  const verifiedRight = registry.getRight(right.id);
  console.log(chalk.gray('    Chain changed:    ') + chalk.green(verifiedRight.chain === 'ethereum' ? 'YES' : 'NO'));
  console.log(chalk.gray('    Owner changed:    ') + chalk.green(verifiedRight.owner === '0xeth_new_owner_address_002' ? 'YES' : 'NO'));
  console.log(chalk.gray('    Status active:    ') + chalk.green(verifiedRight.status === 'active' ? 'YES' : 'NO'));

  // Show history
  console.log(chalk.cyan('\n  Right History:'));
  for (const entry of verifiedRight.history) {
    console.log(chalk.gray(`    [${new Date(entry.timestamp).toISOString()}] ${entry.action}`));
  }

  // Show registry stats
  const stats = registry.getStats();
  console.log(chalk.cyan('\n  Registry Stats:'));
  console.log(chalk.gray('    Total Rights Created:     ') + chalk.white(stats.totalRightsCreated));
  console.log(chalk.gray('    Total Transfers Initiated: ') + chalk.white(stats.totalTransfersInitiated));
  console.log(chalk.gray('    Total Transfers Completed: ') + chalk.green(stats.totalTransfersCompleted));

  console.log(chalk.gray('\n  ' + '='.repeat(55)));
  console.log(chalk.green('\n  Scenario completed successfully!'));
  console.log('');

  return { right: verifiedRight, transfer, registry };
}

module.exports = { execute };
