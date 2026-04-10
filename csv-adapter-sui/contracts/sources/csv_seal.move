/// CSV Seal — Cross-Chain Right Transfer on Sui
///
/// This module implements:
/// - `create_seal()` — Create a new Right anchored to a Sui object
/// - `consume_seal()` — Consume a Right (single-use enforcement via object deletion)
/// - `lock_right()` — Lock a Right for cross-chain transfer (consumes seal, emits event)
/// - `mint_right()` — Mint a new Right from a cross-chain transfer proof
/// - `refund_right()` — Recover a Right after lock timeout (settlement strategy)

module csv_seal::csv_seal {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use std::string::{Self, String};
    use std::hash;

    /// A Right anchored to Sui as an object.
    /// The object's existence = the Right's validity.
    /// Deleting the object = consuming the Right (single-use enforced).
    struct RightObject has key, store {
        id: UID,
        /// Unique Right identifier (preserved across chains)
        right_id: vector<u8>,
        /// Commitment hash (preserved across chains)
        commitment: vector<u8>,
        /// Owner address
        owner: address,
        /// Nullifier (for L3 chains that use nullifiers)
        nullifier: vector<u8>,
        /// State root (off-chain state commitment)
        state_root: vector<u8>,
    }

    /// Lock record for refund tracking
    struct LockRecord has store {
        /// Right identifier
        right_id: vector<u8>,
        /// Commitment hash
        commitment: vector<u8>,
        /// Original owner
        owner: address,
        /// Destination chain ID
        destination_chain: u8,
        /// Lock timestamp (Unix epoch seconds)
        locked_at: u64,
        /// Whether this lock has been refunded
        refunded: bool,
    }

    /// Shared object tracking lock records for settlement
    struct LockRegistry has key {
        id: UID,
        /// Map from right_id to LockRecord
        locks: table::Table<vector<u8>, LockRecord>,
        /// Refund timeout in seconds (24 hours)
        refund_timeout: u64,
    }

    /// Emitted when a Right is created
    event RightCreated {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
        object_id: ID,
    }

    /// Emitted when a Right is consumed
    event RightConsumed {
        right_id: vector<u8>,
        consumer: address,
    }

    /// Emitted when a Right is locked for cross-chain transfer
    event CrossChainLock {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
        destination_chain: u8,
        destination_owner: vector<u8>,
        source_tx_hash: vector<u8>,
        locked_at: u64,
    }

    /// Emitted when a Right is minted from cross-chain transfer
    event CrossChainMint {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
        source_chain: u8,
        source_seal_ref: vector<u8>,
    }

    /// Emitted when a Right is refunded (settlement)
    event CrossChainRefund {
        right_id: vector<u8>,
        commitment: vector<u8>,
        claimant: address,
        refunded_at: u64,
    }

    /// Create a new LockRegistry (called once during deployment)
    public entry fun create_registry(ctx: &mut TxContext) {
        let registry = LockRegistry {
            id: object::new(ctx),
            locks: table::new<vector<u8>, LockRecord>(),
            refund_timeout: 86400, // 24 hours
        };
        transfer::share_object(registry);
    }

    /// Create a new Right on Sui
    public entry fun create_seal(
        right_id: vector<u8>,
        commitment: vector<u8>,
        state_root: vector<u8>,
        ctx: &mut TxContext
    ) {
        let right = RightObject {
            id: object::new(ctx),
            right_id,
            commitment,
            owner: tx_context::sender(ctx),
            nullifier: vector::empty<u8>(),
            state_root,
        };

        event::emit(RightCreated {
            right_id: right.right_id,
            commitment: right.commitment,
            owner: right.owner,
            object_id: object::uid_to_inner(&right.id),
        });

        transfer::public_transfer(right, tx_context::sender(ctx));
    }

    /// Consume a Right (single-use enforcement via object deletion)
    public entry fun consume_seal(
        right: RightObject,
        ctx: &mut TxContext
    ) {
        event::emit(RightConsumed {
            right_id: right.right_id,
            consumer: tx_context::sender(ctx),
        });

        let RightObject { id, right_id: _, commitment: _, owner: _, nullifier: _, state_root: _ } = right;
        object::delete(id);
    }

