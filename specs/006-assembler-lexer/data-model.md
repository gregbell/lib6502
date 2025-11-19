# Data Model: Lexer and Parser Entities

**Phase**: Phase 1 - Design
**Date**: 2025-11-18
**Purpose**: Define core data structures for the lexer/parser architecture

## Entity Overview

This refactoring introduces three primary entities:
1. **Token** - A single lexical unit with type, value, and location
2. **TokenStream** - Sequence of tokens with iterator interface
3. **Lexer** - Stateful tokenizer that converts source text to tokens

These entities replace the ad-hoc string parsing currently scattered throughout `parser.rs`.

---

## Entity 1: Token

**Purpose**: Represents a single lexical unit in assembly source code.

**Fields**:
- `token_type: TokenType` - Classification and optional parsed value
- `line: usize` - Source line number (1-indexed for user display)
- `column: usize` - Column offset within line (0-indexed)
- `length: usize` - Character span for error highlighting

**Relationships**:
- Created by: `Lexer`
- Consumed by: `TokenStream` → `Parser`
- References: Source location (line/column) for error reporting

**Validation Rules** (FR-004):
- `line` must be ≥ 1 (no zero-indexed line numbers in user-facing output)
- `column` must be ≥ 0
- `length` must be > 0 (every token occupies at least one character)
- `token_type` must be valid enum variant

**Invariants**:
- Token location is immutable after creation (preserves source fidelity)
- Numeric token types (DecimalNumber, HexNumber, BinaryNumber) already contain parsed u16 value
- Source location points to first character of token in original source

**Example**:
```rust
// Source: "LDA #$42"
// Produces:
Token { token_type: Identifier("LDA"), line: 1, column: 0, length: 3 }
Token { token_type: Whitespace, line: 1, column: 3, length: 1 }
Token { token_type: Hash, line: 1, column: 4, length: 1 }
Token { token_type: HexNumber(0x42), line: 1, column: 5, length: 3 }  // $42 parsed
```

---

## Entity 2: TokenType (Enumeration)

**Purpose**: Classifies tokens and carries parsed values for numeric types.

**Variants**:

### Identifiers and Keywords
- `Identifier(String)` - Mnemonics, labels, symbol references (uppercase normalized)

### Numbers (Parsed Values)
- `DecimalNumber(u16)` - Decimal literals (0-65535)
- `HexNumber(u16)` - Hex literals with $ prefix (parsed to 0x0000-0xFFFF)
- `BinaryNumber(u16)` - Binary literals with % prefix (parsed to 0-65535)

### Operators and Punctuation
- `Colon` - `:` (label definition suffix)
- `Comma` - `,` (operand separator, indexed addressing)
- `Hash` - `#` (immediate mode prefix)
- `Dollar` - `$` (hex number prefix)
- `Percent` - `%` (binary number prefix)
- `Equal` - `=` (constant assignment operator)
- `LParen` - `(` (indirect addressing open)
- `RParen` - `)` (indirect addressing close)
- `Dot` - `.` (directive prefix)

### Structural
- `Whitespace` - Space/tab sequences (preserved for formatters)
- `Newline` - Line terminator (CRLF or LF normalized to single token)
- `Comment(String)` - Semicolon-prefixed comment text (excluding `;`)
- `Eof` - End of file marker

**Validation Rules** (FR-003):
- Identifier strings must be non-empty, ASCII-only
- Numeric values must fit in u16 (0-65535)
- Comment strings preserve original text after semicolon

**State Transitions**: N/A (immutable after creation)

---

## Entity 3: Lexer

**Purpose**: Converts source text into token stream.

**Fields**:
- `source: &str` - Reference to original source text (lifetime-bound)
- `chars: CharIndices<'_>` - Iterator over (byte_offset, char) pairs
- `current: Option<(usize, char)>` - Current character being examined
- `line: usize` - Current line number (starts at 1)
- `line_start: usize` - Byte offset where current line begins

**Relationships**:
- Input: Assembly source string
- Output: `Vec<Token>` (collected into TokenStream)
- Used by: `assemble()` function (first phase before parsing)

**Validation Rules** (FR-002, FR-008):
- Must accept all valid 6502 assembly syntax
- Must reject malformed tokens (invalid hex digits, etc.) with `LexerError`
- Must preserve source location accuracy (no off-by-one errors)

**State Transitions**:
```
Initial → Reading Token → Token Complete → Reading Token → ... → EOF
                ↓
         (on error) → LexerError
```

