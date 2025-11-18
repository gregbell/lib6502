# Lexer API Contract

**Version**: 1.0.0
**Module**: `src/assembler/lexer.rs`
**Purpose**: Public interface for lexical analysis of 6502 assembly source code

## Overview

The lexer module provides functions to tokenize assembly source text into typed tokens with source location information. This is the first phase of assembly, converting raw text into structured tokens that the parser can consume.

---

## Public Types

### Token

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,        // 1-indexed line number
    pub column: usize,      // 0-indexed column within line
    pub length: usize,      // Character span of this token
}
```

**Guarantees**:
- `line >= 1` (user-facing line numbers start at 1)
- `column >= 0`
- `length > 0` (every token has non-zero width)
- Immutable after construction

### TokenType

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Identifiers
    Identifier(String),          // Mnemonics, labels, symbols (UPPERCASE)

    // Numbers (already parsed)
    DecimalNumber(u16),          // 0-65535
    HexNumber(u16),              // $0000-$FFFF (parsed)
    BinaryNumber(u16),           // %binary (parsed)

    // Operators
    Colon,                       // :
    Comma,                       // ,
    Hash,                        // #
    Dollar,                      // $
    Percent,                     // %
    Equal,                       // =
    LParen,                      // (
    RParen,                      // )
    Dot,                         // .

    // Structural
    Whitespace,                  // Space/tab sequences
    Newline,                     // Line terminator
    Comment(String),             // ; comment text
    Eof,                         // End of file
}
```

**Guarantees**:
- Identifiers are normalized to UPPERCASE
- Numbers are validated to fit in u16 (0-65535)
- Comment strings exclude the leading `;`

### LexerError

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    InvalidHexDigit {
        ch: char,
        line: usize,
        column: usize
    },
    InvalidBinaryDigit {
        ch: char,
        line: usize,
        column: usize
    },
    NumberTooLarge {
        value: String,
        max: u16,
        line: usize,
        column: usize
    },
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize
    },
}
```

**Guarantees**:
- All errors include precise source location (line, column)
- Errors are recoverable (lexer can continue after reporting)

---

## Public Functions

### tokenize()

```rust
pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<LexerError>>
```

**Purpose**: Convert assembly source text into sequence of tokens.

**Parameters**:
- `source: &str` - Assembly source code (UTF-8, but only ASCII tokens recognized)

**Returns**:
- `Ok(Vec<Token>)` - Complete token sequence on success
- `Err(Vec<LexerError>)` - All lexical errors found (may contain multiple errors)

**Behavior**:
- Scans source character-by-character
- Produces token for every syntactic element (including whitespace if requested)
- Normalizes identifiers to UPPERCASE
- Parses numbers immediately (hex, binary, decimal → u16)
- Preserves exact source locations for error reporting
- Handles CRLF and LF line endings uniformly

**Error Handling**:
- Returns ALL lexical errors found (doesn't stop at first error)
- Errors include precise location and context
- Invalid tokens are skipped, scanning continues

**Performance**:
- Single-pass O(n) where n = source length
- Zero allocations except for token vec and identifier/comment strings
- Adds <5% overhead vs direct string parsing

**Examples**:

```rust
// Success case
let source = "LDA #$42";
let tokens = tokenize(source).unwrap();
assert_eq!(tokens.len(), 5);  // Identifier, Whitespace, Hash, HexNumber, Eof

assert_eq!(tokens[0].token_type, TokenType::Identifier("LDA".to_string()));
assert_eq!(tokens[0].line, 1);
assert_eq!(tokens[0].column, 0);

assert_eq!(tokens[3].token_type, TokenType::HexNumber(0x42));
assert_eq!(tokens[3].column, 5);

// Error case
let source = ".byte $ZZ";
let errors = tokenize(source).unwrap_err();
assert_eq!(errors.len(), 1);
match &errors[0] {
    LexerError::InvalidHexDigit { ch, line, column } => {
        assert_eq!(*ch, 'Z');
        assert_eq!(*line, 1);
        assert_eq!(*column, 7);
    }
    _ => panic!("Wrong error type"),
}
```

**Preconditions**:
- Source must be valid UTF-8 (Rust str guarantee)
- None (accepts any UTF-8 string)

**Postconditions**:
- If Ok: Token sequence represents complete tokenization of source
- If Ok: Last token is always `TokenType::Eof`
- If Err: All invalid tokens are reported with locations

---

## Usage Patterns

### Pattern 1: Full Tokenization

```rust
use lib6502::assembler::lexer;

let source = r#"
START:
    LDA #$42
    STA $0200
"#;

match lexer::tokenize(source) {
    Ok(tokens) => {
        // Pass to parser
        let stream = TokenStream::new(tokens);
        // ...
    }
    Err(errors) => {
        for err in errors {
            eprintln!("Lexer error: {:?}", err);
        }
    }
}
```

### Pattern 2: Error Recovery

```rust
// Collect all errors before failing
let source = "LDA #$GG\nSTA #%222";  // Two errors
let errors = lexer::tokenize(source).unwrap_err();

assert_eq!(errors.len(), 2);
// Error 1: Invalid hex digit 'G' at line 1
// Error 2: Invalid binary digit '2' at line 2
```

### Pattern 3: External Tooling (Syntax Highlighter)

```rust
// Reuse lexer for IDE integration
let source = read_file("program.asm");
let tokens = lexer::tokenize(&source)?;

for token in tokens {
    let color = match token.token_type {
        TokenType::Identifier(_) => Color::Blue,
        TokenType::HexNumber(_) | TokenType::DecimalNumber(_) => Color::Green,
        TokenType::Comment(_) => Color::Gray,
        _ => Color::Default,
    };

    highlight(token.line, token.column, token.length, color);
}
```

---

## Implementation Notes

**Character Handling**:
- Recognizes only ASCII tokens (A-Z, 0-9, operators)
- UTF-8 in comments is preserved but not interpreted
- Mixed line endings (CRLF/LF) handled transparently

**Number Parsing**:
- Hex: `$` prefix required, accepts 0-9 A-F a-f
- Binary: `%` prefix required, accepts 0-1 only
- Decimal: No prefix, accepts 0-9
- All overflow (>65535) reported as `NumberTooLarge`

**Whitespace**:
- Spaces and tabs produce `Whitespace` tokens (for formatters)
- Can be skipped by parser using `TokenStream::skip_whitespace()`
- Consecutive whitespace collapses into single token

**Comments**:
- Semicolon `;` starts comment (6502 standard)
- Extends to end of line
- Text preserved (excluding `;` itself)

---

## Compatibility

**Backward Compatibility** (FR-009):
- Internal API only (not exposed in public `assemble()` function)
- Existing assembly programs assemble identically after refactoring
- Error messages improve but error locations remain accurate

**Forward Compatibility**:
- TokenType enum can be extended (e.g., for string literals in future)
- LexerError variants can be added without breaking consumers

---

## Testing Contract

**Unit Test Coverage** (required):
- Each TokenType variant has dedicated test
- Number boundary conditions (0, 255, 256, 65535, 65536)
- Error cases (invalid hex, binary, overflow)
- Line ending variants (LF, CRLF, no final newline)
- Edge cases (empty source, comment-only, whitespace-only)

**Integration Requirements**:
- All existing `assembler_tests.rs` must pass with lexer integration
- Bit-for-bit identical output for all test programs (SC-004)

**Performance Benchmarks**:
- Tokenize 10,000-line program in <50ms (target: 200k lines/sec)
- Memory: Token vec < 2x source string size