    /// Lock a Right for cross-chain transfer.
    /// This consumes the Right (deletes the object) and emits a CrossChainLock event.
    /// The lock is recorded in the registry for refund support.
    public entry fun lock_right(
        right: RightObject,
        destination_chain: u8,
        destination_owner: vector<u8>,
        registry: &mut LockRegistry,
        ctx: &mut TxContext
    ) {
        let locked_at = tx_context::epoch_timestamp_ms(ctx) / 1000;

        // Record the lock in the registry
        let lock = LockRecord {
            right_id: right.right_id,
            commitment: right.commitment,
            owner: right.owner,
            destination_chain,
            locked_at,
            refunded: false,
        };

        table::add(&mut registry.locks, right.right_id, lock);

        event::emit(CrossChainLock {
            right_id: right.right_id,
            commitment: right.commitment,
            owner: right.owner,
            destination_chain,
            destination_owner,
            source_tx_hash: tx_context::digest(ctx),
            locked_at,
        });

        // Consume the Right (object deletion = single-use enforcement)
        let RightObject { id, right_id: _, commitment: _, owner: _, nullifier: _, state_root: _ } = right;
        object::delete(id);
    }

    /// Mint a new Right from a cross-chain transfer proof.
    /// This creates a new RightObject with the same commitment as the source chain's Right.
    public entry fun mint_right(
        right_id: vector<u8>,
        commitment: vector<u8>,
        state_root: vector<u8>,
        source_chain: u8,
        source_seal_ref: vector<u8>,
        ctx: &mut TxContext
    ) {
        let right = RightObject {
            id: object::new(ctx),
            right_id,
            commitment,
            owner: tx_context::sender(ctx),
            nullifier: vector::empty<u8>(),
            state_root,
        };

        event::emit(CrossChainMint {
            right_id: right.right_id,
            commitment: right.commitment,
            owner: right.owner,
            source_chain,
            source_seal_ref,
        });

        transfer::public_transfer(right, tx_context::sender(ctx));
    }

    /// Refund a Right after the lock timeout has elapsed.
    /// This re-creates the RightObject if:
    /// 1. The lock was recorded in the registry
    /// 2. The REFUND_TIMEOUT has elapsed
    /// 3. The Right has not already been refunded
    public entry fun refund_right(
        right_id: vector<u8>,
        commitment: vector<u8>,
        state_root: vector<u8>,
        registry: &mut LockRegistry,
        ctx: &mut TxContext
    ) {
        assert!(
            table::contains(&registry.locks, &right_id),
            1003 // Lock not found in registry
        );

        let lock = table::borrow_mut(&mut registry.locks, &right_id);
        let now = tx_context::epoch_timestamp_ms(ctx) / 1000;

        // Verify timeout has elapsed
        assert!(
            now >= lock.locked_at + registry.refund_timeout,
            1004 // Refund timeout not yet expired
        );

        // Verify not already refunded
        assert!(!lock.refunded, 1005); // Already refunded

        // Mark as refunded
        lock.refunded = true;

        // Re-create the RightObject
        let right = RightObject {
            id: object::new(ctx),
            right_id: lock.right_id,
            commitment: lock.commitment,
            owner: tx_context::sender(ctx),
            nullifier: vector::empty<u8>(),
            state_root,
        };

        event::emit(CrossChainRefund {
            right_id: right.right_id,
            commitment: right.commitment,
            claimant: tx_context::sender(ctx),
            refunded_at: now,
        });

        transfer::public_transfer(right, tx_context::sender(ctx));
    }

    /// Transfer ownership of a Right
    public entry fun transfer_right(
        right: RightObject,
        new_owner: address,
        _ctx: &mut TxContext
    ) {
        transfer::public_transfer(right, new_owner);
    }

    /// Get lock info (for off-chain refund verification)
    public fun get_lock_info(
        registry: &LockRegistry,
        right_id: &vector<u8>,
    ): (vector<u8>, u64, bool) {
        let lock = table::borrow(&registry.locks, right_id);
        (lock.commitment, lock.locked_at, lock.refunded)
    }

    /// Check if refund is available for a Right
    public fun can_refund(
        registry: &LockRegistry,
        right_id: &vector<u8>,
        now: u64,
    ): bool {
        if (!table::contains(&registry.locks, right_id)) {
            return false
        };
        let lock = table::borrow(&registry.locks, right_id);
        !lock.refunded && now >= lock.locked_at + registry.refund_timeout
    }
}
