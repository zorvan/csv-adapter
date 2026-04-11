#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const { EventEmitter } = require('events');

/**
 * Simulated Aptos Chain (testnet-like)
 * 
 * Features:
 * - Block generation
 * - Account state management
 * - Ledger info
 * - Transaction finality
 * - REST API compatible endpoints
 */
class AptosSimulator extends EventEmitter {
  constructor(options = {}) {
    super();
    this.blockInterval = options.blockInterval || 5000;
    this.walletManager = options.walletManager;
    this.registry = options.registry;

    // Chain state
    this.blocks = [];
    this.transactions = [];
    this.accounts = new Map(); // address -> { sequence_number, authentication_key, coin_balance }
    this.resources = new Map(); // address -> { type, data }
    this.mempool = [];
    this.ledgerVersion = 0;
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
      const wallet = this.walletManager.getWallet('aptos');
      if (wallet) {
        const addr = wallet.address;
        this.accounts.set(addr, {
          sequence_number: '0',
          authentication_key: wallet.publicKey || '0x' + crypto.randomBytes(32).toString('hex'),
          coin_balance: 100000000000 // 100 APT (8 decimals)
        });

        // Add account resources
        this.resources.set(`${addr}::0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>`, {
          type: '0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>',
          data: {
            coin: { value: '100000000000' }
          }
        });

        wallet.balance = '100.00000000 APT';
        wallet.rawBalance = BigInt(100 * 1e8);
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
    return {
      block_height: 0,
      block_hash: '0x' + crypto.createHash('sha256').update('aptos-genesis').digest('hex'),
      block_timestamp: Date.now() * 1000, // microseconds
      first_version: '0',
      last_version: '0',
      epoch: 0,
      round: 0
    };
  }

  generateBlock() {
    if (!this.running) return;

    const prevBlock = this.blocks[this.blocks.length - 1];
    const blockTxs = [...this.mempool.splice(0, 30)]; // Max 30 txs per block

    // Process transactions
    const executedTxs = [];
    for (const tx of blockTxs) {
      const result = this.executeTransaction(tx);
      if (result.success) {
        executedTxs.push(result.tx);
        this.transactions.push(result.tx);
        this.stats.totalTransactions++;
      }
    }

    const block = {
      block_height: prevBlock.block_height + 1,
      block_hash: '0x' + crypto.randomBytes(32).toString('hex'),
      block_timestamp: Date.now() * 1000,
      first_version: (this.ledgerVersion + 1).toString(),
      last_version: (this.ledgerVersion + executedTxs.length).toString(),
      epoch: prevBlock.epoch,
      round: prevBlock.round + 1
    };

    this.blocks.push(block);
    this.ledgerVersion += executedTxs.length;
    this.stats.totalBlocks++;

    this.emit('block', block);
    return block;
  }

  executeTransaction(tx) {
    const sender = tx.sender || '0x' + crypto.randomBytes(32).toString('hex');
    const sequenceNumber = tx.sequence_number || '0';
    const maxGasAmount = tx.max_gas_amount || 100000;
    const gasUnitPrice = tx.gas_unit_price || 100;

    // Get or create account
    let account = this.accounts.get(sender);
    if (!account) {
      account = {
        sequence_number: '0',
        authentication_key: '0x' + crypto.randomBytes(32).toString('hex'),
        coin_balance: 0
      };
      this.accounts.set(sender, account);
    }

    // Check sequence number
    if (parseInt(sequenceNumber) < parseInt(account.sequence_number)) {
      return { success: false, error: 'sequence number too old' };
    }
    if (parseInt(sequenceNumber) > parseInt(account.sequence_number)) {
      return { success: false, error: 'sequence number too new' };
    }

    // Calculate gas
    const gasCost = maxGasAmount * gasUnitPrice;
    if (account.coin_balance < gasCost) {
      return { success: false, error: 'insufficient balance for gas' };
    }

    // Update account
    account.sequence_number = (parseInt(account.sequence_number) + 1).toString();
    account.coin_balance -= gasCost;

    // Generate version
    const version = (this.ledgerVersion + this.mempool.length + 1).toString();

    // Generate transaction hash
    const txHash = '0x' + crypto.randomBytes(32).toString('hex');

    const txData = {
      version,
      hash: txHash,
      state_change_hash: '0x' + crypto.randomBytes(32).toString('hex'),
      event_root_hash: '0x' + crypto.randomBytes(32).toString('hex'),
      state_checkpoint_hash: null,
      gas_used: maxGasAmount.toString(),
      success: true,
      vm_status: 'Executed successfully',
      accumulator_root_hash: '0x' + crypto.randomBytes(32).toString('hex'),
      changes: [],
      sender,
      sequence_number: account.sequence_number,
      max_gas_amount: maxGasAmount.toString(),
      gas_unit_price: gasUnitPrice.toString(),
      expiration_timestamp_secs: (Date.now() * 1000 + 60000000).toString(),
      payload: tx.payload || { function: '0x1::coin::transfer' },
      signature: tx.signature || { type: 'ed25519_signature', public_key: '0x' + crypto.randomBytes(32).toString('hex'), signature: '0x' + crypto.randomBytes(64).toString('hex') },
      events: [],
      timestamp: (Date.now() * 1000).toString(),
      type: 'user_transaction'
    };

    return { success: true, tx: txData };
  }

  async handleRpc(request) {
    // Aptos uses REST API style, but we support JSON-RPC-like for convenience
    const method = request.method;
    const params = request.params || {};
    const id = request.id || 1;

    let result = null;
    let error = null;

    try {
      switch (method) {
        case 'get_ledger_info':
          const latestBlock = this.blocks[this.blocks.length - 1];
          result = {
            chain_id: 4,
            epoch: latestBlock.epoch.toString(),
            ledger_version: this.ledgerVersion.toString(),
            ledger_timestamp: Date.now() * 1000,
            oldest_ledger_version: '0',
            oldest_block_height: '0',
            block_height: latestBlock.block_height.toString(),
            git_hash: 'csv-local-dev',
            node_role: 'validator',
            oldest_ledger_state_version: '0',
            block_timestamp: latestBlock.block_timestamp.toString()
          };
          break;

        case 'get_block_by_height':
          const blockHeight = parseInt(params.height, 10);
          const block = this.blocks.find(b => b.block_height === blockHeight);
          result = block || null;
          break;

        case 'get_block_by_version':
          const blockVersion = parseInt(params.version, 10);
          const blockByVersion = this.blocks.find(b => 
            parseInt(b.first_version, 10) <= blockVersion && parseInt(b.last_version, 10) >= blockVersion
          );
          result = blockByVersion || null;
          break;

        case 'get_account':
          const accountAddr = params.address;
          const account = this.accounts.get(accountAddr);
          if (account) {
            result = {
              sequence_number: account.sequence_number,
              authentication_key: account.authentication_key
            };
          } else {
            error = { code: 404, message: `Account not found: ${accountAddr}` };
          }
          break;

        case 'get_account_resources':
          const resourcesAddr = params.address;
          const accountResources = [];
          for (const [key, resource] of this.resources) {
            if (key.startsWith(resourcesAddr + '::')) {
              accountResources.push(resource);
            }
          }
          result = accountResources;
          break;

        case 'get_account_module':
          result = null;
          break;

        case 'get_transactions':
          const startVersion = parseInt(params.start, 10) || 0;
          const txLimit = params.limit || 10;
          result = this.transactions.slice(startVersion, startVersion + txLimit);
          break;

        case 'get_transaction_by_hash':
          const txHash = params.hash;
          const foundTx = this.transactions.find(t => t.hash === txHash);
          result = foundTx || null;
          break;

        case 'get_transaction_by_version':
          const txVersion = parseInt(params.version, 10);
          const foundTxByVersion = this.transactions.find(t => parseInt(t.version, 10) === txVersion);
          result = foundTxByVersion || null;
          break;

        case 'submit_transaction':
          const tx = params.transaction || request;
          const txResult = this.executeTransaction(tx);
          if (txResult.success) {
            this.mempool.push(tx);
            result = {
              hash: txResult.tx.hash,
              committed: false
            };
          } else {
            error = { code: 400, message: txResult.error };
          }
          break;

        case 'simulate_transaction':
          result = {
            version: '0',
            hash: '0x' + crypto.randomBytes(32).toString('hex'),
            state_change_hash: '0x' + crypto.randomBytes(32).toString('hex'),
            event_root_hash: '0x' + crypto.randomBytes(32).toString('hex'),
            gas_used: '100',
            success: true,
            vm_status: 'Executed successfully',
            accumulator_root_hash: '0x' + crypto.randomBytes(32).toString('hex'),
            changes: [],
            events: [],
            timestamp: (Date.now() * 1000).toString(),
            type: 'pending_transaction'
          };
          break;

        case 'get_events':
          const eventHandle = params.event_handle;
          const events = this.transactions.flatMap(tx => tx.events || []);
          result = events;
          break;

        case 'get_events_by_event_handle':
          result = [];
          break;

        case 'get_table_items_by_handle':
          result = { items: [] };
          break;

        case 'view':
          // Simulate a view function call
          result = ['0'];
          break;

        case 'estimate_gas_price':
          result = {
            gas_estimate: 100,
            deprioritized_gas_estimate: 50,
            prioritized_gas_estimate: 200
          };
          break;

        case 'get_genesis_transaction':
          result = {
            version: '0',
            hash: this.genesisBlock.block_hash,
            success: true,
            vm_status: 'Genesis transaction',
            type: 'genesis_transaction'
          };
          break;

        case 'get_validator_set':
          result = {
            total_voting_power: 10000,
            active_validators: [{
              addr: '0x' + crypto.randomBytes(32).toString('hex'),
              config: {
                consensus_pubkey: '0x' + crypto.randomBytes(32).toString('hex'),
                fullnode_addresses: '/ip4/127.0.0.1/tcp/6180'
              },
              voting_power: 10000
            }]
          };
          break;

        default:
          // REST API style routing
          if (request.url) {
            const url = request.url;
            if (url === '/v1') {
              result = await this.handleRpc({ method: 'get_ledger_info', params: {} });
            } else if (url.startsWith('/v1/blocks/by_height/')) {
              const height = parseInt(url.split('/').pop(), 10);
              result = await this.handleRpc({ method: 'get_block_by_height', params: { height } });
            } else if (url.startsWith('/v1/accounts/')) {
              const addr = url.split('/')[3];
              result = await this.handleRpc({ method: 'get_account', params: { address: addr } });
            } else if (url.startsWith('/v1/transactions/by_hash/')) {
              const hash = url.split('/').pop();
              result = await this.handleRpc({ method: 'get_transaction_by_hash', params: { hash } });
            } else {
              error = { code: 404, message: `Not found: ${url}` };
            }
          } else {
            error = { code: -32601, message: `Method not found: ${method}` };
          }
      }
    } catch (e) {
      error = { code: 500, message: e.message };
    }

    // Return in Aptos REST style or JSON-RPC style
    if (request.url) {
      return result;
    }

    return {
      jsonrpc: '2.0',
      result,
      error,
      id
    };
  }

  getStatus() {
    const latestBlock = this.blocks[this.blocks.length - 1];
    return {
      running: this.running,
      chainId: 4,
      blockHeight: latestBlock?.block_height || 0,
      ledgerVersion: this.ledgerVersion,
      totalTransactions: this.stats.totalTransactions,
      accountCount: this.accounts.size,
      mempoolSize: this.mempool.length,
      latency: Math.random() * 6 + 1.5, // Simulated 1.5-7.5ms latency
      tps: this.stats.totalBlocks > 0 ? this.stats.totalTransactions / ((Date.now() - this.stats.startTime) / 1000) : 0
    };
  }
}

module.exports = { AptosSimulator };
