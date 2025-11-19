//! Assembly source parser (syntactic analysis phase)
//!
//! This module provides the second phase of assembly: converting token streams into
//! structured assembly lines. The parser works with pre-tokenized input from the
//! [`lexer`](super::lexer) module, using pattern matching on token types instead of
//! string manipulation.
//!
//! # Architecture
//!
//! The parser follows a token-based design that separates lexical concerns from
//! syntactic analysis:
//!
//! ```text
//! Source Text → Lexer → Token Stream → Parser → AssemblyLine → Assembler → Machine Code
//!                ↓                        ↓                         ↓
//!           TokenType::*           AssemblyLine            Binary output
//!           (characters)           (structure)              (bytes)
//! ```
//!
//! ## Two Parsing Paths
//!
//! 1. **Token-based** ([`parse_token_line`]): Modern path using lexer tokens (recommended)
//! 2. **String-based** ([`parse_line`]): Legacy path for backwards compatibility
//!
//! The assembler uses `parse_token_line()` via token grouping for production code.
//!
//! # Parser Responsibilities
//!
//! **What the parser does:**
//! - Recognize syntactic patterns (label:, mnemonic operand, .directive args)
//! - Distinguish label definitions from constant assignments (NAME: vs NAME =)
//! - Parse directives (.org, .byte, .word) and validate arguments
//! - Build [`AssemblyLine`] structure for each line
//! - Preserve comments and source locations for debugging
//!
//! **What the parser does NOT do:**
//! - Character-level tokenization (lexer's job)
//! - Number parsing (lexer already parsed `$42` → `HexNumber(0x42)`)
//! - Label resolution or address calculation (assembler's job)
//! - Machine code generation (encoder's job)
//!
//! # Token Consumption Model
//!
//! The parser uses a simple left-to-right scan with token pattern matching:
//!
//! ```text
//! Input tokens:  [Identifier("START"), Colon, Whitespace, Identifier("LDA"), ...]
//!                  ↓
//! Pattern match:  Identifier + Colon  → Found a label!
//!                                       ↓
//! Skip whitespace:                     [Identifier("LDA"), ...]
//!                                       ↓
//! Pattern match:  Identifier (no Colon) → Found a mnemonic!
//! ```
//!
//! ## Edge Cases Handled
//!
//! - **Number before identifier** (e.g., `1START:`): Treated as invalid label
//! - **Invalid directives** (e.g., `.unknown`): Treated as mnemonic for validation
//! - **Whitespace flexibility**: Parser skips whitespace between tokens
//! - **Comment preservation**: Comments are extracted and stored, not discarded
//!
//! # Examples
//!
//! ## Basic Parsing
//!
//! ```
//! use lib6502::assembler::parser::parse_line;
//!
//! let line = parse_line("START: LDA #$42", 1).unwrap();
//!
//! assert_eq!(line.label, Some("START".to_string()));
//! assert_eq!(line.mnemonic, Some("LDA".to_string()));
//! assert_eq!(line.operand, Some("#$42".to_string()));
//! assert_eq!(line.line_number, 1);
//! ```
//!
//! ## Directive Parsing
//!
//! ```
//! use lib6502::assembler::parser::parse_directive;
//! use lib6502::assembler::AssemblerDirective;
//!
//! let directive = parse_directive(".org $8000").unwrap();
//!
//! match directive {
//!     AssemblerDirective::Org { address } => assert_eq!(address, 0x8000),
//!     _ => panic!("Expected .org directive"),
//! }
//! ```
//!
//! ## Token-Based Parsing (Internal)
//!
//! The assembler uses token-based parsing internally:
//!
//! ```
//! use lib6502::assembler::lexer::tokenize;
//! use lib6502::assembler::parser::parse_token_line;
//!
//! // Tokenize first
//! let tokens = tokenize("LDA #$42").unwrap();
//!
//! // Group tokens by line (simplified example - real code handles newlines)
//! let line_tokens: Vec<_> = tokens.iter()
//!     .filter(|t| !matches!(t.token_type, lib6502::assembler::lexer::TokenType::Eof))
//!     .cloned()
//!     .collect();
//!
//! // Parse tokens
//! let line = parse_token_line(&line_tokens, 1).unwrap();
//! assert_eq!(line.mnemonic, Some("LDA".to_string()));
//! ```
//!
//! # Migration Notes
//!
//! The parser has been refactored from string-based to token-based parsing:
//!
//! **Old approach** (string manipulation):
//! ```text
//! line.find(':')            → Look for label
//! line.strip_prefix('.')    → Look for directive
//! line.splitn(2, ' ')       → Split mnemonic/operand
//! parse_number("$42")       → Parse hex numbers
//! ```
//!
//! **New approach** (token matching):
//! ```text
//! match (token[0], token[1])
//!   (Identifier, Colon)     → Found label
//!   (Dot, Identifier)       → Found directive
//! match token.token_type
//!   HexNumber(0x42)         → Already parsed!
//! ```
//!
//! Benefits: Simpler code, better errors, no repeated parsing.

use crate::addressing::AddressingMode;
use super::lexer::{Token, TokenType};

/// A parsed line of assembly source
#[derive(Debug, Clone, PartialEq)]
pub struct AssemblyLine {
    /// Line number in source file (1-indexed)
    pub line_number: usize,

