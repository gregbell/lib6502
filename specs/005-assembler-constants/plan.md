# Implementation Plan: Assembler Constants

**Branch**: `005-assembler-constants` | **Date**: 2025-11-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-assembler-constants/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add named constant support to the 6502 assembler using `NAME = VALUE` syntax. Constants enable reusable values (screen addresses, character codes, magic numbers) throughout assembly code. Implementation extends the existing symbol table to distinguish constants (literal values) from labels (memory addresses), with immediate resolution during Pass 1 and substitution during Pass 2 encoding.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (zero external dependencies for core library - `no_std` compatible)
**Storage**: N/A (operates on in-memory byte slices and strings)
**Testing**: cargo test (unit + integration tests), Klaus functional test suite
**Target Platform**: Native (Linux/macOS/Windows) + WebAssembly (browser)
**Project Type**: Single library crate with assembler module
**Performance Goals**: Assemble thousands of lines per second (non-critical - assembler runs at compile time)
**Constraints**:
- Must maintain existing assembler test suite (935 lines, ~2s execution)
- Must not break Klaus functional test (96M+ cycles, ~6s execution)
- Zero external dependencies (maintains `no_std` compatibility)
- WebAssembly portability (no OS dependencies)

**Scale/Scope**:
- Modify 4 core assembler files (parser, symbol_table, encoder, main assembler)
- Add ~300-500 lines of implementation + ~500-800 lines of tests
- Extend existing symbol table with SymbolKind enum
- Maintain backward compatibility with all existing assembly code

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Modularity & Separation of Concerns

**Status**: ✅ **COMPLIANT**

- Constants feature is entirely self-contained within assembler module
- No changes to CPU core or memory abstractions
- Parser extension cleanly separates constant vs. label syntax
- Symbol table extension uses enum to distinguish types

**Rationale**: Assembler is already a separate concern from CPU execution. This feature extends existing assembler capabilities without coupling to other modules.

### II. WebAssembly Portability

**Status**: ✅ **COMPLIANT**

- No new dependencies introduced (maintains zero external deps)
- No OS-level features required
- All operations are pure computation (parsing, symbol table lookup, substitution)
- Deterministic execution (no randomness or system time)

**Rationale**: Constant parsing and resolution are pure string/number operations compatible with `no_std` and WASM environments.

### III. Cycle Accuracy

**Status**: ✅ **NOT APPLICABLE**

- Constants are compile-time assembler feature
- No impact on runtime CPU cycle counts
- Assembled machine code is identical whether constants or literals were used

**Rationale**: This feature affects assembly-time only. The resulting bytecode is cycle-accurate as before.

### IV. Clarity & Hackability

**Status**: ✅ **COMPLIANT**

- Simple, intuitive `NAME = VALUE` syntax matches common assembler conventions
- Clear error messages for undefined/duplicate constants
- Symbol table extension uses explicit enum rather than magic flags
- Parser logic cleanly distinguishes `=` (constant) from `:` (label)
- Implementation follows existing two-pass assembler pattern

**Rationale**: Feature is self-explanatory and follows established patterns. Newcomers can understand constant vs. label distinction from type system.

### V. Table-Driven Design

**Status**: ✅ **NOT APPLICABLE**

- Constants are data (symbol table entries), not opcodes
- No instruction decode logic affected
- Symbol table lookup is already table-driven (Vec of Symbol structs)

**Rationale**: This feature extends data structures, not instruction decoding. No table-driven changes needed.

## Project Structure

### Documentation (this feature)

```text
specs/005-assembler-constants/
├── spec.md              # Feature specification (completed)
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
├── lib.rs                       # No changes needed
├── cpu.rs                       # No changes needed
├── memory.rs                    # No changes needed
├── opcodes.rs                   # No changes needed
├── addressing.rs                # No changes needed
└── assembler/
    ├── mod.rs                   # MODIFY: Add SymbolKind enum, update Symbol struct
    ├── parser.rs                # MODIFY: Parse NAME = VALUE syntax
    ├── symbol_table.rs          # MODIFY: Add constant handling methods
    ├── encoder.rs               # MODIFY: Substitute constants during encoding
    └── source_map.rs            # No changes needed

tests/
├── assembler_tests.rs           # MODIFY: Add constant integration tests
├── functional_assembler_disassembler.rs  # No changes (round-trip tests)
├── functional_klaus.rs          # No changes (but must still pass)
└── [other test files]           # No changes

examples/
├── simple_asm.rs                # Optional: Could add constant example
└── [new] constants.rs           # NEW: Example demonstrating constant usage
```

**Structure Decision**: Single project structure. All changes are confined to the `src/assembler/` module and `tests/assembler_tests.rs`. This is a pure library extension with no API surface changes (assembler already accepts string input, just extends syntax support).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*No violations detected. All constitution principles are compliant or not applicable.*

