# Developer Quickstart: Lexer/Parser Architecture

**Audience**: Contributors implementing or extending the assembler
**Time**: 10 minutes to understand architecture, 30 minutes to make first change
**Prerequisites**: Rust basics, familiarity with compiler concepts (lexer/parser separation)

## Architecture Overview

The assembler now has **three distinct phases**:

```
Phase 1: Lexical Analysis          Phase 2: Parsing             Phase 3: Code Generation
┌──────────────────┐               ┌────────────────┐           ┌──────────────────┐
│                  │               │                │           │                  │
│  Source Text     │──────────────▶│  TokenStream   │──────────▶│  AssemblyLine[]  │
│  (&str)          │   Lexer       │  (Vec<Token>)  │  Parser   │  (syntax tree)   │
│                  │               │                │           │                  │
└──────────────────┘               └────────────────┘           └──────────────────┘
                                                                          │
                                                                          │ Encoder
                                                                          ▼
                                                                ┌──────────────────┐
                                                                │  Machine Code    │
                                                                │  (Vec<u8>)       │
                                                                └──────────────────┘
```

### Key Principle: Separation of Concerns

| Layer | Responsibility | What It Knows | What It Doesn't Know |
|-------|----------------|---------------|----------------------|
| **Lexer** | Character → Token | String formats, number bases, operators | Assembly grammar, addressing modes, instruction validity |
| **Parser** | Token → Syntax Tree | Assembly grammar, operand patterns | Character encoding, number parsing, opcode encoding |
| **Encoder** | Syntax Tree → Bytes | Opcode table, addressing modes, binary format | Source text, tokens |

---

## Quick Reference: When to Modify Each Layer

### Modify the Lexer when...
✅ Adding new token types (e.g., string literals, new operators)
✅ Changing number formats (e.g., support octal with `0` prefix)
✅ Fixing tokenization bugs (e.g., "hex number $AG not rejected")

❌ **Don't modify lexer for**: New directives, new addressing modes, new instructions

### Modify the Parser when...
✅ Adding new directives (`.align`, `.fill`, `.macro`)
✅ Changing grammar rules (e.g., optional commas, case-sensitive labels)
✅ Fixing parsing bugs (e.g., "indirect indexed addressing broken")

❌ **Don't modify parser for**: Number format bugs, operator recognition, comment handling

### Modify the Encoder when...
✅ Adding instruction implementations
✅ Fixing opcode mapping bugs
✅ Changing binary output format

❌ **Don't modify encoder for**: Anything related to source syntax

---

## Example 1: Adding a New Directive (`.fill`)

**Requirement**: Support `.fill COUNT, VALUE` to emit COUNT copies of VALUE byte.

**Which layer?** Parser only (lexer already handles `.` and numbers)

**Steps**:

1. **Add to `AssemblerDirective` enum** (`src/assembler.rs`):
```rust
pub enum AssemblerDirective {
    Origin { address: u16 },
    Byte { values: Vec<DirectiveValue> },
    Word { values: Vec<DirectiveValue> },
    Fill { count: u16, value: u8 },  // NEW
}
```

2. **Add parser rule** (`src/assembler/parser.rs`):
```rust
fn parse_directive(stream: &mut TokenStream) -> Result<AssemblerDirective, ParserError> {
    match directive_name.as_str() {
        ".org" => parse_org_directive(stream),
        ".byte" => parse_byte_directive(stream),
        ".word" => parse_word_directive(stream),
        ".fill" => parse_fill_directive(stream),  // NEW
        _ => Err(ParserError::UnknownDirective { ... }),
    }
}

fn parse_fill_directive(stream: &mut TokenStream) -> Result<AssemblerDirective, ParserError> {
    stream.skip_whitespace();

    // Parse count
    let count = match stream.consume() {
        Some(Token { token_type: TokenType::DecimalNumber(n), .. }) => n,
        Some(Token { token_type: TokenType::HexNumber(n), .. }) => n,
        other => return Err(ParserError::ExpectedNumber { found: other }),
    };

    // Expect comma
    stream.expect(TokenType::Comma)?;
    stream.skip_whitespace();

    // Parse value
    let value = match stream.consume() {
        Some(Token { token_type: TokenType::DecimalNumber(n), .. }) if n <= 255 => n as u8,
        Some(Token { token_type: TokenType::HexNumber(n), .. }) if n <= 255 => n as u8,
        other => return Err(ParserError::InvalidByteValue { found: other }),
    };

    Ok(AssemblerDirective::Fill { count, value })
}
```

