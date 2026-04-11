# AluVM Integration in CSV Adapter

**Version:** 1.0  
**Date:** April 10, 2026  
**Status:** Planning Complete - Implementation Ready

---

## 1. Overview

This document outlines the integration of AluVM (Algebraic Universal Virtual Machine) into the CSV Adapter project. AluVM will serve as a production-grade virtual machine for executing contract validation scripts, replacing the current `PassthroughVM` stub implementation.

### What is AluVM?

AluVM is a deterministic, sandboxed virtual machine designed for executing smart contracts with strong guarantees of:

- **Determinism**: Same inputs always produce same outputs
- **Security**: Sandboxed execution with memory safety
- **Performance**: Optimized bytecode execution
- **Extensibility**: Easy to add new opcodes and features

### Why Integrate AluVM?

The current CSV Adapter uses a simple `PassthroughVM` that only validates basic state conservation rules. AluVM integration will enable:

1. **Complex Contract Logic**: Support for sophisticated validation rules
2. **Enhanced Security**: Sand execution with memory safety guarantees
3. **Deterministic Execution**: Guaranteed consistency across environments
4. **Performance Optimization**: Efficient bytecode execution
5. **Extensibility**: Easy to add new features and opcodes

---

## 2. Current Architecture

### 2.1 VM Layer

The CSV Adapter currently has a generic VM interface defined in `csv-adapter-core/src/vm.rs`:

```rust
pub trait DeterministicVM {
    fn execute(
        &self,
        bytecode: &[u8],
        inputs: VMInputs,
        signatures: &[Vec<u8>],
    ) -> Result<VMOutputs, VMError>;
    
    fn validate_outputs(&self, inputs: &VMInputs, outputs: &VMOutputs) -> Result<(), VMError>;
}
```

### 2.2 Current Implementation

The current `PassthroughVM` implementation:

- Passes inputs through as outputs
- Only validates basic state conservation rules
- Does not execute actual bytecode
- Serves as a placeholder for testing

### 2.3 State Types

The system uses typed state structures:

- `GlobalState`: Contract-wide values
- `OwnedState`: State bound to specific seals
- `Metadata`: Auxiliary data
- `StateAssignment`: Output state assignments

---

## 3. AluVM Integration Architecture

### 3.1 Integration Points

AluVM will integrate at multiple levels:

1. **Core VM Layer**: Replace `PassthroughVM` with `AluVM`
2. **Schema System**: Support AluVM bytecode in transition definitions
3. **Validation Pipeline**: Use AluVM for transition execution
4. **CLI Tools**: Add VM selection and bytecode compilation

### 3.2 State Mapping Strategy

| CSV Type | AluVM Representation | Purpose |
|----------|---------------------|---------|
| `OwnedState` | Memory with seal addresses | Per-seal state storage |
| `GlobalState` | Global memory section | Contract-wide state |
| `Metadata` | Auxiliary data section | Opaque metadata |
| `SealRef` | Memory address identifiers | State ownership |
| `Signatures` | Signature verification context | Authorization |

### 3.3 Memory Layout

```
AluVM Memory Layout:
┌─────────────────────────────────────────┐
│ Global State (0x0000 - 0x0FFF)         │
│ Contract configuration, supply counters  │
├─────────────────────────────────────────┤
│ Owned State (0x1000 - 0x7FFF)          │
│ Per-seal state with seal as address      │
├─────────────────────────────────────────┤
│ Metadata (0x8000 - 0x8FFF)              │
│ Auxiliary data, timestamps, etc.        │
├─────────────────────────────────────────┤
│ Scratch Space (0x9000 - 0xFFFF)         │
│ Temporary calculations                  │
└─────────────────────────────────────────┘
```

---

## 4. Implementation Plan

### Phase 1: Core Integration

#### 1.1 Add AluVM Dependency

```toml
# csv-adapter-core/Cargo.toml
[dependencies]
aluvm = { version = "0.1", optional = true }
```

