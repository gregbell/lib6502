# Advanced Testing Guide

This document describes the advanced testing infrastructure for lib6502, including property-based testing, fuzz testing, and formal verification.

## Overview

The project uses three complementary techniques to achieve high test coverage:

| Technique | Tool | Purpose |
|-----------|------|---------|
| Property-based testing | `proptest` | Generate random inputs to test invariants |
| Fuzz testing | `cargo-fuzz` + `libfuzzer` | Coverage-guided crash detection |
| Formal verification | `kani` | Mathematical proofs of correctness |

## Property-Based Testing

Property-based tests generate thousands of random inputs and verify that certain invariants always hold.

### Running Property Tests

```bash
# Default run (256 cases per property)
cargo test proptest_

# Extended run (100,000 cases)
PROPTEST_CASES=100000 cargo test --release proptest_ -- --test-threads=1

# Run specific property test file
cargo test --test proptest_cpu
cargo test --test proptest_addressing
cargo test --test proptest_assembler
cargo test --test proptest_roundtrip
```

### Test Files

| File | Properties Tested |
|------|-------------------|
| `tests/proptest_cpu.rs` | PC advancement, cycle counting, flags (N, Z, C, V), ALU operations |
| `tests/proptest_addressing.rs` | Zero-page wrap, page crossing, indirect addressing, branches |
| `tests/proptest_assembler.rs` | Number format equivalence, addressing mode selection, error handling |
| `tests/proptest_roundtrip.rs` | Assemble-disassemble-assemble identity |

### Key Properties

**CPU Properties:**
- `new_pc = old_pc.wrapping_add(size_bytes)` for all instructions
- `new_cycles = old_cycles + base_cycles + page_crossing_penalty`
- `flag_n = (result & 0x80) != 0` for all ALU operations
- `flag_z = result == 0` for all ALU operations
- `flag_c = (a + m + carry) > 0xFF` for ADC
- Stack operations wrap correctly at boundaries

**Addressing Mode Properties:**
- Zero-page addressing wraps within page 0
- Page crossing detection adds exactly 1 cycle
- Indirect JMP bug at page boundary is replicated

**Assembler Properties:**
- `$42`, `66`, and `%01000010` produce identical output
- Values 0-255 use zero-page addressing, >255 use absolute
- Invalid input returns `Err`, never panics

### Regression Files

When proptest finds a failing case, it saves it to `proptest-regressions/`. These files are committed to git and re-run on every test execution to prevent regressions.

## Fuzz Testing

Fuzz testing uses coverage-guided mutation to find edge cases and crashes.

### Prerequisites

```bash
# Install cargo-fuzz (requires nightly Rust)
cargo install cargo-fuzz

# Verify installation
cargo +nightly fuzz --help
```

### Running Fuzz Targets

```bash
cd fuzz

# Run a specific target indefinitely
cargo +nightly fuzz run fuzz_cpu_step

# Run for a limited time
cargo +nightly fuzz run fuzz_cpu_step -- -max_total_time=300

# Run all targets (60 seconds each)
cargo +nightly fuzz run fuzz_cpu_step -- -max_total_time=60
cargo +nightly fuzz run fuzz_assembler -- -max_total_time=60
cargo +nightly fuzz run fuzz_disassembler -- -max_total_time=60
```

### Fuzz Targets

| Target | Description |
|--------|-------------|
| `fuzz_cpu_step` | Random CPU state + memory, execute one instruction |
| `fuzz_assembler` | Random ASCII/UTF-8 input to assembler |
| `fuzz_disassembler` | Random byte sequences to disassembler |

### Corpus and Crashes

- **Corpus:** `fuzz/corpus/<target>/` - Interesting inputs found by fuzzer
- **Crashes:** `fuzz/artifacts/<target>/` - Inputs that caused crashes

To reproduce a crash:
```bash
cargo +nightly fuzz run fuzz_cpu_step fuzz/artifacts/fuzz_cpu_step/crash-xxxxx
```

## Formal Verification with Kani

Kani uses bounded model checking to mathematically prove properties hold for ALL possible inputs.

### Prerequisites

```bash
# Install Kani
cargo install --locked kani-verifier
kani setup
```