    /// Optional constant assignment (e.g., ("MAX", "255") from "MAX = 255")
    pub constant: Option<(String, String)>,

    /// Optional label definition (e.g., "START" from "START:")
    pub label: Option<String>,

    /// Optional mnemonic (e.g., "LDA")
    pub mnemonic: Option<String>,

    /// Optional operand text (e.g., "#$42", "$1234,X")
    pub operand: Option<String>,

    /// Optional directive (e.g., .org, .byte, .word)
    pub directive: Option<crate::assembler::AssemblerDirective>,

    /// Optional comment text (after semicolon)
    pub comment: Option<String>,

    /// Character span in source (start, end) for error reporting
    pub span: (usize, usize),
}

/// Parse a line from a sequence of tokens (new token-based parser)
///
/// Takes a slice of tokens representing one logical line (tokens between newlines).
/// This is the new lexer-integrated parser that replaces string manipulation
/// with token pattern matching.
///
/// # Arguments
///
/// * `tokens` - Slice of tokens for this line (excluding the terminating newline)
/// * `line_number` - The 1-indexed line number
///
/// # Returns
///
/// Some(AssemblyLine) if the line contains code, None for empty/comment-only lines
pub fn parse_token_line(tokens: &[Token], line_number: usize) -> Option<AssemblyLine> {
    // Skip leading whitespace
    let mut pos = 0;
    while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
        pos += 1;
    }

    // Empty line or only whitespace
    if pos >= tokens.len() || matches!(tokens[pos].token_type, TokenType::Eof) {
        return None;
    }

    // Calculate span from first to last token
    let span_start = if tokens.is_empty() { 0 } else { tokens[0].column };
    let span_end = if tokens.is_empty() {
        0
    } else {
        let last = &tokens[tokens.len() - 1];
        last.column + last.length
    };

    // Comment-only line
    if matches!(tokens[pos].token_type, TokenType::Comment(_)) {
        if let TokenType::Comment(text) = &tokens[pos].token_type {
            return Some(AssemblyLine {
                line_number,
                constant: None,
                label: None,
                mnemonic: None,
                operand: None,
                directive: None,
                comment: Some(text.clone()),
                span: (span_start, span_end),
            });
        }
    }

    let mut label = None;
    let mut constant = None;
    let mut mnemonic = None;
    let mut operand_tokens = Vec::new();
    let mut directive = None;
    let mut comment = None;

    // Look for label (Identifier followed by Colon)
    // Also handle invalid labels that start with digits (Number + Identifier + Colon)
    if pos + 1 < tokens.len() {
        // Check for Number + Identifier + Colon (invalid label starting with digit)
        if pos + 2 < tokens.len() {
            let is_number_label = matches!(
                (&tokens[pos].token_type, &tokens[pos + 1].token_type, &tokens[pos + 2].token_type),
                (TokenType::DecimalNumber(_) | TokenType::HexNumber(_) | TokenType::BinaryNumber(_),
                 TokenType::Identifier(_),
                 TokenType::Colon)
            );

            if is_number_label {
                // Construct the invalid label name
                let num_str = match &tokens[pos].token_type {
                    TokenType::DecimalNumber(val) => val.to_string(),
                    TokenType::HexNumber(val) => format!("${:X}", val),
                    TokenType::BinaryNumber(val) => format!("%{:b}", val),
                    _ => unreachable!(),
                };
                if let TokenType::Identifier(id) = &tokens[pos + 1].token_type {
                    label = Some(format!("{}{}", num_str, id));
                    pos += 3; // Skip number, identifier, and colon

                    // Skip whitespace after label
                    while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
                        pos += 1;
                    }
                }
            }
        }

        // Normal label case (Identifier + Colon)
        if label.is_none() && matches!(tokens[pos].token_type, TokenType::Identifier(_)) {
            if let TokenType::Identifier(name) = &tokens[pos].token_type {
                if matches!(tokens[pos + 1].token_type, TokenType::Colon) {
                    label = Some(name.clone());
                    pos += 2; // Skip identifier and colon

                    // Skip whitespace after label
                    while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
                        pos += 1;
                    }
                }
            }
        }
    }

    // Check if line ends after label
    if pos >= tokens.len() {
        if label.is_some() {
            return Some(AssemblyLine {
                line_number,
                constant: None,
                label,
                mnemonic: None,
                operand: None,
                directive: None,
                comment: None,
                span: (span_start, span_end),
            });
        }
        return None;
    }

    // Look for constant assignment (Identifier followed by Equal)
    if label.is_none() && pos + 1 < tokens.len() {
        if let TokenType::Identifier(name) = &tokens[pos].token_type {
            // Skip whitespace before potential equals
            let mut check_pos = pos + 1;
            while check_pos < tokens.len() && matches!(tokens[check_pos].token_type, TokenType::Whitespace) {
                check_pos += 1;
            }

            if check_pos < tokens.len() && matches!(tokens[check_pos].token_type, TokenType::Equal) {
                // This is a constant assignment
                let const_name = name.clone();
                pos = check_pos + 1; // Skip past equals

                // Skip whitespace after equals
                while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
                    pos += 1;
                }

                // Collect value tokens until comment or EOF
                let mut value_str = String::new();
                while pos < tokens.len() {
                    match &tokens[pos].token_type {
                        TokenType::Comment(text) => {
                            comment = Some(text.clone());
                            break;
                        }
                        TokenType::Eof => break,
                        TokenType::Whitespace => {
                            if !value_str.is_empty() {
                                value_str.push(' ');
                            }
                        }
                        TokenType::HexNumber(val) => value_str.push_str(&format!("${:X}", val)),
                        TokenType::BinaryNumber(val) => value_str.push_str(&format!("%{:b}", val)),
                        TokenType::DecimalNumber(val) => value_str.push_str(&val.to_string()),
                        TokenType::Identifier(id) => value_str.push_str(id),
                        _ => {}, // Ignore other tokens in constant value
                    }
                    pos += 1;
                }

                constant = Some((const_name, value_str.trim().to_string()));

                return Some(AssemblyLine {
                    line_number,
                    constant,
                    label,
                    mnemonic: None,
                    operand: None,
                    directive: None,
                    comment,
                    span: (span_start, span_end),
                });
            }
        }
    }

    // Check for directive (starts with Dot)
    if pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Dot) {
        pos += 1; // Skip dot

        // Get directive name
        if pos < tokens.len() {
            if let TokenType::Identifier(directive_name) = &tokens[pos].token_type {
                pos += 1; // Skip directive name

                // Skip whitespace
                while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
                    pos += 1;
                }

                // Collect argument tokens until comment or EOF
                let mut args = String::new();
                while pos < tokens.len() {
                    match &tokens[pos].token_type {
                        TokenType::Comment(text) => {
                            comment = Some(text.clone());
                            break;
                        }
                        TokenType::Eof => break,
                        TokenType::HexNumber(val) => {
                            if !args.is_empty() { args.push(' '); }
                            args.push_str(&format!("${:X}", val));
                        }
                        TokenType::DecimalNumber(val) => {
                            if !args.is_empty() { args.push(' '); }
                            args.push_str(&val.to_string());
                        }
                        TokenType::Identifier(id) => {
                            if !args.is_empty() { args.push(' '); }
                            args.push_str(id);
                        }
                        TokenType::Comma => args.push(','),
                        TokenType::Whitespace => {
                            if !args.is_empty() && !args.ends_with(',') {
                                args.push(' ');
                            }
                        }
                        _ => {}
                    }
                    pos += 1;
                }

                // Parse the directive
                let directive_str = format!(".{} {}", directive_name, args).trim().to_string();

                match parse_directive(&directive_str) {
                    Ok(dir) => {
                        directive = Some(dir);
                        return Some(AssemblyLine {
                            line_number,
                            constant: None,
                            label,
                            mnemonic: None,
                            operand: None,
                            directive,
                            comment,
                            span: (span_start, span_end),
                        });
                    }
                    Err(_) => {
                        // Invalid directive - treat as mnemonic so validator can catch it
                        mnemonic = Some(format!(".{}", directive_name));
                        let operand_str = if args.is_empty() {
                            None
                        } else {
                            Some(args.trim().to_string())
                        };

                        return Some(AssemblyLine {
                            line_number,
                            constant: None,
                            label,
                            mnemonic,
                            operand: operand_str,
                            directive: None,
                            comment,
                            span: (span_start, span_end),
                        });
                    }
                }
            }
        }
    }

    // Look for mnemonic (Identifier not followed by colon or equals)
    if pos < tokens.len() {
        if let TokenType::Identifier(name) = &tokens[pos].token_type {
            mnemonic = Some(name.clone());
            pos += 1;

            // Skip whitespace after mnemonic
            while pos < tokens.len() && matches!(tokens[pos].token_type, TokenType::Whitespace) {
                pos += 1;
            }

            // Collect operand tokens until comment or EOF
            while pos < tokens.len() {
                match &tokens[pos].token_type {
                    TokenType::Comment(text) => {
                        comment = Some(text.clone());
                        break;
                    }
                    TokenType::Eof => break,
                    _ => {
                        operand_tokens.push(&tokens[pos]);
                        pos += 1;
                    }
                }
            }
        }
    }

    // Convert operand tokens to string (temporary - until we refactor operand parsing)
    let operand = if operand_tokens.is_empty() {
        None
    } else {
        let mut operand_str = String::new();
        for token in operand_tokens {
            match &token.token_type {
                TokenType::Hash => operand_str.push('#'),
                TokenType::Dollar => operand_str.push('$'),
                TokenType::Percent => operand_str.push('%'),
                TokenType::HexNumber(val) => operand_str.push_str(&format!("${:X}", val)),
                TokenType::BinaryNumber(val) => operand_str.push_str(&format!("%{:b}", val)),
                TokenType::DecimalNumber(val) => operand_str.push_str(&val.to_string()),
                TokenType::Identifier(id) => operand_str.push_str(id),
                TokenType::Comma => operand_str.push(','),
                TokenType::LParen => operand_str.push('('),
                TokenType::RParen => operand_str.push(')'),
                TokenType::Whitespace => operand_str.push(' '),
                _ => {}
            }
        }
        Some(operand_str.trim().to_string())
    };

    Some(AssemblyLine {
        line_number,
        constant,
        label,
        mnemonic,
        operand,
        directive,
        comment,
        span: (span_start, span_end),
    })
}

