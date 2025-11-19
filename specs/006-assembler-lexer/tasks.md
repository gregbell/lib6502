# Implementation Tasks: Assembler Lexer and Parser Architecture

**Feature**: 006-assembler-lexer
**Branch**: `006-assembler-lexer`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Overview

This task breakdown implements a proper lexer/parser architecture for the assembler, separating tokenization (lexer) from syntax analysis (parser). The refactoring is organized by user story priority to enable incremental delivery and independent testing.

**Total Tasks**: 45 tasks across 6 phases
**Estimated Effort**: 3-5 days for full implementation
**MVP Scope**: Phase 3 (US1) - delivers working lexer with token stream inspection

---

## Task Summary by Phase

| Phase | User Story | Task Count | Can Run Independently? |
|-------|------------|-----------|------------------------|
| Phase 1 | Setup | 3 | Yes (project prep) |
| Phase 2 | Foundational | 8 | No (US1 depends on this) |
| Phase 3 | US1 (P1) | 15 | **Yes** (MVP - token debugging) |
| Phase 4 | US2 (P2) | 12 | Depends on US1 |
| Phase 5 | US3 (P3) | 5 | Depends on US1, US2 |
| Phase 6 | Polish | 2 | Depends on all |

---

## Dependencies Between User Stories

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational) ← blocking for all user stories
    ↓
Phase 3 (US1: Token Stream) ← MVP, independently testable
    ↓
Phase 4 (US2: Parser Refactor) ← depends on US1
    ↓
Phase 5 (US3: Code Clarity) ← depends on US1 + US2
    ↓
