# Wallet-Indexer Integration: Priority Address Indexing

## Overview

This implementation enables the **Wallet** to connect to the **CSV Explorer Indexer API** with prioritized indexing for all wallet addresses across all chains (Bitcoin, Ethereum, Sui, Aptos) on both testnet and mainnet networks. The system ensures that the indexer prioritizes indexing all data related to wallet addresses, enabling the wallet to display:

- **Rights** owned by wallet addresses
- **Transactions** (transfers) sent or received by wallet addresses
- **Seals** associated with wallet addresses
- **Assets** sent or received by wallet addresses

## Architecture

### Components

```
┌─────────────────┐
│     Wallet      │
│  (csv-wallet)   │
└────────┬────────┘
         │
         │ ExplorerService (REST API calls)
         │ WebSocket (real-time subscriptions)
         │
┌────────▼────────┐
│  CSV Explorer   │
│     API         │
│  (axum server)  │
└────────┬────────┘
         │
         │ WalletIndexerBridge
         │
┌────────▼────────┐
│  CSV Explorer   │
│   Indexer       │
│  (daemon)       │
└────────┬────────┘
         │
         │ ChainIndexer trait (address-based methods)
         │
┌────────▼────────┐
│  Chain-Specific │
│   Indexers      │
│ (BTC, ETH,      │
│  SUI, APT)      │
└────────┬────────┘
         │
         │ SQLite Storage
         │
┌────────▼────────┐
│   Database      │
│  (SQLite)       │
└─────────────────┘
```

### Key Components

#### 1. Priority Address Registration System

**Location**: `csv-explorer/shared/src/types.rs` and `csv-explorer/storage/src/repositories/priority_addresses.rs`

**Features**:

- Wallet addresses can be registered with different priority levels (High, Normal, Low)
- Supports multiple networks (mainnet, testnet, devnet)
- Tracks indexing timestamps and activity history
- SQLite-based persistence

**Data Model**:

```rust
PriorityAddress {
    address: String,          // The blockchain address
    chain: String,            // Chain identifier (bitcoin, ethereum, sui, aptos)
    network: Network,         // Mainnet, Testnet, or Devnet
    priority: PriorityLevel,  // High, Normal, or Low
    wallet_id: String,        // Owner wallet ID
    registered_at: DateTime,
    last_indexed_at: Option<DateTime>,
    is_active: bool,
}
```

#### 2. Address-Based Indexing

**Location**: `csv-explorer/indexer/src/chain_indexer.rs`

**New ChainIndexer Trait Methods**:

```rust
// Index all rights related to a specific address
async fn index_rights_by_address(&self, address: &str) -> ChainResult<Vec<RightRecord>>;

// Index all seals related to a specific address
async fn index_seals_by_address(&self, address: &str) -> ChainResult<Vec<SealRecord>>;

// Index all transfers related to a specific address
async fn index_transfers_by_address(&self, address: &str) -> ChainResult<Vec<TransferRecord>>;

// Index all data for addresses with priority
async fn index_addresses_with_priority(
    &self,
    addresses: &[String],
    priority: PriorityLevel,
    network: Network,
) -> ChainResult<AddressIndexingResult>;
```

**Implementation Requirements for Each Chain**:

- Bitcoin indexer: Scan UTXOs and transactions for Taproot addresses
- Ethereum indexer: Scan ERC-20 transfers and contract interactions for ETH addresses
- Sui indexer: Scan object ownership and transfers for Sui addresses
- Aptos indexer: Scan resource ownership and transactions for Aptos addresses

#### 3. Wallet-Indexer Bridge Service

**Location**: `csv-explorer/indexer/src/wallet_bridge.rs`

**Features**:

- Manages address registration/unregistration
- Runs priority indexing loop with configurable intervals:
  - High priority: Every 10 seconds
  - Normal priority: Every 1 minute
  - Low priority: Every 5 minutes
- Records indexing activities and errors
- Provides comprehensive status reporting

**Key Methods**:

```rust
// Register an address for priority indexing
async fn register_address(address, chain, network, priority, wallet_id) -> Result<()>;

// Unregister an address
async fn unregister_address(address, chain, network, wallet_id) -> Result<bool>;

// Get all data for an address
async fn get_address_data(address) -> Result<AddressDataResult>;

// Get priority indexing status
async fn get_priority_indexing_status() -> Result<PriorityIndexingStatus>;
```

#### 4. REST API Endpoints

**Location**: `csv-explorer/api/src/rest/handlers.rs` and `routes.rs`

**New Endpoints**:

