# Research: Lexer Design Patterns and Token Architecture

**Phase**: Phase 0 - Research
**Date**: 2025-11-18
**Purpose**: Research lexer implementation patterns, token stream architecture, and error handling strategies for the assembler refactoring

## Research Questions

1. What token types are needed for 6502 assembly language?
2. How should token streams be structured for efficient parsing?
3. What error reporting patterns work best for lexical analysis?
4. How do we maintain source location information through the pipeline?
5. What are best practices for lexer state machines in Rust?

## Decision 1: Token Type Enumeration

### Decision
Use an explicit `TokenType` enum with variants covering all syntactic elements of 6502 assembly:

```rust
pub enum TokenType {
    // Identifiers and keywords
    Identifier(String),          // Labels, mnemonics, symbol names

    // Numbers
    DecimalNumber(u16),           // 0-65535
    HexNumber(u16),              // $0000-$FFFF
    BinaryNumber(u16),           // %00000000-%1111111111111111

    // Operators and punctuation
    Colon,                       // : (label definition)
    Comma,                       // , (operand separator, indexed addressing)
    Hash,                        // # (immediate mode prefix)
    Dollar,                      // $ (hex prefix)
    Percent,                     // % (binary prefix)
    Equal,                       // = (constant assignment)
    LParen,                      // ( (indirect addressing)
    RParen,                      // ) (indirect addressing)
    Dot,                         // . (directive prefix)

    // Whitespace and structure
    Whitespace,                  // Spaces, tabs (preserved for formatting tools)
    Newline,                     // Line terminator
    Comment(String),             // ; comment text

    // Special
    Eof,                         // End of file
}
```

### Rationale
- **Explicit over implicit**: Each token carries its parsed value (numbers already converted from string representation)
- **Whitespace preservation**: Optional whitespace tokens enable formatters/prettifiers to preserve user formatting
- **Parsed numbers**: Lexer handles hex ($42) / binary (%01000010) / decimal (66) conversion, parser doesn't need to know about number formats
- **Comment preservation**: Keeping comment text enables documentation extraction tools

### Alternatives Considered
- **String-only tokens**: Store raw text in all tokens, parse later → rejected because it pushes parsing responsibility to later stages (defeats purpose of lexer)
- **Whitespace-ignoring lexer**: Skip whitespace entirely → rejected because external tools (formatters) need it
- **Separate token for each operator**: Individual `HashToken`, `DollarToken`, etc. → rejected as unnecessarily verbose (enum variants are clear enough)

## Decision 2: Token Structure with Source Location

### Decision
Each token carries its source location for precise error reporting:

```rust
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,           // 1-indexed line number
    pub column: usize,         // 0-indexed column offset
    pub length: usize,         // Character span (for highlighting)
}
```

### Rationale
- **Error precision**: Parser can report "Line 42, Column 15: expected operand" instead of "Line 42: syntax error"
- **IDE integration**: Source locations enable jump-to-error in editors
- **Minimal overhead**: 3 extra usizes per token (~24 bytes on 64-bit) is negligible for assembly source
- **Consistent with existing source_map**: Matches existing source map infrastructure

### Alternatives Considered
- **Span-based locations**: Store byte offsets (start, end) → rejected because line/column is more human-readable and matches existing error reporting
- **Separate location table**: Store locations separately, index into it → rejected due to complexity and pointer chasing overhead
- **No location tracking**: Rely on existing source_map → rejected because lexer errors happen before source_map is populated

## Decision 3: TokenStream Iterator Pattern

### Decision
Implement `TokenStream` as an iterator with lookahead capability:

```rust
pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    pub fn peek(&self) -> Option<&Token> { /* Look ahead without consuming */ }
    pub fn peek_n(&self, n: usize) -> Option<&Token> { /* Look ahead N tokens */ }
    pub fn consume(&mut self) -> Option<Token> { /* Advance and return current token */ }
    pub fn expect(&mut self, expected: TokenType) -> Result<Token, ParseError> { /* Consume or error */ }
    pub fn position(&self) -> usize { /* Current offset for backtracking if needed */ }
}
```

### Rationale
- **Lookahead support**: Parser needs to peek ahead to distinguish addressing modes (e.g., `($FF)` vs `($FF),Y`)
- **Error recovery**: `expect()` method generates helpful errors ("expected ',', found newline")
- **Standard iterator pattern**: Familiar to Rust developers
- **Simple implementation**: Vec-backed stream is trivial to implement and test

### Alternatives Considered
- **Iterator trait only**: Use Rust's `Iterator` trait → rejected because it doesn't support efficient multi-token lookahead
- **Lazy lexing**: Tokenize on demand as parser consumes → rejected due to complexity (error handling becomes stateful)
- **Peekable iterator**: Wrap standard iterator with `.peekable()` → rejected because only supports 1-token lookahead (need 2+ for indirect addressing)

## Decision 4: Lexer State Machine Architecture

### Decision
Single-pass character-by-character lexer with explicit state:

```rust
struct Lexer<'a> {
    source: &'a str,
    chars: std::str::CharIndices<'a>,
    current: Option<(usize, char)>,
    line: usize,
    line_start: usize,  // Byte offset of current line start
}

impl<'a> Lexer<'a> {
    fn next_token(&mut self) -> Result<Token, LexerError> {
        match self.current {
            Some((_, ';')) => self.scan_comment(),
            Some((_, '$')) => self.scan_hex_number(),
            Some((_, '%')) => self.scan_binary_number(),
            Some((_, ch)) if ch.is_ascii_digit() => self.scan_decimal_number(),
            Some((_, ch)) if ch.is_ascii_alphabetic() => self.scan_identifier(),
            Some((_, ':')) => self.scan_single(TokenType::Colon),
            // ... other cases
        }
    }
}
```