Phase 6 (Polish)
```

**Independent Testing**:
- **US1**: Run `cargo test lexer` to verify token stream works
- **US2**: Add `.align` directive without touching lexer.rs
- **US3**: Code review confirms clear layer boundaries

---

## Phase 1: Setup (Project Preparation)

**Goal**: Prepare project for lexer implementation

### Tasks

- [ ] T001 Review existing `src/assembler/parser.rs` to understand current string parsing approach
- [ ] T002 Create `src/assembler/lexer.rs` stub module with module declaration in `src/assembler.rs`
- [ ] T003 Create `tests/lexer_tests.rs` test file for lexer unit tests

**Completion Criteria**: Module structure ready, can run `cargo build` successfully

---

## Phase 2: Foundational (Blocking Prerequisites)

**Goal**: Implement core lexer types that all user stories depend on

**Note**: These tasks must complete before any user story can be independently tested.

### Tasks

- [ ] T004 [P] Define `TokenType` enum in `src/assembler/lexer.rs` with all variants per data-model.md
- [ ] T005 [P] Define `Token` struct in `src/assembler/lexer.rs` with token_type, line, column, length fields
- [ ] T006 [P] Define `LexerError` enum in `src/assembler.rs` with InvalidHexDigit, InvalidBinaryDigit, NumberTooLarge variants
- [ ] T007 [P] Extend `ErrorType` enum in `src/assembler.rs` with `LexicalError(LexerError)` variant
- [ ] T008 Implement `Display` trait for `LexerError` in `src/assembler.rs`
- [ ] T009 [P] Define `Lexer` struct in `src/assembler/lexer.rs` with source, chars, current, line, line_start fields
- [ ] T010 [P] Define `TokenStream` struct in `src/assembler/lexer.rs` with tokens vec and position field
- [ ] T011 Implement `TokenStream::new()` constructor in `src/assembler/lexer.rs`

**Completion Criteria**: Core types compile, no tests failing

---

## Phase 3: User Story 1 - Token Stream Debugging (P1) [MVP]

**Story Goal**: Assembler developers can inspect token streams to distinguish lexical from syntactic errors

**Independent Test**: Feed assembly source to lexer, inspect tokens without running full assembler
- Example: `let tokens = tokenize("LDA #$42")?; assert_eq!(tokens[0].token_type, Identifier("LDA"));`

**Why This Is MVP**: Delivers immediate value—token inspection works independently before parser refactor

### Entity: Lexer Core Implementation

- [ ] T012 [P] [US1] Implement `Lexer::new()` constructor in `src/assembler/lexer.rs`
- [ ] T013 [US1] Implement `Lexer::advance()` helper method to move to next character in `src/assembler/lexer.rs`
- [ ] T014 [US1] Implement `Lexer::peek()` helper method to look at current character in `src/assembler/lexer.rs`
- [ ] T015 [US1] Implement `Lexer::column()` helper to calculate current column offset in `src/assembler/lexer.rs`

### Entity: Token Scanners

- [ ] T016 [P] [US1] Implement `Lexer::scan_identifier()` for [a-zA-Z][a-zA-Z0-9_]* in `src/assembler/lexer.rs`
- [ ] T017 [P] [US1] Implement `Lexer::scan_hex_number()` for $[0-9A-Fa-f]+ with validation in `src/assembler/lexer.rs`
- [ ] T018 [P] [US1] Implement `Lexer::scan_binary_number()` for %[01]+ with validation in `src/assembler/lexer.rs`
- [ ] T019 [P] [US1] Implement `Lexer::scan_decimal_number()` for [0-9]+ with validation in `src/assembler/lexer.rs`
- [ ] T020 [P] [US1] Implement `Lexer::scan_comment()` for ;.* until newline in `src/assembler/lexer.rs`
- [ ] T021 [P] [US1] Implement `Lexer::scan_single_char_token()` for operators (colon, comma, hash, etc.) in `src/assembler/lexer.rs`

### Entity: Main Lexer Logic

- [ ] T022 [US1] Implement `Lexer::next_token()` with match on current character in `src/assembler/lexer.rs`
- [ ] T023 [US1] Implement `Lexer::tokenize()` public function returning `Result<Vec<Token>, Vec<LexerError>>` in `src/assembler/lexer.rs`
- [ ] T024 [US1] Add `pub use lexer::{tokenize, Token, TokenType, LexerError};` to `src/assembler.rs`

### Testing: Lexer Verification

- [ ] T025 [P] [US1] Add test for identifier tokenization in `tests/lexer_tests.rs`
- [ ] T026 [P] [US1] Add test for hex number tokenization ($42 → HexNumber(0x42)) in `tests/lexer_tests.rs`
- [ ] T027 [P] [US1] Add test for binary number tokenization (%01000010 → BinaryNumber(66)) in `tests/lexer_tests.rs`
- [ ] T028 [P] [US1] Add test for decimal number tokenization (42 → DecimalNumber(42)) in `tests/lexer_tests.rs`
- [ ] T029 [P] [US1] Add test for operator tokenization (#, ,, :, etc.) in `tests/lexer_tests.rs`
- [ ] T030 [P] [US1] Add test for comment preservation in `tests/lexer_tests.rs`
- [ ] T031 [P] [US1] Add test for invalid hex digit error ($ZZ → InvalidHexDigit) in `tests/lexer_tests.rs`
- [ ] T032 [P] [US1] Add test for number overflow error (>65535 → NumberTooLarge) in `tests/lexer_tests.rs`
- [ ] T033 [P] [US1] Add test for line/column tracking accuracy in `tests/lexer_tests.rs`

**Phase 3 Completion Criteria** (US1 Independent Test):
- ✅ Can call `tokenize(source)` and get token vector
- ✅ Tokens include correct types and source locations
- ✅ Lexical errors (invalid hex, overflow) reported separately
- ✅ Run `cargo test lexer` passes all tests

---

## Phase 4: User Story 2 - Parser Extensibility (P2)

**Story Goal**: Maintainers can add directives by modifying only parser, not lexer

**Independent Test**: Add `.align` directive using existing Number tokens, verify no lexer.rs changes
- Example: Implement `.align 256` parsing using TokenType::DecimalNumber without modifying lexer

**Depends On**: US1 (needs working tokenize() function)

### Entity: TokenStream Implementation

- [ ] T034 [P] [US2] Implement `TokenStream::peek()` to look at current token in `src/assembler/lexer.rs`
- [ ] T035 [P] [US2] Implement `TokenStream::peek_n(n)` for multi-token lookahead in `src/assembler/lexer.rs`
- [ ] T036 [P] [US2] Implement `TokenStream::consume()` to advance and return token in `src/assembler/lexer.rs`
- [ ] T037 [P] [US2] Implement `TokenStream::skip_whitespace()` helper in `src/assembler/lexer.rs`
- [ ] T038 [P] [US2] Implement `TokenStream::is_eof()` check in `src/assembler/lexer.rs`
- [ ] T039 [P] [US2] Implement `TokenStream::current_location()` for error reporting in `src/assembler/lexer.rs`

### Integration: Parser Refactoring

- [ ] T040 [US2] Update `parse_line()` signature to accept `&mut TokenStream` instead of `&str` in `src/assembler/parser.rs`
- [ ] T041 [US2] Refactor `parse_line()` to consume tokens instead of string slicing in `src/assembler/parser.rs`
- [ ] T042 [US2] Update `assemble()` to call `tokenize()` before parsing in `src/assembler.rs`
- [ ] T043 [US2] Remove inline number parsing from parser (use Token::HexNumber, etc.) in `src/assembler/parser.rs`
- [ ] T044 [US2] Update error handling to preserve LexerError locations in `src/assembler.rs`

### Testing: Integration Verification

- [ ] T045 [P] [US2] Run full test suite `cargo test` to verify all existing tests pass with refactored parser
- [ ] T046 [P] [US2] Add integration test for `.align` directive (parser-only change, no lexer mod) in `tests/assembler_tests.rs`
- [ ] T047 [P] [US2] Verify bit-for-bit identical output (SC-004) with existing programs in `tests/assembler_tests.rs`
- [ ] T048 [US2] Add test for error message clarity (lexical vs syntactic distinction) in `tests/assembler_tests.rs`

**Phase 4 Completion Criteria** (US2 Independent Test):
- ✅ All 1,470+ existing tests pass unchanged
- ✅ Can add `.align` directive without modifying `src/assembler/lexer.rs`
- ✅ Error messages distinguish "invalid hex digit" from "expected operand"
- ✅ Output is bit-for-bit identical to pre-refactor

---

## Phase 5: User Story 3 - Code Clarity (P3)

**Story Goal**: New contributors understand layer boundaries and can locate bugs quickly

**Independent Test**: Code review confirms clear separation, new contributor can answer "where do I fix X?" in < 5 minutes

**Depends On**: US1 + US2 (needs both lexer and refactored parser)

### Documentation

- [ ] T049 [P] [US3] Add module-level documentation to `src/assembler/lexer.rs` with examples
- [ ] T050 [P] [US3] Add module-level documentation to `src/assembler/parser.rs` explaining token consumption
- [ ] T051 [P] [US3] Update `src/lib.rs` or `README.md` with lexer/parser architecture diagram
- [ ] T052 [P] [US3] Add inline comments explaining token type conversions in `src/assembler/lexer.rs`
- [ ] T053 [US3] Create example syntax highlighter using tokenize() in `examples/syntax_highlighter.rs` (SC-005)

**Phase 5 Completion Criteria** (US3 Independent Test):
- ✅ Module docs clearly explain responsibilities (lexer=tokenize, parser=syntax)
- ✅ Code review confirms minimal coupling between layers
- ✅ Example tool demonstrates token stream reusability (SC-005)
- ✅ New contributor test: Can identify "hex parsing bug" location in <5 min

---

## Phase 6: Polish & Cross-Cutting Concerns

**Goal**: Performance verification, final cleanup

### Tasks

- [ ] T054 Run `cargo clippy -- -D warnings` and fix any lints in lexer/parser code
- [ ] T055 Benchmark assembly throughput (verify <5% overhead per Technical Context) using `hyperfine` or criterion

**Completion Criteria**: No clippy warnings, performance target met

---

## Parallel Execution Opportunities

Tasks marked with `[P]` can be executed in parallel within their phase:

**Phase 2 (Foundational)**:
- T004, T005, T006, T007, T009, T010 can run simultaneously (different types, no dependencies)

**Phase 3 (US1)**:
- T016-T021 (token scanners) are fully independent
- T025-T033 (tests) are fully independent after T023 completes

**Phase 4 (US2)**:
- T034-T039 (TokenStream methods) are independent
- T045-T047 (integration tests) are independent after T044 completes

**Phase 5 (US3)**:
- T049-T052 (documentation) are fully independent

**Example Parallel Workflow (US1)**:
```bash
# Step 1: Complete foundational (sequential)
cargo build  # T004-T011

