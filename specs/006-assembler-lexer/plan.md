# Implementation Plan: Assembler Lexer and Parser Architecture

**Branch**: `006-assembler-lexer` | **Date**: 2025-11-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-assembler-lexer/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Refactor the assembler to introduce a proper lexical analysis layer (lexer) that separates tokenization from parsing. The lexer will convert assembly source into a typed token stream with source location tracking, while the parser consumes tokens to build syntax trees. This architecture improves code clarity, simplifies parser logic by 30%+, enables better error reporting (distinguishing lexical vs syntactic errors), and makes the tokenizer reusable for external tooling. The refactoring maintains 100% backward compatibility—all existing programs must assemble to bit-identical output.

**Technical Approach**: Introduce a new `src/assembler/lexer.rs` module that produces a `TokenStream` from source text. Refactor `src/assembler/parser.rs` to consume tokens instead of performing string operations. Extend `AssemblerError` to distinguish lexical errors from syntactic errors. Maintain existing public API (`assemble()` function) while restructuring internals.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (zero external dependencies for core library - `no_std` compatible per Constitution)
**Storage**: N/A (operates on in-memory strings and produces byte vectors)
**Testing**: `cargo test` (unit tests in `src/`, integration tests in `tests/`)
**Target Platform**: Cross-platform (native + WebAssembly browser target per Constitution)
**Project Type**: Single library project with modular assembler subsystem
**Performance Goals**: Assembly throughput >10,000 lines/sec, lexer adds <5% overhead vs current parser
**Constraints**: Zero external dependencies, WASM-compatible (no std::fs/net/process), deterministic execution
**Scale/Scope**: Refactor ~1,200 LOC in assembler module, target 30% reduction in parser.rs size, maintain all 1,470+ passing tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ I. Modularity & Separation of Concerns

**Alignment**: **STRONG** - This feature directly reinforces the modularity principle by cleanly separating lexical analysis (character-level tokenization) from syntactic analysis (grammar parsing). The lexer becomes a reusable module with zero coupling to parser internals.

**Verification**:
- Lexer module operates independently on source text → token stream
- Parser module consumes token stream → syntax tree
- Clear interface boundary (TokenStream) enables independent testing and tooling reuse

### ✅ II. WebAssembly Portability

**Alignment**: **COMPLIANT** - Lexer/parser operate purely on in-memory strings and primitive types. No I/O, no platform-specific code, no external dependencies. Fully WASM-compatible.

**Verification**:
- No `std::fs`, `std::net`, `std::process` usage (only string/vector operations)
- Deterministic tokenization (no system time, no randomness)
- Existing WASM demo will continue to work with refactored assembler

### ✅ III. Cycle Accuracy

**Alignment**: **NOT APPLICABLE** - This feature affects assembler tooling, not CPU emulation. No impact on cycle accuracy guarantees.

### ✅ IV. Clarity & Hackability

**Alignment**: **STRONG** - Primary goal of this feature is improving code clarity. Separating lexer and parser makes each component easier to understand, debug, and extend. Explicit token types replace ad-hoc string parsing.

**Verification**:
- Spec SC-003: Parser LOC reduces by 30%+ through elimination of inline parsing
- Spec User Story 3: New contributors can locate bugs faster through clear layer boundaries
- Each module has single responsibility (lexer = tokenize, parser = build syntax tree)

### ✅ V. Table-Driven Design

**Alignment**: **COMPLIANT** - Lexer uses explicit token type enum and character classification rules. Parser continues to reference existing `OPCODE_TABLE` for instruction metadata. No new duplication introduced.

**Verification**:
- Token types defined in enum (not scattered match statements)
- Lexer state machine uses consistent transition rules
- Parser lookup remains table-driven via existing `OPCODE_TABLE`

**Gate Result**: ✅ **PASS** - No constitutional violations. Feature strongly aligns with Modularity and Clarity principles.

## Project Structure

### Documentation (this feature)

```text
specs/006-assembler-lexer/
├── spec.md              # Feature specification (completed)
├── checklists/          # Quality validation checklists
│   └── requirements.md  # Spec validation checklist (all checks passed)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (lexer design patterns)
├── data-model.md        # Phase 1 output (Token/TokenStream/Lexer entities)
├── quickstart.md        # Phase 1 output (developer guide for using new architecture)
├── contracts/           # Phase 1 output (lexer/parser API contracts)
│   ├── lexer-api.md     # Public lexer interface specification
│   └── parser-api.md    # Updated parser interface specification
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Public API (no changes to public interface)
├── assembler.rs         # Module declarations (add pub mod lexer)
├── assembler/
│   ├── lexer.rs         # NEW: Lexical analysis (source → TokenStream)
│   ├── parser.rs        # REFACTOR: Consume tokens instead of strings
│   ├── encoder.rs       # UNCHANGED: Instruction encoding
│   ├── source_map.rs    # UNCHANGED: Source location tracking
│   └── symbol_table.rs  # UNCHANGED: Label/constant resolution
├── opcodes.rs           # UNCHANGED: Opcode metadata table
├── addressing.rs        # UNCHANGED: Addressing mode enum
├── cpu.rs               # UNCHANGED: CPU emulation
└── memory.rs            # UNCHANGED: Memory bus

tests/
├── assembler_tests.rs       # VERIFY: All existing tests pass unchanged
├── assembler_directives_test.rs  # VERIFY: Directive tests pass
├── functional_assembler_disassembler.rs  # VERIFY: Roundtrip tests pass
└── lexer_tests.rs           # NEW: Lexer-specific unit tests
```

**Structure Decision**: Single library project (Option 1). Assembler is a subsystem within the larger 6502 emulator library. New `lexer.rs` module added to existing `src/assembler/` directory. No project structure changes—only internal refactoring within the assembler subsystem.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - No constitutional violations detected. All gates passed.