/// Parse a number from a string (supports hex $XX, decimal, binary %XXXXXXXX)
pub fn parse_number(s: &str) -> Result<u16, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("empty number string".to_string());
    }

    if let Some(stripped) = s.strip_prefix('$') {
        // Hexadecimal
        u16::from_str_radix(stripped, 16).map_err(|e| format!("invalid hex number: {}", e))
    } else if let Some(stripped) = s.strip_prefix('%') {
        // Binary
        u16::from_str_radix(stripped, 2).map_err(|e| format!("invalid binary number: {}", e))
    } else {
        // Decimal
        s.parse::<u16>()
            .map_err(|e| format!("invalid decimal number: {}", e))
    }
}

/// Parse a single line of assembly source
///
/// # Arguments
///
/// * `line` - The source line to parse
/// * `line_number` - The 1-indexed line number
///
/// # Returns
///
/// Some(AssemblyLine) if the line contains code, None for empty/comment-only lines
pub fn parse_line(line: &str, line_number: usize) -> Option<AssemblyLine> {
    let trimmed = line.trim();

    // Empty line
    if trimmed.is_empty() {
        return None;
    }

    // Comment-only line
    if let Some(stripped) = trimmed.strip_prefix(';') {
        return Some(AssemblyLine {
            line_number,
            constant: None,
            label: None,
            mnemonic: None,
            operand: None,
            directive: None,
            comment: Some(stripped.trim().to_string()),
            span: (0, line.len()),
        });
    }

    // Strip inline comment
    let (code_part, comment_part) = if let Some(comment_pos) = trimmed.find(';') {
        let code = &trimmed[..comment_pos];
        let comment = &trimmed[comment_pos + 1..];
        (code.trim(), Some(comment.trim().to_string()))
    } else {
        (trimmed, None)
    };

    if code_part.is_empty() {
        return Some(AssemblyLine {
            line_number,
            constant: None,
            label: None,
            mnemonic: None,
            operand: None,
            directive: None,
            comment: comment_part,
            span: (0, line.len()),
        });
    }

    // Check for constant assignment (NAME = VALUE) - must be checked before label
    if let Some(eq_pos) = code_part.find('=') {
        let name_part = code_part[..eq_pos].trim();
        let value_part = code_part[eq_pos + 1..].trim();

        // Validate name: not empty and no internal whitespace
        if !name_part.is_empty() && !name_part.contains(char::is_whitespace) {
            return Some(AssemblyLine {
                line_number,
                constant: Some((name_part.to_uppercase(), value_part.to_string())),
                label: None,
                mnemonic: None,
                operand: None,
                directive: None,
                comment: comment_part,
                span: (0, line.len()),
            });
        }
    }

    // Check for label (ends with colon)
    let (label, rest) = if let Some(colon_pos) = code_part.find(':') {
        let label_name = code_part[..colon_pos].trim().to_uppercase();
        let rest = code_part[colon_pos + 1..].trim();
        (Some(label_name), rest)
    } else {
        (None, code_part)
    };

    // Parse mnemonic and operand, or directive
    let (mnemonic, operand, directive) = if !rest.is_empty() {
        // Check if this is a directive (starts with .)
        if rest.starts_with('.') {
            // Parse directive
            match parse_directive(rest) {
                Ok(dir) => (None, None, Some(dir)),
                Err(_) => {
                    // Invalid directive - will be caught as error later
                    let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
                    let directive_name = parts[0].trim().to_uppercase();
                    let operand = if parts.len() > 1 {
                        Some(parts[1].trim().to_string())
                    } else {
                        None
                    };
                    (Some(directive_name), operand, None)
                }
            }
        } else {
            // Parse as mnemonic + operand
            let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
            let mnemonic = parts[0].trim().to_uppercase();
            let operand = if parts.len() > 1 {
                Some(parts[1].trim().to_string())
            } else {
                None
            };
            (Some(mnemonic), operand, None)
        }
    } else {
        (None, None, None)
    };

    Some(AssemblyLine {
        line_number,
        constant: None,
        label,
        mnemonic,
        operand,
        directive,
        comment: comment_part,
        span: (0, line.len()),
    })
}

