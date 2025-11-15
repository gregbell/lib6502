//! Instruction encoder for the 6502 assembler

use crate::addressing::AddressingMode;
use crate::assembler::{AssemblerError, ErrorType};
use crate::opcodes::OPCODE_TABLE;

/// Find the opcode for a given mnemonic and addressing mode
///
/// Returns the opcode byte or an error if the combination is invalid
pub fn find_opcode(mnemonic: &str, mode: AddressingMode) -> Result<u8, AssemblerError> {
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        if metadata.mnemonic == mnemonic && metadata.addressing_mode == mode {
            return Ok(opcode as u8);
        }
    }

    Err(AssemblerError {
        error_type: ErrorType::InvalidOperand,
        line: 0,
        column: 0,
        span: (0, 0),
        message: format!(
            "No opcode found for {} with addressing mode {:?}",
            mnemonic, mode
        ),
    })
}

/// Encode an instruction into bytes
///
/// # Arguments
///
/// * `mnemonic` - The instruction mnemonic (e.g., "LDA")
/// * `mode` - The addressing mode
/// * `operand_value` - The operand value (if any)
///
/// # Returns
///
/// A vector of bytes representing the encoded instruction
pub fn encode_instruction(
    mnemonic: &str,
    mode: AddressingMode,
    operand_value: u16,
) -> Result<Vec<u8>, AssemblerError> {
    // Validate operand range based on addressing mode
    match mode {
        AddressingMode::Immediate => {
            if operand_value > 0xFF {
                return Err(AssemblerError {
                    error_type: ErrorType::RangeError,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!(
                        "Immediate value ${:04X} exceeds 8-bit range (0-255)",
                        operand_value
                    ),
                });
            }
        }
        AddressingMode::ZeroPage | AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
            if operand_value > 0xFF {
                return Err(AssemblerError {
                    error_type: ErrorType::RangeError,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!(
                        "Zero-page address ${:04X} exceeds range (0-255)",
                        operand_value
                    ),
                });
            }
        }
        _ => {}
    }

    // Find the opcode
    let opcode = find_opcode(mnemonic, mode)?;

    // Build the instruction bytes
    let mut bytes = vec![opcode];

    match mode {
        AddressingMode::Implicit | AddressingMode::Accumulator => {
            // No operand bytes
        }
        AddressingMode::Immediate
        | AddressingMode::ZeroPage
        | AddressingMode::ZeroPageX
        | AddressingMode::ZeroPageY
        | AddressingMode::IndirectX
        | AddressingMode::IndirectY
        | AddressingMode::Relative => {
            // 1-byte operand
            bytes.push(operand_value as u8);
        }
        AddressingMode::Absolute
        | AddressingMode::AbsoluteX
        | AddressingMode::AbsoluteY
        | AddressingMode::Indirect => {
            // 2-byte operand (little-endian)
            bytes.push((operand_value & 0xFF) as u8);
            bytes.push((operand_value >> 8) as u8);
        }
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_opcode_lda_immediate() {
        let opcode = find_opcode("LDA", AddressingMode::Immediate).unwrap();
        assert_eq!(opcode, 0xA9);
    }

    #[test]
    fn test_encode_lda_immediate() {
        let bytes = encode_instruction("LDA", AddressingMode::Immediate, 0x42).unwrap();
        assert_eq!(bytes, vec![0xA9, 0x42]);
    }

    #[test]
    fn test_encode_sta_absolute() {
        let bytes = encode_instruction("STA", AddressingMode::Absolute, 0x8000).unwrap();
        assert_eq!(bytes, vec![0x8D, 0x00, 0x80]);
    }

    #[test]
    fn test_encode_immediate_range_error() {
        let result = encode_instruction("LDA", AddressingMode::Immediate, 0x1234);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_type, ErrorType::RangeError);
    }
}