# Step 2: Implement scanners in parallel (split work)
# Developer A: T016, T017 (identifier, hex)
# Developer B: T018, T019 (binary, decimal)
# Developer C: T020, T021 (comment, operators)

# Step 3: Wire up main logic (sequential)
# T022, T023, T024

# Step 4: Tests in parallel (split work)
# Developer A: T025, T026, T027
# Developer B: T028, T029, T030
# Developer C: T031, T032, T033

cargo test lexer  # Verify US1 complete
```

---

## Implementation Strategy

### MVP First (Phase 3 Only)

**Minimum Viable Product**: Deliver US1 (token stream debugging) first
- Value: Developers can immediately inspect tokens to debug issues
- Scope: Lexer implementation + tests (T001-T033)
- Validation: `tokenize("LDA #$42")` returns correct token stream
- Effort: ~1-2 days

**Incremental Delivery**:
1. **Week 1**: US1 (MVP) - token stream works independently
2. **Week 1-2**: US2 - parser refactored, all tests green
3. **Week 2**: US3 - documentation and clarity improvements
4. **Week 2**: Polish - performance and cleanup

### Test-Driven Development

While not strictly TDD (tests-first), the workflow emphasizes early testing:
- Write test stubs early (T003)
- Implement feature + test together (T016 + T026 in same session)
- Verify independently (run `cargo test lexer` after each scanner)

### Risk Mitigation

**Risk**: Parser refactor breaks existing tests
**Mitigation**: US1 delivers working lexer first, parser refactor is separate phase (US2)

**Risk**: Performance regression (>5% overhead)
**Mitigation**: T055 benchmarks early, profile if needed

**Risk**: Incomplete test coverage
**Mitigation**: T025-T033 cover all token types, T045-T048 verify integration

---

## Verification Checklist

Before marking feature complete, verify:

- [ ] All 1,470+ existing tests pass (`cargo test`)
- [ ] Klaus functional test passes (`cargo test klaus_6502_functional_test -- --ignored`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Performance < 5% slower than baseline (T055)
- [ ] Can add `.align` directive without touching `src/assembler/lexer.rs` (US2 test)
- [ ] Example syntax highlighter works (`cargo run --example syntax_highlighter`)
- [ ] Parser LOC reduced by 30%+ (measure `src/assembler/parser.rs` line count)
- [ ] Error messages distinguish lexical vs syntactic (manual test)

---

## Success Metrics (from Spec)

- **SC-001**: 90% of syntax additions require zero lexer changes → Tested by US2
- **SC-002**: Error messages distinguish lexical vs syntactic → Tested by T048
- **SC-003**: Parser LOC reduces 30%+ → Measured before/after T043
- **SC-004**: Bit-for-bit identical output → Tested by T047
- **SC-005**: Token stream reusable for external tools → Demonstrated by T053

---

## Next Steps

1. **Start with MVP**: Complete Phase 1-3 (T001-T033) for token stream debugging
2. **Validate independently**: Run `cargo test lexer` to verify US1 works standalone
3. **Proceed incrementally**: Add US2 (parser refactor) only after US1 is solid
4. **Document as you go**: Write module docs (Phase 5) alongside implementation

**Ready to begin?** Start with T001 (review existing parser) to understand current architecture before adding lexer.
