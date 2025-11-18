# Feature Specification: Assembler Lexer and Parser Architecture

**Feature Branch**: `001-assembler-lexer`
**Created**: 2025-11-18
**Status**: Draft
**Input**: User description: "I want create a proper lexing layer in the assembler to dramatically simplify the code. The parser should become much simpler too. Let's introduce proper assembler architecture."

## User Scenarios & Testing

### User Story 1 - Assembler Developer Debugging Token Stream (Priority: P1)

As an assembler developer, when I encounter a parsing error, I want to see exactly what tokens the lexer produced so I can quickly identify if the issue is in lexical analysis (wrong tokens) or syntactic analysis (wrong grammar rules).

**Why this priority**: This is the foundation for all other improvements. Without proper token separation, debugging and extending the assembler remains unnecessarily difficult. This directly addresses the project's Clarity & Hackability principle.

**Independent Test**: Can be fully tested by feeding assembly source to the lexer and inspecting the token stream output. Delivers immediate value by making tokenization errors visible and diagnosable without running the full assembler pipeline.

**Acceptance Scenarios**:

1. **Given** assembly source `LDA #$42`, **When** lexer processes the line, **Then** produces tokens: [Identifier("LDA"), Hash, Dollar, HexNumber("42"), Newline]
2. **Given** assembly source with syntax error `.byte $ZZ`, **When** lexer processes the line, **Then** produces clear token-level error indicating invalid hex digit 'Z'
3. **Given** assembly source `START: LDA TARGET,X ; comment`, **When** lexer processes the line, **Then** produces distinct tokens for label, mnemonic, operand parts, index register, and preserves comment for debugging context

---

### User Story 2 - Maintainer Adding New Directive (Priority: P2)

As a maintainer, when I want to add a new assembler directive (like `.align` or `.fill`), I want to only modify the parser grammar rules without touching the lexer, because the lexer already handles all the basic token types I need.

**Why this priority**: Extensibility is critical for long-term maintainability. Proper separation of concerns means new features only require changes in one layer, not cascading modifications across multiple parsing stages.

**Independent Test**: Can be tested by implementing a new directive (e.g., `.align 256`) using only existing token types, verifying that no lexer changes are needed and the parser correctly constructs the directive representation.

**Acceptance Scenarios**:

1. **Given** a new `.align` directive syntax, **When** maintainer adds parser rule for it, **Then** can parse `.align $100` using existing Number tokens without lexer modification
2. **Given** existing directives `.byte` and `.word`, **When** adding `.fill` directive, **Then** reuses existing comma-separated value parsing logic from parser layer
3. **Given** a need to extend label syntax (e.g., support `@local` labels), **When** modifying lexer token rules, **Then** all existing directive parsing continues to work without changes

---

### User Story 3 - Developer Understanding Code Flow (Priority: P3)

As a new contributor, when I read the assembler code, I want to clearly see the separation between "what are the raw tokens" (lexer), "what is the structure" (parser), and "what does it mean" (semantic analysis), so I can quickly locate where to make changes for different types of bugs.

**Why this priority**: Code clarity enables community contributions and reduces onboarding time. This aligns with the Constitution's emphasis on hackability and educational value.

**Independent Test**: Can be tested through code review and documentation walkthroughs, verifying that each layer has clear responsibilities and minimal coupling. Success means a new contributor can answer "where do I fix X?" questions in under 5 minutes.

**Acceptance Scenarios**:

1. **Given** a bug report "hex numbers aren't parsing correctly", **When** developer examines codebase, **Then** immediately identifies issue must be in lexer's number tokenization logic
2. **Given** a bug report "constants aren't resolving in indexed addressing", **When** developer examines codebase, **Then** immediately identifies issue must be in parser's operand resolution or semantic analysis, not lexer
3. **Given** a request to improve error messages, **When** developer reviews architecture, **Then** can clearly see error generation happens at appropriate layer (lexer for malformed tokens, parser for invalid syntax, semantic for undefined symbols)

