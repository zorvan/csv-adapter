const crypto = require('crypto');

/**
 * Format a right_id for display (truncate with ellipsis)
 * @param {string} id - The full right_id (hex string)
 * @param {number} [chars=12] - Number of characters to show on each side
 * @returns {string}
 */
function formatRightId(id, chars = 12) {
  if (!id || id.length <= chars * 2 + 3) return id || '';
  return `${id.slice(0, chars)}...${id.slice(-chars)}`;
}

/**
 * Format a blockchain address for display
 * @param {string} addr - The full address
 * @param {number} [chars=10] - Characters on each side
 * @returns {string}
 */
function formatAddress(addr, chars = 10) {
  if (!addr || addr.length <= chars * 2 + 3) return addr || '';
  return `${addr.slice(0, chars)}...${addr.slice(-chars)}`;
}

/**
 * Sleep for a given number of milliseconds
 * @param {number} ms - Milliseconds to sleep
 * @returns {Promise<void>}
 */
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Generate a random hex string of given length
 * @param {number} bytes - Number of random bytes
 * @returns {string}
 */
function randomHex(bytes = 32) {
  return crypto.randomBytes(bytes).toString('hex');
}

/**
 * Generate a simulated BIP-39 mnemonic (for tutorial purposes only)
 * @returns {string}
 */
function generateMnemonic() {
  const wordList = [
    'abandon', 'ability', 'able', 'about', 'above', 'absent', 'absorb', 'abstract',
    'absurd', 'abuse', 'access', 'accident', 'account', 'accuse', 'achieve', 'acid',
    'acoustic', 'acquire', 'across', 'act', 'action', 'actor', 'actress', 'actual',
    'adapt', 'add', 'addict', 'address', 'adjust', 'admit', 'adult', 'advance',
    'advice', 'aerobic', 'affair', 'afford', 'afraid', 'again', 'age', 'agent',
    'agree', 'ahead', 'aim', 'air', 'airport', 'aisle', 'alarm', 'album',
    'alcohol', 'alert', 'alien', 'all', 'alley', 'allow', 'almost', 'alone',
    'alpha', 'already', 'also', 'alter', 'always', 'amateur', 'amazing', 'among'
  ];

  const words = [];
  for (let i = 0; i < 12; i++) {
    words.push(wordList[Math.floor(Math.random() * wordList.length)]);
  }
  return words.join(' ');
}

/**
 * Simulate a cross-chain transfer with realistic phases and delays
 * @param {object} params - Transfer parameters
 * @param {string} params.fromChain - Source chain name
 * @param {string} params.toChain - Destination chain name
 * @param {string} params.rightId - The right_id being transferred
 * @param {string} params.fromAddress - Source address
 * @param {string} params.toAddress - Destination address
 * @param {object} [options] - Display options
 * @param {function} [options.onPhase] - Callback for each phase
 * @returns {Promise<object>} Transfer result
 */
async function simulateTransfer({
  fromChain,
  toChain,
  rightId,
  fromAddress,
  toAddress,
  onPhase
}) {
  const phases = [
    {
      name: 'Creating Right on source chain',
      action: async () => {
        await sleep(800);
        return { status: 'created', chain: fromChain };
      }
    },
    {
      name: 'Generating inclusion proof',
      action: async () => {
        await sleep(1200);
        const proof = {
          root: randomHex(32),
          leaf: rightId,
          path: Array.from({ length: 8 }, () => randomHex(32)),
          index: Math.floor(Math.random() * 1024)
        };
        return { status: 'proof_generated', proof };
      }
    },
    {
      name: 'Building state transition',
      action: async () => {
        await sleep(600);
        return {
          status: 'transition_built',
          from: fromAddress,
          to: toAddress,
          right_id: rightId
        };
      }
    },
    {
      name: 'Verifying proof constraints',
      action: async () => {
        await sleep(1000);
        return {
          status: 'verified',
          constraints_met: true,
          verification_hash: randomHex(16)
        };
      }
    },
    {
      name: `Submitting to ${toChain}`,
      action: async () => {
        await sleep(1500);
        const txHash = '0x' + randomHex(32);
        return {
          status: 'submitted',
          tx_hash: txHash,
          chain: toChain
        };
      }
    },
    {
      name: 'Confirming on destination',
      action: async () => {
        await sleep(800);
        return {
          status: 'confirmed',
          confirmations: 1,
          new_right_id: '0x' + randomHex(32)
        };
      }
    }
  ];

  const results = [];
  for (const phase of phases) {
    if (onPhase) {
      await onPhase(phase.name, phases.indexOf(phase), phases.length);
    }
    const result = await phase.action();
    results.push({ phase: phase.name, ...result });
  }

  return {
    success: true,
    from_chain: fromChain,
    to_chain: toChain,
    right_id: rightId,
    from_address: fromAddress,
    to_address: toAddress,
    phases: results,
    final_right_id: results[results.length - 1].new_right_id,
    tx_hash: results[results.length - 2].tx_hash
  };
}

/**
 * Simulate proof verification (used in tutorial step 5)
 * @param {object} proof - The proof object
 * @param {string} root - Expected Merkle root
 * @returns {Promise<object>}
 */
