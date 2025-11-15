//! Instruction decoder for the 6502 disassembler

use crate::disassembler::Instruction;
use crate::opcodes::OPCODE_TABLE;

/// Decode a single instruction from a byte slice
///
/// # Arguments
///
/// * `bytes` - The byte slice starting at the instruction to decode
/// * `address` - The memory address of this instruction
///
/// # Returns
///
/// Some(Instruction) if the opcode is valid, None for illegal opcodes
pub fn decode_instruction(bytes: &[u8], address: u16) -> Option<Instruction> {
    if bytes.is_empty() {
        return None;
    }

    let opcode = bytes[0];
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Check if this is a valid opcode (illegal opcodes have "???" as mnemonic)
    if metadata.mnemonic == "???" {
        return None;
    }

    // Check if there are enough bytes for the full instruction
    if bytes.len() < metadata.size_bytes as usize {
        return None;
    }

    // Extract operand bytes based on instruction size
    let operand_bytes: Vec<u8> = if metadata.size_bytes > 1 {
        bytes
            .iter()
            .skip(1)
            .take((metadata.size_bytes - 1) as usize)
            .copied()
            .collect()
    } else {
        Vec::new()
    };

    Some(Instruction {
        address,
        opcode,
        mnemonic: metadata.mnemonic,
        addressing_mode: metadata.addressing_mode,
        operand_bytes,
        size_bytes: metadata.size_bytes,
        base_cycles: metadata.base_cycles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addressing::AddressingMode;

    #[test]
    fn test_decode_lda_immediate() {
        let bytes = &[0xA9, 0x42];
        let instr = decode_instruction(bytes, 0x8000).unwrap();

        assert_eq!(instr.address, 0x8000);
        assert_eq!(instr.opcode, 0xA9);
        assert_eq!(instr.mnemonic, "LDA");
        assert_eq!(instr.addressing_mode, AddressingMode::Immediate);
        assert_eq!(instr.operand_bytes, vec![0x42]);
        assert_eq!(instr.size_bytes, 2);
    }

    #[test]
    fn test_decode_sta_absolute() {
        let bytes = &[0x8D, 0x00, 0x80];
        let instr = decode_instruction(bytes, 0x0000).unwrap();

        assert_eq!(instr.opcode, 0x8D);
        assert_eq!(instr.mnemonic, "STA");
        assert_eq!(instr.addressing_mode, AddressingMode::Absolute);
        assert_eq!(instr.operand_bytes, vec![0x00, 0x80]);
        assert_eq!(instr.size_bytes, 3);
    }

    #[test]
    fn test_decode_nop() {
        let bytes = &[0xEA];
        let instr = decode_instruction(bytes, 0x1000).unwrap();

        assert_eq!(instr.address, 0x1000);
        assert_eq!(instr.opcode, 0xEA);
        assert_eq!(instr.mnemonic, "NOP");
        assert_eq!(instr.addressing_mode, AddressingMode::Implicit);
        assert_eq!(instr.operand_bytes.len(), 0);
        assert_eq!(instr.size_bytes, 1);
    }
}
