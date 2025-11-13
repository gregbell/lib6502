# Research: CPU Core Foundation

**Feature**: 001-cpu-core-foundation
**Date**: 2025-11-13
**Phase**: 0 - Outline & Research

This document consolidates architectural decisions, technical research, and design rationale for the CPU core foundation. All "NEEDS CLARIFICATION" items from Technical Context have been resolved.

## Performance Goals Resolution

**Decision**: No specific throughput targets for this foundational feature. Baseline cycle-accurate execution is the goal.

**Rationale**: Performance optimization is explicitly out of scope for this feature (per spec). The constitution prioritizes clarity and hackability over raw performance. Initial implementation focuses on correctness and architectural soundness. Future performance work can profile and optimize hot paths (likely in the instruction execution loop) once a representative instruction set is implemented.

**Alternatives Considered**:
- Setting arbitrary performance targets (e.g., "1M cycles/sec") without benchmarking infrastructure → Premature optimization without data
- Including micro-benchmarks in this feature → Scope creep, contradicts constitution's simplicity principle

## CPU State Representation

**Decision**: Use a simple struct with individual fields for each register (A, X, Y), status flags (as individual bools or bitfield), 16-bit PC, 8-bit SP, and u64 cycle counter.

**Rationale**: Rust's strong typing makes individual fields clearer than arrays or magic indices. The 6502 has few enough registers that field access is more readable than indexing. Status flags can be represented as either individual bool fields (clearer for beginners) or a packed u8 bitfield (more authentic to hardware). Given the constitution's Clarity & Hackability principle, individual bool fields are preferred unless performance profiling demands otherwise.

**6502 Reset Values** (from architecture docs):
- **PC**: Loaded from reset vector at 0xFFFC/0xFFFD (little-endian)
- **SP**: 0xFD (per common NMOS behavior, though technically undefined)
- **Status Register**: 0x24 (Interrupt Disable flag set, bit 5 always 1)
- **A, X, Y**: Undefined (initialize to 0x00 for determinism)

**Alternatives Considered**:
- Packed register array → Less readable, no performance benefit for 3 registers
- Status register as u8 bitfield → More authentic but less hackable (defer to future if profiling demands it)

## Memory Bus Abstraction

**Decision**: Define a `MemoryBus` trait with `read(&self, addr: u16) -> u8` and `write(&mut self, addr: u16, value: u8)` methods.

**Rationale**: Trait-based abstraction enables modularity (constitution principle I) and supports diverse memory implementations (flat RAM, memory-mapped I/O, NES-style banking, debugging wrappers). Immutable self on read enables shared references. Mutable self on write prevents data races and makes memory modification explicit. 16-bit address space is architecturally correct for 6502 (64KB addressable).

**WASM Compatibility**: Traits with simple signatures compile cleanly to WASM. No platform-specific code required.

**Alternatives Considered**:
- Function pointers for read/write → Less ergonomic, no trait-based polymorphism benefits
- Returning `Result<u8, BusError>` from read/write → Adds error handling complexity; 6502 hardware has no bus error mechanism (reads always succeed, may return garbage)

## Opcode Table Design

**Decision**: Use a 256-element const array of `OpcodeMetadata` structs, indexed by opcode byte value.

**Structure**:
```rust
struct OpcodeMetadata {
    mnemonic: &'static str,        // e.g., "LDA", "STA", "???" for illegal
    addressing_mode: AddressingMode,
    base_cycles: u8,
    size_bytes: u8,
    implemented: bool,              // false for this feature, true when instruction added
}
```

**Rationale**: Const array provides O(1) lookup by opcode. Static string slices have zero runtime cost. The `implemented` flag allows the decoder to distinguish between "not yet implemented" (future work) and "illegal opcode" (never will be). Size and cycle information centralizes timing/decode logic per constitution principle V (Table-Driven Design).

**Illegal Opcodes**: NMOS 6502 has 151 documented opcodes and 105 undocumented/illegal opcodes. Mark undocumented opcodes with mnemonic "???" and `implemented: false`. Document this choice—future work can implement undocumented opcodes if needed for specific emulation targets.

**Alternatives Considered**:
- HashMap for opcode lookup → Runtime overhead, unnecessary for dense 256-element space
- Match statement decoder → Violates table-driven principle, duplicates metadata across branches
- Separate arrays for mnemonic/mode/cycles → More C-like, less ergonomic in Rust

## Addressing Mode Representation

**Decision**: Define an enum covering all 13 addressing modes from the 6502 reference docs.

```rust
enum AddressingMode {
    Implicit,        // Implied (no operand)
    Accumulator,     // Operates on accumulator register
    Immediate,       // 8-bit constant operand
    ZeroPage,        // 8-bit zero page address
    ZeroPageX,       // Zero page indexed by X
    ZeroPageY,       // Zero page indexed by Y
    Relative,        // Signed 8-bit branch offset
    Absolute,        // 16-bit address
    AbsoluteX,       // Absolute indexed by X
    AbsoluteY,       // Absolute indexed by Y
    Indirect,        // Indirect jump (JMP only)
    IndirectX,       // Indexed indirect (via X)
    IndirectY,       // Indirect indexed (via Y)
}
```

**Rationale**: Enum provides type safety and exhaustive match checking. Names match reference documentation for clarity. This enum will be used by both the opcode table (to specify mode per instruction) and future addressing mode resolution logic.

**Note on Terminology**: Reference docs use both "Implicit" and "Implied" interchangeably. We standardize on "Implicit" for consistency with assembly language terminology.

**Alternatives Considered**:
- Integer constants for modes → No type safety, easy to misuse
- Splitting into separate addressing vs indexing enums → Overengineered for 13 total modes

