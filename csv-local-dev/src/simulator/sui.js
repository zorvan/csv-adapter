#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const { EventEmitter } = require('events');

/**
 * Simulated Sui Chain (devnet-like)
 * 
 * Features:
 * - Epoch management
 * - Object state management
 * - Checkpoint generation
 * - Transaction finality
 * - JSON-RPC compatible endpoints
 */
class SuiSimulator extends EventEmitter {
  constructor(options = {}) {
    super();
    this.blockInterval = options.blockInterval || 5000;
    this.walletManager = options.walletManager;
    this.registry = options.registry;

    // Chain state
    this.checkpoints = [];
    this.transactions = [];
    this.objects = new Map(); // objectId -> { type, owner, fields, version, digest }
    this.mempool = [];
    this.epoch = 0;
    this.epochStartTimestamp = Date.now();
    this.epochDurationMs = 86400000; // 24 hours in reality, but we simulate faster
    this.running = false;
    this.blockTimer = null;
    this.stats = {
      totalTransactions: 0,
      totalCheckpoints: 0,
      startTime: Date.now()
    };

    // Genesis checkpoint
    this.genesisCheckpoint = this.createGenesisCheckpoint();
  }

  async initialize() {
    this.checkpoints.push(this.genesisCheckpoint);

    // Pre-fund dev wallet
    if (this.walletManager) {
      const wallet = this.walletManager.getWallet('sui');
      if (wallet) {
        // Create a SUI coin object for the dev wallet
        const suiObjectId = '0x' + crypto.randomBytes(32).toString('hex');
        const suiObject = {
          objectId: suiObjectId,
          version: 1,
          digest: '0x' + crypto.randomBytes(32).toString('hex'),
          type: '0x2::coin::Coin<0x2::sui::SUI>',
          owner: { AddressOwner: wallet.address },
          fields: {
            id: { id: suiObjectId },
            balance: '1000000000000' // 1000 SUI (9 decimals)
          },
          previousTransaction: this.genesisCheckpoint.digest
        };
        this.objects.set(suiObjectId, suiObject);
        wallet.balance = '1000.000000000 SUI';
        wallet.rawBalance = BigInt(1000 * 1e9);
      }
    }

    // Start checkpoint generation
    this.running = true;
    this.blockTimer = setInterval(() => this.generateCheckpoint(), this.blockInterval);
  }

  async stop() {
    this.running = false;
    if (this.blockTimer) {
      clearInterval(this.blockTimer);
      this.blockTimer = null;
    }
    this.emit('stopped');
  }

  createGenesisCheckpoint() {
    return {
      sequenceNumber: 0,
      digest: '0x' + crypto.createHash('sha256').update('sui-genesis').digest('hex'),
      epoch: 0,
      timestampMs: Date.now().toString(),
      transactions: [],
      checkpointCommitments: [],
      validatorSignature: '0x' + crypto.randomBytes(64).toString('hex'),
      previousCheckpointDigest: null
    };
  }

  generateCheckpoint() {
    if (!this.running) return;

    const prevCheckpoint = this.checkpoints[this.checkpoints.length - 1];
    const checkpointTxs = [...this.mempool.splice(0, 20)]; // Max 20 txs per checkpoint

    // Process transactions
    const executedTxs = [];
    for (const tx of checkpointTxs) {
      const result = this.executeTransaction(tx);
      if (result.success) {
        executedTxs.push(result.tx);
        this.transactions.push(result.tx);
        this.stats.totalTransactions++;
      }
    }

    // Check for epoch change
    if (Date.now() - this.epochStartTimestamp > this.epochDurationMs) {
      this.epoch++;
      this.epochStartTimestamp = Date.now();
    }

    const checkpoint = {
      sequenceNumber: this.checkpoints.length,
      digest: '0x' + crypto.randomBytes(32).toString('hex'),
      epoch: this.epoch,
      timestampMs: Date.now().toString(),
      transactions: executedTxs.map(tx => tx.digest),
      checkpointCommitments: [],
      validatorSignature: '0x' + crypto.randomBytes(64).toString('hex'),
      previousCheckpointDigest: prevCheckpoint.digest,
      networkTotalTransactions: this.stats.totalTransactions,
      epochRollingGasCostSummary: {
        computationCost: '0',
        storageCost: '0',
        storageRebate: '0',
        nonRefundableStorageFee: '0'
      }
    };

    this.checkpoints.push(checkpoint);
    this.stats.totalCheckpoints++;

    this.emit('checkpoint', checkpoint);
    return checkpoint;
  }

