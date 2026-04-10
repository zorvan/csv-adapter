/// CSV Seal — Cross-Chain Right Transfer on Aptos
///
/// This module implements:
/// - `create_seal()` — Create a new Right anchored to a Move resource
/// - `delete_seal()` — Consume a Right (single-use enforcement via resource destruction)
/// - `lock_right()` — Lock a Right for cross-chain transfer (destroys resource, emits event)
/// - `mint_right()` — Mint a new Right from a cross-chain transfer proof
/// - `refund_right()` — Recover a Right after lock timeout (settlement strategy)

module csv_seal::csv_seal {
    use std::signer;
    use std::vector;
    use aptos_framework::account;
    use aptos_framework::event;
    use aptos_framework::timestamp;

    /// A Right anchored to Aptos as a Move resource.
    /// The resource's existence = the Right's validity.
    /// Destroying the resource = consuming the Right (single-use enforced by Move VM).
    struct RightResource has key {
        /// Unique Right identifier (preserved across chains)
        right_id: vector<u8>,
        /// Commitment hash (preserved across chains)
        commitment: vector<u8>,
        /// Nullifier (for L3 chains that use nullifiers)
        nullifier: vector<u8>,
        /// State root (off-chain state commitment)
        state_root: vector<u8>,
    }

    /// Lock record stored on-chain for refund tracking
    struct LockRecord has store {
        /// Right identifier
        right_id: vector<u8>,
        /// Commitment hash
        commitment: vector<u8>,
        /// Destination chain ID
        destination_chain: u8,
        /// Lock timestamp (Unix epoch seconds)
        locked_at: u64,
        /// Whether this lock has been refunded
        refunded: bool,
    }

    /// Registry storing lock records for settlement
    struct LockRegistry has key {
        /// Map from right_id bytes to LockRecord
        locks: vector<LockRecord>,
        /// Refund timeout in seconds (24 hours = 86400)
        refund_timeout: u64,
    }

    /// Singleton registry resource (exists on deployer's account)
    struct RegistrySingleton has key {}

    /// Emitted when a Right is created
    struct RightCreated has drop, store {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
    }

    /// Emitted when a Right is consumed
    struct RightConsumed has drop, store {
        right_id: vector<u8>,
        consumer: address,
    }

    /// Emitted when a Right is locked for cross-chain transfer
    struct CrossChainLock has drop, store {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
        destination_chain: u8,
        destination_owner: vector<u8>,
        locked_at: u64,
    }

    /// Emitted when a Right is minted from cross-chain transfer
    struct CrossChainMint has drop, store {
        right_id: vector<u8>,
        commitment: vector<u8>,
        owner: address,
        source_chain: u8,
        source_seal_ref: vector<u8>,
    }

    /// Emitted when a Right is refunded (settlement)
    struct CrossChainRefund has drop, store {
        right_id: vector<u8>,
        commitment: vector<u8>,
        claimant: address,
        refunded_at: u64,
    }

    /// Initialize the LockRegistry (called once during deployment)
    public entry fun init_registry(account: &signer) {
        let registry_addr = signer::address_of(account);
        assert!(
            !exists<LockRegistry>(registry_addr),
            2001 // Registry already exists
        );

        move_to(account, LockRegistry {
            locks: vector::empty<LockRecord>(),
            refund_timeout: 86400, // 24 hours
        });

        move_to(account, RegistrySingleton {});
    }

    /// Find a lock record by right_id
    fun find_lock(registry: &LockRegistry, right_id: &vector<u8>): (bool, u64) {
        let i = 0;
        let len = vector::length(&registry.locks);
        while (i < len) {
            let record = vector::borrow(&registry.locks, i);
            if (record.right_id == *right_id) {
                return (true, i)
            };
            i = i + 1;
        };
        (false, 0)
    }

    /// Find and borrow mutable lock record by right_id
    fun find_lock_mut(registry: &mut LockRegistry, right_id: &vector<u8>): (bool, u64) {
        let i = 0;
        let len = vector::length(&registry.locks);
        while (i < len) {
            let record = vector::borrow(&registry.locks, i);
            if (record.right_id == *right_id) {
                return (true, i)
            };
            i = i + 1;
        };
        (false, 0)
    }

    /// Create a new Right on Aptos
    public entry fun create_seal(
        account: &signer,
        right_id: vector<u8>,
        commitment: vector<u8>,
        state_root: vector<u8>,
    ) acquires RightResource {
        let owner = signer::address_of(account);

        assert!(
            !exists<RightResource>(owner),
            1001 // Right already exists
        );

        move_to(account, RightResource {
            right_id,
            commitment,
            nullifier: vector::empty<u8>(),
            state_root,
        });

        event::emit(RightCreated {
            right_id,
            commitment,
            owner,
        });
    }

