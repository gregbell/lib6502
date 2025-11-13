# Implementation Plan: CPU Core Foundation

**Branch**: `001-cpu-core-foundation` | **Date**: 2025-11-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-cpu-core-foundation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create the foundational Rust project structure and CPU core architecture for a cycle-accurate NMOS 6502 emulator. Establishes CPU state structures, trait-based memory bus abstraction, skeletal fetch-decode-execute loop, and table-driven opcode metadata covering all 256 opcodes. No instruction implementations included—this provides the architectural foundation for subsequent per-instruction feature work.

## Technical Context

**Language/Version**: Rust (stable - 1.91.1 at the time of writing)
**Primary Dependencies**: None required for core module (test dependencies: standard Rust test framework)
**Storage**: N/A (CPU core operates on memory abstraction, storage is out of scope)
**Testing**: `cargo test` with standard `#[test]` framework, WASM compatibility validation with `wasm32-unknown-unknown` target
**Target Platform**: Cross-platform (native Linux/macOS/Windows + WebAssembly via wasm32-unknown-unknown)
**Project Type**: Single library project (CPU core module)
**Performance Goals**: NEEDS CLARIFICATION (baseline cycle-accurate execution, specific throughput targets TBD)
**Constraints**: WASM-compatible (no OS dependencies, no threading/async in core), deterministic execution, cycle-accurate timing
**Scale/Scope**: Single CPU core module, ~256 opcode metadata entries, foundational structures for ~50+ instruction implementations in future features

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

### I. Modularity & Separation of Concerns

- ✅ **PASS**: CPU state contains only registers, flags, PC, SP, cycle counter—no memory arrays
- ✅ **PASS**: All memory access goes through trait-based bus abstraction (MemoryBus trait requirement in FR-004)
- ✅ **PASS**: No OS-level features or platform-specific code (WASM portability constraint)
- ✅ **PASS**: Decoupled from specific machine implementations (flat RAM test implementation proves abstraction works)

### II. WebAssembly Portability

- ✅ **PASS**: No `std::fs`, `std::net`, `std::process` dependencies (core module only, FR-014 requires WASM compilation)
- ✅ **PASS**: No threading or async runtime in core (out of scope)
- ✅ **PASS**: Deterministic execution (no system time or randomness)
- ✅ **PASS**: Success criterion SC-002 explicitly requires wasm32-unknown-unknown compilation

### III. Cycle Accuracy

- ✅ **PASS**: Opcode metadata table includes cycle cost for all 256 opcodes (FR-007)
- ✅ **PASS**: Cycle counter tracked and incremented per instruction (FR-010)
- ⚠️ **DEFERRED**: Page-crossing penalties, branch timing—deferred to instruction implementation features (this feature establishes foundation only)
- ✅ **PASS**: Flexible clocking via run-for-cycles method (FR-012)

### IV. Clarity & Hackability

- ✅ **PASS**: Public API requires doc comments (SC-009: documentation for instantiation, memory bus, execution)
- ✅ **PASS**: Structural simplicity (this feature intentionally minimal—no instruction implementations, minimal complexity)
- ✅ **PASS**: Test coverage expectation of 80% for structures (SC-010)

### V. Table-Driven Design

- ✅ **PASS**: Opcode metadata table required (FR-007) covering all 256 opcodes
- ✅ **PASS**: Addressing modes as reusable enumeration (FR-009)
- ✅ **PASS**: Single source of truth for mnemonic, mode, cycle cost, size (opcode table requirement)
- ✅ **PASS**: Unimplemented instructions clearly marked (FR-008)

**GATE STATUS**: ✅ **APPROVED TO PROCEED** (all principles satisfied, one timing detail appropriately deferred to instruction features)

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── lib.rs              # Library root, re-exports CPU and MemoryBus
├── cpu.rs              # CPU struct with state (registers, flags, PC, SP, cycles)
├── memory.rs           # MemoryBus trait definition
├── opcodes.rs          # Opcode metadata table (all 256 opcodes)
├── addressing.rs       # AddressingMode enum and related types
└── instruction.rs      # Instruction struct (decoded opcode + operands)

tests/
├── cpu_init_test.rs    # Verify CPU initialization with correct reset values
├── memory_bus_test.rs  # Test MemoryBus trait with simple RAM implementation
└── execute_loop_test.rs # Skeletal execute loop test (fetch-decode-execute cycle)

examples/
└── simple_ram.rs       # Example: FlatMemory implementation of MemoryBus (64KB)

Cargo.toml              # Project manifest (edition 2021, no external dependencies)
```

**Structure Decision**: Single library project (Option 1). The CPU core is a standalone Rust library crate designed to be embedded in larger projects (fantasy consoles, emulators). Module organization follows separation of concerns: state (cpu.rs), abstractions (memory.rs), data (opcodes.rs/addressing.rs), and behavior (instruction.rs for decode logic). Tests validate structural correctness and trait-based abstraction.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

_No violations detected. Constitution Check passed all gates._