---

### Edge Cases

- What happens when source contains Unicode characters or non-ASCII bytes (UTF-8 sequences)?
- How does lexer handle extremely long identifiers (> 1000 characters)?
- What happens when a line contains only whitespace tokens?
- How are unterminated string literals or malformed tokens handled at lexer boundary cases?
- What happens when source contains mixed line endings (CRLF vs LF) or no final newline?
- How does lexer handle maximum input size constraints (e.g., 100MB source file)?

## Requirements

### Functional Requirements

- **FR-001**: System MUST separate lexical analysis (tokenization) from syntactic analysis (parsing) into distinct phases
- **FR-002**: Lexer MUST convert assembly source text into a stream of typed tokens (identifier, number, operator, whitespace, comment, etc.)
- **FR-003**: Lexer MUST recognize all 6502 assembly token types: identifiers, decimal numbers, hex numbers ($-prefixed), binary numbers (%-prefixed), operators (colon, comma, hash, parentheses), and whitespace/comments
- **FR-004**: Lexer MUST preserve source location information (line, column) for every token to enable accurate error reporting
- **FR-005**: Parser MUST consume token streams instead of raw string slices
- **FR-006**: Parser MUST produce structured intermediate representation (parsed lines with typed operands) without performing string parsing operations
- **FR-007**: System MUST detect and report lexical errors (invalid hex digit, malformed number) separately from syntax errors (wrong operand format)
- **FR-008**: Lexer MUST handle all existing assembly source syntax without breaking changes to accepted inputs
- **FR-009**: System MUST maintain existing public API (`assemble()` function signature) so existing code continues to work
- **FR-010**: Lexer MUST be reusable independently for tooling (syntax highlighters, formatters, analysis tools)

### Key Entities

- **Token**: A lexical unit with type (Identifier, Number, Operator, etc.), value (the matched text or parsed number), and source location (line, column, length)
- **TokenStream**: Ordered sequence of tokens produced by lexer, with ability to peek ahead and consume tokens during parsing
- **Lexer**: Component that reads source text and produces TokenStream, handles character-level validation and number format conversion
- **Parser**: Component that consumes TokenStream and produces AssemblyLine structures, handles syntactic validation and structure building

## Success Criteria

### Measurable Outcomes

- **SC-001**: Developers can add new directives or addressing modes by modifying only parser layer, requiring zero lexer changes (measured by git diff showing no lexer.rs changes for 90% of syntax additions)
- **SC-002**: Assembler error messages distinguish between lexical errors (line 42: invalid hex digit 'G' in token) and syntactic errors (line 42: expected operand after mnemonic)
- **SC-003**: Parser module size decreases by at least 30% (measured in lines of code) by eliminating inline string parsing and number conversion logic
- **SC-004**: All existing assembly programs continue to assemble identically (bit-for-bit identical output) after refactoring
- **SC-005**: Token stream can be extracted and used independently by external tools without running the full assembler (demonstrated by example syntax highlighter utility)

## Assumptions

- The refactoring will be done incrementally in a way that maintains existing tests passing throughout
- Token types will follow standard assembler conventions (similar to other 6502 assemblers)
- Performance impact of adding a lexer pass will be negligible (< 5% assembly time increase) for typical programs
- Lexer will operate on UTF-8 strings but will only recognize ASCII tokens (non-ASCII in comments/strings will be preserved as-is)
- The feature will maintain zero external dependencies in keeping with project principles

## Dependencies

- Requires familiarity with current `src/assembler/parser.rs` implementation
- Depends on existing `AddressingMode` enum and instruction encoding infrastructure
- May impact `AssemblerError` struct to distinguish lexical vs syntactic error types

## Out of Scope

- Macro support or advanced preprocessing directives
- Performance optimizations beyond basic architecture improvements
- Changes to instruction encoding or addressing mode detection logic (only affects how operands are parsed, not what they mean)
- Syntax changes that would break existing valid assembly programs
- Localization or internationalization of error messages
