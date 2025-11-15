# Tasks: Assembler & Disassembler

**Input**: Design documents from `/specs/002-assembler-disassembler/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included based on spec requirements. All user stories specify independent test criteria and acceptance scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

All paths are relative to repository root:
- Core library: `src/`
- Integration tests: `tests/`
- Examples: `examples/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure for assembler/disassembler modules

- [X] T001 Create disassembler module directory structure in src/disassembler/
- [X] T002 Create assembler module directory structure in src/assembler/
- [X] T003 [P] Create integration test directories in tests/ (assembler_tests.rs, disassembler_tests.rs, roundtrip_tests.rs)
- [X] T004 [P] Create examples directory structure in examples/ (simple_disasm.rs, simple_asm.rs)
- [X] T005 Update src/lib.rs to export disassembler and assembler modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and utilities that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T006 Define Instruction struct in src/disassembler.rs with all metadata fields (address, opcode, mnemonic, addressing_mode, operand_bytes, size_bytes, base_cycles)
- [X] T007 [P] Define DisassemblyOptions struct in src/disassembler.rs (start_address, hex_dump, show_offsets)
- [X] T008 [P] Define AssemblyLine struct in src/assembler/parser.rs (line_number, label, mnemonic, operand, comment, span)
- [X] T009 [P] Define Symbol struct in src/assembler/symbol_table.rs (name, address, defined_at)
- [X] T010 [P] Define AssemblerError and ErrorType enums in src/assembler.rs (syntax, undefined_label, duplicate_label, invalid_label, invalid_mnemonic, invalid_operand, range_error, invalid_directive)
- [X] T011 [P] Define SourceLocation and AddressRange structs in src/assembler/source_map.rs
- [X] T012 [P] Define SourceMap struct with bidirectional mappings in src/assembler/source_map.rs
- [X] T013 [P] Define AssemblerOutput struct in src/assembler.rs (bytes, symbol_table, source_map, warnings)
- [X] T014 [P] Define AssemblerDirective enum in src/assembler.rs (Origin, Byte, Word)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Disassemble Binary to Assembly (Priority: P1) 🎯 MVP

**Goal**: Convert raw binary machine code into human-readable assembly mnemonics with operands for debugging

