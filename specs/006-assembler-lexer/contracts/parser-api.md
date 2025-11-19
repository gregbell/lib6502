# Parser API Contract

**Version**: 2.0.0 (Post-Lexer Refactoring)
**Module**: `src/assembler/parser.rs`
**Purpose**: Updated parser interface consuming TokenStream instead of raw strings

## Overview

The parser module transforms a `TokenStream` (produced by the lexer) into structured `AssemblyLine` representations. This is the second phase of assembly, converting tokens into syntax trees.

**Key Change from v1.0**: Parser no longer performs string parsing or number conversion. It consumes typed tokens with pre-parsed values.

---

## Public Types

### AssemblyLine (Updated)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AssemblyLine {
    pub line_number: usize,

    // Constant assignment (NAME = VALUE)
    pub constant: Option<(String, String)>,  // UNCHANGED

    // Label definition (LABEL:)
    pub label: Option<String>,               // UNCHANGED

    // Instruction mnemonic (LDA, STA, etc.)
    pub mnemonic: Option<String>,            // UNCHANGED

    // Operand (now pre-validated by lexer)
    pub operand: Option<Operand>,            // NEW: was Option<String>

    // Directive (.org, .byte, .word)
    pub directive: Option<AssemblerDirective>,  // UNCHANGED

    // Comment text
    pub comment: Option<String>,             // UNCHANGED

    // Source span for error reporting
    pub span: (usize, usize),                // UNCHANGED
}
```

**Breaking Change**:
- `operand` field changes from `Option<String>` to `Option<Operand>`
- Operand is now typed (no string parsing needed)

### Operand (NEW)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    // Direct values
    Immediate(u16),               // #$42
    Absolute(u16),                // $1234
    ZeroPage(u8),                 // $FF

    // Indexed
    AbsoluteX(u16),               // $1234,X
    AbsoluteY(u16),               // $1234,Y
    ZeroPageX(u8),                // $FF,X
    ZeroPageY(u8),                // $FF,Y

    // Indirect
    Indirect(u16),                // ($1234)
    IndirectX(u8),                // ($FF,X)
    IndirectY(u8),                // ($FF),Y

    // Relative (for branches)
    Relative(i8),                 // Calculated offset

    // Symbolic
    Label(String),                // Forward/backward reference
    Constant(String),             // Named constant reference

    // Special
    Accumulator,                  // A (for ASL A, ROL A, etc.)
}
```

**Rationale**:
- Parser receives typed operands instead of strings
- Addressing mode detection moves from runtime string parsing to compile-time type checking
- Enables better error messages (type mismatch vs malformed string)

---

## Public Functions

### parse_line() - Updated Signature

```rust
pub fn parse_line(
    stream: &mut TokenStream,
    line_number: usize
) -> Result<Option<AssemblyLine>, ParserError>
```

**Changes from v1.0**:
- **Old**: `parse_line(line: &str, line_number: usize)`
- **New**: `parse_line(stream: &mut TokenStream, line_number: usize)`

**Parameters**:
- `stream: &mut TokenStream` - Token source (mutable for consume operations)
- `line_number: usize` - Current line number for error reporting

**Returns**:
- `Ok(Some(AssemblyLine))` - Parsed line if non-empty
- `Ok(None)` - Empty line or comment-only line
- `Err(ParserError)` - Syntax error (unexpected token, wrong grammar, etc.)

**Behavior**:
- Consumes tokens from stream until newline or EOF
- Skips whitespace transparently
- Builds AssemblyLine from token sequence
- Returns `None` for blank lines (optimization - no allocations)

**Error Handling**:
- Returns first syntax error encountered
- ParserError includes token location from stream
- Does NOT handle lexical errors (those come from lexer phase)

---

### parse_operand() - New Internal Function

```rust
fn parse_operand(stream: &mut TokenStream) -> Result<Operand, ParserError>
```

**Purpose**: Convert token sequence into typed `Operand`.

**Token Patterns**:

| Token Sequence | Operand Type | Example |
|----------------|--------------|---------|
| `Hash Number` | `Immediate(n)` | `#$42` → `Immediate(0x42)` |
| `Number` | `Absolute(n)` or `ZeroPage(n)` | `$FF` → `ZeroPage(0xFF)`, `$100` → `Absolute(0x100)` |
| `Number Comma Identifier("X")` | `AbsoluteX(n)` or `ZeroPageX(n)` | `$200,X` → `AbsoluteX(0x200)` |
| `Number Comma Identifier("Y")` | `AbsoluteY(n)` or `ZeroPageY(n)` | `$80,Y` → `ZeroPageY(0x80)` |
| `LParen Number RParen` | `Indirect(n)` | `($FFFC)` → `Indirect(0xFFFC)` |
| `LParen Number Comma Identifier("X") RParen` | `IndirectX(n)` | `($40,X)` → `IndirectX(0x40)` |
| `LParen Number RParen Comma Identifier("Y")` | `IndirectY(n)` | `($40),Y` → `IndirectY(0x40)` |
| `Identifier("A")` | `Accumulator` | `A` → `Accumulator` |
| `Identifier(name)` | `Label(name)` or `Constant(name)` | `START` → `Label("START")` |

