#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const { EventEmitter } = require('events');

/**
 * Simulated Bitcoin Chain (regtest-like)
 * 
 * Features:
 * - Block generation at configurable intervals
 * - UTXO set management
 * - Transaction validation
 * - Merkle proof generation
 * - JSON-RPC compatible endpoints
 */
class BitcoinSimulator extends EventEmitter {
  constructor(options = {}) {
    super();
    this.blockInterval = options.blockInterval || 5000;
    this.walletManager = options.walletManager;
    this.registry = options.registry;
    
    // Chain state
    this.blocks = [];
    this.transactions = [];
    this.utxoSet = new Map(); // txid:vout -> { value, script, address }
    this.mempool = [];
    this.chainId = 'regtest';
    this.running = false;
    this.blockTimer = null;
    this.stats = {
      totalTransactions: 0,
      totalBlocks: 0,
      startTime: Date.now()
    };

    // Genesis block
    this.genesisBlock = this.createGenesisBlock();
  }

  async initialize() {
    this.blocks.push(this.genesisBlock);
    
    // Pre-fund dev wallet
    if (this.walletManager) {
      const wallet = this.walletManager.getWallet('bitcoin');
      if (wallet) {
        // Create a coinbase UTXO for the dev wallet
        const genesisUtxo = {
          txid: this.genesisBlock.transactions[0].txid,
          vout: 0,
          value: 50 * 1e8, // 50 BTC in satoshis
          script: `OP_DUP OP_HASH160 ${wallet.address} OP_EQUALVERIFY OP_CHECKSIG`,
          address: wallet.address,
          coinbase: true,
          confirmations: 1
        };
        this.utxoSet.set(`${genesisUtxo.txid}:0`, genesisUtxo);
        wallet.balance = '50.00000000 BTC';
        wallet.rawBalance = 50 * 1e8;
      }
    }

    // Start block generation
    this.running = true;
    this.blockTimer = setInterval(() => this.generateBlock(), this.blockInterval);
  }

  async stop() {
    this.running = false;
    if (this.blockTimer) {
      clearInterval(this.blockTimer);
      this.blockTimer = null;
    }
    this.emit('stopped');
  }

  createGenesisBlock() {
    const genesisTx = {
      txid: crypto.createHash('sha256').update('genesis').digest('hex'),
      version: 1,
      inputs: [{
        coinbase: Buffer.from('CSV Local Dev Genesis').toString('hex'),
        sequence: 0xffffffff
      }],
      outputs: [{
        value: 50 * 1e8,
        scriptPubKey: 'GENESIS'
      }],
      size: 200,
      weight: 800
    };

    return {
      hash: crypto.createHash('sha256').update('genesis-block').digest('hex'),
      prevHash: '0000000000000000000000000000000000000000000000000000000000000000',
      height: 0,
      time: Math.floor(Date.now() / 1000),
      nonce: 0,
      bits: '207fffff',
      difficulty: 1,
      transactions: [genesisTx],
      merkleRoot: genesisTx.txid
    };
  }

  generateBlock() {
    if (!this.running) return;

    const prevBlock = this.blocks[this.blocks.length - 1];
    const blockTransactions = [...this.mempool.splice(0, 10)]; // Max 10 txs per block
    
    // Add coinbase transaction if there are transactions
    const coinbaseTx = {
      txid: crypto.randomBytes(32).toString('hex'),
      version: 1,
      inputs: [{ coinbase: Buffer.from(`block-${prevBlock.height + 1}`).toString('hex') }],
      outputs: [{ value: 6.25 * 1e8, scriptPubKey: 'COINBASE' }],
      size: 150,
      weight: 600,
      coinbase: true
    };

    const allTxs = [coinbaseTx, ...blockTransactions];
    const merkleRoot = this.computeMerkleRoot(allTxs);

    const block = {
      hash: crypto.randomBytes(32).toString('hex'),
      prevHash: prevBlock.hash,
      height: prevBlock.height + 1,
      time: Math.floor(Date.now() / 1000),
      nonce: Math.floor(Math.random() * 0xffffffff),
      bits: '207fffff',
      difficulty: 1,
      transactions: allTxs,
      merkleRoot,
      size: allTxs.reduce((sum, tx) => sum + (tx.size || 0), 0)
    };

    this.blocks.push(block);
    this.stats.totalBlocks++;

    // Update confirmations for transactions in this block
    for (const tx of allTxs) {
      this.transactions.push({ ...tx, blockHash: block.hash, blockHeight: block.height });
      
      // Move UTXOs from inputs to outputs
      if (!tx.coinbase) {
        for (const input of tx.inputs) {
          const utxoKey = `${input.txid}:${input.vout}`;
          this.utxoSet.delete(utxoKey);
        }
      }
      
      // Create new UTXOs
      for (let i = 0; i < tx.outputs.length; i++) {
        const output = tx.outputs[i];
        const utxoKey = `${tx.txid}:${i}`;
        this.utxoSet.set(utxoKey, {
          ...output,
          txid: tx.txid,
          vout: i,
          confirmations: 1,
          address: output.address || 'unknown'
        });
      }
    }

    // Update confirmations for all UTXOs
    for (const [key, utxo] of this.utxoSet) {
      if (!utxo.coinbase || utxo.confirmations > 0) {
        utxo.confirmations = (utxo.confirmations || 0) + 1;
      }
    }

    this.emit('block', block);
    return block;
  }

