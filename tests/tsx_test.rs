//! Comprehensive tests for the TSX (Transfer Stack Pointer to X) instruction.
//!
//! Tests cover:
//! - Basic TSX operation
//! - Flag updates (Z, N)
//! - Various stack pointer values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Register preservation

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic TSX Operation Tests ==========

#[test]
fn test_tsx_basic() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42); // X = SP
    assert_eq!(cpu.sp(), 0x42); // SP unchanged
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_tsx_transfers_stack_pointer_to_x() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x55);
    cpu.set_x(0xFF); // X has different value initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55); // X now equals SP
    assert_eq!(cpu.sp(), 0x55); // SP unchanged
}

#[test]
fn test_tsx_with_default_stack_pointer() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    // SP is 0xFD by default (set during CPU initialization)

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFD); // X = SP (default value)
    assert_eq!(cpu.sp(), 0xFD); // SP unchanged
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0xFD has bit 7 set
}

// ========== Zero Flag Tests ==========

#[test]
fn test_tsx_zero_flag_set() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x00); // Transfer zero

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert_eq!(cpu.sp(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_tsx_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x01); // Non-zero value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_tsx_negative_flag_set() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x80); // 0b10000000 - bit 7 set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80);
    assert_eq!(cpu.sp(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_tsx_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x7F); // 0b01111111 - bit 7 clear

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is 0
}

#[test]
fn test_tsx_negative_flag_with_0xff() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0xFF); // All bits set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Edge Case Tests ==========

#[test]
fn test_tsx_preserves_stack_pointer() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0x99); // Stack pointer should be unchanged
    assert_eq!(cpu.x(), 0x99); // X should equal original SP
}

#[test]
fn test_tsx_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_a(0x88); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert_eq!(cpu.a(), 0x88); // Accumulator should be unchanged
}

#[test]
fn test_tsx_preserves_y_register() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_y(0x88); // Set Y register

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert_eq!(cpu.y(), 0x88); // Y register should be unchanged
}

#[test]
fn test_tsx_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_tsx_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_tsx_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

#[test]
fn test_tsx_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

// ========== Multiple Transfer Tests ==========

#[test]
fn test_tsx_sequence() {
    let mut cpu = setup_cpu();

    // Set up two TSX instructions
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.memory_mut().write(0x8001, 0xBA);

    cpu.set_sp(0x10);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x10);
    assert_eq!(cpu.pc(), 0x8001);

    // Change SP and execute another TSX
    cpu.set_sp(0x20);
    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x20); // X updated with new SP value
    assert_eq!(cpu.pc(), 0x8002);
}

#[test]
fn test_tsx_overwrites_previous_x() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0xAB);
    cpu.set_x(0xCD); // X has a different value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xAB); // X is overwritten with SP
    assert_eq!(cpu.sp(), 0xAB);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_tsx_cycle_count() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // TSX takes 2 cycles
}

#[test]
fn test_tsx_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 TSX instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0xBA);
    }

    cpu.set_sp(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each TSX takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_tsx_program_counter_advance() {
    let mut cpu = setup_cpu();

    // TSX (0xBA) - single byte instruction
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}

// ========== Boundary Value Tests ==========

#[test]
fn test_tsx_with_0x00() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tsx_with_0x7f() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x7F); // Maximum positive signed value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tsx_with_0x80() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x80); // Minimum negative signed value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_tsx_with_0xff() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Flag Combination Tests ==========

#[test]
fn test_tsx_clears_previous_z_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_z(true); // Z flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_z()); // Z flag should be cleared
}

#[test]
fn test_tsx_clears_previous_n_flag() {
    let mut cpu = setup_cpu();

    // TSX (0xBA)
    cpu.memory_mut().write(0x8000, 0xBA);
    cpu.set_sp(0x42);
    cpu.set_flag_n(true); // N flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_n()); // N flag should be cleared
}