/// Parse a directive line (e.g., ".org $8000", ".byte $42, $43")
pub fn parse_directive(line: &str) -> Result<crate::assembler::AssemblerDirective, String> {
    let line = line.trim();

    if !line.starts_with('.') {
        return Err("Directive must start with '.'".to_string());
    }

    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    let directive_name = parts[0].trim().to_lowercase();
    let args = if parts.len() > 1 { parts[1].trim() } else { "" };

    match directive_name.as_str() {
        ".org" => parse_org_directive(args),
        ".byte" => parse_byte_directive(args),
        ".word" => parse_word_directive(args),
        _ => Err(format!("Unknown directive: {}", directive_name)),
    }
}

/// Parse .org directive
pub fn parse_org_directive(args: &str) -> Result<crate::assembler::AssemblerDirective, String> {
    if args.is_empty() {
        return Err(".org directive requires an address argument".to_string());
    }

    let address = parse_number(args)?;
    Ok(crate::assembler::AssemblerDirective::Origin { address })
}

/// Parse a directive value (either a literal number or a symbol reference)
fn parse_directive_value(arg: &str) -> Result<crate::assembler::DirectiveValue, String> {
    let trimmed = arg.trim();

    // Check if it looks like a number (starts with $, %, or digit)
    if trimmed.starts_with('$')
        || trimmed.starts_with('%')
        || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        // Parse as number
        let val = parse_number(trimmed)?;
        Ok(crate::assembler::DirectiveValue::Literal(val))
    } else {
        // Treat as symbol reference
        // Validate it looks like a valid identifier
        if trimmed.is_empty() {
            return Err("Empty symbol name".to_string());
        }

        // Check first character is a letter
        if !trimmed.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(format!("Symbol '{}' must start with a letter", trimmed));
        }

        // Check all characters are alphanumeric or underscore
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(format!("Symbol '{}' contains invalid characters", trimmed));
        }

        Ok(crate::assembler::DirectiveValue::Symbol(
            trimmed.to_uppercase(),
        ))
    }
}

