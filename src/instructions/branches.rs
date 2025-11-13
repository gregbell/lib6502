//! # Branch Instructions
//!
//! This module implements conditional branch operations:
//! - BCC: Branch if Carry Clear
//! - (Future: BCS, BEQ, BNE, BMI, BPL, BVC, BVS)
//!
//! All branch instructions use relative addressing with a signed 8-bit offset.
//! Cycle timing varies based on whether the branch is taken and whether a page boundary is crossed.

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the BCC (Branch if Carry Clear) instruction.
///
/// Branches to a new location if the carry flag is clear (C = 0).
/// Uses relative addressing mode with a signed 8-bit offset.
///
/// Cycle timing:
/// - 2 cycles if branch not taken
/// - 3 cycles if branch taken to same page
/// - 4 cycles if branch taken to different page
///
/// No flags are affected.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this BCC instruction (0x90)
pub(crate) fn execute_bcc<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Read the signed 8-bit offset from PC+1
    let offset = cpu.memory.read(cpu.pc.wrapping_add(1)) as i8;

    // Start with base cycles
    let mut cycles = metadata.base_cycles as u64;

    // Calculate the address after the instruction (PC + 2)
    let pc_after_instruction = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Check if carry flag is clear
    if !cpu.flag_c {
        // Branch is taken
        // Calculate the target address by adding the signed offset
        // Use wrapping_add_signed to handle both positive and negative offsets correctly
        let target_pc = pc_after_instruction.wrapping_add_signed(offset as i16);

        // Check if page boundary was crossed
        // A page boundary is crossed if the high byte of the address changes
        let page_crossed = (pc_after_instruction & 0xFF00) != (target_pc & 0xFF00);

        // Add 1 cycle for branch taken
        cycles += 1;

        // Add 1 more cycle if page boundary was crossed
        if page_crossed {
            cycles += 1;
        }

        // Update PC to target address
        cpu.pc = target_pc;
    } else {
        // Branch not taken, just advance PC normally
        cpu.pc = pc_after_instruction;
    }

    // Update cycle count
    cpu.cycles += cycles;

    Ok(())
}
