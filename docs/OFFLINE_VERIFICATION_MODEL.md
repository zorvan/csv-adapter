# ❄️ Offline Verification Model

## 1. The Core Thesis

The CSV Protocol allows a party to receive an asset and prove its validity without querying a blockchain node at the moment of receipt.

## 2. The Proof Bundle Anatomy

A `ProofBundle` is a self-contained DAG containing:

1. **Transition Proofs:** The "What" (state changes).
2. **Seal Commitments:** The "Who" (the L1 anchor).
3. **Inclusion Proofs:** The "Where" (Merkle branches linking to an L1 block).
4. **Finality Proofs:** The "Weight" (Header chain showing the block is deep enough).

## 3. The Trust Boundary

| Component | Status | Source |
| :--- | :--- | :--- |
| **Header Chain** | Verified Offline | Local Header Cache |
| **State Transition** | Verified Offline | Bundle Logic |
| **Inclusion** | Verified Offline | Merkle Math |
| **L1 Finality** | Assumed | $N$ Confirmations |

## 4. Verification Workflow (Offline)

1. **Hash Verification:** Recompute the `TaggedHash` of the transition.
2. **Seal Verification:** Confirm the transition matches the commitment in the Single-Use Seal.
3. **Path Verification:** Walk the Merkle path from the commitment to the `state_root` (ETH) or `merkle_root` (BTC) provided in the bundle.
4. **Header Verification:** Verify the block header matches the user's local "Checkpoint" (Trusted Block Hash).

## 5. Security Invariant
>
> "An offline verifier with a trusted block hash at height $H$ can verify the entire history of an asset up to height $H$ with the same security as a Full Node."