### Running Kani Proofs

```bash
# Run all proofs
cargo kani --tests

# Run specific proof
cargo kani --tests --harness proof_stack_address_always_in_stack_page
```

### Proofs Included

| Proof | Property Verified |
|-------|-------------------|
| `proof_stack_address_always_in_stack_page` | Stack address in 0x0100-0x01FF |
| `proof_stack_address_high_byte` | Stack address high byte is 0x01 |
| `proof_n_flag_computation` | N flag set iff bit 7 set |
| `proof_z_flag_computation` | Z flag set iff value is zero |
| `proof_carry_flag_addition` | Carry flag correct for ADC |
| `proof_overflow_flag_addition` | Overflow flag correct for signed ADC |
| `proof_zero_page_x_wrap` | ZP+X wraps within zero page |
| `proof_zero_page_y_wrap` | ZP+Y wraps within zero page |
| `proof_page_crossing_detection` | Page crossing detected correctly |
| `proof_forward_branch_calculation` | Forward branch target correct |
| `proof_backward_branch_calculation` | Backward branch offset is negative |
| `proof_all_opcode_sizes_valid` | All opcodes have size 1-3 bytes |
| `proof_all_opcode_cycles_reasonable` | Implemented opcodes have 2-7 cycles |
| `proof_inx_wrap` | INX wraps 0xFF to 0x00 |
| `proof_dex_wrap` | DEX wraps 0x00 to 0xFF |
| `proof_status_register_bit_layout` | Status register bits in correct positions |
| `proof_asl_operation` | ASL doubles value, carry is bit 7 |
| `proof_lsr_operation` | LSR halves value, carry is bit 0 |
| `proof_rol_operation` | ROL is reversible with ROR |
| `proof_ror_operation` | ROR is reversible with ROL |

### Kani vs proptest

| Aspect | proptest | Kani |
|--------|----------|------|
| Coverage | Statistical (random sampling) | Complete (all inputs) |
| Speed | Fast (seconds) | Slow (minutes per proof) |
| Guarantees | High confidence | Mathematical proof |
| Use case | Finding bugs quickly | Proving absence of bugs |

## CI Integration

The CI workflow runs advanced testing automatically:

```yaml
# Standard tests (every push)
cargo test

# Extended property tests (100k cases)
PROPTEST_CASES=100000 cargo test --release proptest_

# Fuzz testing (60s per target)
cargo +nightly fuzz run <target> -- -max_total_time=60

# Kani verification (optional, continue-on-error)
cargo kani --tests
```

## Adding New Tests

### Adding a Property Test

```rust
// In tests/proptest_cpu.rs
proptest! {
    #[test]
    fn test_my_property(value in 0u8..=255u8) {
        // Setup
        let mut memory = FlatMemory::new();
        let mut cpu = CPU::new(memory);

        // Exercise
        // ...

        // Verify property
        prop_assert!(some_invariant_holds);
    }
}
```

### Adding a Fuzz Target

1. Create `fuzz/fuzz_targets/fuzz_new_target.rs`:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercise the code with arbitrary input
    // No panics = success
});
```

2. Add to `fuzz/Cargo.toml`:
```toml
[[bin]]
name = "fuzz_new_target"
path = "fuzz_targets/fuzz_new_target.rs"
test = false
doc = false
bench = false
```

### Adding a Kani Proof

```rust
// In tests/kani_proofs.rs
#[kani::proof]
fn proof_my_invariant() {
    let value: u8 = kani::any();

    // Optionally constrain input
    kani::assume(value > 0);

    // Verify property
    kani::assert(some_property(value), "Property must hold");
}
```

## Troubleshooting

### proptest: "Too many shrink steps"
The test found a failing case but couldn't shrink it. The original failing input is still saved in `proptest-regressions/`.

### cargo-fuzz: "AddressSanitizer: stack-buffer-overflow"
The fuzzer found a real bug. Check `fuzz/artifacts/` for the reproducing input.

### Kani: "Verification failed"
Kani found a counterexample. The output shows the specific values that violate the property.

### Kani: "Unwinding assertion loop"
Add `#[kani::unwind(N)]` attribute to increase the unwind bound for loops.
