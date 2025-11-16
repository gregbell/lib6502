//! Comprehensive tests for the SEC (Set Carry Flag) instruction.
//!
//! Tests cover:
//! - Basic SEC operation sets carry flag
//! - Carry flag set when already set
//! - Carry flag set when clear
//! - All addressing modes (Implied only for SEC)
//! - Processor flags updated correctly (C set to 1, others unchanged)
//! - Correct cycle counts
//! - PC advancement

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic SEC Operation Tests ==========

#[test]
fn test_sec_sets_carry_flag_when_clear() {
    let mut cpu = setup_cpu();

    // SEC (0x38)
    cpu.memory_mut().write(0x8000, 0x38);

    // Clear carry flag
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Carry flag should be set
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 2); // 2 cycles
}

#[test]
fn test_sec_sets_carry_flag_when_already_set() {
    let mut cpu = setup_cpu();

    // SEC (0x38)
    cpu.memory_mut().write(0x8000, 0x38);

    // Ensure carry flag is already set
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    // Carry flag should remain set
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_sec_preserves_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set zero flag
    cpu.set_flag_z(true);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Zero flag should be unchanged
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set negative flag
    cpu.set_flag_n(true);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Negative flag should be unchanged
    assert!(cpu.flag_n());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set overflow flag
    cpu.set_flag_v(true);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Overflow flag should be unchanged
    assert!(cpu.flag_v());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Clear interrupt flag (it's set by default on reset)
    cpu.set_flag_i(false);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Interrupt disable flag should be unchanged
    assert!(!cpu.flag_i());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set decimal flag
    cpu.set_flag_d(true);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Decimal flag should be unchanged
    assert!(cpu.flag_d());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set break flag
    cpu.set_flag_b(true);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Break flag should be unchanged
    assert!(cpu.flag_b());
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_all_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set all flags to known values
    cpu.set_flag_c(false);
    cpu.set_flag_z(true);
    cpu.set_flag_i(false);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // Only carry flag should be set, all others unchanged
    assert!(cpu.flag_c()); // Set
    assert!(cpu.flag_z()); // Unchanged
    assert!(!cpu.flag_i()); // Unchanged
    assert!(cpu.flag_d()); // Unchanged
    assert!(cpu.flag_b()); // Unchanged
    assert!(cpu.flag_v()); // Unchanged
    assert!(cpu.flag_n()); // Unchanged
}

// ========== Register Preservation Tests ==========

#[test]
fn test_sec_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set accumulator to a known value
    cpu.set_a(0x42);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Accumulator should be unchanged
    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set X register to a known value
    cpu.set_x(0x33);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // X register should be unchanged
    assert_eq!(cpu.x(), 0x33);
    assert!(cpu.flag_c());
}

#[test]
fn test_sec_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);

    // Set Y register to a known value
    cpu.set_y(0x44);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Y register should be unchanged
    assert_eq!(cpu.y(), 0x44);
    assert!(cpu.flag_c());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_sec_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);
    cpu.set_flag_c(false);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // SEC should take exactly 2 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

#[test]
fn test_sec_cycle_count_when_already_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);
    cpu.set_flag_c(true);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // SEC should take exactly 2 cycles even when carry is already set
    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

// ========== PC Advancement Tests ==========

#[test]
fn test_sec_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x38);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of SEC instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_sec_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0x38); // SEC at page boundary

    let mut cpu = CPU::new(memory);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
    assert!(cpu.flag_c());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_sec_implied_addressing_mode() {
    let mut cpu = setup_cpu();

    // SEC uses implied addressing mode (opcode 0x38)
    cpu.memory_mut().write(0x8000, 0x38);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_sec_multiple_executions() {
    let mut cpu = setup_cpu();

    // Write multiple SEC instructions
    cpu.memory_mut().write(0x8000, 0x38);
    cpu.memory_mut().write(0x8001, 0x38);
    cpu.memory_mut().write(0x8002, 0x38);

    cpu.set_flag_c(false);

    // First execution
    cpu.step().unwrap();
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);

    // Second execution (carry already set)
    cpu.step().unwrap();
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);

    // Third execution
    cpu.step().unwrap();
    assert!(cpu.flag_c());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}