  executeTransaction(tx) {
    const digest = tx.digest || '0x' + crypto.randomBytes(32).toString('hex');
    const sender = tx.sender || '0x' + crypto.randomBytes(20).toString('hex');

    // Simulate transaction execution based on kind
    let effects = {};
    
    if (tx.kind === 'ProgrammableTransaction') {
      effects = this.executeProgrammableTransaction(tx, sender);
    } else {
      // Default: simple transfer
      effects = {
        status: { status: 'success' },
        gasUsed: { computationCost: '1000000', storageCost: '0', storageRebate: '0' },
        gasObject: null,
        eventsCreated: [],
        modifiedAtVersions: [],
        sharedObjects: [],
        versionAssigned: [],
        changedObjects: [],
        untouchedSharedObjects: []
      };
    }

    const txData = {
      digest,
      transaction: tx,
      effects: {
        ...effects,
        transactionDigest: digest,
        executedEpoch: this.epoch.toString(),
        checkpoint: (this.checkpoints.length).toString(),
        timestampMs: Date.now().toString()
      },
      timestampMs: Date.now().toString()
    };

    return { success: true, tx: txData };
  }

  executeProgrammableTransaction(tx, sender) {
    const inputs = tx.inputs || [];
    const commands = tx.commands || [];

    // Simulate command execution
    for (const cmd of commands) {
      if (cmd.TransferObjects) {
        const [objects, address] = cmd.TransferObjects;
        for (const obj of objects) {
          const objId = this.resolveInput(obj, inputs);
          if (this.objects.has(objId)) {
            const objData = this.objects.get(objId);
            objData.owner = { AddressOwner: this.resolveInput(address, inputs) };
            objData.version++;
          }
        }
      } else if (cmd.SplitCoins) {
        const [coin, amounts] = cmd.SplitCoins;
        // Simulate split
      } else if (cmd.MergeCoins) {
        // Simulate merge
      } else if (cmd.Publish) {
        // Simulate module publish
      } else if (cmd.MoveCall) {
        // Simulate move call
      }
    }

    return {
      status: { status: 'success' },
      gasUsed: { computationCost: '2000000', storageCost: '1000000', storageRebate: '500000' },
      gasObject: null,
      eventsCreated: [],
      modifiedAtVersions: [],
      sharedObjects: [],
      versionAssigned: [],
      changedObjects: [],
      untouchedSharedObjects: []
    };
  }

  resolveInput(input, inputs) {
    if (typeof input === 'object' && input.Input !== undefined) {
      return inputs[input.Input];
    }
    return input;
  }

  getObject(objectId) {
    return this.objects.get(objectId) || null;
  }

  getObjectsByOwner(owner) {
    const ownerObjects = [];
    for (const [id, obj] of this.objects) {
      if (obj.owner && obj.owner.AddressOwner === owner) {
        ownerObjects.push(obj);
      }
    }
    return ownerObjects;
  }

