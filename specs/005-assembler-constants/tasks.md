# Implementation Tasks: Assembler Constants

**Feature**: 005-assembler-constants
**Branch**: `claude/add-assembler-variables-01RFfnC2JwQWx9biDLRcWftG`
**Spec**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

---

## Summary

Implement named constant support for the 6502 assembler using `NAME = VALUE` syntax. This enables assembly programmers to define reusable values (screen addresses, character codes, magic numbers) and reference them throughout their code.

### User Stories from Spec

**User Story 1 (US1)**: Define named constants at the top of assembly file
**User Story 2 (US2)**: Use constants anywhere in code
**User Story 3 (US3)**: Get clear errors for undefined/duplicate constants
**User Story 4 (US4)**: Distinguish between address labels and value constants

### Implementation Approach

- **MVP Scope**: US1 + US2 (basic definition and usage)
- **Incremental Delivery**: Each user story delivers testable functionality
- **Parallel Opportunities**: Parser, symbol table, and encoder tasks can progress in parallel after foundational phase
- **Testing Strategy**: Integration tests after each user story, comprehensive validation in final phase

### Task Statistics

- **Total Tasks**: 34
- **Parallelizable**: 18 tasks (53%)
- **User Story Breakdown**:
  - Setup: 2 tasks
  - Foundational: 6 tasks
  - US1 (Define constants): 4 tasks
  - US2 (Use constants): 3 tasks
  - US3 (Error handling): 7 tasks
  - US4 (Symbol kind system): 6 tasks
  - Testing & Validation: 4 tasks
  - Polish: 2 tasks

---

## Phase 1: Setup

**Goal**: Validate existing codebase structure and ensure clean starting point.

**Prerequisites**: None

**Test Criteria**:
- Existing assembler tests pass (935 tests, ~2s)
- Klaus functional test passes (96M+ cycles, ~6s)

### Tasks

- [X] T001 Run existing assembler test suite and verify all tests pass
- [X] T002 Run Klaus functional test and verify it passes

---

## Phase 2: Foundational - Core Data Structures

**Goal**: Establish the foundational data structures needed for constant support before implementing any user-facing features.

**Prerequisites**: Phase 1 complete

**Why Foundational**: All subsequent user stories depend on these data structure changes. The SymbolKind enum and Symbol struct modifications are used throughout parser, symbol table, and encoder.

**Test Criteria**:
- SymbolKind enum compiles and can be pattern matched
- Symbol struct compiles with new fields
- Existing code using Symbol.address still compiles (after field rename)
- ErrorType enum compiles with new variants

### Tasks

- [X] T003 [P] Add SymbolKind enum to src/assembler.rs with Label and Constant variants
- [X] T004 [P] Extend Symbol struct in src/assembler.rs with kind: SymbolKind field
- [X] T005 Rename Symbol.address field to Symbol.value in src/assembler.rs
- [X] T006 Update all references to symbol.address to symbol.value in src/assembler.rs (5 locations)
- [X] T007 [P] Update references to symbol.address in src/assembler/symbol_table.rs (3 unit tests)
- [X] T008 [P] Update references to symbol.address in tests/assembler_tests.rs (4+ assertions)

**Validation**: Run `cargo build` and ensure no compilation errors

---

## Phase 3: User Story 1 - Define Named Constants

**Goal**: Assembly programmers can define constants at the top of their assembly files using `NAME = VALUE` syntax.

**Prerequisites**: Phase 2 complete (SymbolKind and Symbol struct available)

**Acceptance Criteria**:
- Parser recognizes `NAME = VALUE` syntax
- Constants added to symbol table with SymbolKind::Constant
- Constants normalized to UPPERCASE
- Basic validation (name format, value is literal)

**Test Criteria**:
- Can parse `MAX = 255`
- Can parse `SCREEN = $4000`
- Can parse `BITS = %11110000`
- Constants appear in symbol table with correct kind
- Invalid names rejected (starts with digit, contains spaces)

### Tasks

- [X] T009 [P] [US1] Extend AssemblyLine struct in src/assembler/parser.rs with constant: Option<(String, String)> field
- [X] T010 [US1] Add constant detection logic in src/assembler/parser.rs before label detection (after comment stripping, line ~102)
- [X] T011 [P] [US1] Implement constant parsing logic in src/assembler/parser.rs (detect =, extract name/value, validate name format)
- [X] T012 [US1] Process constant assignments in Pass 1 of src/assembler.rs, adding to symbol table with SymbolKind::Constant