**Independent Test**: Provide a byte array containing known 6502 opcodes and verify the disassembler outputs the correct mnemonic and operand representation for each instruction

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T015 [P] [US1] Integration test for single instruction disassembly (LDA #$42) in tests/disassembler_tests.rs
- [ ] T016 [P] [US1] Integration test for multi-instruction disassembly in tests/disassembler_tests.rs
- [ ] T017 [P] [US1] Integration test for illegal opcode handling (".byte $XX") in tests/disassembler_tests.rs
- [ ] T018 [P] [US1] Integration test for starting address offset in tests/disassembler_tests.rs

### Implementation for User Story 1

- [ ] T019 [P] [US1] Implement opcode lookup in src/disassembler/decoder.rs using OPCODE_TABLE
- [ ] T020 [P] [US1] Implement operand byte extraction based on addressing mode in src/disassembler/decoder.rs
- [ ] T021 [US1] Implement decode_instruction function that creates Instruction struct in src/disassembler/decoder.rs
- [ ] T022 [P] [US1] Implement format_operand helper for each addressing mode (Immediate, ZeroPage, Absolute, etc.) in src/disassembler/formatter.rs
- [ ] T023 [US1] Implement format_instruction function in src/disassembler/formatter.rs using format_operand
- [ ] T024 [US1] Implement main disassemble function that processes byte slice into Vec<Instruction> in src/disassembler.rs
- [ ] T025 [US1] Add illegal opcode handling (".byte $XX" format) in src/disassembler/formatter.rs
- [ ] T026 [US1] Add unit tests for all addressing mode formats in src/disassembler/formatter.rs
- [ ] T027 [US1] Add unit tests for opcode decoding edge cases in src/disassembler/decoder.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - disassembly works for all documented and illegal opcodes

---

## Phase 4: User Story 2 - Assemble Text to Binary (Priority: P2)

**Goal**: Convert assembly language source code (mnemonics and operands) into executable binary machine code

**Independent Test**: Provide assembly source text and verify the assembler produces the exact byte sequence that would execute correctly on the CPU

### Tests for User Story 2

- [X] T028 [P] [US2] Integration test for single instruction assembly (LDA #$42) in tests/assembler_tests.rs
- [X] T029 [P] [US2] Integration test for multi-line assembly in tests/assembler_tests.rs
- [X] T030 [P] [US2] Integration test for number format parsing (hex $42, decimal 66, binary %01000010) in tests/assembler_tests.rs
- [X] T031 [P] [US2] Integration test for case-insensitive and whitespace-tolerant parsing in tests/assembler_tests.rs
- [X] T032 [P] [US2] Integration test for syntax error reporting with line/column info in tests/assembler_tests.rs
- [X] T033 [P] [US2] Integration test for multiple error collection in tests/assembler_tests.rs

### Implementation for User Story 2

- [X] T034 [P] [US2] Implement number parser for hex ($XX), decimal, and binary (%XXXXXXXX) in src/assembler/parser.rs
- [X] T035 [P] [US2] Implement line tokenizer (split on whitespace, handle special chars) in src/assembler/parser.rs
- [X] T036 [US2] Implement parse_line function that creates AssemblyLine struct in src/assembler/parser.rs
- [X] T037 [US2] Implement mnemonic-to-opcode lookup using OPCODE_TABLE in src/assembler/encoder.rs
- [X] T038 [P] [US2] Implement operand parser for each addressing mode (detect mode from operand syntax) in src/assembler/parser.rs
- [X] T039 [US2] Implement encode_instruction function that produces bytes from AssemblyLine in src/assembler/encoder.rs
- [X] T040 [US2] Implement assemble function (Pass 1: parse, Pass 2: encode) in src/assembler.rs
- [X] T041 [US2] Add error recovery logic to continue parsing after errors in src/assembler/parser.rs
- [X] T042 [US2] Add validation for addressing mode operand ranges (immediate 0-255, zero-page 0-255) in src/assembler/encoder.rs
- [X] T043 [US2] Add unit tests for number parsing in src/assembler/parser.rs
- [X] T044 [US2] Add unit tests for operand mode detection in src/assembler/parser.rs
- [X] T045 [US2] Add unit tests for instruction encoding in src/assembler/encoder.rs

**Checkpoint**: At this point, User Stories 1 AND 2 both work independently - basic disassembly and assembly without labels

---

## Phase 5: User Story 6 - Structured Output for IDE Integration (Priority: P2)

**Goal**: Return structured data with source mappings and comprehensive error information for web-based IDE features

**Independent Test**: Assemble code with errors and verify the returned data structure contains all errors with line/column/span information, plus source maps linking bytes to source locations

**Note**: US6 is implemented here (before US3-5) because it affects the core assembler output structure and must be designed in alongside basic assembly to avoid breaking changes later.

### Tests for User Story 6

- [ ] T046 [P] [US6] Integration test for error reporting with line/column/span in tests/assembler_tests.rs
- [ ] T047 [P] [US6] Integration test for source map query by instruction address in tests/assembler_tests.rs
- [ ] T048 [P] [US6] Integration test for source map query by source line in tests/assembler_tests.rs
- [ ] T049 [P] [US6] Integration test for symbol table access in tests/assembler_tests.rs
- [ ] T050 [P] [US6] Integration test for structured Instruction data (not just text) in tests/disassembler_tests.rs

### Implementation for User Story 6

- [ ] T051 [P] [US6] Implement SourceMap::add_mapping to record instruction address → source location in src/assembler/source_map.rs
- [ ] T052 [P] [US6] Implement SourceMap::get_source_location with binary search in src/assembler/source_map.rs
- [ ] T053 [P] [US6] Implement SourceMap::get_address_range with binary search in src/assembler/source_map.rs
- [ ] T054 [US6] Integrate source map building into Pass 2 assembly in src/assembler.rs
- [ ] T055 [P] [US6] Add character span tracking to parser for error reporting in src/assembler/parser.rs
- [ ] T056 [US6] Update AssemblerError creation to include span information in src/assembler.rs
- [ ] T057 [US6] Implement AssemblerOutput::lookup_symbol helper method in src/assembler.rs
- [ ] T058 [US6] Implement AssemblerOutput::get_source_location helper method in src/assembler.rs
- [ ] T059 [US6] Implement AssemblerOutput::get_address_range helper method in src/assembler.rs
- [ ] T060 [US6] Add Display impl for AssemblerError with formatted output in src/assembler.rs
- [ ] T061 [US6] Add unit tests for source map operations in src/assembler/source_map.rs

**Checkpoint**: Assembler now returns rich structured data suitable for IDE integration

---

## Phase 6: User Story 3 - Support Symbolic Labels (Priority: P3)

**Goal**: Enable symbolic labels for addresses to write maintainable assembly code without hard-coding memory addresses

**Independent Test**: Assemble code containing label definitions and references, verifying the assembler correctly resolves label addresses and encodes branch/jump targets

### Tests for User Story 3

- [ ] T062 [P] [US3] Integration test for simple label definition and reference (JMP START) in tests/assembler_tests.rs
- [ ] T063 [P] [US3] Integration test for forward label reference in tests/assembler_tests.rs
- [ ] T064 [P] [US3] Integration test for relative branch to label (BEQ LOOP) in tests/assembler_tests.rs
- [ ] T065 [P] [US3] Integration test for undefined label error in tests/assembler_tests.rs
- [ ] T066 [P] [US3] Integration test for duplicate label error in tests/assembler_tests.rs
- [ ] T067 [P] [US3] Integration test for invalid label validation (starts with digit, too long, invalid chars) in tests/assembler_tests.rs

### Implementation for User Story 3

- [ ] T068 [P] [US3] Implement validate_label function with regex pattern check in src/assembler.rs
- [ ] T069 [P] [US3] Implement SymbolTable::add_symbol with duplicate detection in src/assembler/symbol_table.rs
- [ ] T070 [P] [US3] Implement SymbolTable::lookup_symbol in src/assembler/symbol_table.rs
- [ ] T071 [US3] Update parser to extract label definitions (identifier followed by colon) in src/assembler/parser.rs
- [ ] T072 [US3] Update Pass 1 to build symbol table with label addresses in src/assembler.rs
- [ ] T073 [US3] Update encoder to detect label references in operands in src/assembler/encoder.rs
- [ ] T074 [US3] Implement label address resolution in Pass 2 in src/assembler.rs
- [ ] T075 [US3] Implement relative branch offset calculation for labels in src/assembler/encoder.rs
- [ ] T076 [US3] Add range check for branch offsets (-128 to +127) in src/assembler/encoder.rs
- [ ] T077 [US3] Add undefined label detection and error reporting in src/assembler.rs
- [ ] T078 [US3] Add unit tests for label validation in src/assembler.rs
- [ ] T079 [US3] Add unit tests for symbol table operations in src/assembler/symbol_table.rs
- [ ] T080 [US3] Add unit tests for branch offset calculation in src/assembler/encoder.rs

**Checkpoint**: Assembler now supports full label functionality with forward references and relative branches

---

## Phase 7: User Story 4 - Hexadecimal Dump Formatting (Priority: P4)

**Goal**: View disassembled code alongside hexadecimal byte representations and memory addresses for debugging

**Independent Test**: Disassemble a known byte sequence and verify the output includes formatted addresses, hex bytes, and assembly mnemonics in aligned columns

### Tests for User Story 4

- [ ] T081 [P] [US4] Integration test for hex dump format with single instruction in tests/disassembler_tests.rs
- [ ] T082 [P] [US4] Integration test for hex dump with varying instruction byte lengths in tests/disassembler_tests.rs
- [ ] T083 [P] [US4] Integration test for hex dump with multi-line output and address increments in tests/disassembler_tests.rs

### Implementation for User Story 4

- [ ] T084 [P] [US4] Implement format_hex_bytes helper (up to 3 bytes, left-aligned) in src/disassembler/formatter.rs
- [ ] T085 [P] [US4] Implement format_address helper (4-digit hex) in src/disassembler/formatter.rs
- [ ] T086 [US4] Implement format_hex_dump function for Vec<Instruction> in src/disassembler/formatter.rs
- [ ] T087 [US4] Add column alignment logic for consistent mnemonic positioning in src/disassembler/formatter.rs
- [ ] T088 [US4] Add unit tests for hex dump formatting in src/disassembler/formatter.rs

**Checkpoint**: Disassembler now supports hex dump output format for enhanced debugging

---

## Phase 8: User Story 5 - Comments and Directives (Priority: P5)

**Goal**: Support comments (`;`) and assembler directives (`.org`, `.byte`, `.word`) for code documentation and assembly control

**Independent Test**: Assemble code containing comments and directives, verifying comments are ignored and directives correctly affect assembly behavior

### Tests for User Story 5

- [ ] T089 [P] [US5] Integration test for comment parsing and ignoring in tests/assembler_tests.rs
- [ ] T090 [P] [US5] Integration test for .org directive setting origin address in tests/assembler_tests.rs
- [ ] T091 [P] [US5] Integration test for .byte directive inserting literal bytes in tests/assembler_tests.rs
- [ ] T092 [P] [US5] Integration test for .word directive with little-endian encoding in tests/assembler_tests.rs
- [ ] T093 [P] [US5] Integration test for invalid directive error in tests/assembler_tests.rs

### Implementation for User Story 5

- [ ] T094 [P] [US5] Implement comment detection and stripping in parser (after semicolon) in src/assembler/parser.rs
- [ ] T095 [P] [US5] Implement directive detection (starts with dot) in src/assembler/parser.rs
- [ ] T096 [P] [US5] Implement parse_org_directive in src/assembler/parser.rs
- [ ] T097 [P] [US5] Implement parse_byte_directive in src/assembler/parser.rs
- [ ] T098 [P] [US5] Implement parse_word_directive (little-endian encoding) in src/assembler/parser.rs
- [ ] T099 [US5] Integrate directive handling into Pass 1 address calculation in src/assembler.rs
- [ ] T100 [US5] Integrate directive handling into Pass 2 byte emission in src/assembler.rs
- [ ] T101 [US5] Add directive validation (at least one value for .byte/.word) in src/assembler/parser.rs
- [ ] T102 [US5] Add unit tests for comment stripping in src/assembler/parser.rs
- [ ] T103 [US5] Add unit tests for directive parsing in src/assembler/parser.rs

**Checkpoint**: All user stories (US1-US6) are now complete with full assembler/disassembler functionality

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Quality improvements, examples, and validation that affect multiple user stories

- [ ] T104 [P] Create basic disassembly example in examples/simple_disasm.rs
- [ ] T105 [P] Create basic assembly example in examples/simple_asm.rs
- [ ] T106 [P] Add round-trip tests (assemble → disassemble → re-assemble) in tests/roundtrip_tests.rs
- [ ] T107 [P] Add documentation comments to all public API functions in src/disassembler.rs
- [ ] T108 [P] Add documentation comments to all public API functions in src/assembler.rs
- [ ] T109 [P] Add usage examples to module-level docs in src/disassembler.rs
- [ ] T110 [P] Add usage examples to module-level docs in src/assembler.rs
- [ ] T111 Run cargo clippy and fix all warnings
- [ ] T112 Run cargo fmt to format all code
- [ ] T113 Run cargo test and ensure all tests pass
- [ ] T114 Verify WebAssembly compilation with cargo build --target wasm32-unknown-unknown
- [ ] T115 [P] Validate examples from quickstart.md work correctly
- [ ] T116 [P] Add CHANGELOG.md entry for assembler/disassembler feature
- [ ] T117 Review and update README.md with assembler/disassembler usage

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (P1): Disassembly - can start after Phase 2
  - US2 (P2): Assembly - can start after Phase 2, benefits from US1 for testing but independent
  - US6 (P2): IDE Integration - must complete alongside or immediately after US2 (affects core output structure)
  - US3 (P3): Labels - depends on US2 completion (extends assembler)
  - US4 (P4): Hex Dump - depends on US1 completion (extends disassembler)
  - US5 (P5): Comments/Directives - depends on US2 completion (extends assembler parser)
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories (benefits from US1 for validation but independently testable)
- **User Story 6 (P2)**: MUST complete alongside US2 (one-way door: affects AssemblerOutput structure)
- **User Story 3 (P3)**: Depends on US2 completion (extends assembler with labels)
- **User Story 4 (P4)**: Depends on US1 completion (extends disassembler with formatting)
- **User Story 5 (P5)**: Depends on US2 completion (extends assembler parser with comments/directives)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data structures (foundational phase) before business logic
- Core decoding/encoding before formatting/helpers
- Validation and error handling integrated with implementation
- Story complete and tested before moving to next priority

### Parallel Opportunities

**Setup Phase**:
- T001, T002, T003, T004 can all run in parallel (different directories)

**Foundational Phase**:
- All struct definitions (T006-T014) can run in parallel (different files)

**User Story 1 (Disassembly)**:
- All tests (T015-T018) can run in parallel
- T019, T020, T022 can run in parallel (different aspects of decoding/formatting)
- T026, T027 can run in parallel (different test files)

**User Story 2 (Assembly)**:
- All tests (T028-T033) can run in parallel
- T034, T035, T038 can run in parallel (parsing helpers)
- T043, T044, T045 can run in parallel (unit tests in different areas)

**User Story 6 (IDE Integration)**:
- All tests (T046-T050) can run in parallel
- T051, T052, T053, T055 can run in parallel (different source map/parser features)

**User Story 3 (Labels)**:
- All tests (T062-T067) can run in parallel
- T068, T069, T070 can run in parallel (validation and symbol table operations)
- T078, T079, T080 can run in parallel (unit tests)

**User Story 4 (Hex Dump)**:
- All tests (T081-T083) can run in parallel
- T084, T085 can run in parallel (formatting helpers)

**User Story 5 (Comments/Directives)**:
- All tests (T089-T093) can run in parallel
- T094, T095, T096, T097, T098 can run in parallel (parsing helpers)
- T102, T103 can run in parallel (unit tests)

**Polish Phase**:
- T104, T105, T106, T107, T108, T109, T110, T115, T116 can all run in parallel (different files)

**Across User Stories** (after Phase 2):
- US1 and US2 can be developed in parallel (different modules)
- After US2: US3 and (US4 if US1 complete) can be developed in parallel
- After US2 and US3: US5 can be developed

---

## Parallel Example: User Story 1 (Disassembly)

```bash
# Launch all tests for User Story 1 together:
Task: "Integration test for single instruction disassembly (LDA #$42) in tests/disassembler_tests.rs"
Task: "Integration test for multi-instruction disassembly in tests/disassembler_tests.rs"
Task: "Integration test for illegal opcode handling in tests/disassembler_tests.rs"
Task: "Integration test for starting address offset in tests/disassembler_tests.rs"

# Launch parallel implementation tasks:
Task: "Implement opcode lookup in src/disassembler/decoder.rs using OPCODE_TABLE"
Task: "Implement operand byte extraction based on addressing mode in src/disassembler/decoder.rs"
Task: "Implement format_operand helper for each addressing mode in src/disassembler/formatter.rs"
```

---

## Parallel Example: User Story 2 (Assembly)

```bash
# Launch all tests for User Story 2 together:
Task: "Integration test for single instruction assembly (LDA #$42) in tests/assembler_tests.rs"
Task: "Integration test for multi-line assembly in tests/assembler_tests.rs"
Task: "Integration test for number format parsing in tests/assembler_tests.rs"
Task: "Integration test for case-insensitive and whitespace-tolerant parsing in tests/assembler_tests.rs"
Task: "Integration test for syntax error reporting with line/column info in tests/assembler_tests.rs"
Task: "Integration test for multiple error collection in tests/assembler_tests.rs"

# Launch parallel parsing helpers:
Task: "Implement number parser for hex, decimal, and binary in src/assembler/parser.rs"
Task: "Implement line tokenizer in src/assembler/parser.rs"
Task: "Implement operand parser for each addressing mode in src/assembler/parser.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Disassembly)
4. **STOP and VALIDATE**: Test disassembly independently
5. Complete Phase 4: User Story 2 (Assembly - basic)
6. Complete Phase 5: User Story 6 (IDE Integration - must complete with US2)
7. **STOP and VALIDATE**: Test assembly independently, test round-trip
8. Deploy/demo MVP: Basic assembler and disassembler working

### Incremental Delivery

1. **Foundation**: Setup + Foundational → Core data structures ready
2. **MVP Release**: US1 + US2 + US6 → Basic assemble/disassemble working, IDE-ready
3. **Enhancement 1**: Add US3 (Labels) → Practical assembler with symbolic references
4. **Enhancement 2**: Add US4 (Hex Dump) → Enhanced debugging output
5. **Enhancement 3**: Add US5 (Comments/Directives) → Full-featured assembler
6. Each increment adds value without breaking previous functionality

### Parallel Team Strategy

With multiple developers after Phase 2 complete:

**Option 1: Sequential MVP Focus**
- All developers: Complete US1 → US2 → US6 together for MVP
- Then split: Developer A takes US3, Developer B takes US4, Developer C takes US5

**Option 2: Parallel After Foundation**
- Developer A: User Story 1 (Disassembly)
- Developer B: User Story 2 + 6 (Assembly + IDE Integration)
- After both complete: Developer A takes US4, Developer B takes US3, Developer C takes US5

**Option 3: Feature Teams**
- Disassembler team: US1 → US4 (core disassembly + hex dump)
- Assembler team: US2 → US6 → US3 → US5 (core assembly + IDE + labels + directives)
- Teams integrate via round-trip tests

---

## Notes

- [P] tasks = different files, no dependencies within the phase
- [Story] label maps task to specific user story for traceability (US1-US6)
- Each user story should be independently completable and testable
- Write tests first, verify they fail, then implement to make them pass
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- US6 (IDE Integration) MUST be done with US2 - it's a P2 story that affects core output structure
- Round-trip tests (assemble → disassemble → re-assemble) validate both modules together
- All modules compile to WebAssembly (validate with cargo build --target wasm32-unknown-unknown)
- Zero external dependencies - use only Rust core/alloc libraries