  async handleRpc(request) {
    // Sui uses POST with method in body
    const method = request.method;
    const params = request.params || {};
    const id = request.id || 1;

    let result = null;
    let error = null;

    try {
      switch (method) {
        case 'sui_getCheckpoint':
          const checkpointSeq = parseInt(params.id, 10);
          const checkpoint = this.checkpoints[checkpointSeq];
          result = checkpoint || null;
          break;

        case 'sui_getCheckpoints':
          const cursor = params.cursor ? parseInt(params.cursor, 10) : 0;
          const limit = params.limit || 10;
          const descending = params.descendingOrder || false;
          
          let checkpointsSlice;
          if (descending) {
            checkpointsSlice = this.checkpoints.slice(0, limit).reverse();
          } else {
            checkpointsSlice = this.checkpoints.slice(cursor, cursor + limit);
          }
          
          result = {
            data: checkpointsSlice,
            nextCursor: cursor + limit < this.checkpoints.length ? (cursor + limit).toString() : null,
            hasNextPage: cursor + limit < this.checkpoints.length
          };
          break;

        case 'sui_getObject':
          const objId = params.objectId;
          const showContent = params.options?.showContent;
          const obj = this.objects.get(objId);
          if (obj) {
            result = {
              data: {
                objectId: obj.objectId,
                version: obj.version.toString(),
                digest: obj.digest,
                type: obj.type,
                owner: obj.owner
              }
            };
            if (showContent) {
              result.data.content = {
                dataType: 'moveObject',
                type: obj.type,
                hasPublicTransfer: true,
                fields: obj.fields
              };
            }
          } else {
            result = null;
          }
          break;

        case 'sui_multiGetObjects':
          const objectIds = params.objectIds || [];
          result = objectIds.map(id => {
            const obj = this.objects.get(id);
            return obj ? {
              data: {
                objectId: obj.objectId,
                version: obj.version.toString(),
                digest: obj.digest
              }
            } : null;
          }).filter(Boolean);
          break;

        case 'sui_getOwnedObjects':
          const ownerAddr = params.owner;
          const ownedObjects = this.getObjectsByOwner(ownerAddr);
          result = {
            data: ownedObjects.map(obj => ({
              data: {
                objectId: obj.objectId,
                version: obj.version.toString(),
                digest: obj.digest,
                type: obj.type,
                owner: obj.owner
              }
            })),
            hasNextPage: false,
            nextCursor: null
          };
          break;

        case 'sui_getTransactionBlock':
          const txDigest = params.digest;
          const tx = this.transactions.find(t => t.digest === txDigest);
          result = tx || null;
          break;

        case 'sui_multiGetTransactionBlocks':
          const txDigests = params.digests || [];
          result = txDigests.map(d => this.transactions.find(t => t.digest === d)).filter(Boolean);
          break;

        case 'sui_executeTransactionBlock':
          const txBlock = params.txBytes;
          const signatures = params.signatures || [];
          // Execute and add to mempool
          const newDigest = '0x' + crypto.randomBytes(32).toString('hex');
          const executedTx = this.executeTransaction({
            digest: newDigest,
            ...txBlock,
            sender: params.sender
          });
          this.mempool.push(txBlock);
          result = {
            digest: newDigest,
            effects: executedTx.tx.effects,
            timestampMs: Date.now().toString()
          };
          break;

        case 'sui_dryRunTransactionBlock':
          // Simulate without executing
          result = {
            effects: {
              status: { status: 'success' },
              gasUsed: { computationCost: '1000000', storageCost: '0', storageRebate: '0' },
              transactionDigest: '0x' + crypto.randomBytes(32).toString('hex')
            }
          };
          break;

        case 'sui_getBalance':
          const balanceOwner = params.owner;
          const coinType = params.coinType || '0x2::sui::SUI';
          const balanceObjects = this.getObjectsByOwner(balanceOwner)
            .filter(obj => obj.type === `0x2::coin::Coin<${coinType}>`);
          const totalBalance = balanceObjects.reduce((sum, obj) => {
            return sum + BigInt(obj.fields?.balance || 0);
          }, BigInt(0));
          result = {
            coinType,
            totalBalance: totalBalance.toString(),
            coinObjectCount: balanceObjects.length
          };
          break;

        case 'sui_getAllBalances':
          const allBalOwner = params.owner;
          const allOwnerObjects = this.getObjectsByOwner(allBalOwner);
          const balancesByType = {};
          for (const obj of allOwnerObjects) {
            const coinTypeMatch = obj.type.match(/0x2::coin::Coin<(.+)>/);
            if (coinTypeMatch && obj.fields?.balance) {
              const type = coinTypeMatch[1];
              balancesByType[type] = (balancesByType[type] || BigInt(0)) + BigInt(obj.fields.balance);
            }
          }
          result = Object.entries(balancesByType).map(([type, balance]) => ({
            coinType: type,
            coinObjectCount: 1,
            totalBalance: balance.toString()
          }));
          break;

        case 'sui_getCoins':
          const coinsOwner = params.owner;
          const coinsType = params.coinType;
          let coins = this.getObjectsByOwner(coinsOwner);
          if (coinsType) {
            coins = coins.filter(obj => obj.type === `0x2::coin::Coin<${coinsType}>`);
          }
          result = {
            data: coins.map(obj => ({
              coinType: obj.type.replace(/0x2::coin::Coin<(.+)>/, '$1'),
              coinObjectId: obj.objectId,
              version: obj.version.toString(),
              digest: obj.digest,
              balance: obj.fields?.balance || '0'
            })),
            nextCursor: null,
            hasNextPage: false
          };
          break;

        case 'sui_getLatestCheckpointSequenceNumber':
          result = (this.checkpoints.length - 1).toString();
          break;

        case 'sui_getTotalTransactionBlocks':
          result = this.stats.totalTransactions.toString();
          break;

        case 'sui_getCommitteeInfo':
          result = {
            epoch: this.epoch.toString(),
            validators: [{
              name: 'simulated-validator-1',
              votingPower: 10000,
              protocolPubKey: '0x' + crypto.randomBytes(32).toString('hex'),
              networkPubKey: '0x' + crypto.randomBytes(32).toString('hex')
            }]
          };
          break;

        case 'sui_getCurrentEpoch':
          result = {
            epoch: this.epoch.toString(),
            startTimestampMs: this.epochStartTimestamp.toString(),
            endTimestampMs: (this.epochStartTimestamp + this.epochDurationMs).toString()
          };
          break;

        case 'sui_getReferenceGasPrice':
          result = '1000';
          break;

        case 'sui_getProtocolConfig':
          result = {
            minSupportedProtocolVersion: '1',
            maxSupportedProtocolVersion: '40',
            protocolVersion: '40',
            featureFlags: {
              'sui::transfer': true,
              'deepbook': true
            },
            attributes: {}
          };
          break;

        case 'rpc.discover':
          result = {
            info: {
              title: 'Sui RPC API',
              version: '1.0.0',
              description: 'CSV Local Dev Sui Simulator'
            }
          };
          break;

        default:
          error = { code: -32601, message: `Method not found: ${method}` };
      }
    } catch (e) {
      error = { code: -32603, message: e.message };
    }

    return {
      jsonrpc: '2.0',
      result,
      error,
      id
    };
  }

  getStatus() {
    const latestCheckpoint = this.checkpoints[this.checkpoints.length - 1];
    return {
      running: this.running,
      epoch: this.epoch,
      blockHeight: this.checkpoints.length - 1,
      checkpointDigest: latestCheckpoint?.digest,
      totalTransactions: this.stats.totalTransactions,
      objectCount: this.objects.size,
      mempoolSize: this.mempool.length,
      latency: Math.random() * 8 + 2, // Simulated 2-10ms latency
      tps: this.stats.totalCheckpoints > 0 ? this.stats.totalTransactions / ((Date.now() - this.stats.startTime) / 1000) : 0
    };
  }
}

module.exports = { SuiSimulator };
