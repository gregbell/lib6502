//! Comprehensive tests for the BRK (Force Interrupt) instruction.
//!
//! Tests cover:
//! - Basic BRK operation
//! - Stack operations (PC and status pushed correctly)
//! - IRQ vector loading
//! - I flag set after BRK
//! - B flag set in pushed status (but not in CPU flag)
//! - Correct cycle count (7 cycles)
//! - PC+2 pushed to stack (not PC+1)

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic BRK Operation Tests ==========

#[test]
fn test_brk_basic_operation() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector to point to 0x9000
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // PC should now point to IRQ handler at 0x9000
    assert_eq!(cpu.pc(), 0x9000);

    // Stack pointer should have been decremented by 3 (PC high, PC low, status)
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(3));

    // Interrupt disable flag should be set
    assert!(cpu.flag_i());
}

#[test]
fn test_brk_pushes_pc_plus_2() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read the return address from stack
    // Stack grows downward: first push is at SP, then SP-1, then SP-2
    // Order: PC_high at SP, PC_low at SP-1, status at SP-2
    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    let return_address = ((pc_high as u16) << 8) | (pc_low as u16);

    // BRK should push PC+2, not PC+1 (this is 0x8000 + 2 = 0x8002)
    assert_eq!(return_address, 0x8002);
}

#[test]
fn test_brk_loads_irq_vector() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector to point to 0xABCD
    cpu.memory_mut().write(0xFFFE, 0xCD); // Low byte
    cpu.memory_mut().write(0xFFFF, 0xAB); // High byte

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    cpu.step().unwrap();

    // PC should now point to IRQ handler at 0xABCD
    assert_eq!(cpu.pc(), 0xABCD);
}

// ========== Status Register Tests ==========

#[test]
fn test_brk_pushes_status_with_b_flag_set() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Set some flags before BRK
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read the status byte from stack (at SP-2)
    let status = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(2) as u16));

    // Check that B flag (bit 4) is set in the pushed status
    assert_eq!(
        status & 0b00010000,
        0b00010000,
        "B flag should be set in pushed status"
    );

    // Bit 5 should always be set
    assert_eq!(
        status & 0b00100000,
        0b00100000,
        "Bit 5 should always be set"
    );

    // Check that other flags were preserved
    assert_eq!(status & 0b10000000, 0b10000000, "N flag should be set");
    assert_eq!(status & 0b01000000, 0b01000000, "V flag should be set");
    assert_eq!(status & 0b00000010, 0b00000010, "Z flag should be set");
    assert_eq!(status & 0b00000001, 0b00000001, "C flag should be set");
}

#[test]
fn test_brk_sets_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Clear interrupt disable flag before BRK
    cpu.set_flag_i(false);

    cpu.step().unwrap();

    // I flag should now be set
    assert!(cpu.flag_i());
}

#[test]
fn test_brk_preserves_other_flags() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Set various flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    // All flags except I should remain unchanged
    assert!(cpu.flag_c(), "Carry flag should be preserved");
    assert!(cpu.flag_z(), "Zero flag should be preserved");
    assert!(cpu.flag_v(), "Overflow flag should be preserved");
    assert!(cpu.flag_n(), "Negative flag should be preserved");
    assert!(cpu.flag_d(), "Decimal flag should be preserved");
}

#[test]
fn test_brk_does_not_set_cpu_b_flag() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Ensure B flag is not set initially
    cpu.set_flag_b(false);

    cpu.step().unwrap();

    // B flag in CPU should still be false (only set in pushed status)
    assert!(!cpu.flag_b(), "CPU B flag should not be set by BRK");
}

// ========== Stack Operation Tests ==========

#[test]
fn test_brk_stack_push_order() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Stack grows downward, so:
    // SP+0: PC high byte
    // SP-1: PC low byte
    // SP-2: Status register

    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    let status = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(2) as u16));

    // Verify PC high byte
    assert_eq!(pc_high, 0x80, "High byte of PC+2 should be 0x80");

    // Verify PC low byte
    assert_eq!(pc_low, 0x02, "Low byte of PC+2 should be 0x02");

    // Verify status has B flag set
    assert_eq!(
        status & 0b00010000,
        0b00010000,
        "Status should have B flag set"
    );
}

#[test]
fn test_brk_stack_pointer_update() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // SP should have decremented by 3 (one for each push)
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(3));
}

#[test]
fn test_brk_stack_wrapping() {
    // Start with SP at 0x02, BRK will wrap SP to 0xFF
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    let mut cpu = CPU::new(memory);

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Manually set SP to 0x02
    cpu.set_sp(0x02);

    cpu.step().unwrap();

    // SP should wrap: 0x02 - 3 = 0xFF (wrapping subtraction)
    assert_eq!(cpu.sp(), 0xFF);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_brk_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    let initial_cycles = cpu.cycles();

    cpu.step().unwrap();

    // BRK should take exactly 7 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 7);
}

// ========== Edge Cases ==========

#[test]
fn test_brk_with_all_flags_clear() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Clear all flags
    cpu.set_flag_n(false);
    cpu.set_flag_v(false);
    cpu.set_flag_b(false);
    cpu.set_flag_d(false);
    cpu.set_flag_i(false);
    cpu.set_flag_z(false);
    cpu.set_flag_c(false);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read status from stack
    let status = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(2) as u16));

    // Should have bits 5 and 4 set (B flag and bit 5), all others clear
    assert_eq!(status, 0b00110000);
}

#[test]
fn test_brk_with_all_flags_set() {
    let mut cpu = setup_cpu();

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00);

    // Set all flags
    cpu.set_flag_n(true);
    cpu.set_flag_v(true);
    cpu.set_flag_b(true); // This won't affect the pushed status
    cpu.set_flag_d(true);
    cpu.set_flag_i(true);
    cpu.set_flag_z(true);
    cpu.set_flag_c(true);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read status from stack
    let status = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(2) as u16));

    // Should have all bits set
    assert_eq!(status, 0b11111111);
}

#[test]
fn test_brk_at_end_of_memory() {
    // BRK at 0xFFFF
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0xFF);

    let mut cpu = CPU::new(memory);

    // Set up IRQ vector
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x90);

    // Note: We can't actually put BRK at 0xFFFF because that's the IRQ vector high byte
    // Instead, test at 0xFFFE
    cpu.set_pc(0xFFFE);
    cpu.memory_mut().write(0xFFFE, 0x00); // Temporarily overwrite IRQ vector

    // This will read 0x90 as the IRQ high byte, which we'll restore
    cpu.memory_mut().write(0xFFFE, 0x00);
    cpu.memory_mut().write(0xFFFF, 0x80);

    cpu.step().unwrap();

    // PC should wrap: 0xFFFE + 2 = 0x0000, then load from IRQ vector
    assert_eq!(cpu.pc(), 0x8000);
}
