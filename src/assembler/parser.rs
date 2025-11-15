//! Assembly source parser

use crate::assembler::{AssemblerDirective, AssemblerError, ErrorType};

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