3. **Add code generation** (`src/assembler.rs`, pass 2):
```rust
// In encode phase
match directive {
    AssemblerDirective::Origin { .. } => { /* ... */ },
    AssemblerDirective::Byte { .. } => { /* ... */ },
    AssemblerDirective::Word { .. } => { /* ... */ },
    AssemblerDirective::Fill { count, value } => {
        bytes.extend(std::iter::repeat(*value).take(*count as usize));
        current_address = current_address.wrapping_add(*count);
    },
}
```

4. **Add test** (`tests/assembler_directives_test.rs`):
```rust
#[test]
fn test_fill_directive() {
    let source = ".fill 5, $AA";
    let output = assemble(source).unwrap();
    assert_eq!(output.bytes, vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA]);
}
```

**What we didn't touch**: Lexer (already handles `.`, numbers, comma)

---

## Example 2: Fixing a Lexer Bug (Invalid Hex Digits)

**Bug**: Lexer accepts `$AG` as valid hex number instead of erroring on 'G'.

**Which layer?** Lexer only

**Steps**:

1. **Write failing test** (`tests/lexer_tests.rs`):
```rust
#[test]
fn test_invalid_hex_digit_rejected() {
    let source = "$AG";
    let result = lexer::tokenize(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        LexerError::InvalidHexDigit { ch, line, column } => {
            assert_eq!(*ch, 'G');
            assert_eq!(*line, 1);
            assert_eq!(*column, 2);
        }
        _ => panic!("Expected InvalidHexDigit error"),
    }
}
```

2. **Fix lexer** (`src/assembler/lexer.rs`):
```rust
fn scan_hex_number(&mut self) -> Result<Token, LexerError> {
    self.advance(); // Skip '$'
    let start_col = self.column();
    let mut digits = String::new();

    while let Some((_, ch)) = self.current {
        if ch.is_ascii_hexdigit() {
            digits.push(ch);
            self.advance();
        } else if ch.is_ascii_alphabetic() {
            // INVALID hex digit (G-Z, g-z)
            return Err(LexerError::InvalidHexDigit {
                ch,
                line: self.line,
                column: self.column(),
            });
        } else {
            break;  // End of number
        }
    }

    // ... parse and return token
}
```

3. **Verify fix**: Run `cargo test` - failing test now passes, existing tests still green

**What we didn't touch**: Parser, encoder (bug was purely lexical)

---

## Example 3: Debugging a Parse Error

**Scenario**: User reports "LDA START,X fails with syntax error"

**Debugging Workflow**:

1. **Add debug test**:
```rust
#[test]
fn test_debug_indexed_label() {
    let source = r#"
START = $80
    LDA START,X
"#;
    let output = assemble(source);
    println!("{:#?}", output);  // Debug print
}
```

2. **Run test**: `cargo test test_debug_indexed_label -- --nocapture`

3. **Check token stream** (if lexer suspect):
```rust
let tokens = lexer::tokenize(source).unwrap();
for token in &tokens {
    println!("{:?}", token);
}
// Verify: Identifier("START"), Comma, Identifier("X") are produced
```

4. **Check parser logic** (if parser suspect):
```rust
// Add trace in parse_operand()
println!("Parsing operand, next tokens: {:#?}", [
    stream.peek(),
    stream.peek_n(1),
    stream.peek_n(2),
]);
```

5. **Identify layer**: If tokens are correct → parser bug. If tokens are wrong → lexer bug.

---

## Common Patterns

### Pattern: Peeking Ahead for Grammar Disambiguation

```rust
// Distinguish "($FF)" (indirect) from "($FF,X)" (indirect,X)
if stream.peek() == TokenType::LParen {
    stream.consume();  // Consume '('
    let addr = parse_number(stream)?;

    if stream.peek() == TokenType::Comma {
        // Indirect indexed: ($FF,X) or ($FF),Y
        stream.consume();  // Consume ','
        let index = stream.expect_identifier()?;
        // ...
    } else if stream.peek() == TokenType::RParen {
        // Simple indirect: ($FF)
        stream.consume();  // Consume ')'
        return Ok(Operand::Indirect(addr));
    }
}
```