**Validation**:
- Zero-page values must be ≤ 0xFF
- Indirect indexed must use zero-page addresses
- Index register must be exactly "X" or "Y"

**Errors**:
- `UnexpectedToken` - Wrong token type at position
- `InvalidAddressingMode` - Token sequence doesn't match any pattern
- `ValueOutOfRange` - Number too large for addressing mode

---

## TokenStream Interface

### Required Methods for Parser

```rust
impl TokenStream {
    // Lookahead (no state change)
    pub fn peek(&self) -> Option<&Token>;
    pub fn peek_n(&self, n: usize) -> Option<&Token>;

    // Consumption (advances position)
    pub fn consume(&mut self) -> Option<Token>;
    pub fn expect(&mut self, expected: TokenType) -> Result<Token, ParserError>;

    // Utility
    pub fn skip_whitespace(&mut self);
    pub fn skip_until_newline(&mut self);
    pub fn is_eof(&self) -> bool;
    pub fn current_location(&self) -> (usize, usize);  // (line, column)
}
```

**Contract**:
- `peek()` never returns `Eof` if tokens remain
- `consume()` advances position by exactly 1
- `skip_whitespace()` is idempotent (safe to call multiple times)
- `expect()` generates helpful error with token type name

---

## Migration Guide (Internal API)

### Before (v1.0 - String Parsing)

```rust
// Old approach: parse strings inline
pub fn parse_line(line: &str, line_number: usize) -> Option<AssemblyLine> {
    let trimmed = line.trim();

    // Manual string parsing
    if let Some(colon_pos) = trimmed.find(':') {
        let label = trimmed[..colon_pos].trim().to_uppercase();
        // ... more string slicing ...
    }

    // Parse operand from string
    if operand_str.starts_with('#') {
        let num_str = &operand_str[1..];
        if num_str.starts_with('$') {
            // Parse hex manually...
        }
    }
}
```

### After (v2.0 - Token Consumption)

```rust
// New approach: consume typed tokens
pub fn parse_line(
    stream: &mut TokenStream,
    line_number: usize
) -> Result<Option<AssemblyLine>, ParserError> {
    stream.skip_whitespace();

    // Token-based parsing
    let label = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = stream.peek() {
        if matches!(stream.peek_n(1), Some(Token { token_type: TokenType::Colon, .. })) {
            let name = stream.consume().unwrap();  // Consume identifier
            stream.consume();  // Consume colon
            Some(name)
        } else {
            None
        }
    } else {
        None
    };

    // Parse operand from tokens
    let operand = if let Some(Token { token_type: TokenType::Hash, .. }) = stream.peek() {
        stream.consume();  // Consume '#'
        let value = match stream.consume() {
            Some(Token { token_type: TokenType::HexNumber(n), .. }) => n,
            Some(Token { token_type: TokenType::DecimalNumber(n), .. }) => n,
            _ => return Err(ParserError::ExpectedNumber),
        };
        Some(Operand::Immediate(value))
    } else {
        // ... other patterns
    };
}
```

**Benefits**:
- No string slicing (safer, no panics on bad indices)
- No manual number parsing (lexer already did it)
- Better error locations (token-level precision)
- 30%+ less code (target per SC-003)

---

## Error Handling (Updated)

### ParserError (Extended)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    // Existing variants
    UnexpectedToken { expected: String, found: Token },
    InvalidMnemonic { mnemonic: String, line: usize },

    // New variants for token-based parsing
    ExpectedNumber { found: Token },
    ExpectedIdentifier { found: Token },
    InvalidAddressingMode { tokens: Vec<Token> },
    ValueOutOfRange { value: u16, max: u16, mode: String },
}
```

**Distinction from LexerError**:
- `LexerError` - Malformed token ($ZZ, %222)
- `ParserError` - Wrong token type or sequence (expected number, found identifier)

---

## Performance Contract

**Targets** (from Technical Context):
- Parser throughput: >10,000 lines/sec
- Lexer + Parser combined: <5% slower than old parser
- Memory: No parser-side allocations for number parsing (lexer did it)

**Measurements**:
- Benchmark on 1,000-line real-world program
- Compare total assembly time (old vs new)
- Profile to verify parser complexity reduction

---

## Testing Contract

**Unit Tests Required**:
- Each addressing mode has parse test
- Error cases (wrong token type, out of range value)
- Edge cases (accumulator mode "A", label-only lines)

**Integration Tests**:
- All existing `tests/assembler_tests.rs` pass unchanged (FR-009)
- Output is bit-for-bit identical to pre-refactoring (SC-004)

**Regression Prevention**:
- Klaus functional test continues to pass
- All 1,470+ existing tests pass without modification

---

## Compatibility Guarantees

**Public API** (maintained):
- `assemble(source: &str) -> Result<AssemblerOutput, Vec<AssemblerError>>` - UNCHANGED
- `AssemblerOutput` struct - UNCHANGED
- `AssemblerError` display format - IMPROVED (better locations)

**Internal API** (breaking changes):
- `parse_line()` signature changes (consumers must update)
- `AssemblyLine::operand` type changes (String → Operand)
- New dependency on `TokenStream` type

**Migration Path**:
- Internal consumers (only `assemble()` function) updated in same PR
- External consumers (none - parser.rs is internal module) unaffected