### Rationale
- **Simple and debuggable**: Character-level matching is easy to trace and test
- **Zero allocations**: CharIndices iterator is zero-copy over source string
- **Line tracking**: Explicit line/column tracking enables precise error locations
- **Rust idioms**: Uses standard library iterators and match expressions

### Alternatives Considered
- **Regex-based lexer**: Use regex crate to match tokens → rejected due to external dependency constraint (Constitution II)
- **Macro-based lexer generator**: Use `logos` or similar → rejected due to external dependency and "magic" that violates Clarity principle
- **Hand-rolled DFA**: Explicit state transition table → rejected as over-engineered for assembly language (simple grammar doesn't need it)

## Decision 5: Error Handling - Lexical vs Syntactic Separation

### Decision
Introduce `LexerError` separate from existing `AssemblerError`:

```rust
pub enum LexerError {
    InvalidHexDigit { ch: char, line: usize, column: usize },
    InvalidBinaryDigit { ch: char, line: usize, column: usize },
    NumberTooLarge { value: String, max: u16, line: usize, column: usize },
    UnexpectedCharacter { ch: char, line: usize, column: usize },
    UnterminatedString { line: usize, column: usize },  // Future-proofing
}

// Existing AssemblerError gets new variant:
pub enum ErrorType {
    // ... existing variants ...
    LexicalError(LexerError),  // Wrap lexer errors
}
```

### Rationale
- **Clear error source**: Users immediately know if problem is malformed token vs incorrect grammar
- **Spec requirement FR-007**: "System MUST detect and report lexical errors separately from syntax errors"
- **Better error messages**: "Invalid hex digit 'G' in $12G0" vs generic "syntax error on line 42"
- **Backward compatibility**: Wrapping LexerError in AssemblerError maintains existing error API

### Alternatives Considered
- **Reuse existing ErrorType**: Add `InvalidToken` variant → rejected because it doesn't distinguish token-type errors from parsing errors
- **String-only errors**: Just return error messages → rejected because structured errors enable better IDE integration
- **Panic on lexer errors**: Fail-fast → rejected violates error recovery (want to collect all errors in one pass if possible)

## Decision 6: Number Parsing and Validation

### Decision
Lexer performs full number parsing and range validation:

```rust
fn scan_hex_number(&mut self) -> Result<Token, LexerError> {
    self.advance(); // Skip '$'
    let start_col = self.column();
    let mut value_str = String::new();

    while let Some((_, ch)) = self.current {
        if ch.is_ascii_hexdigit() {
            value_str.push(ch);
            self.advance();
        } else {
            break;
        }
    }

    let value = u16::from_str_radix(&value_str, 16)
        .map_err(|_| LexerError::NumberTooLarge { ... })?;

    Ok(Token {
        token_type: TokenType::HexNumber(value),
        line: self.line,
        column: start_col,
        length: value_str.len() + 1,  // +1 for '$' prefix
    })
}
```

### Rationale
- **Single responsibility**: Lexer handles character-level concerns, parser handles semantic concerns
- **Early error detection**: Invalid numbers caught immediately, not during semantic analysis
- **Simplified parser**: Parser receives parsed u16 values, doesn't need parsing logic
- **Consistent with spec SC-003**: "Parser module size decreases by 30% by eliminating inline parsing logic"

### Alternatives Considered
- **Defer to parser**: Return string tokens, parse in parser → rejected because it violates lexer/parser separation
- **String validation only**: Check format but don't parse → rejected because parser would duplicate parsing logic
- **Lazy parsing**: Parse on first use → rejected due to complexity and error handling issues

## Best Practices Summary

Based on research into Rust lexer patterns (rustc lexer, syn crate, logos patterns):

1. **Keep it simple**: Character-by-character matching with explicit state
2. **Fail fast with good errors**: Detect invalid tokens immediately with precise locations
3. **Zero-copy where possible**: Use string slices, CharIndices iterator
4. **Test thoroughly**: Each token type needs comprehensive unit tests
5. **Document assumptions**: Clarify ASCII-only tokenization (UTF-8 preserved in comments)

## Open Questions (Resolved)

~~Q: Should we handle Unicode identifiers?~~
**A**: No. Labels/mnemonics are ASCII-only per 6502 conventions. UTF-8 in comments is preserved as-is but not tokenized.

~~Q: Should lexer handle macro expansion?~~
**A**: No. Out of scope per spec. Macros would be a preprocessing layer if added in future.

~~Q: Should we support alternative comment syntax (# or //)?~~
**A**: No. Semicolon-only per 6502 standard. Alternative syntax would break compatibility with existing code.

## Implementation Notes

- **Testing strategy**: TDD approach with one test per token type, edge cases for number boundaries, error conditions
- **Migration path**: Introduce lexer alongside existing parser, switch over incrementally, maintain all tests green
- **Performance**: Profile lexer separately to verify <5% overhead target (expect lexer to be fastest component)
- **Documentation**: Add module-level docs with examples showing token stream for common assembly snippets

## References

- Rust rustc lexer: https://github.com/rust-lang/rust/tree/master/compiler/rustc_lexer
- Crafting Interpreters (Lexer chapter): https://craftinginterpreters.com/scanning.html
- 6502 assembly syntax standards: https://www.masswerk.at/6502/assembler.html