/// Parse .byte directive
pub fn parse_byte_directive(args: &str) -> Result<crate::assembler::AssemblerDirective, String> {
    if args.is_empty() {
        return Err(".byte directive requires at least one value".to_string());
    }

    let mut values = Vec::new();
    for arg in args.split(',') {
        let directive_val = parse_directive_value(arg)?;

        // For literals, validate they fit in a byte
        if let crate::assembler::DirectiveValue::Literal(val) = directive_val {
            if val > 0xFF {
                return Err(format!(
                    "Byte value ${:04X} is too large (must be 0-255)",
                    val
                ));
            }
        }

        values.push(directive_val);
    }

    if values.is_empty() {
        return Err(".byte directive requires at least one value".to_string());
    }

    Ok(crate::assembler::AssemblerDirective::Byte { values })
}

/// Parse .word directive
pub fn parse_word_directive(args: &str) -> Result<crate::assembler::AssemblerDirective, String> {
    if args.is_empty() {
        return Err(".word directive requires at least one value".to_string());
    }

    let mut values = Vec::new();
    for arg in args.split(',') {
        let directive_val = parse_directive_value(arg)?;
        values.push(directive_val);
    }

    if values.is_empty() {
        return Err(".word directive requires at least one value".to_string());
    }

    Ok(crate::assembler::AssemblerDirective::Word { values })
}

