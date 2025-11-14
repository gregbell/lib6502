//! # Control Flow Instructions
//!
//! This module implements control flow operations:
//! - BRK: Force Interrupt
//! - (Future: JMP, JSR, RTS, RTI, NOP)
//!
//! BRK is a software interrupt that:
//! 1. Pushes PC+2 to the stack (high byte first, then low byte)
//! 2. Pushes processor status to stack with B flag set
//! 3. Sets the I (interrupt disable) flag
//! 4. Loads PC from IRQ vector at $FFFE/F

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the BRK (Force Interrupt) instruction.
///
/// BRK forces a software interrupt by:
/// 1. Incrementing PC by 2 (BRK is 1 byte, but PC+2 is pushed for compatibility)
/// 2. Pushing the high byte of PC to the stack
/// 3. Pushing the low byte of PC to the stack
/// 4. Pushing the processor status register to the stack (with B flag set to 1)
/// 5. Setting the I (interrupt disable) flag
/// 6. Loading the PC from the IRQ vector at addresses $FFFE (low) and $FFFF (high)
///
/// Cycle timing: 7 cycles (fixed)
///
/// Flags affected:
/// - B: Set to 1 (in the pushed status byte, not in the actual flag)
/// - I: Set to 1
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this BRK instruction (0x00)
pub(crate) fn execute_brk<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // BRK pushes PC+2 (even though BRK is only 1 byte)
    // This is a quirk of the 6502 hardware
    let return_address = cpu.pc.wrapping_add(2);

    // Push high byte of return address to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, (return_address >> 8) as u8);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Push low byte of return address to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, (return_address & 0xFF) as u8);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Build status byte with B flag set (bit 4) and bit 5 always set
    // Note: This doesn't actually set cpu.flag_b, it only sets the B flag
    // in the pushed status byte
    let mut status: u8 = 0b00110000; // Bits 5 and 4 set

    if cpu.flag_n {
        status |= 0b10000000;
    }
    if cpu.flag_v {
        status |= 0b01000000;
    }
    if cpu.flag_d {
        status |= 0b00001000;
    }
    if cpu.flag_i {
        status |= 0b00000100;
    }
    if cpu.flag_z {
        status |= 0b00000010;
    }
    if cpu.flag_c {
        status |= 0b00000001;
    }

    // Push status byte to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, status);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Set the interrupt disable flag
    cpu.flag_i = true;

    // Load PC from IRQ vector at $FFFE/F (little-endian)
    let pc_low = cpu.memory.read(0xFFFE) as u16;
    let pc_high = cpu.memory.read(0xFFFF) as u16;
    cpu.pc = (pc_high << 8) | pc_low;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
