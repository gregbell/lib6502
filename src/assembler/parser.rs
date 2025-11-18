//! Assembly source parser

use crate::addressing::AddressingMode;

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
