//! 6502 Disassembler Module
//!
//! Converts binary machine code into human-readable assembly mnemonics.

pub mod decoder;
pub mod formatter;

use crate::addressing::AddressingMode;

/// A single disassembled instruction with full metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// Memory address where this instruction starts
    pub address: u16,

    /// The opcode byte value (0x00-0xFF)
    pub opcode: u8,

    /// Instruction mnemonic (e.g., "LDA", "STA", "JMP")
    pub mnemonic: &'static str,

    /// Addressing mode used by this instruction
    pub addressing_mode: AddressingMode,

    /// Operand bytes (0-2 bytes depending on addressing mode)
    pub operand_bytes: Vec<u8>,

    /// Total size in bytes (1-3 bytes: opcode + operands)
    pub size_bytes: u8,

    /// Base cycle cost (excluding page-crossing penalties)
    pub base_cycles: u8,
}

/// Options controlling disassembly output
#[derive(Debug, Clone, Copy)]
pub struct DisassemblyOptions {
    /// Starting address for disassembly (affects address display)
    pub start_address: u16,

    /// Whether to format output as hex dump
    pub hex_dump: bool,

    /// Whether to include byte offsets in output
    pub show_offsets: bool,
}

impl Default for DisassemblyOptions {
    fn default() -> Self {
        Self {
            start_address: 0x0000,
            hex_dump: false,
            show_offsets: false,
        }
    }
}

/// Disassemble a byte slice into a vector of instructions
///
/// # Arguments
///
/// * `bytes` - The machine code to disassemble
/// * `options` - Disassembly options controlling output format
///
/// # Returns
///
/// A vector of `Instruction` structs, one for each decoded instruction
pub fn disassemble(bytes: &[u8], options: DisassemblyOptions) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut pc = 0;
    let mut address = options.start_address;

    while pc < bytes.len() {
        match decoder::decode_instruction(&bytes[pc..], address) {
            Some(instr) => {
                pc += instr.size_bytes as usize;
                address = address.wrapping_add(instr.size_bytes as u16);
                instructions.push(instr);
            }
            None => {
                // Invalid opcode - create a .byte directive
                instructions.push(Instruction {
                    address,
                    opcode: bytes[pc],
                    mnemonic: ".byte",
                    addressing_mode: AddressingMode::Implicit,
                    operand_bytes: vec![bytes[pc]],
                    size_bytes: 1,
                    base_cycles: 0,
                });
                pc += 1;
                address = address.wrapping_add(1);
            }
        }
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_empty() {
        let bytes = &[];
        let instructions = disassemble(bytes, DisassemblyOptions::default());
        assert_eq!(instructions.len(), 0);
    }
}