/// Normalize operand for matching: remove internal whitespace and convert to uppercase
///
/// This allows case-insensitive matching and tolerance for spaces around commas and parentheses.
/// Examples:
/// - "$10 , x" -> "$10,X"
/// - "( $20 ),y" -> "($20),Y"
/// - "lda" -> "LDA"
fn normalize_operand(operand: &str) -> String {
    // Remove all whitespace and convert to uppercase
    operand
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

/// Detect the addressing mode from operand syntax (for labels, assume Absolute/Relative)
///
/// Returns addressing mode without resolving values (for Pass 1 size calculation)
pub fn detect_addressing_mode_or_label(operand: &str) -> Result<AddressingMode, String> {
    let operand = operand.trim();
    let normalized = normalize_operand(operand);

    if normalized.is_empty() {
        return Ok(AddressingMode::Implicit);
    }

    // Accumulator mode: just "A"
    if normalized == "A" {
        return Ok(AddressingMode::Accumulator);
    }

    // Immediate: #$XX or #value
    if normalized.starts_with('#') {
        return Ok(AddressingMode::Immediate);
    }

    // Indirect: ($XXXX)
    if normalized.starts_with('(') && normalized.ends_with(')') && !normalized.contains(',') {
        return Ok(AddressingMode::Indirect);
    }

    // Indexed Indirect: ($XX,X)
    if normalized.starts_with('(') && normalized.contains(",X)") {
        return Ok(AddressingMode::IndirectX);
    }

    // Indirect Indexed: ($XX),Y
    if normalized.starts_with('(') && normalized.contains("),Y") {
        return Ok(AddressingMode::IndirectY);
    }

    // Indexed modes: $XXXX,X or $XXXX,Y
    if normalized.contains(",X") {
        let comma_pos = normalized.find(',').unwrap();
        let addr_str = &normalized[..comma_pos];

        // Try to parse the value to determine zero-page vs absolute
        if let Ok(addr) = parse_number(addr_str) {
            // Check the number of hex digits to distinguish zero-page from absolute
            // This handles cases like $0013 (4 digits = absolute) vs $13 (2 digits = zero-page)
            if let Some(hex_part) = addr_str.strip_prefix('$') {
                if hex_part.len() <= 2 && addr <= 0xFF {
                    return Ok(AddressingMode::ZeroPageX);
                } else {
                    return Ok(AddressingMode::AbsoluteX);
                }
            } else if addr <= 0xFF {
                return Ok(AddressingMode::ZeroPageX);
            } else {
                return Ok(AddressingMode::AbsoluteX);
            }
        }
        // If it's a label, assume absolute
        return Ok(AddressingMode::AbsoluteX);
    }

    if normalized.contains(",Y") {
        let comma_pos = normalized.find(',').unwrap();
        let addr_str = &normalized[..comma_pos];

        // Try to parse the value to determine zero-page vs absolute
        if let Ok(addr) = parse_number(addr_str) {
            // Check the number of hex digits to distinguish zero-page from absolute
            // This handles cases like $0013 (4 digits = absolute) vs $13 (2 digits = zero-page)
            if let Some(hex_part) = addr_str.strip_prefix('$') {
                if hex_part.len() <= 2 && addr <= 0xFF {
                    return Ok(AddressingMode::ZeroPageY);
                } else {
                    return Ok(AddressingMode::AbsoluteY);
                }
            } else if addr <= 0xFF {
                return Ok(AddressingMode::ZeroPageY);
            } else {
                return Ok(AddressingMode::AbsoluteY);
            }
        }
        // If it's a label, assume absolute
        return Ok(AddressingMode::AbsoluteY);
    }

    // Plain value or label
    if let Ok(value) = parse_number(&normalized) {
        // Choose zero-page or absolute based on value and hex digit count
        // $0013 (4 digits) = absolute, $13 (2 digits) = zero-page
        if let Some(hex_part) = normalized.strip_prefix('$') {
            if hex_part.len() <= 2 && value <= 0xFF {
                Ok(AddressingMode::ZeroPage)
            } else {
                Ok(AddressingMode::Absolute)
            }
        } else if value <= 0xFF {
            Ok(AddressingMode::ZeroPage)
        } else {
            Ok(AddressingMode::Absolute)
        }
    } else {
        // Must be a label - assume absolute (branches will be detected later)
        Ok(AddressingMode::Absolute)
    }
}

/// Detect the addressing mode from operand syntax
///
/// Returns (addressing_mode, operand_value) where operand_value is the parsed number
pub fn detect_addressing_mode(operand: &str) -> Result<(AddressingMode, u16), String> {
    let operand = operand.trim();
    let normalized = normalize_operand(operand);

    if normalized.is_empty() {
        return Ok((AddressingMode::Implicit, 0));
    }

    // Accumulator mode: just "A"
    if normalized == "A" {
        return Ok((AddressingMode::Accumulator, 0));
    }

    // Immediate: #$XX or #value
    if let Some(stripped) = normalized.strip_prefix('#') {
        let value = parse_number(stripped)?;
        return Ok((AddressingMode::Immediate, value));
    }

    // Indirect: ($XXXX)
    if normalized.starts_with('(') && normalized.ends_with(')') && !normalized.contains(',') {
        let addr_str = &normalized[1..normalized.len() - 1];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::Indirect, addr));
    }

    // Indexed Indirect: ($XX,X)
    if normalized.starts_with('(') && normalized.contains(",X)") {
        let comma_pos = normalized.find(',').unwrap();
        let addr_str = &normalized[1..comma_pos];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::IndirectX, addr));
    }

    // Indirect Indexed: ($XX),Y
    if normalized.starts_with('(') && normalized.contains("),Y") {
        let paren_pos = normalized.find(')').unwrap();
        let addr_str = &normalized[1..paren_pos];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::IndirectY, addr));
    }

    // Indexed modes: $XXXX,X or $XXXX,Y
    if normalized.contains(",X") {
        let comma_pos = normalized.find(',').unwrap();
        let addr_str = &normalized[..comma_pos];
        let addr = parse_number(addr_str)?;

        // Choose zero-page or absolute based on value and hex digit count
        // $0013,X (4 digits) = absolute, $13,X (2 digits) = zero-page
        if let Some(hex_part) = addr_str.strip_prefix('$') {
            if hex_part.len() <= 2 && addr <= 0xFF {
                return Ok((AddressingMode::ZeroPageX, addr));
            } else {
                return Ok((AddressingMode::AbsoluteX, addr));
            }
        } else if addr <= 0xFF {
            return Ok((AddressingMode::ZeroPageX, addr));
        } else {
            return Ok((AddressingMode::AbsoluteX, addr));
        }
    }

    if normalized.contains(",Y") {
        let comma_pos = normalized.find(',').unwrap();
        let addr_str = &normalized[..comma_pos];
        let addr = parse_number(addr_str)?;

        // Choose zero-page or absolute based on value and hex digit count
        // $0013,Y (4 digits) = absolute, $13,Y (2 digits) = zero-page
        if let Some(hex_part) = addr_str.strip_prefix('$') {
            if hex_part.len() <= 2 && addr <= 0xFF {
                return Ok((AddressingMode::ZeroPageY, addr));
            } else {
                return Ok((AddressingMode::AbsoluteY, addr));
            }
        } else if addr <= 0xFF {
            return Ok((AddressingMode::ZeroPageY, addr));
        } else {
            return Ok((AddressingMode::AbsoluteY, addr));
        }
    }

    // Plain address: $XXXX or value (could be zero-page, absolute, or relative)
    let value = parse_number(&normalized)?;

    // Choose zero-page or absolute based on value and hex digit count
    // $0013 (4 digits) = absolute, $13 (2 digits) = zero-page
    if let Some(hex_part) = normalized.strip_prefix('$') {
        if hex_part.len() <= 2 && value <= 0xFF {
            Ok((AddressingMode::ZeroPage, value))
        } else {
            Ok((AddressingMode::Absolute, value))
        }
    } else if value <= 0xFF {
        Ok((AddressingMode::ZeroPage, value))
    } else {
        Ok((AddressingMode::Absolute, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_hex() {
        assert_eq!(parse_number("$FF").unwrap(), 255);
        assert_eq!(parse_number("$42").unwrap(), 66);
        assert_eq!(parse_number("$1234").unwrap(), 0x1234);
    }

    #[test]
    fn test_parse_number_decimal() {
        assert_eq!(parse_number("255").unwrap(), 255);
        assert_eq!(parse_number("42").unwrap(), 42);
        assert_eq!(parse_number("1234").unwrap(), 1234);
    }

    #[test]
    fn test_parse_number_binary() {
        assert_eq!(parse_number("%11111111").unwrap(), 255);
        assert_eq!(parse_number("%01000010").unwrap(), 66);
        assert_eq!(parse_number("%00000001").unwrap(), 1);
    }

    #[test]
    fn test_parse_number_invalid() {
        assert!(parse_number("$XY").is_err());
        assert!(parse_number("%202").is_err());
        assert!(parse_number("ABC").is_err());
        assert!(parse_number("").is_err());
    }

    // T102: Unit tests for comment stripping

    #[test]
    fn test_comment_only_line() {
        let line = parse_line("; This is a comment", 1).unwrap();
        assert_eq!(line.comment, Some("This is a comment".to_string()));
        assert_eq!(line.mnemonic, None);
        assert_eq!(line.label, None);
    }

    #[test]
    fn test_inline_comment_stripping() {
        let line = parse_line("LDA #$42 ; Load value", 1).unwrap();
        assert_eq!(line.mnemonic, Some("LDA".to_string()));
        assert_eq!(line.operand, Some("#$42".to_string()));
        assert_eq!(line.comment, Some("Load value".to_string()));
    }

    #[test]
    fn test_no_comment() {
        let line = parse_line("LDA #$42", 1).unwrap();
        assert_eq!(line.mnemonic, Some("LDA".to_string()));
        assert_eq!(line.operand, Some("#$42".to_string()));
        assert_eq!(line.comment, None);
    }

    // T103: Unit tests for directive parsing

    #[test]
    fn test_parse_org_directive() {
        let result = parse_org_directive("$8000").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Origin { address } => {
                assert_eq!(address, 0x8000);
            }
            _ => panic!("Expected Origin directive"),
        }
    }

    #[test]
    fn test_parse_org_directive_missing_arg() {
        let result = parse_org_directive("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires an address"));
    }

    #[test]
    fn test_parse_byte_directive() {
        let result = parse_byte_directive("$42, $43, $44").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Byte { values } => {
                assert_eq!(
                    values,
                    vec![
                        crate::assembler::DirectiveValue::Literal(0x42),
                        crate::assembler::DirectiveValue::Literal(0x43),
                        crate::assembler::DirectiveValue::Literal(0x44)
                    ]
                );
            }
            _ => panic!("Expected Byte directive"),
        }
    }

    #[test]
    fn test_parse_byte_directive_single() {
        let result = parse_byte_directive("$FF").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Byte { values } => {
                assert_eq!(
                    values,
                    vec![crate::assembler::DirectiveValue::Literal(0xFF)]
                );
            }
            _ => panic!("Expected Byte directive"),
        }
    }

    #[test]
    fn test_parse_byte_directive_range_error() {
        let result = parse_byte_directive("$1234");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn test_parse_byte_directive_missing_arg() {
        let result = parse_byte_directive("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires at least one value"));
    }

    #[test]
    fn test_parse_word_directive() {
        let result = parse_word_directive("$1234, $5678").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Word { values } => {
                assert_eq!(
                    values,
                    vec![
                        crate::assembler::DirectiveValue::Literal(0x1234),
                        crate::assembler::DirectiveValue::Literal(0x5678)
                    ]
                );
            }
            _ => panic!("Expected Word directive"),
        }
    }

    #[test]
    fn test_parse_word_directive_single() {
        let result = parse_word_directive("$ABCD").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Word { values } => {
                assert_eq!(
                    values,
                    vec![crate::assembler::DirectiveValue::Literal(0xABCD)]
                );
            }
            _ => panic!("Expected Word directive"),
        }
    }

    #[test]
    fn test_parse_word_directive_missing_arg() {
        let result = parse_word_directive("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires at least one value"));
    }

    #[test]
    fn test_parse_directive_unknown() {
        let result = parse_directive(".unknown $1234");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown directive"));
    }

    // T013-T015: Unit tests for constant parsing

    #[test]
    fn test_parse_constant_simple() {
        let line = parse_line("MAX = 255", 1).unwrap();
        assert_eq!(line.constant, Some(("MAX".to_string(), "255".to_string())));
        assert_eq!(line.label, None);
        assert_eq!(line.mnemonic, None);
        assert_eq!(line.operand, None);
    }

    #[test]
    fn test_parse_constant_hex() {
        let line = parse_line("SCREEN = $4000", 1).unwrap();
        assert_eq!(
            line.constant,
            Some(("SCREEN".to_string(), "$4000".to_string()))
        );
        assert_eq!(line.label, None);
        assert_eq!(line.mnemonic, None);
    }

    #[test]
    fn test_parse_constant_binary() {
        let line = parse_line("BITS = %11110000", 1).unwrap();
        assert_eq!(
            line.constant,
            Some(("BITS".to_string(), "%11110000".to_string()))
        );
        assert_eq!(line.label, None);
        assert_eq!(line.mnemonic, None);
    }

    #[test]
    fn test_parse_constant_with_whitespace() {
        let line = parse_line("  MAX   =   $FF", 1).unwrap();
        assert_eq!(line.constant, Some(("MAX".to_string(), "$FF".to_string())));
    }

    #[test]
    fn test_parse_constant_with_comment() {
        let line = parse_line("PAGE_SIZE = 256  ; bytes per page", 1).unwrap();
        assert_eq!(
            line.constant,
            Some(("PAGE_SIZE".to_string(), "256".to_string()))
        );
        assert_eq!(line.comment, Some("bytes per page".to_string()));
    }

    #[test]
    fn test_parse_constant_lowercase_normalized() {
        let line = parse_line("max = 100", 1).unwrap();
        assert_eq!(line.constant, Some(("MAX".to_string(), "100".to_string())));
    }

    #[test]
    fn test_parse_directive_integration() {
        let result = parse_directive(".org $8000").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Origin { address } => {
                assert_eq!(address, 0x8000);
            }
            _ => panic!("Expected Origin directive"),
        }

        let result = parse_directive(".byte $42, $43").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Byte { values } => {
                assert_eq!(
                    values,
                    vec![
                        crate::assembler::DirectiveValue::Literal(0x42),
                        crate::assembler::DirectiveValue::Literal(0x43)
                    ]
                );
            }
            _ => panic!("Expected Byte directive"),
        }

        let result = parse_directive(".word $1234").unwrap();
        match result {
            crate::assembler::AssemblerDirective::Word { values } => {
                assert_eq!(
                    values,
                    vec![crate::assembler::DirectiveValue::Literal(0x1234)]
                );
            }
            _ => panic!("Expected Word directive"),
        }
    }

    // Tests for hex digit count logic (discovered via Klaus round-trip test)
    // This ensures that $13 (2 digits) is treated as zero page, while
    // $0013 (4 digits) is treated as absolute, even when the value is the same.

    #[test]
    fn test_hex_digit_count_zero_page_x() {
        // 2 hex digits → Zero Page,X
        let (mode, value) = detect_addressing_mode("$13,X").unwrap();
        assert_eq!(mode, AddressingMode::ZeroPageX);
        assert_eq!(value, 0x13);
    }

    #[test]
    fn test_hex_digit_count_absolute_x() {
        // 4 hex digits → Absolute,X (even though value could fit in zero page)
        let (mode, value) = detect_addressing_mode("$0013,X").unwrap();
        assert_eq!(mode, AddressingMode::AbsoluteX);
        assert_eq!(value, 0x0013);
    }

    #[test]
    fn test_hex_digit_count_zero_page_y() {
        // 2 hex digits → Zero Page,Y
        let (mode, value) = detect_addressing_mode("$13,Y").unwrap();
        assert_eq!(mode, AddressingMode::ZeroPageY);
        assert_eq!(value, 0x13);
    }

    #[test]
    fn test_hex_digit_count_absolute_y() {
        // 4 hex digits → Absolute,Y (even though value could fit in zero page)
        let (mode, value) = detect_addressing_mode("$0013,Y").unwrap();
        assert_eq!(mode, AddressingMode::AbsoluteY);
        assert_eq!(value, 0x0013);
    }

    #[test]
    fn test_hex_digit_count_zero_page() {
        // 2 hex digits → Zero Page
        let (mode, value) = detect_addressing_mode("$13").unwrap();
        assert_eq!(mode, AddressingMode::ZeroPage);
        assert_eq!(value, 0x13);
    }

    #[test]
    fn test_hex_digit_count_absolute() {
        // 4 hex digits → Absolute (even though value could fit in zero page)
        let (mode, value) = detect_addressing_mode("$0013").unwrap();
        assert_eq!(mode, AddressingMode::Absolute);
        assert_eq!(value, 0x0013);
    }

    #[test]
    fn test_decimal_values_still_use_value_based_detection() {
        // Decimal values still use value-based detection (no hex prefix)
        let (mode, value) = detect_addressing_mode("19,X").unwrap();
        assert_eq!(mode, AddressingMode::ZeroPageX); // 19 < 256 → zero page
        assert_eq!(value, 19);

        let (mode, value) = detect_addressing_mode("256,X").unwrap();
        assert_eq!(mode, AddressingMode::AbsoluteX); // 256 >= 256 → absolute
        assert_eq!(value, 256);
    }

    #[test]
    fn test_hex_digit_count_with_leading_zeros() {
        // 4 digits with leading zeros → Absolute
        let (mode, value) = detect_addressing_mode("$0001").unwrap();
        assert_eq!(mode, AddressingMode::Absolute);
        assert_eq!(value, 0x0001);

        // 2 digits → Zero Page
        let (mode, value) = detect_addressing_mode("$01").unwrap();
        assert_eq!(mode, AddressingMode::ZeroPage);
        assert_eq!(value, 0x01);
    }
}