#### 1.2 Implement AluVM Adapter

```rust
// csv-adapter-core/src/vm.rs
pub struct AluVM {
    execution_engine: aluvm::ExecutionEngine,
    gas_limit: u64,
    max_steps: u64,
}

impl DeterministicVM for AluVM {
    fn execute(&self, bytecode: &[u8], inputs: VMInputs, signatures: &[Vec<u8>]) -> Result<VMOutputs, VMError> {
        // Map CSV inputs to AluVM memory
        let mut memory = Self::map_inputs_to_memory(&inputs);
        
        // Set up execution context
        let mut context = aluvm::ExecutionContext::new();
        context.set_gas_limit(self.gas_limit);
        context.set_max_steps(self.max_steps);
        
        // Execute bytecode
        let result = self.execution_engine.execute(bytecode, &mut memory, &mut context)?;
        
        // Map results back to VMOutputs
        Self::map_memory_to_outputs(&memory, &result)
    }
}
```

#### 1.3 State Mapping Implementation

```rust
impl AluVM {
    fn map_inputs_to_memory(inputs: &VMInputs) -> aluvm::Memory {
        let mut memory = aluvm::Memory::new();
        
        // Map global state (0x0000 - 0x0FFF)
        for (i, state) in inputs.global_state.iter().enumerate() {
            let address = 0x0000 + (i as u16) * 256;
            memory.write(address, &state.data);
        }
        
        // Map owned state (0x1000 - 0x7FFF)
        for (i, state) in inputs.owned_inputs.iter().enumerate() {
            let address = Self::seal_to_address(&state.seal);
            memory.write(address, &state.data);
        }
        
        // Map metadata (0x8000 - 0x8FFF)
        for (i, meta) in inputs.metadata.iter().enumerate() {
            let address = 0x8000 + (i as u16) * 256;
            memory.write(address, &meta.value);
        }
        
        memory
    }
    
    fn seal_to_address(seal: &SealRef) -> u16 {
        // Use hash of seal ID as memory address
        let hash = crate::hash::Hash::new(seal.seal_id.clone());
        u16::from_be_bytes([hash.as_bytes()[0], hash.as_bytes()[1]]) | 0x1000
    }
}
```

### Phase 2: Schema Integration

#### 2.1 Update Schema System

```rust
// csv-adapter-core/src/schema.rs
pub struct TransitionDef {
    pub transition_id: u16,
    pub name: String,
    pub owned_inputs: Vec<StateTypeId>,
    pub owned_outputs: Vec<StateTypeId>,
    pub global_updates: Vec<StateTypeId>,
    pub validation_script: Vec<u8>, // AluVM bytecode
    pub script_type: ScriptType,     // NEW: Track script type
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    PassThrough,  // Current simple pass-through
    AluVM,        // AluVM bytecode
    AluVMSource,  // High-level source (to be compiled)
}
```

#### 2.2 Script Compilation

```rust
impl Schema {
    pub fn compile_script(&self, script: &[u8], script_type: ScriptType) -> Result<Vec<u8>, ScriptError> {
        match script_type {
            ScriptType::PassThrough => Ok(script.to_vec()),
            ScriptType::AluVM => Ok(script.to_vec()), // Already bytecode
            ScriptType::AluVMSource => {
                // Compile high-level source to bytecode
                aluvm::compile(script).map_err(|e| ScriptError::CompilationFailed(e.to_string()))
            }
        }
    }
}
```

### Phase 3: Validation Pipeline Integration

#### 3.1 Update Consignment Validator

```rust
// csv-adapter-core/src/validator.rs
impl ConsignmentValidator {
    pub fn validate_with_vm(
        &self,
        consignment: &Consignment,
        vm: &impl DeterministicVM,
    ) -> ValidationReport {
        // Use provided VM for transition execution
        // instead of hardcoded validation logic
    }
}
```