```
POST   /api/v1/wallet/addresses
       Register address for priority indexing
       Body: { address, chain, network, priority, wallet_id }

DELETE /api/v1/wallet/addresses
       Unregister address from priority indexing
       Body: { address, chain, network, wallet_id }

GET    /api/v1/wallet/{wallet_id}/addresses
       Get all registered addresses for a wallet

GET    /api/v1/wallet/address/{address}/data
       Get complete data (rights, seals, transfers) for address

GET    /api/v1/wallet/address/{address}/rights
       Get all rights for address

GET    /api/v1/wallet/address/{address}/seals
       Get all seals for address

GET    /api/v1/wallet/address/{address}/transfers
       Get all transfers for address

GET    /api/v1/wallet/priority/status
       Get priority indexing status
```

#### 5. Wallet ExplorerService Integration

**Location**: `csv-wallet/src/services/explorer.rs`

**New Methods**:

```rust
// Register address for priority indexing
async fn register_priority_address(address, chain, network, priority, wallet_id);

// Unregister address from priority indexing
async fn unregister_priority_address(address, chain, network, wallet_id);

// Get all registered addresses for wallet
async fn get_wallet_addresses(wallet_id);

// Get complete data for address
async fn get_address_data(address);

// Get rights/seals/transfers for address
async fn get_address_rights(address);
async fn get_address_seals(address);
async fn get_address_transfers(address);

// Get priority indexing status
async fn get_priority_indexing_status();
```

#### 6. WebSocket Subscription System

**Location**: `csv-explorer/api/src/websocket.rs`

**Features**:

- Real-time event notifications for indexed data
- Subscribe/unsubscribe to address-specific events
- Event types:
  - `NewRight`: When a new right is indexed
  - `NewSeal`: When a new seal is indexed
  - `NewTransfer`: When a new transfer is indexed
  - `IndexingComplete`: When indexing cycle completes
  - `IndexingError`: When indexing fails

**WebSocket Endpoint**:

```
ws://explorer-host:port/ws/subscriptions
```

**Message Format**:

```json
// Subscribe
{
  "action": "subscribe",
  "address": "0x123...",
  "chain": "ethereum",
  "network": "mainnet"
}

// Event received
{
  "success": true,
  "message": "Event received",
  "event": {
    "type": "new_right",
    "address": "0x123...",
    "chain": "ethereum",
    "right_id": "abc...",
    "data": {...}
  }
}
```

## Usage Flow

### 1. Wallet Connects and Registers Addresses

```rust
// In wallet application
let explorer_service = ExplorerService::new(config);

// Get all wallet addresses
let wallet = ExtendedWallet::from_mnemonic(phrase);
let addresses = wallet.all_addresses(); // [(Bitcoin, "bc1q..."), (Ethereum, "0x..."), ...]

// Register each address for priority indexing
for (chain, address) in addresses {
    explorer_service.register_priority_address(
        &address,
        &chain.to_string(),
        "testnet",  // or "mainnet"
        "high",     // high priority for wallet addresses
        &wallet.metadata.id,
    ).await?;
}
```

### 2. Indexer Prioritizes Wallet Addresses

The `WalletIndexerBridge` runs a continuous loop:

```rust
// High priority addresses indexed every 10 seconds
// Normal priority every 1 minute
// Low priority every 5 minutes

for each priority_level in [High, Normal, Low]:
    addresses = get_addresses_by_priority(priority_level)
    
    for each chain in addresses:
        indexer.index_addresses_with_priority(
            addresses[chain],
            priority_level,
            network
        )
        
        // Record results
        record_indexing_activity(...)
```

### 3. Wallet Queries Indexed Data

```rust
// Get all data for an address
let data = explorer_service.get_address_data(&address).await?;

println!("Rights: {}", data.rights.len());
println!("Seals: {}", data.seals.len());
println!("Transfers: {}", data.transfers.len());

// Or get specific data types
let rights = explorer_service.get_address_rights(&address).await?;
let seals = explorer_service.get_address_seals(&address).await?;
let transfers = explorer_service.get_address_transfers(&address).await?;
```

### 4. Wallet Subscribes to Real-Time Updates

```rust
// Connect to WebSocket
let ws_url = "ws://explorer-host:8181/ws/subscriptions";
let ws_connection = connect(ws_url).await?;

// Subscribe to address
ws_connection.send(json!({
    "action": "subscribe",
    "address": "0x123...",
    "chain": "ethereum",
    "network": "mainnet"
})).await?;

// Receive real-time events
while let Some(message) = ws_connection.next().await {
    let event: SubscriptionEvent = parse(message)?;
    match event {
        NewRight { right_id, .. } => {
            // Update UI with new right
        }
        NewTransfer { transfer_id, .. } => {
            // Update UI with new transfer
        }
        ...
    }
}
```

## Multi-Network Support

The system fully supports indexing across multiple networks:

### Configuration

Each chain can be configured for different networks in `config.toml`:

```toml
[chains.bitcoin]
enabled = true
network = "testnet"  # or "mainnet"
rpc_url = "https://mempool.space/testnet/api"

[chains.ethereum]
enabled = true
network = "testnet"
rpc_url = "https://rpc.sepolia.org"

[chains.sui]
enabled = true
network = "testnet"
rpc_url = "https://fullnode.testnet.sui.io:443"

[chains.aptos]
enabled = true
network = "testnet"
rpc_url = "https://fullnode.testnet.aptoslabs.com/v1"
```

