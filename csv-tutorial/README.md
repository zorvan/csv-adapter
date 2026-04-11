# CSV Adapter Interactive Tutorial

A guided, hands-on CLI tutorial that teaches you how to build cross-chain applications with CSV Adapter. Step by step, you will learn everything from wallet generation to advanced zero-knowledge proofs.

## Quick Start

```bash
# Install and run in one command
npx csv-tutorial

# Or install globally
npm install -g csv-tutorial
csv-tutorial
```

## Available Tutorials

### 1. Cross-Chain Basics (7 steps)

The foundational tutorial. Learn the core concepts of CSV cross-chain transfers.

```bash
npx csv-tutorial cross-chain-basics
```

**What you will learn:**
- BIP-39 wallet generation and key derivation
- Testnet faucet usage
- Creating Rights (state commitments)
- Cross-chain transfer phases
- Merkle proof verification
- Round-trip transfers

### 2. Cross-Chain NFT Transfer (6 steps)

Learn how to move NFTs across blockchains while preserving metadata, royalties, and provenance.

```bash
npx csv-tutorial nft-transfer
```

**What you will learn:**
- NFT Right creation with metadata
- Metadata hashing and commitment
- Cross-chain NFT transfer mechanics
- Metadata verification on destination
- Provenance chain tracking
- Bidirectional NFT transfers

### 3. Advanced Proofs Deep Dive (7 steps)

Go deep into the cryptographic proofs that power CSV. Understand Merkle trees, constraint systems, and security guarantees.

```bash
npx csv-tutorial advanced-proofs
```

**What you will learn:**
- Merkle tree structure and properties
- Proof generation and verification
- Constraint systems (ownership, double-spend, value)
- Proof composition for atomic operations
- Fraud detection and security analysis

## Commands

```bash
# Run default tutorial (cross-chain-basics)
npx csv-tutorial

# Run specific tutorial
npx csv-tutorial nft-transfer
npx csv-tutorial advanced-proofs

# List available tutorials
npx csv-tutorial --list

# Restart a tutorial (ignore saved progress)
npx csv-tutorial --restart

# Run without waiting for user input
npx csv-tutorial --non-interactive

# Run from package scripts
npm run tutorial:cross-chain
npm run tutorial:nft
npm run tutorial:advanced
```

## Features

- **Interactive**: Press Enter to progress through each step at your own pace
- **Visual**: ASCII art, progress bars, and formatted code blocks
- **Simulated**: Works without real chain connections (uses realistic simulations)
- **Progress Saving**: Resume where you left off (progress saved for 24 hours)
- **Certificates**: Earn a completion certificate with badges and stats

## Tutorial Structure

Each tutorial step follows a consistent pattern:

1. **Title** - What step you are on
2. **Description** - What you will learn
3. **Concept** - The CSV concept being taught
4. **Code Example** - Copy-pasteable code snippet
5. **Pro Tip** - Extra context or best practice
6. **Action** - Simulated execution with realistic delays
7. **Result** - What was accomplished

## Requirements

- Node.js 18 or later
- A terminal with at least 80 columns width
- About 5-15 minutes per tutorial

## Progress Saving

Tutorial progress is automatically saved to `~/.csv-tutorial/progress/`. If you exit mid-tutorial, your progress will be remembered for 24 hours. Use `--restart` to start fresh.

## Certificates

Upon completing a tutorial, you receive a completion certificate with:
- Tutorial name and completion date
- Time taken
- Steps completed
- Achievement badges
- Unique certificate ID

Certificates are saved to `~/.csv-tutorial/certificate-<ID>.txt`.

## Project Structure

```
csv-tutorial/
├── package.json                    # Package configuration
├── README.md                       # This file
├── src/
│   ├── index.js                    # CLI entry point
│   ├── engine.js                   # TutorialRunner class
│   ├── renderer.js                 # Terminal rendering utilities
│   ├── utils.js                    # Shared utilities
│   ├── certificate.js              # Certificate generation
│   └── tutorials/
│       ├── cross-chain-basics.js   # 7-step foundational tutorial
│       ├── nft-transfer.js         # 6-step NFT tutorial
│       └── advanced-proofs.js      # 7-step advanced proofs tutorial
```

## Development

```bash
# Clone and install
cd csv-tutorial
npm install

# Run from source
node src/index.js

# Run specific tutorial
node src/index.js nft-transfer

# List tutorials
node src/index.js --list
```

## Adding New Tutorials

Create a new file in `src/tutorials/` following this structure:

```javascript
module.exports = {
  id: 'my-tutorial',
  title: 'My Tutorial',
  description: 'What this tutorial teaches',
  prerequisites: ['Prerequisite 1', 'Prerequisite 2'],
  steps: [
    {
      title: 'Step 1/N: Step Title',
      description: 'What this step teaches',
      concept: 'The core concept',
      code: {
        language: 'javascript',
        code: `// Code example`
      },
      proTip: 'Extra tip for this step',
      action: async () => {
        // Simulated action
        return { result: 'data' };
      },
      result: 'What was accomplished'
    }
    // ... more steps
  ]
};
```

Then register it in `src/index.js`:

```javascript
const tutorials = {
  // ... existing tutorials
  'my-tutorial': require('./tutorials/my-tutorial')
};
```

## License

MIT
