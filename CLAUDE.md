# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A cycle-accurate NMOS 6502 CPU emulator built in Rust. The project prioritizes modularity, clarity, WebAssembly portability, and hackability. No external dependencies are used in the core library.

## Constitution

The project uses a CONSTITUTION.md file (.specify/memory/constitution.md) to manage a set of core principles to follow. Review them to ensure that new work matches hte project style, architecture, and vision.

## Architecture

### Core Abstractions

The emulator uses a **trait-based architecture** to decouple CPU logic from memory implementation:

- **CPU<M: MemoryBus>**: Generic over memory implementation, contains all processor state (registers, flags, PC, SP, cycle counter)
- **MemoryBus trait**: Provides `read(&self, addr: u16) -> u8` and `write(&mut self, addr: u16, value: u8)`
- **FlatMemory**: Simple 64KB RAM implementation of MemoryBus
- **OPCODE_TABLE**: Static 256-entry metadata table mapping opcodes to mnemonic, addressing mode, cycle cost, and size

### Execution Model

The fetch-decode-execute loop is driven by:

- `CPU::step()` - Execute one instruction and return Result
- `CPU::run_for_cycles(budget)` - Execute until cycle budget exhausted
- Unimplemented opcodes return `Err(ExecutionError::UnimplementedOpcode(u8))`

### Module Structure

```
src/
  lib.rs         - Public API and error types
  cpu.rs         - CPU state and execution logic
  memory.rs      - MemoryBus trait and FlatMemory impl
  opcodes.rs     - OPCODE_TABLE metadata
  addressing.rs  - AddressingMode enum
tests/           - Integration tests (separate from unit tests in src/)
examples/        - Usage examples
specs/           - Feature specifications and planning docs
```

### Table-Driven Design

All opcode information lives in `OPCODE_TABLE`. When implementing instructions:

1. Look up metadata via `OPCODE_TABLE[opcode as usize]`
2. Use `metadata.addressing_mode` to determine how to fetch operands
3. Add `metadata.base_cycles` to cycle counter (plus page-crossing penalties)
4. Advance PC by `metadata.size_bytes`

The `get_operand_value()` helper handles all addressing modes and returns `(value, page_crossed)`.

## Common Commands

```bash
# Build the library
cargo build

# Run fast tests (TDD workflow - skips slow functional test)
cargo test

# Run all tests including slow functional test
cargo test -- --include-ignored

# Run only the Klaus functional test
cargo test --test functional_klaus klaus_6502_functional_test -- --ignored --nocapture

# Run a specific test
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt

# Run examples
cargo run --example simple_ram
```

## Test Suites

The project has two test categories:

**Fast Tests** (default: `cargo test`)
- 1,470+ unit and integration tests
- Complete in ~2 seconds
- Perfect for TDD workflow
- Run automatically on every `cargo test`

**Functional Tests** (run with `--ignored`)
- Klaus Dormann's 6502 functional test (~6 seconds)
- Validates all 151 opcodes with 96M+ instruction cycles
- Marked as `#[ignore]` to skip during TDD
- Run explicitly with: `cargo test -- --ignored`
- CI runs both test suites separately

See [docs/KLAUS_FUNCTIONAL_TEST.md](docs/KLAUS_FUNCTIONAL_TEST.md) for details.

## Testing Patterns

- Unit tests live in `mod tests` at bottom of source files
- Integration tests live in `tests/` directory
- CPU state is inspectable via public getter methods: `cpu.a()`, `cpu.pc()`, `cpu.flag_c()`, etc.
- CPU state can be set via public setters for testing: `cpu.set_a(value)`, `cpu.set_flag_c(true)`
- Use `cpu.memory_mut()` to access memory for test setup

## Implementation Workflow

When adding new instructions:

1. Mark `implemented: true` in OPCODE_TABLE for the relevant opcodes
2. Add match arm in `CPU::step()` to dispatch based on `metadata.mnemonic`
3. Implement instruction logic in a private `execute_xxx()` method
4. Use `get_operand_value()` to handle addressing modes
5. Update flags (N, Z, C, V as appropriate)
6. Update cycle counter (base + page crossing)
7. Advance PC by instruction size
8. Add comprehensive tests in `tests/` directory
9. Run `cargo test`, `cargo clippy`, and `cargo fmt`

## Key Design Constraints

- **No external dependencies** in core library (only dev-dependencies for testing)
- **No OS dependencies** - must work in WebAssembly
- **No panics in MemoryBus** - reads/writes always succeed (matching real 6502 hardware)
- **Individual bool fields for flags** - not packed into status register byte (stored as `flag_n`, `flag_z`, etc.)
- **Cycle accuracy** - track exact cycle counts including page-crossing penalties

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

## Active Technologies
- Rust 1.75+ (edition 2021) + None (zero external dependencies for core library - `no_std` compatible) (002-assembler-disassembler)
- N/A (operates on in-memory byte slices and strings) (002-assembler-disassembler)
- Rust 1.75+ (for WASM compilation), HTML5/CSS3/JavaScript ES6+ (for frontend) (003-wasm-web-demo)
- N/A (fully client-side, no persistence) (003-wasm-web-demo)
- N/A (in-memory state only, no persistence) (004-memory-mapping-module)

## Recent Changes
- 002-assembler-disassembler: Added Rust 1.75+ (edition 2021) + None (zero external dependencies for core library - `no_std` compatible)