### Address Registration

When registering addresses, specify the network:

```rust
// Testnet address
explorer_service.register_priority_address(
    &testnet_address,
    "bitcoin",
    "testnet",  // Explicitly specify testnet
    "high",
    &wallet_id,
).await?;

// Mainnet address
explorer_service.register_priority_address(
    &mainnet_address,
    "ethereum",
    "mainnet",  // Explicitly specify mainnet
    "high",
    &wallet_id,
).await?;
```

## Database Schema

### Priority Addresses Table

```sql
CREATE TABLE priority_addresses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,
    chain TEXT NOT NULL,
    network TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'normal',
    wallet_id TEXT NOT NULL,
    registered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_indexed_at DATETIME,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    UNIQUE(address, chain, network, wallet_id)
);

CREATE INDEX idx_priority_addr_address ON priority_addresses(address);
CREATE INDEX idx_priority_addr_wallet ON priority_addresses(wallet_id);
CREATE INDEX idx_priority_addr_chain_network ON priority_addresses(chain, network);
```

### Indexing Activities Table

```sql
CREATE TABLE indexing_activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,
    chain TEXT NOT NULL,
    network TEXT NOT NULL,
    indexed_type TEXT NOT NULL,
    items_count INTEGER NOT NULL DEFAULT 0,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL DEFAULT 1,
    error TEXT
);

CREATE INDEX idx_indexing_activity_address ON indexing_activities(address, chain, network);
CREATE INDEX idx_indexing_activity_timestamp ON indexing_activities(timestamp);
```

## Implementation Checklist for Chain Indexers

Each chain-specific indexer (Bitcoin, Ethereum, Sui, Aptos) must implement the new address-based indexing methods:

### Bitcoin Indexer

- [ ] Implement `index_rights_by_address` - Scan for CSV rights in Taproot outputs
- [ ] Implement `index_seals_by_address` - Scan UTXOs owned by address
- [ ] Implement `index_transfers_by_address` - Scan transactions involving address
- [ ] Implement `index_addresses_with_priority` - Batch process addresses

### Ethereum Indexer

- [ ] Implement `index_rights_by_address` - Scan CsvSealRegistry events
- [ ] Implement `index_seals_by_address` - Scan nullifier registrations
- [ ] Implement `index_transfers_by_address` - Scan Transfer events
- [ ] Implement `index_addresses_with_priority` - Batch process addresses

### Sui Indexer

- [ ] Implement `index_rights_by_address` - Scan for right objects owned by address
- [ ] Implement `index_seals_by_address` - Scan objects created/deleted by address
- [ ] Implement `index_transfers_by_address` - Scan object transfers
- [ ] Implement `index_addresses_with_priority` - Batch process addresses

### Aptos Indexer

- [ ] Implement `index_rights_by_address` - Scan for right resources owned by address
- [ ] Implement `index_seals_by_address` - Scan resource creation/destruction
- [ ] Implement `index_transfers_by_address` - Scan resource transfers
- [ ] Implement `index_addresses_with_priority` - Batch process addresses

## Benefits

1. **Prioritized Indexing**: Wallet addresses are indexed with high priority, ensuring fast data availability
2. **Multi-Chain Support**: Works across Bitcoin, Ethereum, Sui, and Aptos
3. **Multi-Network Support**: Supports both testnet and mainnet simultaneously
4. **Real-Time Updates**: WebSocket subscriptions for instant notifications
5. **Comprehensive Data**: Indexes rights, seals, transfers, and assets
6. **Scalable**: Priority-based system ensures resources are used efficiently
7. **Persistent**: All indexing activity is logged and trackable

## Next Steps

1. **Implement Chain-Specific Methods**: Each chain indexer must implement the address-based indexing methods
2. **Add WebSocket to Wallet UI**: Integrate real-time subscriptions in the Dioxus UI
3. **Add GraphQL Support**: Add GraphQL queries for address-based indexing
4. **Performance Optimization**: Implement caching and batch processing
5. **Testing**: Add comprehensive tests for all new functionality
6. **Monitoring**: Add Prometheus metrics for priority indexing

## API Examples

### Register Address

```bash
curl -X POST http://localhost:8181/api/v1/wallet/addresses \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
    "chain": "ethereum",
    "network": "testnet",
    "priority": "high",
    "wallet_id": "wallet-123"
  }'
```

### Get Address Data

```bash
curl http://localhost:8181/api/v1/wallet/address/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18/data
```

### Get Wallet Addresses

```bash
curl http://localhost:8181/api/v1/wallet/wallet-123/addresses
```

### Get Priority Status

```bash
curl http://localhost:8181/api/v1/wallet/priority/status
```
