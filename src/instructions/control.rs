//! # Control Flow Instructions
//!
//! This module implements control flow operations:
//! - BRK: Force Interrupt
//! - JMP: Jump to address
//! - JSR: Jump to Subroutine
//! - NOP: No Operation
//! - RTI: Return from Interrupt
//! - RTS: Return from Subroutine
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

/// Executes the JSR (Jump to Subroutine) instruction.
///
/// JSR pushes the address (minus one) of the return point onto the stack and then
/// sets the program counter to the target memory address.
///
/// The return address pushed is PC+2, which is the address of the last byte of the
/// JSR instruction. When RTS is executed, it pulls this address and adds 1 to get
/// the address of the next instruction.
///
/// Addressing modes:
/// - Absolute (0x20): JSR $1234 - Jump to subroutine at address $1234
///
/// Cycle timing: 6 cycles (fixed)
///
/// Flags affected: None
///
/// Stack operations:
/// 1. Push high byte of return address (PC+2)
/// 2. Push low byte of return address (PC+2)
/// 3. Set PC to target address
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this JSR instruction (0x20)
pub(crate) fn execute_jsr<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Read the target address from operand (little-endian)
    let addr_lo = cpu.memory.read(cpu.pc.wrapping_add(1)) as u16;
    let addr_hi = cpu.memory.read(cpu.pc.wrapping_add(2)) as u16;
    let target_address = (addr_hi << 8) | addr_lo;

    // Calculate return address (PC + 2, which is the address of the last byte of JSR)
    let return_address = cpu.pc.wrapping_add(2);

    // Push high byte of return address to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, (return_address >> 8) as u8);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Push low byte of return address to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, (return_address & 0xFF) as u8);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Set PC to target address
    cpu.pc = target_address;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the NOP (No Operation) instruction.
///
/// NOP causes no changes to the processor other than the normal incrementing
/// of the program counter to the next instruction.
///
/// Addressing Mode: Implicit
/// Opcode: 0xEA
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected: None
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this NOP instruction (0xEA)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0xEA); // NOP
///
/// let mut cpu = CPU::new(memory);
///
/// cpu.step().unwrap();
///
/// // NOP does nothing except advance PC and consume cycles
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_nop<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // NOP does nothing - just advance PC and add cycles
    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for NOP)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the RTI (Return from Interrupt) instruction.
///
/// RTI returns from an interrupt handler by:
/// 1. Pulling the processor status flags from the stack
/// 2. Pulling the program counter from the stack
///
/// This is the complementary instruction to BRK and hardware interrupts (IRQ/NMI).
/// It restores the CPU state that was saved when the interrupt was triggered.
///
/// Stack operations (in order):
/// 1. Increment SP (pull operation)
/// 2. Pull status byte from stack and restore all flags
/// 3. Increment SP (pull operation)
/// 4. Pull low byte of PC from stack
/// 5. Increment SP (pull operation)
/// 6. Pull high byte of PC from stack
/// 7. Set PC from pulled values
///
/// Addressing Mode: Implicit (opcode 0x40)
/// Bytes: 1
/// Cycles: 6
///
/// Flags affected:
/// - C: Set from bit 0 of pulled status
/// - Z: Set from bit 1 of pulled status
/// - I: Set from bit 2 of pulled status
/// - D: Set from bit 3 of pulled status
/// - B: Set from bit 4 of pulled status
/// - V: Set from bit 6 of pulled status
/// - N: Set from bit 7 of pulled status
///
/// (Bit 5 is always ignored)
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this RTI instruction (0x40)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x40); // RTI
///
/// let mut cpu = CPU::new(memory);
///
/// // Setup: Simulate an interrupt by pushing status and PC onto stack
/// // BRK pushes: PC_high at SP, PC_low at SP-1, status at SP-2
/// // So with final SP=0xFA: PC_high at 0x01FD, PC_low at 0x01FC, status at 0x01FB
/// cpu.memory_mut().write(0x01FD, 0x12); // PC high byte
/// cpu.memory_mut().write(0x01FC, 0x34); // PC low byte -> PC = 0x1234
/// cpu.memory_mut().write(0x01FB, 0b00100011); // Status (with C and Z flags set)
/// cpu.set_sp(0xFA); // SP points below the pushed values
///
/// cpu.step().unwrap();
///
/// // CPU state should be restored from stack
/// assert_eq!(cpu.pc(), 0x1234); // PC restored
/// assert!(cpu.flag_c()); // Carry flag restored
/// assert!(cpu.flag_z()); // Zero flag restored
/// assert_eq!(cpu.sp(), 0xFD); // SP incremented 3 times
/// assert_eq!(cpu.cycles(), 6);
/// ```
pub(crate) fn execute_rti<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Pull status byte from stack
    cpu.sp = cpu.sp.wrapping_add(1);
    let stack_addr = 0x0100 | (cpu.sp as u16);
    let status = cpu.memory.read(stack_addr);

    // Restore all flags from the status byte
    // Bit 7: N (Negative)
    cpu.flag_n = (status & 0b10000000) != 0;
    // Bit 6: V (Overflow)
    cpu.flag_v = (status & 0b01000000) != 0;
    // Bit 5: Always ignored (unused bit)
    // Bit 4: B (Break)
    cpu.flag_b = (status & 0b00010000) != 0;
    // Bit 3: D (Decimal mode)
    cpu.flag_d = (status & 0b00001000) != 0;
    // Bit 2: I (Interrupt disable)
    cpu.flag_i = (status & 0b00000100) != 0;
    // Bit 1: Z (Zero)
    cpu.flag_z = (status & 0b00000010) != 0;
    // Bit 0: C (Carry)
    cpu.flag_c = (status & 0b00000001) != 0;

    // Pull low byte of PC from stack
    cpu.sp = cpu.sp.wrapping_add(1);
    let stack_addr = 0x0100 | (cpu.sp as u16);
    let pc_low = cpu.memory.read(stack_addr) as u16;

    // Pull high byte of PC from stack
    cpu.sp = cpu.sp.wrapping_add(1);
    let stack_addr = 0x0100 | (cpu.sp as u16);
    let pc_high = cpu.memory.read(stack_addr) as u16;

    // Restore PC from pulled values
    cpu.pc = (pc_high << 8) | pc_low;

    // Update cycle count (6 cycles for RTI)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the RTS (Return from Subroutine) instruction.
