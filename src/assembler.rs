//! 6502 Assembler Module
//!
//! Converts assembly language source code into binary machine code.

pub mod encoder;
pub mod parser;
pub mod source_map;
pub mod symbol_table;

use crate::opcodes;

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

impl AssemblerOutput {
    /// Look up a symbol by name
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.iter().find(|s| s.name == name)
    }

    /// Get source location for a given instruction address
    pub fn get_source_location(&self, address: u16) -> Option<source_map::SourceLocation> {
        self.source_map.get_source_location(address)
    }

    /// Get address range for a given source line
    pub fn get_address_range(&self, line: usize) -> Option<source_map::AddressRange> {
        self.source_map.get_address_range(line)
    }
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

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line {}, Column {}: {} - {}",
            self.line,
            self.column,
            match self.error_type {
                ErrorType::SyntaxError => "Syntax Error",
                ErrorType::UndefinedLabel => "Undefined Label",
                ErrorType::DuplicateLabel => "Duplicate Label",
                ErrorType::InvalidLabel => "Invalid Label",
                ErrorType::InvalidMnemonic => "Invalid Mnemonic",
                ErrorType::InvalidOperand => "Invalid Operand",
                ErrorType::RangeError => "Range Error",
                ErrorType::InvalidDirective => "Invalid Directive",
            },
            self.message
        )
    }
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

    // Parse all lines
    let parsed_lines: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| parser::parse_line(line, idx + 1))
        .collect();

    // Pass 1: Build symbol table
    let mut symbol_table = symbol_table::SymbolTable::new();
    let mut current_address = 0u16;

    for line in &parsed_lines {
        // Check for label definition
        if let Some(ref label) = line.label {
            // Validate label name
            if let Err(e) = validate_label(label) {
                errors.push(AssemblerError {
                    error_type: ErrorType::InvalidLabel,
                    line: line.line_number,
                    column: 0,
                    span: line.span,
                    message: e,
                });
            } else {
                // Add to symbol table
                if let Err(existing) =
                    symbol_table.add_symbol(label.clone(), current_address, line.line_number)
                {
                    errors.push(AssemblerError {
                        error_type: ErrorType::DuplicateLabel,
                        line: line.line_number,
                        column: 0,
                        span: line.span,
                        message: format!(
                            "Duplicate label '{}' (previously defined at line {})",
                            label, existing.defined_at
                        ),
                    });
                }
            }
        }

        // Handle directives
        if let Some(ref directive) = line.directive {
            match directive {
                AssemblerDirective::Origin { address } => {
                    current_address = *address;
                }
                AssemblerDirective::Byte { values } => {
                    current_address += values.len() as u16;
                }
                AssemblerDirective::Word { values } => {
                    current_address += (values.len() * 2) as u16;
                }
            }
            continue;
        }

        // Skip lines with only labels or comments
        if line.mnemonic.is_none() {
            continue;
        }

        // Calculate instruction size to advance address
        let mnemonic = line.mnemonic.as_ref().unwrap();

        // Determine addressing mode (without resolving labels yet)
        let mode = if let Some(ref operand) = line.operand {
            match parser::detect_addressing_mode_or_label(operand) {
                Ok(m) => m,
                Err(_) => {
                    // Error will be caught in Pass 2
                    crate::addressing::AddressingMode::Implicit
                }
            }
        } else {
            crate::addressing::AddressingMode::Implicit
        };

        // Look up instruction size
        if let Ok(opcode_meta) = encoder::find_opcode_metadata(mnemonic, mode) {
            current_address += opcode_meta.size_bytes as u16;
        } else {
            // Error will be caught in Pass 2
            current_address += 1;
        }
    }

    // Return early if there were label validation errors
    if !errors.is_empty() {
        return Err(errors);
    }

    // Pass 2: Encode instructions with label resolution
    let mut bytes = Vec::new();
    let mut source_map = source_map::SourceMap::new();
    current_address = 0u16;

    for line in &parsed_lines {
        // Handle directives
        if let Some(ref directive) = line.directive {
            match directive {
                AssemblerDirective::Origin { address } => {
                    current_address = *address;
                }
                AssemblerDirective::Byte { values } => {
                    let instruction_start_address = current_address;

                    // Add bytes directly to output
                    bytes.extend(values);

                    // Add to source map
                    source_map.add_mapping(
                        instruction_start_address,
                        source_map::SourceLocation {
                            line: line.line_number,
                            column: 0,
                            length: line.span.1 - line.span.0,
                        },
                    );
                    source_map.add_line_mapping(
                        line.line_number,
                        source_map::AddressRange {
                            start: instruction_start_address,
                            end: instruction_start_address + values.len() as u16,
                        },
                    );

                    current_address += values.len() as u16;
                }
                AssemblerDirective::Word { values } => {
                    let instruction_start_address = current_address;

                    // Add words in little-endian format
                    for word in values {
                        bytes.push((word & 0xFF) as u8); // Low byte
                        bytes.push(((word >> 8) & 0xFF) as u8); // High byte
                    }

                    // Add to source map
                    source_map.add_mapping(
                        instruction_start_address,
                        source_map::SourceLocation {
                            line: line.line_number,
                            column: 0,
                            length: line.span.1 - line.span.0,
                        },
                    );
                    source_map.add_line_mapping(
                        line.line_number,
                        source_map::AddressRange {
                            start: instruction_start_address,
                            end: instruction_start_address + (values.len() * 2) as u16,
                        },
                    );

                    current_address += (values.len() * 2) as u16;
                }
            }
            continue;
        }

        // Skip lines with only labels or comments
        if line.mnemonic.is_none() {
            continue;
        }

        let mnemonic = line.mnemonic.as_ref().unwrap();

        // Check for invalid directive (mnemonic starts with .)
        if mnemonic.starts_with('.') {
            errors.push(AssemblerError {
                error_type: ErrorType::InvalidDirective,
                line: line.line_number,
                column: 0,
                span: line.span,
                message: format!("Invalid or unknown directive: {}", mnemonic),
            });
            continue;
        }

        let instruction_start_address = current_address;

        // Detect addressing mode and resolve operand value (including labels)
        let (mode, value) = if let Some(ref operand) = line.operand {
            match resolve_operand(operand, &symbol_table, instruction_start_address, mnemonic) {
                Ok((m, v)) => (m, v),
                Err(e) => {
                    errors.push(AssemblerError {
                        error_type: e.error_type,
                        line: line.line_number,
                        column: 0,
                        span: line.span,
                        message: e.message,
                    });
                    continue; // Error recovery: skip this line
                }
            }
        } else {
            (crate::addressing::AddressingMode::Implicit, 0)
        };

        // Encode the instruction
        match encoder::encode_instruction(mnemonic, mode, value) {
            Ok(instruction_bytes) => {
                let instruction_size = instruction_bytes.len() as u16;

                // Add to source map (address → source location)
                source_map.add_mapping(
                    instruction_start_address,
                    source_map::SourceLocation {
                        line: line.line_number,
                        column: 0,
                        length: line.span.1 - line.span.0,
                    },
                );

                // Add to source map (line → address range)
                source_map.add_line_mapping(
                    line.line_number,
                    source_map::AddressRange {
                        start: instruction_start_address,
                        end: instruction_start_address + instruction_size,
                    },
                );

                bytes.extend(instruction_bytes);
                current_address += instruction_size;
            }
            Err(mut e) => {
                // Update error with correct line info
                e.line = line.line_number;
                e.span = line.span;

                // Check if this is an invalid mnemonic
                let is_valid_mnemonic = opcodes::OPCODE_TABLE
                    .iter()
                    .any(|op| op.mnemonic == mnemonic.as_str());

                if !is_valid_mnemonic {
                    e.error_type = ErrorType::InvalidMnemonic;
                    e.message = format!("Invalid mnemonic '{}'", mnemonic);
                }

                errors.push(e);
                // Error recovery: continue to collect more errors
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Finalize source map (sort for binary search)
    source_map.finalize();

    Ok(AssemblerOutput {
        bytes,
        symbol_table: symbol_table.symbols().to_vec(),
        source_map,
        warnings: Vec::new(),
    })
}

/// Resolve an operand, handling both numeric values and label references
fn resolve_operand(
    operand: &str,
    symbol_table: &symbol_table::SymbolTable,
    current_address: u16,
    mnemonic: &str,
) -> Result<(crate::addressing::AddressingMode, u16), AssemblerError> {
    // Check if operand is a label reference (no prefix like $, #, (, etc.)
    let operand_trimmed = operand.trim();

    // If it starts with a special character, it's not a plain label
    if operand_trimmed.starts_with('$')
        || operand_trimmed.starts_with('#')
        || operand_trimmed.starts_with('(')
        || operand_trimmed.starts_with('%')
        || operand_trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    {
        // Parse as normal addressing mode
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // Must be a label reference - look it up
    if let Some(symbol) = symbol_table.lookup_symbol(operand_trimmed) {
        let target_address = symbol.address;

        // Determine if this is a branch instruction (needs relative addressing)
        let is_branch = matches!(
            mnemonic,
            "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BVC" | "BVS"
        );

        if is_branch {
            // Calculate relative offset
            // Branch offset is relative to the address of the next instruction
            let next_instruction_address = current_address + 2; // Branch instructions are 2 bytes
            let offset = target_address as i32 - next_instruction_address as i32;

            // Check if offset is in range (-128 to +127)
            if !(-128..=127).contains(&offset) {
                return Err(AssemblerError {
                    error_type: ErrorType::RangeError,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!(
                        "Branch to '{}' is out of range (offset: {}, must be -128 to +127)",
                        operand_trimmed, offset
                    ),
                });
            }

            // Convert to unsigned byte (two's complement)
            let offset_byte = (offset as i8) as u8;
            Ok((
                crate::addressing::AddressingMode::Relative,
                offset_byte as u16,
            ))
        } else {
            // Absolute addressing for JMP, JSR, etc.
            Ok((crate::addressing::AddressingMode::Absolute, target_address))
        }
    } else {
        // Undefined label
        Err(AssemblerError {
            error_type: ErrorType::UndefinedLabel,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Undefined label '{}'", operand_trimmed),
        })
    }
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
        return Err(format!("label must start with a letter, not '{}'", first));
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
