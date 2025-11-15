//! # Control Flow Instructions
//!
//! This module implements control flow operations:
//! - BRK: Force Interrupt
//! - JMP: Jump to address
//! - (Future: JSR, RTS, RTI, NOP)
//!
//! BRK is a software interrupt that:
//! 1. Pushes PC+2 to the stack (high byte first, then low byte)
//! 2. Pushes processor status to stack with B flag set
//! 3. Sets the I (interrupt disable) flag
//! 4. Loads PC from IRQ vector at $FFFE/F

use crate::{AddressingMode, ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

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
    let status = cpu.status() | 0b00110000; // Set B flag

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

/// Executes the JMP (Jump) instruction.
///
/// JMP sets the program counter to the address specified by the operand.
/// This is an unconditional jump that does not affect any flags or the stack.
///
/// Addressing modes:
/// - Absolute (0x4C): JMP $1234 - Jump to address $1234
/// - Indirect (0x6C): JMP ($1234) - Jump to address stored at $1234/$1235
///
/// Cycle timing:
/// - Absolute: 3 cycles
/// - Indirect: 5 cycles
///
/// Flags affected: None
///
/// Note: The Indirect addressing mode has a hardware bug in the original 6502:
/// If the low byte of the indirect address is 0xFF, the high byte is read from
/// the same page (wraps within page) instead of crossing to the next page.
/// For example, JMP ($10FF) reads from $10FF and $1000 (not $1100).
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this JMP instruction (0x4C or 0x6C)
pub(crate) fn execute_jmp<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    let target_address = match metadata.addressing_mode {
        AddressingMode::Absolute => {
            // Read 16-bit address from operand (little-endian)
            let addr_lo = cpu.memory.read(cpu.pc.wrapping_add(1)) as u16;
            let addr_hi = cpu.memory.read(cpu.pc.wrapping_add(2)) as u16;
            (addr_hi << 8) | addr_lo
        }
        AddressingMode::Indirect => {
            // Read 16-bit pointer address from operand
            let ptr_lo = cpu.memory.read(cpu.pc.wrapping_add(1)) as u16;
            let ptr_hi = cpu.memory.read(cpu.pc.wrapping_add(2)) as u16;
            let ptr = (ptr_hi << 8) | ptr_lo;

            // Read the target address from the pointer location
            // Note: 6502 hardware bug - if low byte is 0xFF, high byte wraps within same page
            let target_lo = cpu.memory.read(ptr) as u16;
            let target_hi_addr = if (ptr & 0xFF) == 0xFF {
                // Bug: wrap within same page instead of crossing page boundary
                ptr & 0xFF00
            } else {
                ptr.wrapping_add(1)
            };
            let target_hi = cpu.memory.read(target_hi_addr) as u16;

            (target_hi << 8) | target_lo
        }
        _ => {
            panic!("Invalid addressing mode for JMP");
        }
    };

    // Set PC to target address
    cpu.pc = target_address;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
