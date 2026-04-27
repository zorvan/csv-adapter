# CSV Protocol Specification

## Overview

CSV (Cross-Seal Validation) is a proof portability protocol for cross-chain rights. Unlike bridges that require trusted validators, CSV uses cryptographic proofs that can be verified client-side.

**Core Principle**: The source chain enforces single-use, the sender produces evidence, and the receiver verifies that evidence.

## Protocol Objects

| Object | Definition | Role |
|--------|------------|------|
| `Right` | Portable client-side claim or state object | Represents ownership of an asset |
| `Seal` | Chain-specific primitive consumable at most once | Guarantees single-use on-chain |
| `Commitment` | Hash of a state transition bound to a seal | Links right to chain state |
| `Anchor` | Published chain reference tied to a commitment | Proof of inclusion |
| `ProofBundle` | Inclusion + finality evidence + context | Portable verification package |

## Transfer Flow

### Single-Chain (Baseline)

1. Right is associated with an active seal
2. Owner consumes the seal through a valid chain action
3. Chain records proof data
4. Receiver verifies proof and updates local state

### Cross-Chain (The CSV Innovation)

```
┌─────────────┐     Consume      ┌─────────────┐
│   Right     │ ───────────────→ │   Seal      │
│  (Source)   │                  │  (Source)   │
└─────────────┘                  └──────┬──────┘
                                       │
                                       ↓ Proof Bundle
                              ┌─────────────────┐
                              │  Source Chain   │
                              │  Evidence       │
                              └────────┬────────┘
                                       │
                                       ↓ Verify
┌─────────────┐                  ┌──────┴──────┐
│   Right     │ ←────────────── │  Receiver   │
│  (Target)   │    Re-anchor    │  (Target)   │
└─────────────┘                 └─────────────┘
```

1. **Consume** the source-chain seal
2. **Produce** inclusion and finality evidence from source chain
3. **Bind** evidence to the expected commitment and state transition
4. **Verify** the seal hasn't been replayed elsewhere
5. **Accept** or mint destination-side representation

## Proof Bundle Structure

A valid proof bundle answers four questions:

1. Which chain enforced single-use?
2. Which seal was consumed?
3. Where is the on-chain evidence?
4. Why is that evidence final enough?

```rust
ProofBundle {
    source_chain: ChainId,
    seal_ref: SealRef,
    anchor_ref: AnchorRef,
    inclusion_proof: InclusionProof,
    finality_proof: FinalityProof,
    commitment: Commitment,
    transition_context: TransitionContext,
}
```

## Verification Pipeline

```
Decode → Confirm → Verify Inclusion → Verify Finality → 
Check Commitment → Check Replay Guard → Accept
```

## Chain-Specific Implementations

### Bitcoin

- **Seal**: UTXO spend
- **Evidence**: Transaction + Merkle branch
- **Finality**: Confirmation depth (typically 6)
- **Security**: Structural single-use (strongest)

### Sui

- **Seal**: Object deletion/mutation
- **Evidence**: Transaction effects in checkpoint
- **Finality**: Certified checkpoint
- **Security**: Validator-certified object lifecycle

### Aptos

- **Seal**: Resource destruction
- **Evidence**: Transaction + ledger proof
- **Finality**: Ledger consensus output
- **Security**: Type-enforced linearity

### Ethereum

- **Seal**: Nullifier registration in contract
- **Evidence**: Receipt + log path
- **Finality**: Confirmation threshold
- **Security**: Contract-mediated (not structural)

## Invariants

| Invariant | Why It Matters |
|-----------|----------------|
| Seal consumed at most once | Prevents replay/double-spend |
| Commitment domain-separated | Prevents cross-chain confusion |
| Inclusion binds to chain history | Prevents forged claims |
| Finality is chain-specific | Prevents false equivalence |
| Receiver decides after verification | Preserves client-side model |

## Security Properties

- **No trusted validators**: Cryptography, not attestations
- **No central point of failure**: Client-side verification
- **Chain-native guarantees**: Each chain's strengths preserved
- **Proof replay impossible**: Seals are single-use by construction

## What CSV Does NOT Do

- Assume all chains have equal security
- Require destination minting
- Trust RPC responses
- Use bridge-style messaging