  computeMerkleRoot(transactions) {
    if (transactions.length === 0) return '0'.repeat(64);
    if (transactions.length === 1) return transactions[0].txid;

    let hashes = transactions.map(tx => tx.txid);
    
    while (hashes.length > 1) {
      if (hashes.length % 2 !== 0) {
        hashes.push(hashes[hashes.length - 1]);
      }
      
      const newHashes = [];
      for (let i = 0; i < hashes.length; i += 2) {
        const combined = hashes[i] + hashes[i + 1];
        newHashes.push(crypto.createHash('sha256').update(combined).digest('hex'));
      }
      hashes = newHashes;
    }
    
    return hashes[0];
  }

  generateMerkleProof(txid, block) {
    if (!block) return null;
    
    const txIndex = block.transactions.findIndex(tx => tx.txid === txid);
    if (txIndex === -1) return null;

    // Simplified merkle proof
    return {
      txid,
      blockHash: block.hash,
      blockHeight: block.height,
      index: txIndex,
      proof: [block.merkleRoot]
    };
  }

  validateTransaction(tx) {
    // Basic validation
    if (!tx.inputs || tx.inputs.length === 0) {
      return { valid: false, error: 'No inputs' };
    }
    if (!tx.outputs || tx.outputs.length === 0) {
      return { valid: false, error: 'No outputs' };
    }

    // Skip coinbase validation
    if (tx.inputs[0].coinbase) {
      return { valid: true };
    }

    // Check inputs exist in UTXO set
    let inputSum = 0;
    let outputSum = 0;

    for (const input of tx.inputs) {
      const utxoKey = `${input.txid}:${input.vout}`;
      const utxo = this.utxoSet.get(utxoKey);
      if (!utxo) {
        return { valid: false, error: `Input ${utxoKey} not found (already spent)` };
      }
      if (utxo.confirmations < 1) {
        return { valid: false, error: `Input ${utxoKey} has insufficient confirmations` };
      }
      inputSum += utxo.value;
    }

    for (const output of tx.outputs) {
      outputSum += output.value;
    }

    if (outputSum > inputSum) {
      return { valid: false, error: 'Output value exceeds input value' };
    }

    return { valid: true, fee: inputSum - outputSum };
  }

  submitTransaction(tx) {
    const validation = this.validateTransaction(tx);
    if (!validation.valid) {
      throw new Error(validation.error);
    }

    tx.txid = tx.txid || crypto.randomBytes(32).toString('hex');
    tx.time = Math.floor(Date.now() / 1000);
    this.mempool.push(tx);
    this.stats.totalTransactions++;

    return { txid: tx.txid, fee: validation.fee };
  }

  getUtxo(address) {
    const utxos = [];
    for (const [key, utxo] of this.utxoSet) {
      if (utxo.address === address) {
        utxos.push(utxo);
      }
    }
    return utxos;
  }

  getBalance(address) {
    const utxos = this.getUtxo(address);
    return utxos.reduce((sum, utxo) => sum + utxo.value, 0);
  }