#### 3.2 Transition Execution

```rust
// csv-adapter-core/src/transition.rs
pub fn execute_transition_with_vm(
    vm: &impl DeterministicVM,
    transition: &Transition,
    state_store: &impl StateStore,
) -> Result<VMOutputs, VMError> {
    // Resolve state references to actual state
    let inputs = resolve_transition_inputs(transition, state_store)?;
    
    // Execute through VM
    vm.execute(
        &transition.validation_script,
        inputs,
        &transition.signatures,
    )
}
```

### Phase 4: CLI Integration

#### 4.1 VM Configuration

```rust
// csv-cli/src/config.rs
pub struct Config {
    // ... existing fields ...
    pub vm_type: VMType,
    pub gas_limit: Option<u64>,
    pub max_steps: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMType {
    PassThrough,
    AluVM { gas_limit: u64, max_steps: u64 },
}
```

#### 4.2 Command Line Options

```bash
# Select VM type
csv right create --vm aluvm --gas-limit 1000000 --max-steps 10000

# Compile high-level scripts
csv script compile --source transfer.alu --output transfer.bytecode

# Execute with specific VM
csv proof verify --vm aluvm --proof proof.json
```

---

## 5. Testing Strategy

### 5.1 Unit Tests

```rust
#[cfg(test)]
mod aluvm_tests {
    use super::*;

    #[test]
    fn test_simple_transfer() {
        let vm = AluVM::new(1000, 100);
        let bytecode = compile_aluvm_script("transfer.alu");
        
        let inputs = VMInputs::new(
            vec![OwnedState::new(10, seal_a, 1000u64.to_le_bytes().to_vec())],
            vec![],
            vec![],
            vec![],
        );
        
        let outputs = vm.execute(&bytecode, inputs, &[]).unwrap();
        
        assert_eq!(outputs.owned_outputs.len(), 1);
        assert_eq!(outputs.owned_outputs[0].data, 1000u64.to_le_bytes().to_vec());
    }

    #[test]
    fn test_state_conservation() {
        let vm = AluVM::new(1000, 100);
        let bytecode = compile_aluvm_script("transfer.alu");
        
        let inputs = VMInputs::new(
            vec![OwnedState::new(10, seal_a, 1000u64.to_le_bytes().to_vec())],
            vec![GlobalState::new(1, 10000u64.to_le_bytes().to_vec())],
            vec![],
            vec![],
        );
        
        let outputs = vm.execute(&bytecode, inputs, &[]).unwrap();
        
        // Verify total supply conservation
        let total_input = 1000u64;
        let total_output = decode_integer(&outputs.owned_outputs[0].data).unwrap();
        assert_eq!(total_input, total_output);
    }
}
```

