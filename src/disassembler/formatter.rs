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

/// Format a 4-digit hexadecimal address
///
/// # Arguments
///
/// * `address` - The address to format
///
/// # Returns
///
/// A string in the format "XXXX"
pub fn format_address(address: u16) -> String {
    format!("{:04X}", address)
}

/// Format instruction bytes as hexadecimal (up to 3 bytes, left-aligned)
///
/// # Arguments
///
/// * `instr` - The instruction to format bytes for
///
/// # Returns
///
/// A string with hex bytes, padded to 9 characters for alignment
pub fn format_hex_bytes(instr: &Instruction) -> String {
    let mut bytes = vec![instr.opcode];
    bytes.extend(&instr.operand_bytes);

    let hex_str = bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    // Pad to 9 characters (3 bytes with spaces: "XX XX XX")
    format!("{:<9}", hex_str)
}

/// Format a vector of instructions as hex dump
///
/// # Arguments
///
/// * `instructions` - Vector of instructions to format
///
/// # Returns
///
/// Multi-line string with formatted hex dump
pub fn format_hex_dump(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|instr| {
            let addr = format_address(instr.address);
            let bytes = format_hex_bytes(instr);
            let asm = format_instruction(instr);
            format!("{}: {}  {}", addr, bytes, asm)
        })
        .collect::<Vec<_>>()
        .join("\n")
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

    // T088: Unit tests for hex dump formatting

    #[test]
    fn test_format_address() {
        assert_eq!(format_address(0x0000), "0000");
        assert_eq!(format_address(0x8000), "8000");
        assert_eq!(format_address(0xFFFF), "FFFF");
        assert_eq!(format_address(0x1234), "1234");
    }

    #[test]
    fn test_format_hex_bytes_one_byte() {
        let instr = Instruction {
            address: 0x0000,
            opcode: 0xEA,
            mnemonic: "NOP",
            addressing_mode: AddressingMode::Implicit,
            operand_bytes: vec![],
            size_bytes: 1,
            base_cycles: 2,
        };

        let bytes = format_hex_bytes(&instr);
        assert_eq!(bytes, "EA       "); // Padded to 9 chars
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn test_format_hex_bytes_two_bytes() {
        let instr = Instruction {
            address: 0x0000,
            opcode: 0xA9,
            mnemonic: "LDA",
            addressing_mode: AddressingMode::Immediate,
            operand_bytes: vec![0x42],
            size_bytes: 2,
            base_cycles: 2,
        };

        let bytes = format_hex_bytes(&instr);
        assert_eq!(bytes, "A9 42    "); // Padded to 9 chars
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn test_format_hex_bytes_three_bytes() {
        let instr = Instruction {
            address: 0x0000,
            opcode: 0x8D,
            mnemonic: "STA",
            addressing_mode: AddressingMode::Absolute,
            operand_bytes: vec![0x00, 0x80],
            size_bytes: 3,
            base_cycles: 4,
        };

        let bytes = format_hex_bytes(&instr);
        assert_eq!(bytes, "8D 00 80 "); // Padded to 9 chars
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn test_format_hex_dump_single() {
        let instructions = vec![Instruction {
            address: 0x8000,
            opcode: 0xA9,
            mnemonic: "LDA",
            addressing_mode: AddressingMode::Immediate,
            operand_bytes: vec![0x42],
            size_bytes: 2,
            base_cycles: 2,
        }];

        let output = format_hex_dump(&instructions);
        assert_eq!(output, "8000: A9 42      LDA #$42");
    }

    #[test]
    fn test_format_hex_dump_multiple() {
        let instructions = vec![
            Instruction {
                address: 0xC000,
                opcode: 0xEA,
                mnemonic: "NOP",
                addressing_mode: AddressingMode::Implicit,
                operand_bytes: vec![],
                size_bytes: 1,
                base_cycles: 2,
            },
            Instruction {
                address: 0xC001,
                opcode: 0xA9,
                mnemonic: "LDA",
                addressing_mode: AddressingMode::Immediate,
                operand_bytes: vec![0x42],
                size_bytes: 2,
                base_cycles: 2,
            },
            Instruction {
                address: 0xC003,
                opcode: 0x8D,
                mnemonic: "STA",
                addressing_mode: AddressingMode::Absolute,
                operand_bytes: vec![0x00, 0x80],
                size_bytes: 3,
                base_cycles: 4,
            },
        ];

        let output = format_hex_dump(&instructions);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "C000: EA         NOP");
        assert_eq!(lines[1], "C001: A9 42      LDA #$42");
        assert_eq!(lines[2], "C003: 8D 00 80   STA $8000");
    }

    #[test]
    fn test_format_hex_dump_alignment() {
        // Test that different instruction lengths are properly aligned
        let instructions = vec![
            Instruction {
                address: 0x0000,
                opcode: 0xEA,
                mnemonic: "NOP",
                addressing_mode: AddressingMode::Implicit,
                operand_bytes: vec![],
                size_bytes: 1,
                base_cycles: 2,
            },
            Instruction {
                address: 0x0001,
                opcode: 0x8D,
                mnemonic: "STA",
                addressing_mode: AddressingMode::Absolute,
                operand_bytes: vec![0x00, 0x80],
                size_bytes: 3,
                base_cycles: 4,
            },
        ];

        let output = format_hex_dump(&instructions);

        // Both lines should have proper column alignment
        let lines: Vec<&str> = output.lines().collect();
        // The assembly mnemonics should start at the same column position
        for line in lines {
            // Check format: "XXXX: YYYYYYYY  ZZZZZ"
            // where XXXX is address (4 chars), YYYYYYYY is bytes (9 chars), ZZZZZ is assembly
            assert!(line.contains(": "));
            assert!(line.len() > 16); // At least "XXXX: YYY       Z"
        }
    }
}