### Pattern: Skipping Whitespace/Comments

```rust
// Parser ignores whitespace (lexer preserves it for formatters)
pub fn parse_line(stream: &mut TokenStream) -> Result<AssemblyLine, ParserError> {
    stream.skip_whitespace();  // Skip leading space

    let label = parse_label(stream)?;
    stream.skip_whitespace();  // Skip space between label and mnemonic

    let mnemonic = parse_mnemonic(stream)?;
    stream.skip_whitespace();  // Skip space between mnemonic and operand

    // ...
}
```

### Pattern: Error Recovery

```rust
// Collect all errors instead of failing on first one
fn parse_all_lines(stream: &mut TokenStream) -> Result<Vec<AssemblyLine>, Vec<ParserError>> {
    let mut lines = Vec::new();
    let mut errors = Vec::new();

    while !stream.is_eof() {
        match parse_line(stream) {
            Ok(Some(line)) => lines.push(line),
            Ok(None) => {},  // Blank line, skip
            Err(e) => {
                errors.push(e);
                stream.skip_until_newline();  // Recover: skip to next line
            }
        }
    }

    if errors.is_empty() {
        Ok(lines)
    } else {
        Err(errors)
    }
}
```

---

## Testing Workflow

### Running Tests

```bash
# Run all assembler tests
cargo test assembler

# Run only lexer tests
cargo test lexer

# Run specific test
cargo test test_indexed_addressing

# Run with output visible
cargo test test_debug -- --nocapture
```

### Test Organization

- `tests/lexer_tests.rs` - Lexer unit tests (token type coverage)
- `tests/assembler_tests.rs` - Full assembly integration tests
- `tests/assembler_directives_test.rs` - Directive-specific tests
- `tests/functional_assembler_disassembler.rs` - Round-trip tests

### TDD Workflow

1. Write failing test showing desired behavior
2. Run test, verify it fails for right reason
3. Implement minimal fix
4. Run test, verify it passes
5. Run full suite, verify no regressions
6. Refactor if needed

---

## Performance Tips

**Lexer Optimization**:
- Use `CharIndices` iterator (zero-copy)
- Avoid string allocations (use slices where possible)
- Pre-allocate token vector with capacity hint

**Parser Optimization**:
- Avoid cloning tokens (use references)
- Skip whitespace in bulk (don't check every token)
- Use `Vec::with_capacity()` for known-size collections

**Profiling**:
```bash
# Profile assembly of large program
cargo build --release
hyperfine --warmup 3 'target/release/assembler large_program.asm'
```

---

## FAQs

**Q: Why not use a lexer generator like `logos`?**
A: Constitution II (WebAssembly Portability) and IV (Clarity & Hackability) - zero external dependencies, hand-written code is easier to understand and debug.

**Q: Can I add UTF-8 identifier support?**
A: Not recommended. 6502 assembly convention is ASCII-only. UTF-8 in comments is already preserved.

**Q: Should I add error recovery to the lexer?**
A: Current design already does this - lexer collects ALL errors instead of stopping at first one.

**Q: How do I handle backward compatibility?**
A: Public API (`assemble()`) is unchanged. All existing tests must pass with bit-identical output (SC-004).

---

## Next Steps

1. **Read the contracts**: [lexer-api.md](contracts/lexer-api.md), [parser-api.md](contracts/parser-api.md)
2. **Read the data model**: [data-model.md](data-model.md)
3. **Review existing parser**: `src/assembler/parser.rs` to see what needs refactoring
4. **Pick a task**: See `tasks.md` (generated by `/speckit.tasks`)
5. **Start coding**: Follow TDD workflow, one small change at a time

---

## Getting Help

- **Architecture questions**: See [research.md](research.md) for design decisions
- **Bugs**: Check layer (lexer vs parser vs encoder) using debug workflow above
- **Tests failing**: Run `git diff` to see what changed, review test expectations
- **Performance**: Profile before optimizing, measure actual impact

Happy hacking! 🎉
