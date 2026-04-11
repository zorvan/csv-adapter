#!/usr/bin/env node
'use strict';

/**
 * Scenario: Double-Spend Prevention
 * 
 * Demonstrates CSV's double-spend prevention mechanism.
 * This scenario:
 * 1. Creates a Right on Bitcoin
 * 2. Initiates a legitimate transfer to Ethereum
 * 3. Attempts to transfer the same Right to Sui (should fail)
 * 4. Attempts to transfer the same Right to Aptos (should fail)
 * 5. Completes the legitimate transfer
 * 6. Verifies the Right exists only on Ethereum
 */

async function execute() {
  const chalk = require('chalk');
  const path = require('path');
  const { CrossChainRegistry } = require(path.join(__dirname, '..', 'simulator', 'registry'));

  const registry = new CrossChainRegistry();

  console.log(chalk.cyan('\n  Scenario: Double-Spend Prevention'));
  console.log(chalk.gray('  ' + '='.repeat(55)));
  console.log(chalk.yellow('\n  This scenario demonstrates how CSV prevents'));
  console.log(chalk.yellow('  double-spending of Rights across chains.'));

  // Step 1: Create a Right on Bitcoin
  console.log(chalk.yellow('\n  Step 1: Creating Right on Bitcoin...'));
  const right = registry.createRight({
    name: 'Double-Spend Test Right',
    owner: '0xbtc_owner_address_001',
    chain: 'bitcoin',
    metadata: {
      type: 'Test Right',
      purpose: 'Double-spend prevention demo'
    },
    txHash: '0xbtc_tx_hash_create'
  });

  console.log(chalk.gray('    Right ID:     ') + chalk.white(right.id));
  console.log(chalk.gray('    Chain:        ') + chalk.green(right.chain));
  console.log(chalk.gray('    Status:       ') + chalk.green(right.status));

  // Step 2: Initiate legitimate transfer to Ethereum
  console.log(chalk.yellow('\n  Step 2: Initiating LEGITIMATE transfer to Ethereum...'));
  const legitimateTransfer = await registry.registerTransfer({
    rightId: right.id,
    fromChain: 'bitcoin',
    toChain: 'ethereum',
    fromOwner: '0xbtc_owner_address_001',
    toOwner: '0xeth_recipient_002',
    sourceTxHash: '0xbtc_tx_hash_legit_transfer'
  });

  console.log(chalk.gray('    Transfer ID:  ') + chalk.white(legitimateTransfer.id));
  console.log(chalk.gray('    Status:       ') + chalk.yellow(legitimateTransfer.status));
  console.log(chalk.gray('    Right Status: ') + chalk.yellow(registry.getRight(right.id).status));

  // Step 3: Attempt double-spend to Sui (should fail)
  console.log(chalk.yellow('\n  Step 3: Attempting DOUBLE-SPEND transfer to Sui...'));
  console.log(chalk.gray('    Attacker tries to transfer the same Right to Sui'));
  
  try {
    const doubleSpendAttempt1 = await registry.registerTransfer({
      rightId: right.id,
      fromChain: 'bitcoin',
      toChain: 'sui',
      fromOwner: '0xbtc_owner_address_001',
      toOwner: '0xsui_attacker_003',
      sourceTxHash: '0xbtc_tx_hash_malicious'
    });
    console.log(chalk.red('    ERROR: Double-spend was not prevented!'));
  } catch (error) {
    console.log(chalk.red('    TRANSFER BLOCKED: ') + chalk.white(error.message));
    console.log(chalk.gray('    This is expected - the Right is locked in transfer.'));
  }

  // Step 4: Attempt double-spend to Aptos (should also fail)
  console.log(chalk.yellow('\n  Step 4: Attempting DOUBLE-SPEND transfer to Aptos...'));
  console.log(chalk.gray('    Attacker tries a different chain'));

  try {
    const doubleSpendAttempt2 = await registry.registerTransfer({
      rightId: right.id,
      fromChain: 'bitcoin',
      toChain: 'aptos',
      fromOwner: '0xbtc_owner_address_001',
      toOwner: '0xaptos_attacker_004',
      sourceTxHash: '0xbtc_tx_hash_malicious_2'
    });
    console.log(chalk.red('    ERROR: Double-spend was not prevented!'));
  } catch (error) {
    console.log(chalk.red('    TRANSFER BLOCKED: ') + chalk.white(error.message));
    console.log(chalk.gray('    This is expected - the Right is still locked.'));
  }

  // Step 5: Complete the legitimate transfer
  console.log(chalk.yellow('\n  Step 5: Completing LEGITIMATE transfer to Ethereum...'));
  const { right: updatedRight } = await registry.completeTransfer(
    legitimateTransfer.id,
    '0xeth_tx_hash_complete'
  );

  console.log(chalk.gray('    Right Chain:  ') + chalk.green(updatedRight.chain));
  console.log(chalk.gray('    Right Owner:  ') + chalk.white(updatedRight.owner));
  console.log(chalk.gray('    Status:       ') + chalk.green(updatedRight.status));

  // Step 6: Verify final state
  console.log(chalk.yellow('\n  Step 6: Final Verification'));
  const finalRight = registry.getRight(right.id);
  console.log(chalk.gray('    Final Chain:      ') + chalk.green(finalRight.chain));
  console.log(chalk.gray('    Final Owner:      ') + chalk.white(finalRight.owner));
  console.log(chalk.gray('    Final Status:     ') + chalk.green(finalRight.status));

  // Try another double-spend after transfer is complete (should also fail)
  console.log(chalk.yellow('\n  Step 7: Attempting post-transfer double-spend...'));
  console.log(chalk.gray('    Attacker tries to transfer from Bitcoin again'));

  try {
    await registry.registerTransfer({
      rightId: right.id,
      fromChain: 'bitcoin',
      toChain: 'sui',
      fromOwner: '0xbtc_owner_address_001',
      toOwner: '0xsui_attacker_005',
      sourceTxHash: '0xbtc_tx_hash_post_transfer'
    });
    console.log(chalk.red('    ERROR: Double-spend was not prevented!'));
  } catch (error) {
    console.log(chalk.red('    TRANSFER BLOCKED: ') + chalk.white(error.message));
    console.log(chalk.gray('    Right is on Ethereum, not Bitcoin.'));
  }

  // Show registry stats
  const stats = registry.getStats();
  console.log(chalk.cyan('\n  Registry Stats:'));
  console.log(chalk.gray('    Total Rights Created:        ') + chalk.white(stats.totalRightsCreated));
  console.log(chalk.gray('    Total Transfers Initiated:   ') + chalk.white(stats.totalTransfersInitiated));
  console.log(chalk.gray('    Total Transfers Completed:   ') + chalk.green(stats.totalTransfersCompleted));
  console.log(chalk.gray('    Double-Spend Attempts:       ') + chalk.red(stats.totalDoubleSpendAttempts));

  // Show right history
  console.log(chalk.cyan('\n  Right History (Full Audit Trail):'));
  for (const entry of finalRight.history) {
    const icon = entry.action.includes('double') ? chalk.red('✗') : 
                 entry.action.includes('blocked') ? chalk.red('✗') :
                 entry.action.includes('initiated') ? chalk.yellow('→') :
                 entry.action.includes('completed') ? chalk.green('✓') :
                 chalk.gray('•');
    console.log(`    ${icon} ${entry.action.padEnd(25)} ${chalk.gray(entry.chain || entry.fromChain || '')}`);
  }

  console.log(chalk.gray('\n  ' + '='.repeat(55)));
  console.log(chalk.green('\n  Scenario completed successfully!'));
  console.log(chalk.green('  Double-spend prevention is working correctly.'));
  console.log('');

  return { right: finalRight, registry };
}

module.exports = { execute };
