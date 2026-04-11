#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const { EventEmitter } = require('events');

/**
 * Simulated Ethereum Chain (anvil-like)
 * 
 * Features:
 * - Block generation at configurable intervals
 * - Account state management
 * - Gas estimation and limits
 * - Transaction receipts
 * - JSON-RPC compatible endpoints (Ethereum-compatible)
 */
class EthereumSimulator extends EventEmitter {
  constructor(options = {}) {
    super();
    this.blockInterval = options.blockInterval || 5000;
    this.walletManager = options.walletManager;
    this.registry = options.registry;
    
    // Chain state
    this.blocks = [];
    this.transactions = [];
    this.receipts = new Map();
    this.accounts = new Map(); // address -> { balance, nonce, code, storage }
    this.mempool = [];
    this.chainId = 31337; // Default anvil chain ID
    this.gasLimit = 30000000;
    this.baseFeePerGas = 1000000000; // 1 gwei
    this.running = false;
    this.blockTimer = null;
    this.stats = {
      totalTransactions: 0,
      totalBlocks: 0,
      startTime: Date.now()
    };

    // Genesis state
    this.genesisState = this.createGenesisState();
  }

  async initialize() {
    // Initialize accounts from genesis
    for (const [address, account] of Object.entries(this.genesisState)) {
      this.accounts.set(address.toLowerCase(), {
        balance: BigInt(account.balance),
        nonce: account.nonce || 0,
        code: account.code || '0x',
        storage: new Map()
      });
    }

    // Pre-fund dev wallet
    if (this.walletManager) {
      const wallet = this.walletManager.getWallet('ethereum');
      if (wallet) {
        const addr = wallet.address.toLowerCase();
        if (!this.accounts.has(addr)) {
          this.accounts.set(addr, {
            balance: BigInt(10000 * 1e18), // 10,000 ETH
            nonce: 0,
            code: '0x',
            storage: new Map()
          });
        }
        wallet.balance = '10000.0 ETH';
        wallet.rawBalance = BigInt(10000 * 1e18);
      }
    }

    // Create genesis block
    const genesisBlock = this.createBlock(null, []);
    genesisBlock.number = 0;
    genesisBlock.hash = '0x' + crypto.createHash('sha256').update('ethereum-genesis').digest('hex');
    this.blocks.push(genesisBlock);

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

  createGenesisState() {
    // Pre-funded accounts for testing
    return {
      '0x0000000000000000000000000000000000000001': { balance: BigInt(1000 * 1e18).toString() },
      '0x0000000000000000000000000000000000000002': { balance: BigInt(1000 * 1e18).toString() }
    };
  }

  createBlock(parentBlock, transactions) {
    const parent = parentBlock || this.blocks[this.blocks.length - 1];
    const timestamp = Math.floor(Date.now() / 1000);
    
    const block = {
      number: parent ? parent.number + 1 : 0,
      hash: '0x' + crypto.randomBytes(32).toString('hex'),
      parentHash: parent ? parent.hash : '0x' + '0'.repeat(64),
      nonce: '0x' + crypto.randomBytes(8).toString('hex'),
      sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
      logsBloom: '0x' + '0'.repeat(512),
      transactionsRoot: '0x' + crypto.createHash('sha256').update(timestamp.toString()).digest('hex'),
      stateRoot: '0x' + crypto.createHash('sha256').update('state-' + timestamp).digest('hex'),
      receiptsRoot: '0x' + crypto.createHash('sha256').update('receipts-' + timestamp).digest('hex'),
      miner: '0x0000000000000000000000000000000000000000',
      difficulty: '0x0',
      totalDifficulty: '0x0',
      extraData: '0x',
      size: '0x0',
      gasLimit: '0x' + this.gasLimit.toString(16),
      gasUsed: '0x0',
      timestamp: '0x' + timestamp.toString(16),
      transactions: transactions.map(tx => tx.hash),
      uncles: [],
      baseFeePerGas: '0x' + this.baseFeePerGas.toString(16),
      mixHash: '0x' + crypto.randomBytes(32).toString('hex')
    };

    return block;
  }

  generateBlock() {
    if (!this.running) return;

    const parentBlock = this.blocks[this.blocks.length - 1];
    const blockTxs = [];
    let gasUsed = 0;

    // Process mempool transactions
    const txsToProcess = [...this.mempool];
    this.mempool = [];

    for (const tx of txsToProcess) {
      if (gasUsed + (tx.gas || 21000) > this.gasLimit) {
        // Put back in mempool if gas limit exceeded
        this.mempool.push(tx);
        continue;
      }

      // Execute transaction
      const result = this.executeTransaction(tx);
      if (result.success) {
        blockTxs.push(result.tx);
        gasUsed += parseInt(result.receipt.gasUsed, 16);
        this.receipts.set(result.tx.hash, result.receipt);
        this.transactions.push(result.tx);
        this.stats.totalTransactions++;
      }
    }

    const block = this.createBlock(parentBlock, blockTxs);
    block.gasUsed = '0x' + gasUsed.toString(16);
    this.blocks.push(block);
    this.stats.totalBlocks++;

    this.emit('block', block);
    return block;
  }

  executeTransaction(tx) {
    const from = (tx.from || '0x0000000000000000000000000000000000000000').toLowerCase();
    const to = tx.to ? tx.to.toLowerCase() : null;
    const value = BigInt(tx.value || '0x0');
    const gas = parseInt(tx.gas || '0x5208', 16); // Default 21000
    const gasPrice = BigInt(tx.gasPrice || tx.maxFeePerGas || '0x3b9aca00');

    // Get sender account
    let sender = this.accounts.get(from);
    if (!sender) {
      sender = { balance: BigInt(0), nonce: 0, code: '0x', storage: new Map() };
      this.accounts.set(from, sender);
    }

    // Check balance
    const totalCost = value + gasPrice * BigInt(gas);
    if (sender.balance < totalCost) {
      return { success: false, error: 'insufficient funds' };
    }

    // Deduct gas and value from sender
    sender.balance -= totalCost;
    sender.nonce++;

    // If to address exists, credit value
    if (to) {
      let receiver = this.accounts.get(to);
      if (!receiver) {
        receiver = { balance: BigInt(0), nonce: 0, code: '0x', storage: new Map() };
        this.accounts.set(to, receiver);
      }
      receiver.balance += value;
    }

    // Generate transaction hash
    const txHash = '0x' + crypto.randomBytes(32).toString('hex');
    
    const transaction = {
      hash: txHash,
      nonce: '0x' + (sender.nonce - 1).toString(16),
      blockHash: null, // Set when included in block
      blockNumber: null,
      transactionIndex: '0x0',
      from,
      to: to || null,
      value: '0x' + value.toString(16),
      gas: '0x' + gas.toString(16),
      gasPrice: '0x' + gasPrice.toString(16),
      input: tx.input || tx.data || '0x',
      v: '0x1b',
      r: '0x' + crypto.randomBytes(32).toString('hex'),
      s: '0x' + crypto.randomBytes(32).toString('hex'),
      type: tx.type || '0x2',
      maxFeePerGas: tx.maxFeePerGas,
      maxPriorityFeePerGas: tx.maxPriorityFeePerGas,
      chainId: '0x' + this.chainId.toString(16)
    };

    const gasUsed = tx.input && tx.input !== '0x' ? gas : 21000;
    const gasRefund = (BigInt(gas) - BigInt(gasUsed)) * gasPrice;
    sender.balance += gasRefund;

    const receipt = {
      transactionHash: txHash,
      transactionIndex: '0x0',
      blockHash: null,
      blockNumber: null,
      from,
      to: to || null,
      cumulativeGasUsed: '0x' + gasUsed.toString(16),
      gasUsed: '0x' + gasUsed.toString(16),
      effectiveGasPrice: '0x' + gasPrice.toString(16),
      contractAddress: !to ? '0x' + crypto.randomBytes(20).toString('hex') : null,
      logs: [],
      logsBloom: '0x' + '0'.repeat(512),
      status: '0x1',
      type: '0x2'
    };

    return { success: true, tx: transaction, receipt };
  }

  estimateGas(params) {
    // Simplified gas estimation
    if (params.data && params.data !== '0x') {
      return '0x' + (21000 + params.data.length / 2 * 16).toString(16);
    }
    return '0x5208'; // 21000 for simple transfer
  }

  async handleRpc(request) {
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
          case 'eth_chainId':
            result = '0x' + this.chainId.toString(16);
            break;

          case 'eth_blockNumber':
            result = '0x' + (this.blocks.length - 1).toString(16);
            break;

          case 'eth_getBlockByNumber':
            const blockNum = params[0];
            const fullTxs = params[1];
            
            if (blockNum === 'latest' || blockNum === 'pending') {
              const latest = this.blocks[this.blocks.length - 1];
              result = fullTxs ? { ...latest, transactions: this.transactions.filter(tx => tx.blockHash === latest.hash) } : latest;
            } else if (blockNum === 'earliest') {
              result = this.blocks[0];
            } else {
              const num = parseInt(blockNum, 16);
              const block = this.blocks[num];
              result = block || null;
            }
            break;

          case 'eth_getBlockByHash':
            const blockHash = params[0];
            const blockByHash = this.blocks.find(b => b.hash === blockHash);
            result = blockByHash || null;
            break;

          case 'eth_getBalance':
            const balanceAddr = (params[0] || '').toLowerCase();
            const account = this.accounts.get(balanceAddr);
            result = account ? '0x' + account.balance.toString(16) : '0x0';
            break;

          case 'eth_getTransactionCount':
            const nonceAddr = (params[0] || '').toLowerCase();
            const nonceAccount = this.accounts.get(nonceAddr);
            result = nonceAccount ? '0x' + nonceAccount.nonce.toString(16) : '0x0';
            break;

          case 'eth_sendTransaction':
            const tx = params[0];
            const txResult = this.executeTransaction(tx);
            if (txResult.success) {
              this.mempool.push(tx);
              result = txResult.tx.hash;
            } else {
              error = { code: -32000, message: txResult.error };
            }
            break;

          case 'eth_sendRawTransaction':
            const rawTx = params[0];
            // Simplified - assume valid and add to mempool
            const txHash = '0x' + crypto.randomBytes(32).toString('hex');
            this.mempool.push({ hash: txHash, from: '0x' + crypto.randomBytes(20).toString('hex') });
            result = txHash;
            break;

          case 'eth_getTransactionByHash':
            const txHashParam = params[0];
            const foundTx = this.transactions.find(t => t.hash === txHashParam);
            result = foundTx || null;
            break;

          case 'eth_getTransactionReceipt':
            const receiptTxHash = params[0];
            const receipt = this.receipts.get(receiptTxHash);
            // Update block info if transaction is mined
            if (receipt) {
              const tx = this.transactions.find(t => t.hash === receiptTxHash);
              if (tx && tx.blockHash) {
                result = {
                  ...receipt,
                  blockHash: tx.blockHash,
                  blockNumber: tx.blockNumber
                };
              } else {
                result = receipt;
              }
            } else {
              result = null;
            }
            break;

          case 'eth_gasPrice':
            result = '0x' + this.baseFeePerGas.toString(16);
            break;

          case 'eth_maxPriorityFeePerGas':
            result = '0x3b9aca00'; // 1 gwei
            break;

          case 'eth_estimateGas':
            result = this.estimateGas(params[0] || {});
            break;

          case 'eth_call':
            // Simulate a call without state change
            result = '0x';
            break;

          case 'eth_getCode':
            const codeAddr = (params[0] || '').toLowerCase();
            const codeAccount = this.accounts.get(codeAddr);
            result = codeAccount ? codeAccount.code : '0x';
            break;

          case 'eth_getStorageAt':
            const storageAddr = (params[0] || '').toLowerCase();
            const storageKey = params[1];
            const storageAccount = this.accounts.get(storageAddr);
            result = storageAccount && storageAccount.storage.has(storageKey) 
              ? storageAccount.storage.get(storageKey) 
              : '0x' + '0'.repeat(64);
            break;

          case 'eth_getLogs':
            // Return empty logs for now
            result = [];
            break;

          case 'eth_getTransactionByBlockHashAndIndex':
          case 'eth_getTransactionByBlockNumberAndIndex':
            result = null;
            break;

          case 'net_version':
            result = this.chainId.toString();
            break;

          case 'net_listening':
            result = true;
            break;

          case 'net_peerCount':
            result = '0x1';
            break;

          case 'web3_clientVersion':
            result = 'csv-local-dev/1.0.0/ethereum-simulator';
            break;

          case 'eth_syncing':
            result = false;
            break;

          case 'eth_accounts':
            if (this.walletManager) {
              const wallet = this.walletManager.getWallet('ethereum');
              result = wallet ? [wallet.address] : [];
            } else {
              result = [];
            }
            break;

          case 'eth_mining':
            result = false;
            break;

          case 'eth_hashrate':
            result = '0x0';
            break;

          case 'eth_coinbase':
            result = '0x0000000000000000000000000000000000000000';
            break;

          case 'eth_subscribe':
            result = '0x' + crypto.randomBytes(16).toString('hex');
            break;

          case 'eth_unsubscribe':
            result = true;
            break;

          case 'evm_mine':
            this.generateBlock();
            result = null;
            break;

          case 'evm_setNextBlockTimestamp':
            result = null;
            break;

          case 'evm_increaseTime':
            result = parseInt(params[0], 16);
            break;

          case 'evm_snapshot':
            result = '0x1';
            break;

          case 'evm_revert':
            result = true;
            break;

          case 'anvil_setBalance':
            const setAddr = (params[0] || '').toLowerCase();
            const setBalance = BigInt(params[1]);
            let setAccount = this.accounts.get(setAddr);
            if (!setAccount) {
              setAccount = { balance: setBalance, nonce: 0, code: '0x', storage: new Map() };
              this.accounts.set(setAddr, setAccount);
            } else {
              setAccount.balance = setBalance;
            }
            result = null;
            break;

          case 'anvil_setCode':
            const codeAddr2 = (params[0] || '').toLowerCase();
            const newCode = params[1];
            let codeAcc = this.accounts.get(codeAddr2);
            if (!codeAcc) {
              codeAcc = { balance: BigInt(0), nonce: 0, code: newCode, storage: new Map() };
              this.accounts.set(codeAddr2, codeAcc);
            } else {
              codeAcc.code = newCode;
            }
            result = null;
            break;

          case 'debug_traceTransaction':
            result = { gas: 21000, returnValue: '0x', structLogs: [] };
            break;

          default:
            error = { code: -32601, message: `Method not found: ${method}` };
        }
      } catch (e) {
        error = { code: -32603, message: e.message };
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
      gasUsed: parseInt(latestBlock?.gasUsed || '0x0', 16),
      gasLimit: this.gasLimit,
      baseFeePerGas: this.baseFeePerGas,
      mempoolSize: this.mempool.length,
      totalTransactions: this.stats.totalTransactions,
      accountCount: this.accounts.size,
      latency: Math.random() * 3 + 0.5, // Simulated 0.5-3.5ms latency
      tps: this.stats.totalBlocks > 0 ? this.stats.totalTransactions / ((Date.now() - this.stats.startTime) / 1000) : 0
    };
  }
}

module.exports = { EthereumSimulator };