### 5.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use csv_adapter_bitcoin::BitcoinAnchorLayer;

    #[test]
    fn test_bitcoin_with_aluvm() {
        let adapter = BitcoinAnchorLayer::signet().unwrap();
        let vm = AluVM::new(100000, 1000);
        
        // Create a right with AluVM validation
        let seal = adapter.create_seal(Some(1000)).unwrap();
        let bytecode = compile_aluvm_script("bitcoin_transfer.alu");
        
        let right = Right::new(
            Hash::new([1u8; 32]),
            Hash::new([2u8; 32]),
            OwnershipProof::new(secp256k1::PublicKey::from_slice(&[0u8; 33]).unwrap()),
            vec![],
            None,
            None,
            Some(VMExecutionInfo {
                vm_type: VMType::AluVM,
                bytecode: bytecode.clone(),
            }),
        );
        
        // Execute transfer through AluVM
        let transition = Transition::new(
            1,
            vec![StateRef::new(10, right.commitment, 0)],
            vec![],
            vec![],
            vec![],
            bytecode,
            vec![],
        );
        
        let result = execute_transition_with_vm(&vm, &transition, &adapter);
        assert!(result.is_ok());
    }
}
```

### 5.3 Cross-Chain Tests

```rust
#[test]
fn test_cross_chain_aluvm() {
    // Test that AluVM execution works across different chain adapters
    let bitcoin_adapter = BitcoinAnchorLayer::signet().unwrap();
    let ethereum_adapter = EthereumAnchorLayer::sepolia().unwrap();
    
    let vm = AluVM::new(100000, 1000);
    
    // Create right on Bitcoin with AluVM validation
    let bitcoin_right = create_right_with_aluvm(&bitcoin_adapter, &vm);
    
    // Transfer to Ethereum
    let proof = transfer_right(&bitcoin_right, &ethereum_adapter, &vm);
    
    // Verify on Ethereum side
    assert!(verify_proof(&proof, &ethereum_adapter, &vm));
}
```

### 5.4 Performance Tests

```rust
#[test]
fn test_aluvm_performance() {
    let vm = AluVM::new(1000000, 10000);
    let complex_script = load_complex_script("complex_contract.alu");
    
    let start = std::time::Instant::now();
    let result = vm.execute(&complex_script, test_inputs(), &[]);
    let duration = start.elapsed();
    
    println!("Execution time: {:?}", duration);
    assert!(duration.as_millis() < 1000); // Should complete within 1 second
    assert!(result.is_ok());
}
```

---

## 6. Migration Path

### 6.1 Backward Compatibility

The integration will maintain full backward compatibility:

1. **Default Behavior**: Continue using `PassThroughVM` by default
2. **Feature Flags**: Make AluVM integration optional
3. **Schema Compatibility**: Existing schemas work unchanged
4. **CLI Compatibility**: Existing commands work unchanged

### 6.2 Migration Steps

#### Step 1: Enable AluVM Support

```bash
# Add to Cargo.toml
[features]
aluvm = ["dep:aluvm"]

# Build with AluVM support
cargo build --features aluvm
```

#### Step 2: Update Schema Definitions

```rust
// Old schema (still works)
let schema = Schema::fungible_token(schema_id, "MyToken");

// New schema with AluVM
let schema = Schema::new(
    schema_id,
    "MyToken",
    global_types,
    owned_types,
    vec![
        TransitionDef::new(
            1,
            "transfer",
            vec![10],
            vec![10],
            vec![],
            compile_aluvm_script("transfer.alu"),
            ScriptType::AluVM,
        ),
    ],
    vec![],
);
```

#### Step 3: Update CLI Usage

```bash
# Old command (still works)
csv right create --chain bitcoin --value 1000

