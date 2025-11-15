//! 6502 Assembler Module
//!
//! Converts assembly language source code into binary machine code.

pub mod encoder;
pub mod parser;
pub mod source_map;
pub mod symbol_table;

use crate::addressing::AddressingMode;

/// Complete output from assembling source code
#[derive(Debug, Clone)]
pub struct AssemblerOutput {
    /// Assembled machine code bytes
    pub bytes: Vec<u8>,

    /// Symbol table with all defined labels
    pub symbol_table: Vec<Symbol>,

    /// Source map for debugging
    pub source_map: source_map::SourceMap,

    /// Non-fatal warnings encountered during assembly
    pub warnings: Vec<AssemblerWarning>,
}

/// A symbol table entry mapping a label to an address
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Label name (case-sensitive after normalization)
    pub name: String,

    /// Resolved memory address for this label
    pub address: u16,

    /// Source line where label was defined
    pub defined_at: usize,
}

/// A non-fatal warning from the assembler
#[derive(Debug, Clone)]
pub struct AssemblerWarning {
    /// Line number where warning occurred
    pub line: usize,

    /// Warning message
    pub message: String,
}

/// An error encountered during assembly
#[derive(Debug, Clone, PartialEq)]
pub struct AssemblerError {
    /// Error type classification
    pub error_type: ErrorType,

    /// Line number where error occurred (1-indexed)
    pub line: usize,

    /// Column number where error starts (0-indexed)
    pub column: usize,

    /// Character span (start, end) in the source line
    pub span: (usize, usize),

    /// Human-readable error message
    pub message: String,
}

/// Classification of assembly errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Syntax error (invalid format, unexpected character)
    SyntaxError,

    /// Undefined label reference
    UndefinedLabel,

    /// Duplicate label definition
    DuplicateLabel,

    /// Invalid label name (too long, starts with digit, etc.)
    InvalidLabel,

    /// Invalid mnemonic (not a recognized instruction)
    InvalidMnemonic,

    /// Invalid operand format for addressing mode
    InvalidOperand,

    /// Operand value out of range (e.g., immediate value > 255, branch too far)
    RangeError,

    /// Invalid directive usage
    InvalidDirective,
}

/// Assembler directive types
#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerDirective {
    /// Set origin address (.org $XXXX)
    Origin { address: u16 },

    /// Insert literal bytes (.byte $XX, $YY, ...)
    Byte { values: Vec<u8> },

    /// Insert literal 16-bit words (.word $XXXX, $YYYY, ...)
    Word { values: Vec<u16> },
}

/// Assemble source code into machine code
///
/// # Arguments
///
/// * `source` - The assembly source code text
///
/// # Returns
///
/// Ok(AssemblerOutput) on success, Err(Vec<AssemblerError>) on failure
pub fn assemble(source: &str) -> Result<AssemblerOutput, Vec<AssemblerError>> {
    let mut errors = Vec::new();

    // TODO: Implement two-pass assembly
    // Pass 1: Parse and build symbol table
    // Pass 2: Encode and emit bytes

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(AssemblerOutput {
        bytes: Vec::new(),
        symbol_table: Vec::new(),
        source_map: source_map::SourceMap::new(),
        warnings: Vec::new(),
    })
}

/// Validate a label name according to 6502 conventions
///
/// Labels must:
/// - Start with a letter [a-zA-Z]
/// - Contain only alphanumeric characters and underscores
/// - Not exceed 32 characters in length
fn validate_label(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("label name cannot be empty".to_string());
    }

    if name.len() > 32 {
        return Err(format!("label name too long (max 32 characters): {}", name));
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() {
        return Err(format!(
            "label must start with a letter, not '{}'",
            first
        ));
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(format!(
                "label contains invalid character '{}' (only letters, digits, and underscores allowed)",
                ch
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_label_valid() {
        assert!(validate_label("START").is_ok());
        assert!(validate_label("loop_1").is_ok());
        assert!(validate_label("MyLabel").is_ok());
        assert!(validate_label("A").is_ok());
    }

    #[test]
    fn test_validate_label_invalid() {
        assert!(validate_label("").is_err());
        assert!(validate_label("1START").is_err());
        assert!(validate_label("MY-LABEL").is_err());
        assert!(validate_label("LABEL!").is_err());
        assert!(validate_label(&"A".repeat(33)).is_err());
    }
}