async function simulateProofVerification(proof, root) {
  const steps = [
    { name: 'Loading proof data', delay: 400 },
    { name: 'Validating proof structure', delay: 300 },
    { name: 'Checking Merkle path length', delay: 200 },
    { name: 'Verifying hash chain', delay: 600 },
    { name: 'Comparing computed root', delay: 400 },
    { name: 'Checking constraint satisfaction', delay: 500 },
    { name: 'Final verification', delay: 300 }
  ];

  for (const step of steps) {
    await sleep(step.delay);
  }

  // Compute what the root would be from this proof
  const computedRoot = crypto
    .createHash('sha256')
    .update(proof.leaf + proof.path.join(''))
    .digest('hex');

  return {
    valid: computedRoot === root,
    computed_root: computedRoot,
    expected_root: root,
    steps_verified: steps.length,
    leaf_hash: proof.leaf,
    proof_path_length: proof.path.length,
    leaf_index: proof.index
  };
}

/**
 * Simulate generating a fraudulent proof for the advanced tutorial
 * @returns {Promise<object>}
 */
async function simulateFraudulentProof() {
  await sleep(500);

  const fraudulentProof = {
    leaf: '0x' + randomHex(32),
    path: Array.from({ length: 8 }, () => randomHex(32)),
    index: Math.floor(Math.random() * 1024)
  };

  await sleep(400);
  const computedRoot = crypto
    .createHash('sha256')
    .update(fraudulentProof.leaf + fraudulentProof.path.join(''))
    .digest('hex');

  const legitimateRoot = '0x' + randomHex(32); // Different root

  await sleep(600);

  return {
    proof: fraudulentProof,
    computed_root: computedRoot,
    expected_root: legitimateRoot,
    valid: false,
    failure_reason: 'Root mismatch: computed proof does not match chain state',
    failure_step: 'Comparing computed root'
  };
}

/**
 * Generate data for the completion certificate
 * @param {string} tutorialName
 * @param {number} totalSteps
 * @param {number} startTime - Timestamp when tutorial started
 * @returns {object}
 */
function generateCertificateData(tutorialName, totalSteps, startTime) {
  const elapsed = Date.now() - startTime;
  const minutes = Math.floor(elapsed / 60000);
  const seconds = Math.floor((elapsed % 60000) / 1000);

  const badges = [];
  if (minutes < 5) badges.push('Speed Demon');
  if (minutes < 10) badges.push('Fast Learner');
  badges.push('CSV Explorer');
  if (totalSteps >= 7) badges.push('Cross-Chain Certified');

  const displayName = tutorialName
    .split('-')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');

  return {
    tutorialName: displayName,
    completionDate: new Date().toISOString().split('T')[0],
    completionTime: `${minutes}m ${seconds}s`,
    stepsCompleted: totalSteps,
    totalTimeMs: elapsed,
    badges,
    certificateId: 'CSV-' + randomHex(4).toUpperCase()
  };
}

/**
 * Generate a simulated NFT metadata object
 * @returns {object}
 */
function generateNFTMetadata() {
  return {
    name: 'CSV Pioneer #001',
    description: 'A commemorative NFT for completing the CSV cross-chain tutorial',
    image: 'ipfs://QmCSV' + randomHex(10),
    attributes: [
      { trait_type: 'Chain', value: 'Multi-Chain' },
      { trait_type: 'Rarity', value: 'Tutorial Exclusive' },
      { trait_type: 'Tutorial', value: 'NFT Transfer' },
      { trait_type: 'Edition', value: 'Genesis' }
    ],
    royalty_percentage: 500, // 5% in basis points
    royalty_recipient: '0x' + randomHex(20)
  };
}

/**
 * Simulate faucet request
 * @param {string} chain
 * @param {string} address
 * @returns {Promise<object>}
 */
async function simulateFaucetRequest(chain, address) {
  await sleep(1500);
  return {
    success: true,
    chain,
    address,
    amount: '10.0',
    token: chain === 'Ethereum' ? 'ETH' : chain === 'Sui' ? 'SUI' : chain === 'Aptos' ? 'APT' : 'BTC',
    tx_hash: '0x' + randomHex(32),
    faucet_url: `https://faucet.${chain.toLowerCase()}.testnet/csv-adapter`
  };
}

/**
 * Simulate Right creation
 * @param {string} chain
 * @param {string} address
 * @param {object} [data] - Optional data for the Right
 * @returns {Promise<object>}
 */
async function simulateRightCreation(chain, address, data = null) {
  await sleep(1000);
  const rightId = '0x' + randomHex(32);
  const commitment = crypto.createHash('sha256').update(rightId + chain).digest('hex');

  return {
    success: true,
    chain,
    owner: address,
    right_id: rightId,
    commitment: '0x' + commitment,
    state_root: '0x' + randomHex(32),
    data: data || { type: 'transfer', value: '1000000000000000000' },
    block_number: Math.floor(Math.random() * 1000000) + 10000000,
    timestamp: Date.now()
  };
}

module.exports = {
  formatRightId,
  formatAddress,
  sleep,
  randomHex,
  generateMnemonic,
  simulateTransfer,
  simulateProofVerification,
  simulateFraudulentProof,
  generateCertificateData,
  generateNFTMetadata,
  simulateFaucetRequest,
  simulateRightCreation
};