///
/// RTS returns from a subroutine by:
/// 1. Pulling the program counter (minus one) from the stack
/// 2. Incrementing the pulled address by 1 to get the next instruction
///
/// This is the complementary instruction to JSR (Jump to Subroutine).
/// JSR pushes PC+2 (the address of the last byte of the JSR instruction) to the stack,
/// so RTS needs to pull this value and add 1 to get the address of the next instruction.
///
/// Stack operations (in order):
/// 1. Increment SP (pull operation)
/// 2. Pull low byte of PC from stack
/// 3. Increment SP (pull operation)
/// 4. Pull high byte of PC from stack
/// 5. Set PC from pulled values
/// 6. Increment PC by 1
///
/// Addressing Mode: Implicit (opcode 0x60)
/// Bytes: 1
/// Cycles: 6
///
/// Flags affected: None
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this RTS instruction (0x60)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x60); // RTS
///
/// let mut cpu = CPU::new(memory);
///
/// // Setup: Simulate JSR by pushing return address onto stack
/// // JSR pushes PC+2, so if we want to return to 0x8003, we push 0x8002
/// cpu.memory_mut().write(0x01FD, 0x80); // PC high byte
/// cpu.memory_mut().write(0x01FC, 0x02); // PC low byte -> 0x8002
/// cpu.set_sp(0xFB); // SP points below the pushed values
///
/// cpu.step().unwrap();
///
/// // CPU state should be restored from stack
/// assert_eq!(cpu.pc(), 0x8003); // 0x8002 + 1
/// assert_eq!(cpu.sp(), 0xFD); // SP incremented 2 times
/// assert_eq!(cpu.cycles(), 6);
/// ```
pub(crate) fn execute_rts<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Pull low byte of PC from stack
    cpu.sp = cpu.sp.wrapping_add(1);
    let stack_addr = 0x0100 | (cpu.sp as u16);
    let pc_low = cpu.memory.read(stack_addr) as u16;

    // Pull high byte of PC from stack
    cpu.sp = cpu.sp.wrapping_add(1);
    let stack_addr = 0x0100 | (cpu.sp as u16);
    let pc_high = cpu.memory.read(stack_addr) as u16;

    // Restore PC from pulled values and add 1
    // JSR pushes PC+2 (address of last byte of JSR), so we need to add 1
    // to get the address of the next instruction
    cpu.pc = ((pc_high << 8) | pc_low).wrapping_add(1);

    // Update cycle count (6 cycles for RTS)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
