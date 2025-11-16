# Implementation Plan: Assembler & Disassembler

**Branch**: `002-assembler-disassembler` | **Date**: 2025-11-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-assembler-disassembler/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a 6502 assembler and disassembler as a library module, compiling to WebAssembly. The disassembler converts binary machine code to assembly mnemonics using the existing OPCODE_TABLE. The assembler parses 6502 assembly source (mnemonics, labels, directives) and encodes it to binary, with multi-pass label resolution, comprehensive error collection, and structured output for IDE integration including source maps and symbol tables.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (zero external dependencies for core library - `no_std` compatible)
**Storage**: N/A (operates on in-memory byte slices and strings)
**Testing**: cargo test (unit tests in source files + integration tests in tests/ directory)
**Target Platform**: WebAssembly (wasm32-unknown-unknown) + native (Linux, macOS, Windows)
**Project Type**: Single library crate with modular architecture
**Performance Goals**: Disassemble 10,000 bytes/ms, assemble 8KB programs without degradation
**Constraints**: Zero external dependencies, no OS dependencies, no panics in public APIs, deterministic execution
**Scale/Scope**: Handle 64KB programs (full 6502 address space), support all 151 documented opcodes + 105 illegal opcodes

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

### I. Modularity & Separation of Concerns ✅ PASS

- Assembler and disassembler are independent modules with clean public APIs
- No direct dependency on CPU core (uses shared OPCODE_TABLE and AddressingMode enum)
- Can be used independently or together
- No memory implementation coupling - operates on byte slices and strings

### II. WebAssembly Portability ✅ PASS

- Pure Rust string/byte processing with no OS dependencies
- No `std::fs`, `std::net`, `std::process`, or syscalls
- No threading or async runtime requirements
- Deterministic execution (no system time or randomness)
- All core dependencies are `no_std` compatible (zero dependencies)
- Explicitly targets `wasm32-unknown-unknown`

### III. Cycle Accuracy ⚠️ NOT APPLICABLE

- Assembler/disassembler are static analysis tools, not runtime CPU components
- They reference cycle costs from OPCODE_TABLE but do not execute instructions
- No timing-sensitive behavior

### IV. Clarity & Hackability ✅ PASS

- Table-driven design leverages existing OPCODE_TABLE for encoding/decoding
- Parser and formatter logic is straightforward Rust string processing
- Public APIs have clear documentation with examples
- Educational value: developers can learn 6502 assembly syntax and encoding rules
- No clever optimizations that obscure logic

### V. Table-Driven Design ✅ PASS

- Disassembler uses OPCODE_TABLE as single source of truth for decoding
- Assembler uses OPCODE_TABLE to map (mnemonic, addressing mode) → opcode byte
- No duplication of opcode metadata
- Adding new instructions only requires updating OPCODE_TABLE (already complete)

**Overall Assessment**: All applicable principles satisfied. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/002-assembler-disassembler/
├── plan.md              # This file (implementation plan)
├── spec.md              # Feature specification
├── research.md          # Phase 0: Research findings and design decisions
├── data-model.md        # Phase 1: Core data structures and relationships
├── quickstart.md        # Phase 1: Quick examples and usage guide
├── contracts/           # Phase 1: API contracts
│   ├── assembler-api.md
│   └── disassembler-api.md
└── tasks.md             # Phase 2: Implementation tasks (NOT YET CREATED - use /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Public API exports
├── cpu.rs               # Existing: CPU core
├── memory.rs            # Existing: MemoryBus trait and FlatMemory
├── opcodes.rs           # Existing: OPCODE_TABLE (used by assembler/disassembler)
├── addressing.rs        # Existing: AddressingMode enum (used by assembler/disassembler)
├── assembler.rs         # NEW: Assembler module
│   ├── parser.rs        # NEW: Assembly source parser
│   ├── encoder.rs       # NEW: Instruction encoder
│   ├── symbol_table.rs  # NEW: Symbol table management
│   └── source_map.rs    # NEW: Source-to-binary mapping
└── disassembler.rs      # NEW: Disassembler module
    ├── decoder.rs       # NEW: Instruction decoder
    └── formatter.rs     # NEW: Output formatting

tests/
├── assembler_tests.rs   # NEW: Assembler integration tests
├── disassembler_tests.rs # NEW: Disassembler integration tests
└── roundtrip_tests.rs   # NEW: Assemble → disassemble → re-assemble tests

examples/
├── simple_disasm.rs     # NEW: Basic disassembler usage
└── simple_asm.rs        # NEW: Basic assembler usage
```

**Structure Decision**: Single library crate with modular architecture. Assembler and disassembler are independent modules that both leverage the existing OPCODE_TABLE and AddressingMode enum. Each module is split into focused sub-modules for parsing, encoding, formatting, etc. Integration tests verify end-to-end behavior separately from unit tests in source files.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. This section is not applicable.

---

## Summary of Planning Artifacts

This planning phase has produced the following deliverables:

✅ **Phase 0 - Research** (`research.md`):

- Parser architecture decisions (hand-written recursive descent)
- Number format support (hex, decimal, binary)
- Multi-pass assembly strategy (two-pass)
- Error recovery approach (collect all errors)
- Source mapping granularity (instruction-level)
- Operand formatting conventions
- Symbol table design
- WebAssembly compatibility patterns

✅ **Phase 1 - Design** (multiple files):

- **Data Model** (`data-model.md`): Complete entity definitions with validation rules, state transitions, and relationships
- **API Contracts** (`contracts/`):
  - `assembler-api.md`: Public assembler functions, error handling, directives
  - `disassembler-api.md`: Public disassembler functions, formatting options
- **Quickstart Guide** (`quickstart.md`): Working examples for all major use cases
- **Project Structure**: Defined module layout in `src/` and `tests/`

✅ **Constitution Validation**: All principles satisfied with no violations

**Next Step**: Run `/speckit.tasks` to generate the dependency-ordered implementation task list in `tasks.md`.
