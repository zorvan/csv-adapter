const chalk = require('chalk');
const {
  generateMnemonic,
  simulateFaucetRequest,
  simulateRightCreation,
  simulateTransfer,
  simulateProofVerification,
  randomHex,
  sleep,
  formatAddress,
  formatRightId
} = require('../utils');

const tutorial = {
  id: 'cross-chain-basics',
  title: 'Cross-Chain Basics',
  description: 'Learn the fundamentals of cross-chain transfers with CSV Adapter. By the end of this tutorial, you will have sent assets from one blockchain to another using zero-knowledge state commitments.',
  prerequisites: [
    'Node.js 18+',
    'Basic understanding of blockchain',
    'Curiosity about zero-knowledge proofs'
  ],

  steps: [
    // Step 1: Generate your wallet
    {
      title: 'Step 1/7: Generate Your Wallet',
      description: 'Every journey begins with an identity. We will generate a cryptographic wallet that will be your passport across all chains.',
      concept: 'CSV uses BIP-39 mnemonics and BIP-32 key derivation. One seed phrase controls identities on every supported chain.',

      code: {
        language: 'javascript',
        code: `import { CSVWallet } from '@csv-adapter/core';

// Generate a new BIP-39 mnemonic (12 words)
const wallet = CSVWallet.generate();
console.log(wallet.mnemonic);
// => "abandon ability able about above absent..."

// Derive addresses for different chains
const ethAddress = wallet.getAddress('ethereum');
const suiAddress = wallet.getAddress('sui');
const aptosAddress = wallet.getAddress('aptos');`
      },

      proTip: 'In production, never log your mnemonic! This is for educational purposes only. Store mnemonics in secure hardware wallets or encrypted vaults.',

      action: async () => {
        const mnemonic = generateMnemonic();
        const ethAddress = '0x' + randomHex(20);
        const suiAddress = '0x' + randomHex(32);
        const aptosAddress = '0x' + randomHex(32);

        console.log(chalk.gray('\n  Generating cryptographic wallet...\n'));
        await sleep(800);

        console.log(chalk.yellow('  Your BIP-39 Mnemonic (12 words):'));
        console.log(chalk.cyanBright('  ┌' + '─'.repeat(54) + '┐'));
        const words = mnemonic.split(' ');
        for (let i = 0; i < 4; i++) {
          const line = words.slice(i * 3, i * 3 + 3)
            .map((w, j) => `${chalk.dim(i * 3 + j + 1 + '.')}. ${chalk.whiteBright(w)}`)
            .join('  ');
          console.log(chalk.yellow('  │') + '  ' + line + '   ' + chalk.yellow('│'));
        }
        console.log(chalk.cyanBright('  └' + '─'.repeat(54) + '┘'));

        await sleep(400);

        console.log(chalk.yellow('\n  Derived Addresses:'));
        console.log(chalk.gray('  ├─ Ethereum: ') + chalk.green(ethAddress));
        console.log(chalk.gray('  ├─ Sui:      ') + chalk.green(suiAddress));
        console.log(chalk.gray('  └─ Aptos:    ') + chalk.green(aptosAddress));

        await sleep(300);
        console.log(chalk.green('\n  ✓ Wallet generated successfully!'));

        return {
          mnemonic_words: 12,
          chains_supported: 4,
          eth_address: formatAddress(ethAddress),
          sui_address: formatAddress(suiAddress),
          aptos_address: formatAddress(aptosAddress)
        };
      },

      result: 'Your wallet is ready! The same mnemonic gives you addresses on every chain.'
    },

    // Step 2: Claim test tokens
    {
      title: 'Step 2/7: Claim Test Tokens',
      description: 'We need funds to work with. Let us request tokens from a testnet faucet - this simulates getting test tokens for development.',
      concept: 'CSV works on testnets first. Faucets provide free tokens so developers can test without spending real money.',

      code: {
        language: 'javascript',
        code: `import { FaucetClient } from '@csv-adapter/testnet';

// Request test tokens
const faucet = new FaucetClient('ethereum-testnet');
const result = await faucet.request(wallet.ethAddress, {
  amount: '10.0',
  token: 'ETH'
});

console.log(result.tx_hash);
// => "0xabc123..."

console.log(result.balance);
// => "10.0 ETH"`
      },

      proTip: 'Always test on testnets before mainnet. CSV Adapter supports Ethereum Sepolia, Sui Testnet, Aptos Testnet, and Bitcoin Signet.',

      action: async () => {
        const address = '0x' + randomHex(20);

        console.log(chalk.gray('\n  Requesting test tokens from faucet...\n'));

        const results = [];
        const chains = [
          { name: 'Ethereum Sepolia', token: 'ETH', amount: '10.0' },
          { name: 'Sui Testnet', token: 'SUI', amount: '50.0' },
          { name: 'Aptos Testnet', token: 'APT', amount: '25.0' }
        ];

        for (const chain of chains) {
          console.log(chalk.dim(`  Requesting ${chain.amount} ${chain.token} on ${chain.name}...`));
          await sleep(600);
          console.log(chalk.green(`  ✓ Received ${chain.amount} ${chain.token}`));
          results.push(chain);
        }

        console.log(chalk.green('\n  ✓ All test tokens received!'));

        return {
          total_chains: chains.length,
          tokens_received: chains.map(c => `${c.amount} ${c.token}`).join(', ')
        };
      },

      result: 'Test tokens received on all chains! You now have funds to create Rights and transfer them.'
    },

    // Step 3: Create your first Right
    {
      title: 'Step 3/7: Create Your First Right',
      description: 'A "Right" is the core primitive in CSV. It represents ownership of something on a chain. Let us create one.',
      concept: 'A Right is a state commitment: a cryptographic promise that "X exists on chain Y with value Z." It is represented by a right_id and a commitment hash.',

      code: {
        language: 'javascript',
        code: `import { CSVClient } from '@csv-adapter/core';

const client = new CSVClient('ethereum');

// Create a Right representing 1 ETH
const right = await client.createRight({
  owner: wallet.ethAddress,
  type: 'native_token',
  value: '1000000000000000000', // 1 ETH in wei
  metadata: { chain: 'ethereum' }
});

console.log(right.right_id);
// => "0x7f8a9b2c..."

console.log(right.commitment);
// => "0x3d4e5f6a..."`
      },

      proTip: 'The right_id is a unique identifier. The commitment is a hash that hides the details while still being verifiable. This is the "ZK" in ZK-powered cross-chain!',

      action: async () => {
        const address = '0x' + randomHex(20);

        console.log(chalk.gray('\n  Creating Right on Ethereum...\n'));

        const right = await simulateRightCreation('Ethereum', address, {
          type: 'native_token',
          value: '1000000000000000000'
        });

        console.log(chalk.gray('  ┌─ Right Details ──────────────────────┐'));
        console.log(chalk.gray('  │') + chalk.yellow(' Right ID:    ') + chalk.cyan(formatRightId(right.right_id)));
        console.log(chalk.gray('  │') + chalk.yellow(' Commitment:  ') + chalk.cyan(formatRightId(right.commitment)));
        console.log(chalk.gray('  │') + chalk.yellow(' State Root:  ') + chalk.cyan(formatRightId(right.state_root)));
        console.log(chalk.gray('  │') + chalk.yellow(' Block:       ') + chalk.white(String(right.block_number)));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        console.log(chalk.green('\n  ✓ Right created!'));

        return {
          right_id: formatRightId(right.right_id),
          commitment: formatRightId(right.commitment),
          block_number: right.block_number,
          owner: formatAddress(right.owner)
        };
      },

      result: 'Your first Right exists! It cryptographically represents your asset on Ethereum.'
    },

    // Step 4: Transfer to another chain
    {
      title: 'Step 4/7: Transfer to Another Chain',
      description: 'Now the magic: we will transfer this Right from Ethereum to Sui. Watch each phase of the cross-chain process.',
      concept: 'CSV transfers work in phases: (1) Create Right, (2) Generate inclusion proof, (3) Build state transition, (4) Verify constraints, (5) Submit to destination, (6) Confirm on destination.',

      code: {
        language: 'javascript',
        code: `import { CrossChainTransfer } from '@csv-adapter/core';

const transfer = new CrossChainTransfer({
  from: 'ethereum',
  to: 'sui',
  right_id: right.right_id,
  from_address: wallet.ethAddress,
  to_address: wallet.suiAddress
});

// Execute the transfer (this handles all phases)
const result = await transfer.execute();

console.log(result.status);
// => "confirmed"

console.log(result.new_right_id);
// => "0xnew_right_id on Sui"`
      },

      proTip: 'Each phase generates cryptographic proofs. The destination chain verifies these proofs before accepting the transfer. No trusted relayers needed!',

      action: async () => {
        const ethAddress = '0x' + randomHex(20);
        const suiAddress = '0x' + randomHex(32);
        const rightId = '0x' + randomHex(32);

        const result = await simulateTransfer({
          fromChain: 'Ethereum',
          toChain: 'Sui',
          rightId,
          fromAddress: ethAddress,
          toAddress: suiAddress,
          onPhase: async (name, current, total) => {
            console.log(`\n  ${chalk.dim(`[${current + 1}/${total}]`)} ${chalk.cyan(name)}...`);
            await sleep(400);
          }
        });

        console.log(chalk.gray('\n  ┌─ Transfer Complete ────────────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('From: ') + chalk.white('Ethereum → Sui'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('TX:   ') + chalk.cyan(formatAddress(result.tx_hash, 16)));
        console.log(chalk.gray('  │') + '  ' + chalk.green('New Right ID:'));
        console.log(chalk.gray('  │') + '  ' + chalk.cyan('  ' + formatRightId(result.final_right_id)));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          from_chain: result.from_chain,
          to_chain: result.to_chain,
          phases_completed: result.phases.length,
          tx_hash: formatAddress(result.tx_hash, 16),
          new_right_id: formatRightId(result.final_right_id)
        };
      },

      result: 'Cross-chain transfer complete! Your Right now exists on Sui, proven by ZK state commitments.'
    },

    // Step 5: Verify the proof
    {
      title: 'Step 5/7: Verify the Proof',
      description: 'Let us peek under the hood and see how the destination chain verifies the cross-chain proof step by step.',
      concept: 'Proof verification checks: (1) Proof structure is valid, (2) Merkle path is correct length, (3) Hash chain computes correctly, (4) Computed root matches chain state, (5) All constraints are satisfied.',

      code: {
        language: 'javascript',
        code: `import { ProofVerifier } from '@csv-adapter/core';

const verifier = new ProofVerifier('sui');

// Verify the cross-chain proof
const verification = await verifier.verify({
  proof: transfer.proof,
  expected_root: transfer.state_root,
  constraints: transfer.constraints
});

console.log(verification.valid);
// => true

console.log(verification.steps_verified);
// => 7

// The proof is valid only if the computed
// Merkle root matches the chain's state root`
      },

      proTip: 'The Merkle proof connects your Right to the chain state. If even one hash in the path is wrong, the computed root will differ and verification fails.',

      action: async () => {
        const rightId = '0x' + randomHex(32);
        const root = '0x' + randomHex(32);

        const proof = {
          leaf: rightId,
          path: Array.from({ length: 8 }, () => randomHex(32)),
          index: 42
        };

        console.log(chalk.gray('\n  Verifying proof step by step...\n'));

        // Show proof components first
        console.log(chalk.dim('  Proof Components:'));
        console.log(chalk.gray('  ├─ Leaf hash:    ') + chalk.cyan(formatRightId(proof.leaf)));
        console.log(chalk.gray('  ├─ Path length:  ') + chalk.cyan(String(proof.path.length)) + chalk.dim(' hashes'));
        console.log(chalk.gray('  └─ Leaf index:   ') + chalk.cyan(String(proof.index)));

        await sleep(500);

        const verification = await simulateProofVerification(proof, root);

        console.log(chalk.gray('\n  ┌─ Verification Steps ─────────────────┐'));
        const stepNames = [
          'Loading proof data',
          'Validating proof structure',
          'Checking Merkle path length',
          'Verifying hash chain',
          'Comparing computed root',
          'Checking constraints',
          'Final verification'
        ];
        for (const stepName of stepNames) {
          console.log(chalk.gray('  │') + '  ' + chalk.green('✓ ') + chalk.white(stepName));
          await sleep(200);
        }
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          valid: true,
          steps_verified: verification.steps_verified,
          proof_path_length: verification.proof_path_length,
          leaf_index: verification.leaf_index
        };
      },

      result: 'Proof verified! All 7 checks passed. This is how CSV ensures trustless cross-chain transfers.'
    },

    // Step 6: Transfer back
    {
      title: 'Step 6/7: Transfer Back (Round Trip)',
      description: 'Let us complete the circle: transfer the Right from Sui back to Ethereum. The same process works in reverse!',
      concept: 'CSV transfers are symmetric. The same proof system works in both directions. Every chain can be a source or destination.',

      code: {
        language: 'javascript',
        code: `import { CrossChainTransfer } from '@csv-adapter/core';

// Transfer back: Sui → Ethereum
const returnTransfer = new CrossChainTransfer({
  from: 'sui',
  to: 'ethereum',
  right_id: result.new_right_id,
  from_address: wallet.suiAddress,
  to_address: wallet.ethAddress
});

const returnResult = await returnTransfer.execute();

console.log(returnResult.status);
// => "confirmed"

// The Right is back on Ethereum!
// Same process, reverse direction.`
      },

      proTip: 'CSV is chain-agnostic. The proof system does not care about direction - it only cares that the source chain state commits to the Right existing.',

      action: async () => {
        const suiAddress = '0x' + randomHex(32);
        const ethAddress = '0x' + randomHex(20);
        const rightId = '0x' + randomHex(32);

        console.log(chalk.gray('\n  Transferring back: Sui → Ethereum\n'));

        const result = await simulateTransfer({
          fromChain: 'Sui',
          toChain: 'Ethereum',
          rightId,
          fromAddress: suiAddress,
          toAddress: ethAddress,
          onPhase: async (name, current, total) => {
            console.log(`\n  ${chalk.dim(`[${current + 1}/${total}]`)} ${chalk.cyan(name)}...`);
            await sleep(300);
          }
        });

        console.log(chalk.gray('\n  ┌─ Round Trip Complete ────────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Path: ') + chalk.white('ETH → SUI → ETH'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Final Right ID:'));
        console.log(chalk.gray('  │') + '  ' + chalk.cyan('  ' + formatRightId(result.final_right_id)));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          round_trip: 'ETH → SUI → ETH',
          phases_completed: result.phases.length,
          final_right_id: formatRightId(result.final_right_id),
          status: 'confirmed'
        };
      },

      result: 'Round trip complete! Your Right traveled Ethereum → Sui → Ethereum, proven every step of the way.'
    },

    // Step 7: Celebrate
    {
      title: 'Step 7/7: Celebrate!',
      description: 'You have completed the Cross-Chain Basics tutorial! Let us review what you have learned.',
      concept: 'You now understand: wallet generation, testnet usage, Right creation, cross-chain transfers, proof verification, and round-trip transfers.',

      code: {
        language: '',
        code: `// You have learned:
//
// 1. Wallet Generation    - One mnemonic, all chains
// 2. Testnet Faucets      - Free tokens for testing
// 3. Right Creation       - Cryptographic state commitments
// 4. Cross-Chain Transfer - 6-phase ZK-powered transfer
// 5. Proof Verification   - Merkle proofs & constraint checks
// 6. Round-Trip Transfer  - Symmetric chain-agnostic proofs
//
// Next steps:
// → Try the NFT Transfer tutorial
// → Dive into Advanced Proofs
// → Build your own cross-chain app!`
      },

      proTip: 'The real power of CSV is composability. You can chain multiple transfers together to move assets across any number of chains.',

      action: async () => {
        console.log(chalk.gray('\n  ╔═══════════════════════════════════════════════════╗'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(12)) + chalk.bold.yellow('🎉 TUTORIAL COMPLETE! 🎉') + chalk.gray(' '.repeat(12)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));

        const learnings = [
          '✓ Wallet generation (BIP-39/BIP-32)',
          '✓ Testnet faucet usage',
          '✓ Right creation & state commitments',
          '✓ Cross-chain transfer (6 phases)',
          '✓ Proof verification (Merkle proofs)',
          '✓ Round-trip transfers'
        ];

        console.log(chalk.gray('  ║') + '  ' + chalk.green('Skills Acquired:') + chalk.gray(' '.repeat(28)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        for (const learning of learnings) {
          console.log(chalk.gray('  ║') + '  ' + chalk.white(learning) + chalk.gray(' '.repeat(Math.max(0, 32 - learning.replace(/\x1b\[[0-9;]*m/g, '').length))) + chalk.gray('║'));
        }

        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ╚═══════════════════════════════════════════════════╝'));

        return {
          skills_learned: learnings.length,
          tutorial: 'Cross-Chain Basics',
          ready_for: 'NFT Transfer & Advanced Proofs'
        };
      },

      result: 'You are now a CSV cross-chain developer! Ready for the next tutorial?'
    }
  ]
};

module.exports = tutorial;