# New command with AluVM
csv right create --chain bitcoin --value 1000 --vm aluvm --script transfer.alu
```

#### Step 4: Gradual Migration

1. **Phase 1**: Deploy new contracts with AluVM alongside existing ones
2. **Phase 2**: Migrate simple contracts to AluVM for enhanced security
3. **Phase 3**: Migrate complex contracts to leverage AluVM's capabilities
4. **Phase 4**: Make AluVM the default for new contracts

### 6.3 Testing Migration

```rust
#[test]
fn test_migration_compatibility() {
    // Test that old schemas still work
    let old_schema = Schema::fungible_token(old_id, "OldToken");
    assert!(old_schema.validate_transition(&old_transition).is_ok());
    
    // Test that new schemas work with AluVM
    let new_schema = Schema::with_aluvm(new_id, "NewToken");
    assert!(new_schema.validate_transition(&new_transition).is_ok());
    
    // Test cross-compatibility
    let old_right = Right::from_schema(&old_schema);
    let new_right = Right::from_schema(&new_schema);
    
    // Both should work in the same system
    assert!(validate_right(&old_right).is_ok());
    assert!(validate_right(&new_right).is_ok());
}
```

---

## 7. Security Considerations

### 7.1 VM Security

1. **Memory Safety**: AluVM provides sandboxed execution
2. **Gas Limits**: Prevent infinite loops and resource exhaustion
3. **Step Limits**: Prevent computational DoS attacks
4. **Signature Verification**: Proper cryptographic verification

### 7.2 State Validation

1. **Type Safety**: Strong typing prevents invalid state transitions
2. **Conservation Rules**: Enforce supply and state conservation
3. **Seal Binding**: Ensure state is properly bound to seals
4. **Cross-Chain Validation**: Prevent double-spending across chains

### 7.3 Error Handling

1. **Graceful Failure**: VM errors should not crash the system
2. **Detailed Logging**: Provide sufficient debugging information
3. **Recovery Mechanisms**: Handle execution failures gracefully
4. **Audit Trail**: Maintain logs of all VM executions

---

## 8. Performance Considerations

### 8.1 Optimization Opportunities

1. **Bytecode Caching**: Cache compiled bytecode for reuse
2. **Memory Management**: Efficient memory allocation strategies
3. **Parallel Execution**: Execute independent transitions in parallel
4. **Lazy Loading**: Load bytecode only when needed

### 8.2 Benchmarking

1. **Execution Speed**: Measure time per operation
2. **Memory Usage**: Track memory consumption
3. **Gas Efficiency**: Optimize gas usage patterns
4. **Throughput**: Measure transactions per second

### 8.3 Scalability

1. **Large Contracts**: Handle contracts with many state transitions
2. **Complex Logic**: Support sophisticated validation rules
3. **Cross-Chain**: Maintain performance across chain boundaries
4. **Concurrent Access**: Support multiple concurrent validations

---

## 9. Future Enhancements

### 9.1 Advanced Features

1. **ZK Integration**: Support for zero-knowledge proofs
2. **Multi-Party Computation**: Support for collaborative contracts
3. **Oracle Integration**: External data feeds
4. **Cross-VM Communication**: Interoperability with other VMs

### 9.2 Developer Experience

1. **High-Level Languages**: Support for languages like Rust or Python
2. **Tooling**: Development tools and IDE integration
3. **Documentation**: Comprehensive documentation and examples
4. **Testing Framework**: Enhanced testing capabilities

### 9.3 Ecosystem Integration

1. **Standard Libraries**: Common contract patterns and utilities
2. **Community Contributions**: Open-source development
3. **Industry Standards**: Alignment with blockchain standards
4. **Academic Research**: Collaboration with research institutions

---

## 10. Conclusion

AluVM integration will significantly enhance the CSV Adapter project by providing:

1. **Enhanced Security**: Sandboxed execution with memory safety
2. **Expressive Power**: Support for complex contract logic
3. **Determinism**: Guaranteed consistency across environments
4. **Performance**: Efficient bytecode execution
5. **Extensibility**: Easy to add new features and capabilities

The integration plan maintains full backward compatibility while providing a clear migration path for existing users. The comprehensive testing strategy ensures reliability and performance across all supported chains.

### Next Steps

1. **Implementation**: Begin with Phase 1 core integration
2. **Testing**: Implement the comprehensive test suite
3. **Documentation**: Create developer documentation and examples
4. **Community**: Engage with the community for feedback and contributions
5. **Deployment**: Plan for gradual deployment and migration

---

## Appendix

### A. AluVM Resources

- [AluVM Documentation](https://aluvm.dev/)
- [AluVM GitHub Repository](https://github.com/aluvm/aluvm)
- [AluVM Specification](https://aluvm.dev/specification)

### B. CSV Adapter Resources

- [CSV Adapter Documentation](./README.md)
- [Cross-Chain Specification](./CROSS_CHAIN_SPEC.md)
- [Blueprint](./Blueprint.md)

### C. Contact Information

For questions or contributions:

- GitHub Issues: [csv-adapter Issues](https://github.com/zorvan/csv-adapter/issues)
- AluVM Community: [AluVM Discord](https://discord.gg/aluvm)
- Email: <dev@csv-adapter.com>