## Fetch-Decode-Execute Loop

**Decision**: Implement a `step(&mut self) -> Result<(), ExecutionError>` method that executes one instruction.

**Execution Flow**:
1. **Fetch**: Read opcode byte at PC, increment PC
2. **Decode**: Look up opcode in metadata table
3. **Check implementation**: If `implemented == false`, return `Err(ExecutionError::UnimplementedOpcode(opcode))`
4. **Execute**: For this feature, since no instructions are implemented, always return UnimplementedOpcode
5. **Update cycles**: Increment cycle counter by base_cycles from table

**Error Handling**: Use `Result<(), ExecutionError>` to signal unimplemented opcodes or future error conditions. This allows callers to detect and handle errors (e.g., halt emulation, log error) rather than panicking.

**Alternatives Considered**:
- Panicking on unimplemented opcodes → Not production-ready, violates Rust error handling conventions
- Returning bool (true = success, false = error) → Loses error context
- No-op for unimplemented instructions → Silently incorrect, violates cycle accuracy

## Cycle Counting

**Decision**: Use `u64` for cycle counter to avoid overflow in long-running programs.

**Rationale**: 6502 at 1 MHz executes ~1 million cycles/second. A u32 would overflow after ~71 minutes. A u64 supports centuries of runtime at realistic clock speeds. Cycle-accurate timing is a core constitutional requirement (principle III), so the counter must never overflow during practical use.

**Frame-Based Execution**: Provide a `run_for_cycles(&mut self, cycle_budget: u64) -> Result<u64, ExecutionError>` method that executes instructions until the cycle budget is exhausted, returning cycles consumed. This supports frame-locked execution models (e.g., running CPU for exactly 29780 cycles per 60Hz NTSC frame).

**Alternatives Considered**:
- u32 cycle counter → Overflow risk in long-running programs
- Separate frame cycle counter → Unnecessary complexity, caller can track delta

## Rust Project Structure

**Decision**: Use standard Rust library crate (Cargo.toml with `[lib]` section), edition 2021, no external dependencies.

**Cargo.toml**:
```toml
[package]
name = "cpu6502"
version = "0.1.0"
edition = "2021"

[lib]
name = "cpu6502"
path = "src/lib.rs"

[dependencies]
# None - core module is dependency-free

[dev-dependencies]
# None - use std test framework
```

**WASM Validation**: Include a CI check (or manual verification step) to ensure `cargo build --target wasm32-unknown-unknown` succeeds. This enforces constitution principle II (WebAssembly Portability).

**Alternatives Considered**:
- Including serde for state serialization → Out of scope, can be added later if needed
- Using nightly Rust features → Violates portability, stable is required

## Testing Strategy

**Decision**: Use standard `#[test]` functions in `tests/` directory, organized by concern (cpu_init_test.rs, memory_bus_test.rs, execute_loop_test.rs).

**Test Coverage Goals** (per SC-010):
- CPU initialization: Verify all registers initialized to correct reset values
- MemoryBus trait: Implement `FlatMemory` (64KB array) and verify read/write through trait
- Execute loop: Load simple program (e.g., single opcode), call step(), verify UnimplementedOpcode error
- Opcode table: Verify all 256 opcodes have metadata entries (even if not implemented)

**Cycle Accuracy Testing**: For this feature, only verify cycle counter increments by base_cycles per instruction. Detailed cycle accuracy tests (page crossings, branch timing) deferred to instruction implementation features.

**Alternatives Considered**:
- Using external test framework (e.g., rstest) → Unnecessary dependency for simple tests
- Embedding tests in src/ modules → Less clear separation, tests/ is Rust convention

## Documentation Requirements

**Decision**: Use rustdoc comments (`///`) for all public APIs. Include examples for CPU initialization and memory bus implementation.

**Required Docs** (per SC-009):
- How to instantiate CPU struct
- How to implement MemoryBus trait
- How to execute instructions via step() method
- How to inspect CPU state (registers, flags, PC, SP, cycle count)

**Example Code**: Provide `examples/simple_ram.rs` demonstrating FlatMemory implementation and basic CPU execution loop.

**Alternatives Considered**:
- Separate markdown docs → Rustdoc keeps code and docs synchronized
- No examples → Violates SC-009 documentation requirement

## Open Questions & Future Work

**Interrupt Handling**: This feature establishes CPU state structures but does not implement interrupt logic (IRQ, NMI, BRK execution paths). The status register includes the Interrupt Disable flag, but no interrupt state machine. This is explicitly out of scope per the spec.

**Undocumented Opcodes**: All 105 illegal/undocumented NMOS 6502 opcodes are marked as unimplemented in the opcode table. Future work can implement these if needed for specific emulation targets (e.g., Commodore 64 software that relies on undocumented opcodes).

**Status Register Representation**: Initial implementation uses individual bool fields for clarity. If profiling reveals performance issues, the status register can be refactored to a packed u8 bitfield with getter/setter methods. This is a non-breaking internal change.

**Reset Behavior**: PC is initialized by reading the reset vector (0xFFFC/0xFFFD). For this feature, we'll document that the memory bus must be initialized with a valid reset vector before calling CPU initialization. Future work might add a `reset(&mut self)` method that re-reads the reset vector.

## References

- 6502 Architecture: docs/6502-reference/Architecture.md
- 6502 Registers: docs/6502-reference/Registers.md
- 6502 Addressing Modes: docs/6502-reference/Addressing-Modes.md
- 6502 Instruction Set: docs/6502-reference/Instructions.md
- Project Constitution: .specify/memory/constitution.md