**Invariants**:
- `line` increments only on newline characters
- `line_start` updated only when `line` increments
- `column` always computed as `current_byte_offset - line_start`
- `current` is Some(...) until EOF, then None

**Methods** (behavioral interface):
- `new(source: &str) -> Lexer` - Initialize lexer on source
- `tokenize() -> Result<Vec<Token>, LexerError>` - Full tokenization pass
- `next_token() -> Result<Token, LexerError>` - Single token extraction (private)
- `scan_identifier()` - Match [a-zA-Z][a-zA-Z0-9_]*
- `scan_hex_number()` - Match $[0-9A-Fa-f]+ and parse to u16
- `scan_binary_number()` - Match %[01]+ and parse to u16
- `scan_decimal_number()` - Match [0-9]+ and parse to u16
- `scan_comment()` - Match ;.* until newline

---

## Entity 4: TokenStream

**Purpose**: Provides parser interface to token sequence with lookahead.

**Fields**:
- `tokens: Vec<Token>` - Complete token sequence (pre-parsed by lexer)
- `position: usize` - Current read position (index into tokens vec)

**Relationships**:
- Created from: `Vec<Token>` produced by `Lexer`
- Consumed by: `Parser` (via iterator-like interface)

**Validation Rules** (FR-005):
- Must support multi-token lookahead (peek, peek_n)
- Must not modify tokens (immutable iteration)
- Position must never exceed tokens.len()

**State Transitions**:
```
Start (pos=0) → Consume → Advance → Consume → ... → End (pos=len)
                   ↓                    ↓
                Peek (no advance)   Peek (no advance)
```

**Invariants**:
- `position ≤ tokens.len()` at all times
- Peek operations never modify `position`
- Consume increments `position` and returns owned token

**Methods**:
- `new(tokens: Vec<Token>) -> TokenStream` - Constructor
- `peek(&self) -> Option<&Token>` - Look at current token without advancing
- `peek_n(&self, n: usize) -> Option<&Token>` - Look ahead N tokens
- `consume(&mut self) -> Option<Token>` - Advance and return current token
- `expect(&mut self, expected: TokenType) -> Result<Token, ParseError>` - Consume or error
- `skip_whitespace(&mut self)` - Advance past whitespace/comments
- `is_eof(&self) -> bool` - Check if at end

---

## Entity 5: LexerError (Error Type)

**Purpose**: Represents lexical analysis errors (FR-007).

**Variants**:
- `InvalidHexDigit { ch: char, line: usize, column: usize }` - Non-hex char in $...
- `InvalidBinaryDigit { ch: char, line: usize, column: usize }` - Non-01 char in %...
- `NumberTooLarge { value: String, max: u16, line: usize, column: usize }` - Overflow
- `UnexpectedCharacter { ch: char, line: usize, column: usize }` - Invalid token start
- `UnterminatedString { line: usize, column: usize }` - Future-proofing for string literals

**Relationships**:
- Produced by: `Lexer` methods
- Wrapped by: `AssemblerError::LexicalError(LexerError)` variant
- Consumed by: Error reporting in `assemble()` function

**Validation Rules**:
- All errors must include precise source location (line, column)
- Error messages must be actionable (tell user what's wrong and where)

---

## Entity Lifecycle

```
Source Text (&str)
    ↓
Lexer::new(source)
    ↓
Lexer::tokenize() → Result<Vec<Token>, LexerError>
    ↓
TokenStream::new(tokens)
    ↓
Parser::parse(stream) → Result<Vec<AssemblyLine>, AssemblerError>
    ↓
Encoder → Vec<u8> (machine code)
```

---

## Validation Summary

**Cross-Entity Constraints**:
- Token source locations must reference valid positions in original source
- TokenStream position must never advance beyond token vec length
- Lexer line/column tracking must match Token location fields
- All numeric tokens (Decimal/Hex/Binary) must fit in u16 range

**Performance Constraints** (per Technical Context):
- Lexer tokenization: <5% overhead vs current parser
- TokenStream peek/consume: O(1) operations (vec indexing)
- Memory: Token vec size ≈ source_length / 3 (estimate 1 token per ~3 chars)

**Testing Requirements**:
- Each token type needs unit test demonstrating correct lexing
- Boundary conditions (max u16, line endings, EOF) must be tested
- Error cases (invalid hex, number overflow) must be tested
- Round-trip: tokenize → detokenize must preserve source semantics