**Tests**:
- [X] T013 [P] [US1] Add unit test in src/assembler/parser.rs for parsing simple constant (MAX = 255)
- [X] T014 [P] [US1] Add unit test in src/assembler/parser.rs for parsing hex constant (SCREEN = $4000)
- [X] T015 [P] [US1] Add unit test in src/assembler/parser.rs for parsing binary constant (BITS = %11110000)
- [X] T016 [P] [US1] Add integration test in tests/assembler_tests.rs for basic constant definition

**Validation**: Run `cargo test` and verify new tests pass

---

## Phase 4: User Story 2 - Use Constants in Code

**Goal**: Assembly programmers can reference defined constants anywhere in their code as operands.

**Prerequisites**: Phase 3 complete (constants can be defined and stored)

**Acceptance Criteria**:
- Operand resolution looks up constants in symbol table
- Constant values substituted during encoding
- Works in all addressing modes (immediate, zero page, absolute, indexed)
- Constants must be defined before use (no forward references)

**Test Criteria**:
- `LDA #MAX` substitutes constant value
- `STA SCREEN` uses constant as address
- `LDA ZP_TEMP,X` works with indexed addressing
- Undefined constant produces error

### Tasks

- [ ] T017 [P] [US2] Modify resolve_operand() in src/assembler/encoder.rs to look up identifiers in symbol table
- [ ] T018 [US2] Add constant value substitution logic in src/assembler/encoder.rs (if SymbolKind::Constant, use value as literal)
- [ ] T019 [US2] Ensure addressing mode detection works with resolved constant values in src/assembler/encoder.rs

