//! Comprehensive tests for the TXS (Transfer X to Stack Pointer) instruction.
//!
//! Tests cover:
//! - Basic TXS operation
//! - Flag behavior (NO flags affected - this is unique to TXS)
//! - Various X register values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Register preservation

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic TXS Operation Tests ==========

#[test]
fn test_txs_basic() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42); // SP = X
    assert_eq!(cpu.x(), 0x42); // X unchanged
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_txs_transfers_x_to_stack_pointer() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x55);
    cpu.set_sp(0xFF); // SP has different value initially

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x55); // SP now equals X
    assert_eq!(cpu.x(), 0x55); // X unchanged
}

#[test]
fn test_txs_with_zero() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x00);
    cpu.set_sp(0xFD); // Default SP value

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x00); // SP = 0
    assert_eq!(cpu.x(), 0x00); // X unchanged
}

// ========== Flag Tests - CRITICAL: TXS Does NOT Affect Any Flags ==========

#[test]
fn test_txs_does_not_affect_zero_flag_when_result_is_zero() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x00); // Transfer zero
    cpu.set_flag_z(false); // Z flag is clear initially

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x00);
    assert!(!cpu.flag_z()); // Z flag should remain clear (not affected by TXS)
}

#[test]
fn test_txs_does_not_clear_zero_flag_when_result_is_non_zero() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42); // Non-zero value
    cpu.set_flag_z(true); // Z flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert!(cpu.flag_z()); // Z flag should remain set (not affected by TXS)
}

#[test]
fn test_txs_does_not_affect_negative_flag_when_result_is_negative() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x80); // 0b10000000 - bit 7 set
    cpu.set_flag_n(false); // N flag is clear initially

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x80);
    assert!(!cpu.flag_n()); // N flag should remain clear (not affected by TXS)
}

#[test]
fn test_txs_does_not_clear_negative_flag_when_result_is_positive() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x7F); // 0b01111111 - bit 7 clear
    cpu.set_flag_n(true); // N flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x7F);
    assert!(cpu.flag_n()); // N flag should remain set (not affected by TXS)
}

// ========== Edge Case Tests ==========

#[test]
fn test_txs_preserves_x_register() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x99); // X register should be unchanged
    assert_eq!(cpu.sp(), 0x99); // SP should equal original X
}

#[test]
fn test_txs_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_a(0x88); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert_eq!(cpu.a(), 0x88); // Accumulator should be unchanged
}

#[test]
fn test_txs_preserves_y_register() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_y(0x88); // Set Y register

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert_eq!(cpu.y(), 0x88); // Y register should be unchanged
}

#[test]
fn test_txs_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_txs_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_txs_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

#[test]
fn test_txs_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

// ========== Multiple Transfer Tests ==========

#[test]
fn test_txs_sequence() {
    let mut cpu = setup_cpu();

    // Set up two TXS instructions
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.memory_mut().write(0x8001, 0x9A);

    cpu.set_x(0x10);

    cpu.step().unwrap();
    assert_eq!(cpu.sp(), 0x10);
    assert_eq!(cpu.pc(), 0x8001);

    // Change X and execute another TXS
    cpu.set_x(0x20);
    cpu.step().unwrap();
    assert_eq!(cpu.sp(), 0x20); // SP updated with new X value
    assert_eq!(cpu.pc(), 0x8002);
}

#[test]
fn test_txs_overwrites_previous_sp() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0xAB);
    cpu.set_sp(0xCD); // SP has a different value

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0xAB); // SP is overwritten with X
    assert_eq!(cpu.x(), 0xAB);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_txs_cycle_count() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // TXS takes 2 cycles
}

#[test]
fn test_txs_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 TXS instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x9A);
    }

    cpu.set_x(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each TXS takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_txs_program_counter_advance() {
    let mut cpu = setup_cpu();

    // TXS (0x9A) - single byte instruction
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}

// ========== Boundary Value Tests ==========

#[test]
fn test_txs_with_0x00() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x00);
    // Set flags to verify they aren't affected
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x00);
    assert!(!cpu.flag_z()); // TXS does not affect Z flag
    assert!(!cpu.flag_n()); // TXS does not affect N flag
}

#[test]
fn test_txs_with_0x7f() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x7F); // Maximum positive signed value
    cpu.set_flag_z(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x7F);
    assert!(cpu.flag_z()); // TXS does not affect Z flag
    assert!(cpu.flag_n()); // TXS does not affect N flag
}

#[test]
fn test_txs_with_0x80() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x80); // Minimum negative signed value
    cpu.set_flag_z(true);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x80);
    assert!(cpu.flag_z()); // TXS does not affect Z flag
    assert!(!cpu.flag_n()); // TXS does not affect N flag
}

#[test]
fn test_txs_with_0xff() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0xFF);
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0xFF);
    assert!(!cpu.flag_z()); // TXS does not affect Z flag
    assert!(!cpu.flag_n()); // TXS does not affect N flag
}

// ========== Stack Operation Tests ==========

#[test]
fn test_txs_sets_stack_for_push_operations() {
    let mut cpu = setup_cpu();

    // TXS (0x9A) followed by PHA (0x48)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.memory_mut().write(0x8001, 0x48);

    cpu.set_x(0xFF); // Set SP to top of stack
    cpu.set_a(0x42);

    cpu.step().unwrap(); // TXS
    assert_eq!(cpu.sp(), 0xFF);

    cpu.step().unwrap(); // PHA
    assert_eq!(cpu.sp(), 0xFE); // SP decremented after push
    assert_eq!(cpu.memory_mut().read(0x01FF), 0x42); // Value pushed to stack
}

#[test]
fn test_txs_initializes_empty_stack() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0xFF); // Initialize SP to empty stack

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0xFF); // SP at top of stack (empty)
}

// ========== Comparison with TSX Tests ==========

#[test]
fn test_txs_is_opposite_of_tsx() {
    let mut cpu = setup_cpu();

    // TXS (0x9A) followed by TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.memory_mut().write(0x8001, 0xBA);

    cpu.set_x(0x55);
    cpu.set_sp(0xAA);

    cpu.step().unwrap(); // TXS: SP = X = 0x55
    assert_eq!(cpu.sp(), 0x55);
    assert_eq!(cpu.x(), 0x55);

    cpu.set_x(0x00); // Change X to verify TSX works

    cpu.step().unwrap(); // TSX: X = SP = 0x55
    assert_eq!(cpu.x(), 0x55);
    assert_eq!(cpu.sp(), 0x55);
}

#[test]
fn test_txs_does_not_set_flags_unlike_tsx() {
    let mut cpu = setup_cpu();

    // TXS (0x9A)
    cpu.memory_mut().write(0x8000, 0x9A);
    cpu.set_x(0x00); // Zero value
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // TXS does NOT set flags, even with zero value
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());

    // Now test TSX with same value
    cpu.memory_mut().write(0x8001, 0xBA);
    cpu.set_sp(0x00);
    cpu.set_x(0xFF);

    cpu.step().unwrap(); // TSX: X = SP = 0

    // TSX DOES set flags
    assert!(cpu.flag_z()); // TSX sets Z flag for zero
    assert!(!cpu.flag_n());
}