    /// Consume a Right (single-use enforcement via resource destruction)
    public entry fun delete_seal(account: &signer) acquires RightResource {
        let owner = signer::address_of(account);
        let RightResource { right_id, commitment: _, nullifier: _, state_root: _ } =
            move_from<RightResource>(owner);

        event::emit(RightConsumed {
            right_id,
            consumer: owner,
        });
    }

    /// Lock a Right for cross-chain transfer.
    /// This destroys the Right resource (single-use enforced by Move VM) and emits event.
    /// The lock is recorded in the registry for refund support.
    public entry fun lock_right(
        account: &signer,
        destination_chain: u8,
        destination_owner: vector<u8>,
    ) acquires RightResource, LockRegistry {
        let owner = signer::address_of(account);
        let registry_addr = owner;

        assert!(
            exists<LockRegistry>(registry_addr),
            2002 // Registry not initialized
        );

        let RightResource { right_id, commitment, nullifier: _, state_root: _ } =
            move_from<RightResource>(owner);

        let locked_at = timestamp::now_seconds();

        // Record the lock
        let registry = borrow_global_mut<LockRegistry>(registry_addr);
        vector::push_back(&mut registry.locks, LockRecord {
            right_id: right_id,
            commitment: commitment,
            destination_chain,
            locked_at,
            refunded: false,
        });

        event::emit(CrossChainLock {
            right_id,
            commitment,
            owner,
            destination_chain,
            destination_owner,
            locked_at,
        });
    }

    /// Mint a new Right from a cross-chain transfer proof.
    /// This creates a new RightResource with the same commitment as the source chain's Right.
    public entry fun mint_right(
        account: &signer,
        right_id: vector<u8>,
        commitment: vector<u8>,
        state_root: vector<u8>,
        source_chain: u8,
        source_seal_ref: vector<u8>,
    ) acquires RightResource {
        let owner = signer::address_of(account);

        assert!(
            !exists<RightResource>(owner),
            1002 // Right already exists at this address
        );

        move_to(account, RightResource {
            right_id,
            commitment,
            nullifier: vector::empty<u8>(),
            state_root,
        });

        event::emit(CrossChainMint {
            right_id,
            commitment,
            owner,
            source_chain,
            source_seal_ref,
        });
    }

    /// Refund a Right after the lock timeout has elapsed.
    /// This re-creates the RightResource if:
    /// 1. The lock was recorded in the registry
    /// 2. The REFUND_TIMEOUT has elapsed
    /// 3. The Right has not already been refunded
    public entry fun refund_right(
        account: &signer,
        right_id: vector<u8>,
        state_root: vector<u8>,
    ) acquires RightResource, LockRegistry {
        let owner = signer::address_of(account);
        let registry_addr = owner;

        assert!(
            exists<LockRegistry>(registry_addr),
            2002 // Registry not initialized
        );

        let registry = borrow_global_mut<LockRegistry>(registry_addr);
        let (found, index) = find_lock_mut(registry, &right_id);
        assert!(found, 2003); // Lock not found in registry

        let now = timestamp::now_seconds();
        let record = vector::borrow_mut(&mut registry.locks, index);

        // Verify timeout has elapsed
        assert!(
            now >= record.locked_at + registry.refund_timeout,
            2004 // Refund timeout not yet expired
        );

        // Verify not already refunded
        assert!(!record.refunded, 2005); // Already refunded

        // Mark as refunded
        record.refunded = true;

        // Re-create the RightResource
        assert!(
            !exists<RightResource>(owner),
            1001 // Right already exists
        );

        move_to(account, RightResource {
            right_id: record.right_id,
            commitment: record.commitment,
            nullifier: vector::empty<u8>(),
            state_root,
        });

        event::emit(CrossChainRefund {
            right_id,
            commitment: record.commitment,
            claimant: owner,
            refunded_at: now,
        });
    }

    /// Check if refund is available for a Right (view function)
    #[view]
    public fun can_refund(
        account: address,
        right_id: vector<u8>,
    ): bool acquires LockRegistry {
        if (!exists<LockRegistry>(account)) {
            return false
        };
        let registry = borrow_global<LockRegistry>(account);
        let (found, index) = find_lock(registry, &right_id);
        if (!found) {
            return false
        };
        let record = vector::borrow(&registry.locks, index);
        let now = timestamp::now_seconds();
        !record.refunded && now >= record.locked_at + registry.refund_timeout
    }

    /// Get lock info (for off-chain refund verification)
    #[view]
    public fun get_lock_info(
        account: address,
        right_id: vector<u8>,
    ): (vector<u8>, u64, bool) acquires LockRegistry {
        let registry = borrow_global<LockRegistry>(account);
        let (found, index) = find_lock(registry, &right_id);
        assert!(found, 2003); // Lock not found
        let record = vector::borrow(&registry.locks, index);
        (record.commitment, record.locked_at, record.refunded)
    }
}