**Tests**:
- [ ] T020 [P] [US2] Add integration test in tests/assembler_tests.rs for immediate addressing with constant (LDA #MAX)
- [ ] T021 [P] [US2] Add integration test in tests/assembler_tests.rs for zero page with constant (STA ZP_TEMP)
- [ ] T022 [P] [US2] Add integration test in tests/assembler_tests.rs for absolute with constant (STA SCREEN)
- [ ] T023 [P] [US2] Add integration test in tests/assembler_tests.rs for indexed with constant (STA IO_BASE,X)

**Validation**: Run `cargo test` and verify constants substitute correctly

---

## Phase 5: User Story 3 - Clear Error Messages

**Goal**: Assembly programmers get clear, actionable error messages for constant-related issues.

**Prerequisites**: Phases 3 & 4 complete (constants can be defined and used)

**Acceptance Criteria**:
- Undefined constant produces specific error with line number
- Duplicate constant produces error with original line reference
- Name collision (constant vs label) detected and reported
- Invalid constant value (out of range, not literal) produces error

**Test Criteria**:
- `LDA #MISSING` → "Undefined constant 'MISSING'"
- `MAX = 100` then `MAX = 200` → "Duplicate constant 'MAX' (previously defined at line 1)"
- `FOO = 42` then `FOO:` → "Name collision: 'FOO' is already defined as a constant at line 1"
- `BAR = $10000` → "Constant value out of range"

### Tasks

#### Error Types

- [ ] T024 [P] [US3] Add UndefinedConstant variant to ErrorType enum in src/assembler.rs
- [ ] T025 [P] [US3] Add DuplicateConstant variant to ErrorType enum in src/assembler.rs
- [ ] T026 [P] [US3] Add NameCollision variant to ErrorType enum in src/assembler.rs
- [ ] T027 [P] [US3] Add InvalidConstantValue variant to ErrorType enum in src/assembler.rs

#### Error Detection

- [ ] T028 [US3] Implement collision detection in add_symbol() in src/assembler/symbol_table.rs (check existing symbol kind)
- [ ] T029 [US3] Add undefined constant error handling in resolve_operand() in src/assembler/encoder.rs (Pass 2)
- [ ] T030 [US3] Update Display impl for AssemblerError in src/assembler.rs with new error message templates

**Tests**:
- [ ] T031 [P] [US3] Add integration test in tests/assembler_tests.rs for undefined constant error
- [ ] T032 [P] [US3] Add integration test in tests/assembler_tests.rs for duplicate constant error
- [ ] T033 [P] [US3] Add integration test in tests/assembler_tests.rs for name collision error (constant then label)
- [ ] T034 [P] [US3] Add integration test in tests/assembler_tests.rs for name collision error (label then constant)

**Validation**: Run `cargo test` and verify error messages match templates

---

## Phase 6: User Story 4 - Distinguish Constants from Labels

**Goal**: Assembly programmers can understand the difference between constants (literal values) and labels (memory addresses).

**Prerequisites**: Phases 2-5 complete (all infrastructure in place)

**Acceptance Criteria**:
- Symbol table clearly distinguishes constants from labels via SymbolKind
- Constants and labels can coexist with different names
- Lookup returns correct kind for each symbol type
- Documentation explains the distinction

**Test Criteria**:
- Can define both `MAX = 255` and `START:` in same program
- Symbol table lookup returns correct kind
- Both constants and labels work as operands

### Tasks

- [ ] T035 [P] [US4] Add unit test in src/assembler/symbol_table.rs for adding constant to table
- [ ] T036 [P] [US4] Add unit test in src/assembler/symbol_table.rs for adding label to table
- [ ] T037 [P] [US4] Add unit test in src/assembler/symbol_table.rs for lookup returning correct kind
- [ ] T038 [P] [US4] Add integration test in tests/assembler_tests.rs for mixed constants and labels in same program
- [ ] T039 [P] [US4] Add integration test in tests/assembler_tests.rs for constants in all addressing modes
- [ ] T040 [P] [US4] Verify existing Klaus functional test still passes with new symbol table changes

**Validation**: Run full test suite including Klaus functional test

---

## Phase 7: Testing & Validation

**Goal**: Comprehensive testing to ensure feature completeness, backward compatibility, and no regressions.

**Prerequisites**: Phases 3-6 complete (all functionality implemented)

**Acceptance Criteria**:
- All new tests pass
- All existing tests still pass (backward compatibility)
- Klaus functional test still passes (no regression)
- Edge cases handled correctly

**Test Criteria**:
- Constants work in all addressing modes
- Error messages are clear and actionable
- Existing assembly code (without constants) works unchanged
- Performance acceptable (no significant slowdown)

### Tasks

- [ ] T041 [P] Run full cargo test suite and verify all 935+ tests pass
- [ ] T042 [P] Run Klaus functional test with --ignored flag and verify it passes
- [ ] T043 Add integration test in tests/assembler_tests.rs for complex program with many constants
- [ ] T044 Add integration test in tests/assembler_tests.rs for backward compatibility (existing code without constants)

**Validation**: All tests green, no regressions

---

## Phase 8: Polish & Documentation

**Goal**: Update documentation and provide examples for users.

**Prerequisites**: Phase 7 complete (all functionality tested)

**Acceptance Criteria**:
- CLAUDE.md updated with constant syntax
- Example program demonstrating constants
- Code comments clear and helpful

### Tasks

- [ ] T045 [P] Update CLAUDE.md in repository root with constant syntax and examples
- [ ] T046 [P] Create examples/constants.rs demonstrating constant usage patterns

**Validation**: Documentation clear and examples run successfully

---

## Dependencies & Execution Order

### Critical Path

```
T001-T002 (Setup)
    ↓
T003-T008 (Foundational - must complete before any user story)
    ↓
T009-T016 (US1 - Define constants)
    ↓
T017-T023 (US2 - Use constants - depends on US1)
    ↓
T024-T034 (US3 - Error handling - depends on US1 & US2)
    ↓
T035-T040 (US4 - Symbol kind validation - depends on US1-US3)
    ↓
T041-T044 (Testing & Validation)
    ↓
T045-T046 (Polish & Documentation)
```

### Parallel Opportunities

**Within Foundational Phase (after T003-T004)**:
- T005-T006 (Symbol.address rename in main module) can run in parallel with T007-T008 (update tests)

**Within US1 (after T009-T010)**:
- T011 (parser logic) + T012 (Pass 1 processing) can progress in parallel
- Tests T013-T016 are all parallelizable

**Within US2 (after T017)**:
- Tests T020-T023 are all parallelizable

**Within US3 (before error detection)**:
- T024-T027 (add error variants) are all parallelizable
- Tests T031-T034 are all parallelizable

**Within US4**:
- All tests T035-T040 are parallelizable

**Within Phase 7**:
- T041-T042 can run in parallel
- T043-T044 can run in parallel

**Within Phase 8**:
- T045-T046 are parallelizable

### Blocking Dependencies

**Foundational phase blocks everything**:
- T003-T008 must complete before any user story work
- Reason: SymbolKind enum and Symbol struct changes are used everywhere

**US1 blocks US2**:
- T009-T016 must complete before T017-T023
- Reason: Can't use constants until they can be defined and stored

**US1 & US2 block US3**:
- T009-T023 must complete before T024-T034
- Reason: Error handling requires working definition and usage

**US1-US3 block US4**:
- T009-T034 must complete before T035-T040
- Reason: Validation tests require full functionality

### Per-Story Independence

**US1 (Define)**: Independently testable
- After T016, can test that constants are parsed and stored
- No dependencies on other user stories

**US2 (Use)**: Depends on US1 only
- After T023, can test that constants work as operands
- Independent of US3 & US4

**US3 (Errors)**: Depends on US1 & US2
- After T034, can test all error scenarios
- Independent of US4

**US4 (Distinction)**: Validation layer
- Depends on US1-US3 for full functionality
- Tests the integration of all features

---

## MVP Definition

**Minimum Viable Product** = US1 + US2

Delivers:
- Define constants with `NAME = VALUE`
- Use constants in code as operands
- Basic validation (name format)

Deferred to post-MVP:
- Comprehensive error handling (US3)
- Full symbol kind validation (US4)

**MVP Task Range**: T001-T023 (23 tasks)

**Rationale**: US1 + US2 provide core value - programmers can define and use constants. Error handling and validation can be added incrementally.

---

## Implementation Strategy

### Phase Ordering Rationale

1. **Setup (T001-T002)**: Ensure clean baseline
2. **Foundational (T003-T008)**: Blocking changes that all stories need
3. **US1 (T009-T016)**: Core functionality - must define before use
4. **US2 (T017-T023)**: Natural next step - use what's defined
5. **US3 (T024-T034)**: Error handling improves usability
6. **US4 (T035-T040)**: Validation ensures correctness
7. **Testing (T041-T044)**: Comprehensive validation
8. **Polish (T045-T046)**: User-facing documentation

### Incremental Delivery

Each user story delivers a testable increment:
- After US1: Can define constants (testable)
- After US2: Can use constants (testable)
- After US3: Get clear errors (testable)
- After US4: Full validation (testable)

### Risk Mitigation

- **Early testing**: Integration tests after each user story
- **Backward compatibility**: Existing tests run throughout
- **Klaus test**: Final validation of no regression

---

## File Path Reference

| Component | File Path |
|-----------|-----------|
| Symbol struct | src/assembler.rs |
| SymbolKind enum | src/assembler.rs |
| ErrorType enum | src/assembler.rs |
| AssemblyLine struct | src/assembler/parser.rs |
| Parser logic | src/assembler/parser.rs |
| Symbol table | src/assembler/symbol_table.rs |
| Encoder/resolver | src/assembler/encoder.rs |
| Integration tests | tests/assembler_tests.rs |
| Klaus test | tests/functional_klaus.rs |
| Documentation | CLAUDE.md |
| Examples | examples/constants.rs (new) |

---

## Success Criteria

### Functional

- ✅ Can define constants with `NAME = VALUE`
- ✅ Can use constants in all addressing modes
- ✅ Constants work in immediate, zero page, absolute, indexed addressing
- ✅ Constants must be defined before use (no forward refs)
- ✅ Clear errors for undefined, duplicate, collision
- ✅ Backward compatible (existing code works)

### Non-Functional

- ✅ Zero external dependencies added
- ✅ All existing tests pass (935+ tests)
- ✅ Klaus functional test passes (96M+ cycles)
- ✅ No performance regression (<5% slowdown acceptable)
- ✅ Clear documentation and examples

### Quality

- ✅ Code follows Rust best practices
- ✅ Clear error messages with line numbers
- ✅ Comprehensive test coverage
- ✅ Constitution principles maintained

---

## Notes

- **Breaking Change**: Symbol.address renamed to Symbol.value (12 locations to update)
- **Test Strategy**: Integration tests after each user story, comprehensive validation at end
- **Parallel Execution**: 18 of 46 tasks (39%) can be parallelized
- **MVP Scope**: US1 + US2 (23 tasks) delivers core functionality

**Total Estimated Effort**: ~8-12 hours for full implementation + testing + documentation
