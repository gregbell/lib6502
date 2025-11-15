//! Assembly source parser

use crate::addressing::AddressingMode;

/// A parsed line of assembly source
#[derive(Debug, Clone, PartialEq)]
pub struct AssemblyLine {
    /// Line number in source file (1-indexed)
    pub line_number: usize,

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

    if s.starts_with('$') {
        // Hexadecimal
        u16::from_str_radix(&s[1..], 16).map_err(|e| format!("invalid hex number: {}", e))
    } else if s.starts_with('%') {
        // Binary
        u16::from_str_radix(&s[1..], 2).map_err(|e| format!("invalid binary number: {}", e))
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
    if trimmed.starts_with(';') {
        return Some(AssemblyLine {
            line_number,
            label: None,
            mnemonic: None,
            operand: None,
            comment: Some(trimmed[1..].trim().to_string()),
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
            label: None,
            mnemonic: None,
            operand: None,
            comment: comment_part,
            span: (0, line.len()),
        });
    }

    // Check for label (ends with colon)
    let (label, rest) = if let Some(colon_pos) = code_part.find(':') {
        let label_name = code_part[..colon_pos].trim().to_uppercase();
        let rest = code_part[colon_pos + 1..].trim();
        (Some(label_name), rest)
    } else {
        (None, code_part)
    };

    // Parse mnemonic and operand
    let (mnemonic, operand) = if !rest.is_empty() {
        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        let mnemonic = parts[0].trim().to_uppercase();
        let operand = if parts.len() > 1 {
            Some(parts[1].trim().to_string())
        } else {
            None
        };
        (Some(mnemonic), operand)
    } else {
        (None, None)
    };

    Some(AssemblyLine {
        line_number,
        label,
        mnemonic,
        operand,
        comment: comment_part,
        span: (0, line.len()),
    })
}

/// Detect the addressing mode from operand syntax (for labels, assume Absolute/Relative)
///
/// Returns addressing mode without resolving values (for Pass 1 size calculation)
pub fn detect_addressing_mode_or_label(operand: &str) -> Result<AddressingMode, String> {
    let operand = operand.trim();

    if operand.is_empty() {
        return Ok(AddressingMode::Implicit);
    }

    // Accumulator mode: just "A"
    if operand.eq_ignore_ascii_case("A") {
        return Ok(AddressingMode::Accumulator);
    }

    // Immediate: #$XX or #value
    if operand.starts_with('#') {
        return Ok(AddressingMode::Immediate);
    }

    // Indirect: ($XXXX)
    if operand.starts_with('(') && operand.ends_with(')') && !operand.contains(',') {
        return Ok(AddressingMode::Indirect);
    }

    // Indexed Indirect: ($XX,X)
    if operand.starts_with('(') && operand.contains(",X)") {
        return Ok(AddressingMode::IndirectX);
    }

    // Indirect Indexed: ($XX),Y
    if operand.starts_with('(') && operand.contains("),Y") {
        return Ok(AddressingMode::IndirectY);
    }

    // Indexed modes: $XXXX,X or $XXXX,Y
    if operand.contains(",X") {
        let comma_pos = operand.find(',').unwrap();
        let addr_str = &operand[..comma_pos];

        // Try to parse the value to determine zero-page vs absolute
        if let Ok(addr) = parse_number(addr_str) {
            if addr <= 0xFF {
                return Ok(AddressingMode::ZeroPageX);
            } else {
                return Ok(AddressingMode::AbsoluteX);
            }
        }
        // If it's a label, assume absolute
        return Ok(AddressingMode::AbsoluteX);
    }

    if operand.contains(",Y") {
        let comma_pos = operand.find(',').unwrap();
        let addr_str = &operand[..comma_pos];

        // Try to parse the value to determine zero-page vs absolute
        if let Ok(addr) = parse_number(addr_str) {
            if addr <= 0xFF {
                return Ok(AddressingMode::ZeroPageY);
            } else {
                return Ok(AddressingMode::AbsoluteY);
            }
        }
        // If it's a label, assume absolute
        return Ok(AddressingMode::AbsoluteY);
    }

    // Plain value or label
    if let Ok(value) = parse_number(operand) {
        // Choose zero-page or absolute based on value
        if value <= 0xFF {
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

    if operand.is_empty() {
        return Ok((AddressingMode::Implicit, 0));
    }

    // Accumulator mode: just "A"
    if operand.eq_ignore_ascii_case("A") {
        return Ok((AddressingMode::Accumulator, 0));
    }

    // Immediate: #$XX or #value
    if operand.starts_with('#') {
        let value = parse_number(&operand[1..])?;
        return Ok((AddressingMode::Immediate, value));
    }

    // Indirect: ($XXXX)
    if operand.starts_with('(') && operand.ends_with(')') && !operand.contains(',') {
        let addr_str = &operand[1..operand.len() - 1];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::Indirect, addr));
    }

    // Indexed Indirect: ($XX,X)
    if operand.starts_with('(') && operand.contains(",X)") {
        let comma_pos = operand.find(',').unwrap();
        let addr_str = &operand[1..comma_pos];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::IndirectX, addr));
    }

    // Indirect Indexed: ($XX),Y
    if operand.starts_with('(') && operand.contains("),Y") {
        let paren_pos = operand.find(')').unwrap();
        let addr_str = &operand[1..paren_pos];
        let addr = parse_number(addr_str)?;
        return Ok((AddressingMode::IndirectY, addr));
    }

    // Indexed modes: $XXXX,X or $XXXX,Y
    if operand.contains(",X") {
        let comma_pos = operand.find(',').unwrap();
        let addr_str = &operand[..comma_pos];
        let addr = parse_number(addr_str)?;

        // Choose zero-page or absolute based on value
        if addr <= 0xFF {
            return Ok((AddressingMode::ZeroPageX, addr));
        } else {
            return Ok((AddressingMode::AbsoluteX, addr));
        }
    }

    if operand.contains(",Y") {
        let comma_pos = operand.find(',').unwrap();
        let addr_str = &operand[..comma_pos];
        let addr = parse_number(addr_str)?;

        // Choose zero-page or absolute based on value
        if addr <= 0xFF {
            return Ok((AddressingMode::ZeroPageY, addr));
        } else {
            return Ok((AddressingMode::AbsoluteY, addr));
        }
    }

    // Plain address: $XXXX or value (could be zero-page, absolute, or relative)
    let value = parse_number(operand)?;

    // Choose zero-page or absolute based on value
    if value <= 0xFF {
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
}
