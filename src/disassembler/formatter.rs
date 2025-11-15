//! Formatting functions for disassembled instructions

use crate::addressing::AddressingMode;
use crate::disassembler::Instruction;

/// Format a single instruction as assembly text
///
/// # Arguments
///
/// * `instr` - The instruction to format
///
/// # Returns
///
/// A string containing the formatted assembly instruction
pub fn format_instruction(instr: &Instruction) -> String {
    let operand = format_operand(instr);

    if operand.is_empty() {
        instr.mnemonic.to_string()
    } else {
        format!("{} {}", instr.mnemonic, operand)
    }
}

/// Format the operand based on addressing mode
fn format_operand(instr: &Instruction) -> String {
    use AddressingMode::*;

    // Special case for .byte directive (illegal opcodes)
    if instr.mnemonic == ".byte" {
        return format!("${:02X}", instr.opcode);
    }

    match instr.addressing_mode {
        Implicit => String::new(),
        Accumulator => "A".to_string(),
        Immediate => {
            if instr.operand_bytes.is_empty() {
                "#$??".to_string()
            } else {
                format!("#${:02X}", instr.operand_bytes[0])
            }
        }
        ZeroPage => {
            if instr.operand_bytes.is_empty() {
                "$??".to_string()
            } else {
                format!("${:02X}", instr.operand_bytes[0])
            }
        }
        ZeroPageX => {
            if instr.operand_bytes.is_empty() {
                "$??,X".to_string()
            } else {
                format!("${:02X},X", instr.operand_bytes[0])
            }
        }
        ZeroPageY => {
            if instr.operand_bytes.is_empty() {
                "$??,Y".to_string()
            } else {
                format!("${:02X},Y", instr.operand_bytes[0])
            }
        }
        Relative => {
            if instr.operand_bytes.is_empty() {
                "$????".to_string()
            } else {
                // Calculate target address from relative offset
                let offset = instr.operand_bytes[0] as i8;
                let target = (instr.address as i32 + 2 + offset as i32) as u16;
                format!("${:04X}", target)
            }
        }
        Absolute => {
            if instr.operand_bytes.len() < 2 {
                "$????".to_string()
            } else {
                let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
                format!("${:04X}", addr)
            }
        }
        AbsoluteX => {
            if instr.operand_bytes.len() < 2 {
                "$????,X".to_string()
            } else {
                let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
                format!("${:04X},X", addr)
            }
        }
        AbsoluteY => {
            if instr.operand_bytes.len() < 2 {
                "$????,Y".to_string()
            } else {
                let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
                format!("${:04X},Y", addr)
            }
        }
        Indirect => {
            if instr.operand_bytes.len() < 2 {
                "($????)".to_string()
            } else {
                let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
                format!("(${:04X})", addr)
            }
        }
        IndirectX => {
            if instr.operand_bytes.is_empty() {
                "($??,X)".to_string()
            } else {
                format!("(${:02X},X)", instr.operand_bytes[0])
            }
        }
        IndirectY => {
            if instr.operand_bytes.is_empty() {
                "($??),Y".to_string()
            } else {
                format!("(${:02X}),Y", instr.operand_bytes[0])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_immediate() {
        let instr = Instruction {
            address: 0x8000,
            opcode: 0xA9,
            mnemonic: "LDA",
            addressing_mode: AddressingMode::Immediate,
            operand_bytes: vec![0x42],
            size_bytes: 2,
            base_cycles: 2,
        };

        assert_eq!(format_instruction(&instr), "LDA #$42");
    }

    #[test]
    fn test_format_absolute() {
        let instr = Instruction {
            address: 0x0000,
            opcode: 0x8D,
            mnemonic: "STA",
            addressing_mode: AddressingMode::Absolute,
            operand_bytes: vec![0x00, 0x80],
            size_bytes: 3,
            base_cycles: 4,
        };

        assert_eq!(format_instruction(&instr), "STA $8000");
    }

    #[test]
    fn test_format_implied() {
        let instr = Instruction {
            address: 0x1000,
            opcode: 0xEA,
            mnemonic: "NOP",
            addressing_mode: AddressingMode::Implicit,
            operand_bytes: vec![],
            size_bytes: 1,
            base_cycles: 2,
        };

        assert_eq!(format_instruction(&instr), "NOP");
    }

    #[test]
    fn test_format_illegal_opcode() {
        let instr = Instruction {
            address: 0x2000,
            opcode: 0xFF,
            mnemonic: ".byte",
            addressing_mode: AddressingMode::Implicit,
            operand_bytes: vec![0xFF],
            size_bytes: 1,
            base_cycles: 0,
        };

        assert_eq!(format_instruction(&instr), ".byte $FF");
    }
}
