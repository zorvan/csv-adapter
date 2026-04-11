const chalk = require('chalk');
const {
  generateMnemonic,
  simulateRightCreation,
  simulateTransfer,
  simulateProofVerification,
  generateNFTMetadata,
  randomHex,
  sleep,
  formatAddress,
  formatRightId
} = require('../utils');

const tutorial = {
  id: 'nft-transfer',
  title: 'Cross-Chain NFT Transfer',
  description: 'Learn how to transfer NFTs across blockchains using CSV Adapter. Discover how metadata, royalties, and provenance are preserved across chains.',
  prerequisites: [
    'Completed Cross-Chain Basics tutorial',
    'Understanding of ERC-721 / NFT standards',
    'Familiarity with metadata formats'
  ],

  steps: [
    // Step 1: NFT Basics
    {
      title: 'Step 1/6: NFT Cross-Chain Architecture',
      description: 'Before transferring an NFT, let us understand how CSV handles NFT metadata and provenance across chains.',
      concept: 'NFTs in CSV carry their metadata as part of the Right data. The metadata is included in the commitment, ensuring it cannot be tampered with during transfer.',

      code: {
        language: 'javascript',
        code: `import { NFTClient } from '@csv-adapter/nft';

// NFT Rights include metadata in the commitment
const nftRight = await nftClient.createRight({
  standard: 'ERC-721',
  contract: '0xYourContract',
  token_id: '1',
  metadata: {
    name: 'CSV Pioneer #001',
    image: 'ipfs://QmCSV...',
    attributes: [...]
  },
  royalty: {
    percentage: 500,     // 5% in basis points
    recipient: '0xArtist'
  }
});`
      },

      proTip: 'The metadata hash is included in the Right commitment. If anyone modifies the metadata during transfer, the proof verification will fail.',

      action: async () => {
        console.log(chalk.gray('\n  Understanding NFT cross-chain architecture...\n'));
        await sleep(500);

        console.log(chalk.dim('  How NFT Transfer Works:'));
        console.log(chalk.gray('  ┌────────────────────────────────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.cyan('Source Chain'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ NFT exists as token'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ Metadata hashed'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ Right created with metadata'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('└─ Token locked (not burned!)'));
        console.log(chalk.gray('  │'));
        console.log(chalk.gray('  │') + '  ' + chalk.yellow('↓ Cross-chain proof ↓'));
        console.log(chalk.gray('  │'));
        console.log(chalk.gray('  │') + '  ' + chalk.cyan('Destination Chain'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ Proof verified'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ Metadata validated'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('├─ Royalty info preserved'));
        console.log(chalk.gray('  │') + '  ' + chalk.dim('└─ NFT minted with same metadata'));
        console.log(chalk.gray('  └────────────────────────────────────────┘'));

        return {
          architecture: 'Lock-and-Prove',
          metadata_handling: 'Hashed in commitment',
          royalty_support: 'Preserved across chains'
        };
      },

      result: 'NFT transfers lock the source token and mint a verified copy on the destination with identical metadata.'
    },

    // Step 2: Create NFT Right
    {
      title: 'Step 2/6: Create NFT Right with Metadata',
      description: 'Let us create a Right that represents an NFT, including all its metadata and royalty information.',
      concept: 'The NFT Right contains the contract address, token ID, metadata hash, and royalty data. All of this is cryptographically committed.',

      code: {
        language: 'javascript',
        code: `const metadata = {
  name: 'CSV Pioneer #001',
  description: 'A commemorative NFT',
  image: 'ipfs://QmCSV...',
  attributes: [
    { trait_type: 'Chain', value: 'Multi-Chain' },
    { trait_type: 'Rarity', value: 'Genesis' }
  ]
};

// Create Right with full NFT data
const nftRight = await client.createNFTRight({
  contract: nftContract,
  token_id: '1',
  metadata: metadata,
  royalty: {
    percentage: 500,  // 5%
    recipient: artistAddress
  }
});

// The commitment includes:
// SHA256(contract + token_id + metadata_hash + royalty)`
      },

      proTip: 'Royalty percentages are stored in basis points (1/100th of a percent). 500 = 5.00%. This standard matches EIP-2981.',

      action: async () => {
        const address = '0x' + randomHex(20);
        const nftMetadata = generateNFTMetadata();

        console.log(chalk.gray('\n  Creating NFT Right with metadata...\n'));
        await sleep(600);

        console.log(chalk.dim('  NFT Metadata:'));
        console.log(chalk.gray('  ┌────────────────────────────────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.yellow('Name:        ') + chalk.white(nftMetadata.name));
        console.log(chalk.gray('  │') + '  ' + chalk.yellow('Image:       ') + chalk.cyan(nftMetadata.image));
        console.log(chalk.gray('  │') + '  ' + chalk.yellow('Royalty:     ') + chalk.green((nftMetadata.royalty_percentage / 100).toFixed(1) + '%'));
        console.log(chalk.gray('  │') + '  ' + chalk.yellow('Attributes:  ') + chalk.white(`${nftMetadata.attributes.length} traits`));
        console.log(chalk.gray('  └────────────────────────────────────────┘'));

        await sleep(400);

        const right = await simulateRightCreation('Ethereum', address, {
          type: 'nft',
          contract: '0x' + randomHex(20),
          token_id: '1',
          metadata_hash: '0x' + randomHex(32),
          royalty: nftMetadata.royalty_percentage
        });

        console.log(chalk.gray('\n  ┌─ NFT Right Created ──────────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Right ID:   ') + chalk.cyan(formatRightId(right.right_id)));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Commitment: ') + chalk.cyan(formatRightId(right.commitment)));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          right_id: formatRightId(right.right_id),
          nft_name: nftMetadata.name,
          royalty_pct: (nftMetadata.royalty_percentage / 100).toFixed(1) + '%',
          metadata_hash: formatRightId(right.data.metadata_hash),
          attributes_count: nftMetadata.attributes.length
        };
      },

      result: 'NFT Right created! The metadata and royalty info are cryptographically committed.'
    },

    // Step 3: Transfer NFT
    {
      title: 'Step 3/6: Transfer NFT Across Chains',
      description: 'Now let us transfer this NFT from Ethereum to Sui. Watch how the metadata travels with the Right.',
      concept: 'The NFT transfer follows the same 6 phases as token transfers, but with additional metadata validation at each step.',

      code: {
        language: 'javascript',
        code: `import { CrossChainNFT } from '@csv-adapter/nft';

const nftTransfer = new CrossChainNFT({
  from: 'ethereum',
  to: 'sui',
  right_id: nftRight.right_id,
  metadata: nftMetadata,
  verify_metadata: true  // Validate on destination
});

const result = await nftTransfer.execute();

// Verify metadata preserved
console.log(result.metadata_preserved);
// => true

console.log(result.royalty_preserved);
// => true`
      },

      proTip: 'Setting verify_metadata: true ensures the destination chain validates the metadata matches the original hash. This prevents metadata tampering.',

      action: async () => {
        const ethAddress = '0x' + randomHex(20);
        const suiAddress = '0x' + randomHex(32);
        const rightId = '0x' + randomHex(32);

        console.log(chalk.gray('\n  Initiating cross-chain NFT transfer...\n'));

        const phases = [
          { name: 'Locking NFT on Ethereum', icon: '🔒' },
          { name: 'Generating NFT inclusion proof', icon: '📜' },
          { name: 'Building state transition', icon: '🔄' },
          { name: 'Verifying metadata integrity', icon: '✓' },
          { name: 'Submitting to Sui', icon: '📤' },
          { name: 'Minting NFT on Sui', icon: '🖼️' }
        ];

        for (let i = 0; i < phases.length; i++) {
          const phase = phases[i];
          console.log(`\n  ${chalk.dim(`[${i + 1}/${phases.length}]`)} ${phase.icon} ${chalk.cyan(phase.name)}...`);
          await sleep(500);
          console.log(`  ${chalk.green('✓')} ${chalk.greenBright(phase.name + ' complete')}`);
        }

        const newRightId = '0x' + randomHex(32);

        console.log(chalk.gray('\n  ┌─ NFT Transfer Complete ──────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('From:     ') + chalk.white('Ethereum'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('To:       ') + chalk.white('Sui'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Metadata: ') + chalk.green('Verified ✓'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Royalty:  ') + chalk.green('Preserved ✓'));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          from_chain: 'Ethereum',
          to_chain: 'Sui',
          metadata_verified: true,
          royalty_preserved: true,
          new_right_id: formatRightId(newRightId)
        };
      },

      result: 'NFT transferred successfully! Metadata and royalties are intact on Sui.'
    },

    // Step 4: Verify Metadata
    {
      title: 'Step 4/6: Verify Metadata Preservation',
      description: 'Let us verify that the NFT metadata on Sui matches exactly what was on Ethereum. This proves the transfer was faithful.',
      concept: 'Metadata verification compares the hash of the destination metadata against the original hash in the Right commitment. If they match, the metadata is identical.',

      code: {
        language: 'javascript',
        code: `import { MetadataVerifier } from '@csv-adapter/nft';

const verifier = new MetadataVerifier('sui');

// Verify the NFT on the destination chain
const verification = await verifier.verify({
  right_id: result.new_right_id,
  expected_metadata_hash: originalMetadataHash,
  check_royalty: true
});

console.log(verification.metadata_match);
// => true

console.log(verification.royalty_match);
// => true

// Prove provenance
const provenance = await verifier.getProvenance(result.new_right_id);
console.log(provenance.chain_history);
// => ['ethereum', 'sui']`
      },

      proTip: 'The provenance chain records every chain the NFT has been on. This creates an unbreakable history of the NFT journey.',

      action: async () => {
        console.log(chalk.gray('\n  Verifying metadata preservation...\n'));

        const verificationSteps = [
          { name: 'Fetching destination NFT metadata', delay: 400 },
          { name: 'Computing metadata hash', delay: 300 },
          { name: 'Comparing with original commitment', delay: 500 },
          { name: 'Verifying royalty information', delay: 300 },
          { name: 'Checking provenance chain', delay: 400 }
        ];

        for (const step of verificationSteps) {
          console.log(`  ${chalk.dim('▸')} ${chalk.white(step.name)}...`);
          await sleep(step.delay);
          console.log(`  ${chalk.green('✓')} ${chalk.greenBright('Passed')}`);
        }

        console.log(chalk.gray('\n  ┌─ Metadata Verification ──────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Metadata hash match: ') + chalk.greenBright('✓'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Royalty match:       ') + chalk.greenBright('✓'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Provenance chain:    ') + chalk.cyan('ETH → SUI'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Attributes intact:   ') + chalk.greenBright('✓ (4/4)'));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          metadata_verified: true,
          royalty_verified: true,
          provenance: 'ETH → SUI',
          attributes_intact: '4/4'
        };
      },

      result: 'All metadata verified! The NFT on Sui is provably identical to the original on Ethereum.'
    },

    // Step 5: Transfer Back
    {
      title: 'Step 5/6: Transfer NFT Back (Round Trip)',
      description: 'Let us send the NFT back to Ethereum, proving the process works bidirectionally.',
      concept: 'The return transfer uses the same mechanism. The Sui chain state now commits to the NFT, and Ethereum verifies the Sui proof.',

      code: {
        language: 'javascript',
        code: `// Transfer NFT back to Ethereum
const returnTransfer = new CrossChainNFT({
  from: 'sui',
  to: 'ethereum',
  right_id: suiNFTRight.right_id,
  metadata: nftMetadata,
  verify_metadata: true
});

const returnResult = await returnTransfer.execute();

// Full provenance after round trip
const provenance = await verifier.getProvenance(returnResult.new_right_id);
console.log(provenance.chain_history);
// => ['ethereum', 'sui', 'ethereum']

// Original metadata preserved through entire journey
console.log(provenance.metadata_hash);
// => "0xoriginal_hash" (same throughout!)`
      },

      proTip: 'Each transfer in the provenance chain adds a new link. The original metadata hash stays the same, proving authenticity across the entire journey.',

      action: async () => {
        const suiAddress = '0x' + randomHex(32);
        const ethAddress = '0x' + randomHex(20);
        const rightId = '0x' + randomHex(32);

        console.log(chalk.gray('\n  Transferring NFT back: Sui → Ethereum\n'));

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

        console.log(chalk.gray('\n  ┌─ NFT Round Trip Complete ────────────┐'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Journey:  ') + chalk.white('ETH → SUI → ETH'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Metadata: ') + chalk.green('Intact throughout'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Royalty:  ') + chalk.green('Preserved'));
        console.log(chalk.gray('  │') + '  ' + chalk.green('Provenance: 3 chains recorded'));
        console.log(chalk.gray('  └─────────────────────────────────────┘'));

        return {
          journey: 'ETH → SUI → ETH',
          metadata_intact: true,
          royalty_preserved: true,
          provenance_length: 3
        };
      },

      result: 'NFT round trip complete! The NFT traveled ETH → SUI → ETH with metadata intact.'
    },

    // Step 6: Completion
    {
      title: 'Step 6/6: Cross-Chain NFT Mastery',
      description: 'You have mastered cross-chain NFT transfers! Let us review what makes CSV unique for NFT movement.',
      concept: 'CSV provides: metadata preservation, royalty enforcement, provenance tracking, and trustless verification. No bridges, no wrapping, no trusted parties.',

      code: {
        language: '',
        code: `// CSV NFT Transfer Summary
//
// What you learned:
// ✓ NFT Right creation with metadata
// ✓ Metadata hashing & commitment
// ✓ Cross-chain NFT transfer
// ✓ Metadata verification on destination
// ✓ Provenance chain tracking
// ✓ Round-trip NFT transfers
//
// Key advantages:
// • No wrapping needed (native NFTs)
// • Metadata cannot be tampered with
// • Royalties enforced across chains
// • Full provenance history
// • Trustless: verified by ZK proofs`
      },

      proTip: 'CSV NFT transfers work for any NFT standard: ERC-721, ERC-1155, Sui NFTs, Aptos Objects, and Ordinals. The proof system abstracts over the specifics.',

      action: async () => {
        console.log(chalk.gray('\n  ╔═══════════════════════════════════════════════════╗'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(10)) + chalk.bold.magenta('🖼️ NFT TRANSFER MASTER! 🖼️') + chalk.gray(' '.repeat(12)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));

        const learnings = [
          '✓ NFT Right creation with metadata',
          '✓ Metadata hashing & commitment',
          '✓ Cross-chain NFT transfer',
          '✓ Metadata verification',
          '✓ Provenance chain tracking',
          '✓ Round-trip NFT transfers'
        ];

        console.log(chalk.gray('  ║') + '  ' + chalk.magenta('Skills Acquired:') + chalk.gray(' '.repeat(30)) + chalk.gray('║'));
        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        for (const learning of learnings) {
          console.log(chalk.gray('  ║') + '  ' + chalk.white(learning) + chalk.gray(' '.repeat(Math.max(0, 32 - learning.replace(/\x1b\[[0-9;]*m/g, '').length))) + chalk.gray('║'));
        }

        console.log(chalk.gray('  ║') + chalk.gray(' '.repeat(49)) + chalk.gray('║'));
        console.log(chalk.gray('  ╚═══════════════════════════════════════════════════╝'));

        return {
          skills_learned: learnings.length,
          tutorial: 'Cross-Chain NFT Transfer',
          ready_for: 'Advanced Proofs tutorial'
        };
      },

      result: 'You are now a cross-chain NFT transfer expert! Metadata, royalties, and provenance - all preserved.'
    }
  ]
};

module.exports = tutorial;