---

## Phase 0: Research & Decisions

**Status**: Phase 0 research tasks identified below. Execute research agents to resolve all "NEEDS CLARIFICATION" items.

### Research Tasks

1. **Parser Implementation Pattern**
   - **Unknown**: How to detect `=` vs `:` without breaking existing label parsing
   - **Research**: Analyze current parser.rs line parsing order and tokenization
   - **Deliverable**: Decision on where to insert constant detection in parse flow

2. **Symbol Table Refactoring Strategy**
   - **Unknown**: Whether to rename `address` field to `value` (breaking change) or add separate field
   - **Research**: Analyze impact on existing code using Symbol struct
   - **Deliverable**: Decision on backward-compatible vs. breaking change approach

3. **Error Message Design**
   - **Unknown**: Exact error message format and line/column reporting for constant errors
   - **Research**: Review existing assembler error patterns
   - **Deliverable**: Consistent error message templates

4. **Forward Reference Handling**
   - **Unknown**: Whether to support `A = B + 1` where B is defined later
   - **Research**: Spec says "no forward references" but need to clarify constant-to-constant refs
   - **Deliverable**: Decision on constant expression evaluation order

5. **Name Collision Edge Cases**
   - **Unknown**: What happens if constant defined, then label with same name, then usage?
   - **Research**: Define resolution order and error detection timing
   - **Deliverable**: Collision detection algorithm and error reporting point

### Technology Best Practices

1. **Rust enum best practices for SymbolKind**
   - **Topic**: Pattern matching ergonomics, avoiding boilerplate
   - **Research**: Review Rust enum design patterns for type discrimination
   - **Deliverable**: SymbolKind API design (methods, matching patterns)

2. **Assembler two-pass architecture patterns**
   - **Topic**: How other assemblers handle constant vs. label resolution
   - **Research**: Survey 6502 assembler implementations (ca65, DASM, etc.)
   - **Deliverable**: Confirmation that Pass 1 constant resolution is standard

---

## Phase 1: Design Artifacts

**Prerequisites**: research.md complete

### Data Model (data-model.md)

**Entities to design**:

1. **SymbolKind** (enum)
   - Variants: Label, Constant
   - Methods: TBD based on research

2. **Symbol** (struct extension)
   - Fields: name, value (renamed from address), kind, defined_at
   - Validation: value range 0-65535, name validation

3. **AssemblyLine** (existing struct, may need extension)
   - Check if needs new field for constant assignment
   - Or if constant assignment is treated as directive

4. **Error types** (extend ErrorType enum)
   - UndefinedConstant, DuplicateConstant, NameCollision, InvalidConstantValue

### API Contracts (contracts/)

**No public API changes**:
- Assembler function signature remains: `pub fn assemble(source: &str) -> Result<AssemblerOutput, Vec<AssemblerError>>`
- AssemblerOutput struct unchanged (symbol table already public)
- Backward compatible: existing assembly code works unchanged

**Internal contracts** (for testing):
- Parser contract: `parse_line(&str) -> Option<AssemblyLine>` must recognize `NAME = VALUE`
- Symbol table contract: `add_constant(name, value, line)` must detect duplicates
- Encoder contract: `resolve_operand(operand, symbol_table, ...)` must substitute constants

### Quickstart (quickstart.md)

**User-facing guide**:
1. How to define constants
2. How to use constants in different addressing modes
3. Common error scenarios and fixes
4. Migration guide (none needed - feature is additive)

---

## Phase 2: Task Generation

**Note**: Phase 2 (`/speckit.tasks`) is NOT executed by this command. After Phase 0 research and Phase 1 design artifacts are reviewed, run `/speckit.tasks` to generate tasks.md.

**Expected task categories**:
1. Parser modifications (detect `=`, validate syntax)
2. Symbol table changes (SymbolKind enum, add_constant method)
3. Encoder modifications (constant substitution)
4. Error handling (new error types, messages)
5. Testing (unit tests, integration tests, edge cases)
6. Documentation (CLAUDE.md update, examples)

---

## Validation & Next Steps

**Completion checklist for /speckit.plan**:
- [x] Technical Context filled (no NEEDS CLARIFICATION)
- [x] Constitution Check evaluated (all principles assessed)
- [x] Project Structure concrete (no Option labels remaining)
- [x] Phase 0 research tasks identified
- [x] Phase 1 design artifacts scoped

**Next actions**:
1. ✅ Review this plan for completeness
2. ⏳ Execute Phase 0: Generate research.md (resolve unknowns via research agents)
3. ⏳ Execute Phase 1: Generate data-model.md, contracts/, quickstart.md
4. ⏳ Update agent context (run update-agent-context.sh)
5. ⏳ Re-evaluate Constitution Check post-design
6. ⏳ Ready for `/speckit.tasks` (task breakdown generation)