  async handleRpc(request) {
    // Support both single request and batch
    const requests = Array.isArray(request) ? request : [request];
    const responses = [];

    for (const req of requests) {
      const method = req.method;
      const params = req.params || [];
      const id = req.id || null;
      
      let result = null;
      let error = null;

      try {
        switch (method) {
          case 'getblockchaininfo':
            result = {
              chain: this.chainId,
              blocks: this.blocks.length - 1,
              headers: this.blocks.length - 1,
              bestBlockHash: this.blocks[this.blocks.length - 1].hash,
              difficulty: 1,
              mediantime: this.blocks[this.blocks.length - 1].time,
              verificationprogress: 1,
              pruned: false,
              size_on_disk: this.blocks.reduce((sum, b) => sum + (b.size || 0), 0)
            };
            break;

          case 'getblockcount':
            result = this.blocks.length - 1;
            break;

          case 'getblockhash':
            const height = params[0];
            if (height >= 0 && height < this.blocks.length) {
              result = this.blocks[height].hash;
            } else {
              error = { code: -8, message: 'Block height out of range' };
            }
            break;

          case 'getblock':
            const hashOrHeight = params[0];
            let block;
            if (typeof hashOrHeight === 'number') {
              block = this.blocks[hashOrHeight];
            } else {
              block = this.blocks.find(b => b.hash === hashOrHeight);
            }
            result = block || null;
            break;

          case 'getrawtransaction':
            const txid = params[0];
            const tx = this.transactions.find(t => t.txid === txid) || 
                       this.mempool.find(t => t.txid === txid);
            result = tx || null;
            break;

          case 'sendrawtransaction':
            const rawTx = params[0];
            // Simplified - just parse and submit
            try {
              const tx = JSON.parse(rawTx);
              const submitted = this.submitTransaction(tx);
              result = submitted.txid;
            } catch (e) {
              error = { code: -22, message: e.message };
            }
            break;

          case 'getmempoolinfo':
            result = {
              size: this.mempool.length,
              bytes: this.mempool.reduce((sum, tx) => sum + (tx.size || 0), 0),
              usage: 0,
              maxmempool: 300000000,
              mempoolminfee: 0.00001
            };
            break;

          case 'getwalletinfo':
            if (this.walletManager) {
              const wallet = this.walletManager.getWallet('bitcoin');
              result = {
                walletname: 'dev-wallet',
                walletversion: 169900,
                balance: this.getBalance(wallet.address) / 1e8,
                txcount: this.transactions.length,
                keypoolsize: 1000
              };
            }
            break;

          case 'getbalance':
            if (this.walletManager) {
              const wallet = this.walletManager.getWallet('bitcoin');
              result = this.getBalance(wallet.address) / 1e8;
            } else {
              result = 0;
            }
            break;

          case 'getnewaddress':
            if (this.walletManager) {
              const wallet = this.walletManager.getWallet('bitcoin');
              result = wallet.address;
            }
            break;

          case 'gettxout':
            const outTxid = params[0];
            const outVout = params[1];
            const utxoKey2 = `${outTxid}:${outVout}`;
            result = this.utxoSet.get(utxoKey2) || null;
            break;

          case 'getblockchaininfo':
            result = {
              chain: this.chainId,
              blocks: this.blocks.length - 1,
              headers: this.blocks.length - 1,
              bestBlockHash: this.blocks[this.blocks.length - 1].hash,
              difficulty: 1,
              mediantime: this.blocks[this.blocks.length - 1].time,
              pruned: false
            };
            break;

          case 'generate':
            const numBlocks = params[0] || 1;
            const generated = [];
            for (let i = 0; i < numBlocks; i++) {
              generated.push(this.generateBlock());
            }
            result = generated.map(b => b.hash);
            break;

          case 'getnetworkinfo':
            result = {
              version: 250000,
              subversion: '/Satoshi:25.0.0/',
              protocolversion: 70016,
              connections: 1,
              networkactive: true,
              networks: [{ name: 'ipv4', limited: false, reachable: true }],
              relayfee: 0.00001,
              incrementalfee: 0.00001
            };
            break;

          default:
            error = { code: -32601, message: `Method not found: ${method}` };
        }
      } catch (e) {
        error = { code: -1, message: e.message };
      }

      responses.push({
        jsonrpc: '2.0',
        result,
        error,
        id
      });
    }

    return requests.length === 1 ? responses[0] : responses;
  }

  getStatus() {
    const latestBlock = this.blocks[this.blocks.length - 1];
    return {
      running: this.running,
      chainId: this.chainId,
      blockHeight: this.blocks.length - 1,
      blockHash: latestBlock?.hash,
      mempoolSize: this.mempool.length,
      totalTransactions: this.stats.totalTransactions,
      utxoCount: this.utxoSet.size,
      latency: Math.random() * 5 + 1, // Simulated 1-6ms latency
      tps: this.stats.totalBlocks > 0 ? this.stats.totalTransactions / ((Date.now() - this.stats.startTime) / 1000) : 0
    };
  }
}

module.exports = { BitcoinSimulator };
