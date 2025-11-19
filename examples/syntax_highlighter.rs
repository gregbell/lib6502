//! Syntax highlighter using the assembler lexer
//!
//! This example demonstrates how external tools can use the assembler's
//! tokenization layer for purposes other than assembly. Here, we use it
//! to create a simple syntax highlighter that colors assembly code for
//! terminal display.
//!
//! Run with: `cargo run --example syntax_highlighter`

use lib6502::assembler::lexer::{tokenize, TokenType};

/// ANSI color codes for terminal output
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const MNEMONIC: &str = "\x1b[1;36m"; // Bright cyan (bold)
    pub const NUMBER: &str = "\x1b[1;33m"; // Bright yellow
    pub const LABEL: &str = "\x1b[1;35m"; // Bright magenta
    pub const COMMENT: &str = "\x1b[2;37m"; // Dim white
    pub const OPERATOR: &str = "\x1b[1;32m"; // Bright green
    pub const DIRECTIVE: &str = "\x1b[1;34m"; // Bright blue
}

/// Highlight a single line of assembly source
fn highlight_line(source: &str) -> String {
    let tokens = match tokenize(source) {
        Ok(tokens) => tokens,
        Err(_errors) => {
            // On lexer error, return source with error color
            return format!("\x1b[1;31m{}\x1b[0m", source); // Bright red
        }
    };

    let mut output = String::new();
    let mut prev_was_dot = false;

    for token in &tokens {
        let colored = match &token.token_type {
            // Identifiers can be mnemonics or labels/symbols
            TokenType::Identifier(id) => {
                if prev_was_dot {
                    // After a dot, this is a directive name
                    format!("{}{}{}", colors::DIRECTIVE, id, colors::RESET)
                } else {
                    // Treat as mnemonic (uppercase 3-letter = instruction)
                    if id.len() == 3 && id.chars().all(|c| c.is_ascii_uppercase()) {
                        format!("{}{}{}", colors::MNEMONIC, id, colors::RESET)
                    } else {
                        // Likely a label/symbol reference
                        format!("{}{}{}", colors::LABEL, id, colors::RESET)
                    }
                }
            }

            // Numbers in all formats
            TokenType::HexNumber(val) => {
                format!("{}${:X}{}", colors::NUMBER, val, colors::RESET)
            }
            TokenType::BinaryNumber(val) => {
                format!("{}%{:b}{}", colors::NUMBER, val, colors::RESET)
            }
            TokenType::DecimalNumber(val) => {
                format!("{}{}{}", colors::NUMBER, val, colors::RESET)
            }

            // Operators and punctuation
            TokenType::Colon => {
                format!("{}:{}", colors::LABEL, colors::RESET)
            }
            TokenType::Dot => {
                prev_was_dot = true;
                format!("{}.{}", colors::DIRECTIVE, colors::RESET)
            }
            TokenType::Hash
            | TokenType::Comma
            | TokenType::Dollar
            | TokenType::Percent
            | TokenType::Equal
            | TokenType::LParen
            | TokenType::RParen => {
                let ch = match &token.token_type {
                    TokenType::Hash => '#',
                    TokenType::Comma => ',',
                    TokenType::Dollar => '$',
                    TokenType::Percent => '%',
                    TokenType::Equal => '=',
                    TokenType::LParen => '(',
                    TokenType::RParen => ')',
                    _ => unreachable!(),
                };
                format!("{}{}{}", colors::OPERATOR, ch, colors::RESET)
            }

            // Comments are dimmed
            TokenType::Comment(text) => {
                format!("{};{}{}", colors::COMMENT, text, colors::RESET)
            }

            // Structural tokens preserved as-is
            TokenType::Whitespace => " ".to_string(),
            TokenType::Newline => "\n".to_string(),
            TokenType::Eof => String::new(),
        };

        // Reset the dot flag for non-dot tokens
        if !matches!(token.token_type, TokenType::Dot) {
            prev_was_dot = false;
        }

        output.push_str(&colored);
    }

    output
}

fn main() {
    println!("6502 Assembly Syntax Highlighter");
    println!("=================================\n");

    // Example assembly program
    let source = r#"
; Simple 6502 program demonstrating syntax highlighting
    .org $8000

SCREEN = $4000          ; Screen memory address
CHAR_A = 65             ; ASCII 'A'

START:
    LDA #CHAR_A         ; Load character
    STA SCREEN          ; Store to screen
    LDX #$FF            ; Initialize stack
    TXS
    JMP LOOP

LOOP:
    LDA $1234,X         ; Load with indexed addressing
    STA ($20),Y         ; Store with indirect indexed
    BEQ START           ; Branch if equal
    JMP LOOP            ; Infinite loop

    .byte $00, $01, $02 ; Data bytes
    .word $1234, $5678  ; Data words
"#;

    println!("Original source:\n{}", source);
    println!("\n\nHighlighted output:\n");

    for line in source.lines() {
        println!("{}", highlight_line(line));
    }

    println!("\n\nColor legend:");
    println!(
        "  {}Mnemonics{} - Cyan (instructions)",
        colors::MNEMONIC,
        colors::RESET
    );
    println!(
        "  {}Numbers{} - Yellow (all formats)",
        colors::NUMBER,
        colors::RESET
    );
    println!(
        "  {}Labels{} - Magenta (symbols)",
        colors::LABEL,
        colors::RESET
    );
    println!(
        "  {}Operators{} - Green (#, :, etc.)",
        colors::OPERATOR,
        colors::RESET
    );
    println!(
        "  {}Directives{} - Blue (.org, .byte)",
        colors::DIRECTIVE,
        colors::RESET
    );
    println!(
        "  {}Comments{} - Gray (dimmed)",
        colors::COMMENT,
        colors::RESET
    );
    println!("\nThis example demonstrates Success Criterion SC-005:");
    println!("External tools can use tokenize() for syntax analysis!");
}
