//! Comprehensive tests for the TAX (Transfer Accumulator to X) instruction.
//!
//! Tests cover:
//! - Basic TAX operation
//! - Flag updates (Z, N)
//! - Various accumulator values (0, positive, negative, edge cases)
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

// ========== Basic TAX Operation Tests ==========

#[test]
fn test_tax_basic() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42); // X = A
    assert_eq!(cpu.a(), 0x42); // A unchanged
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_tax_transfers_accumulator_to_x() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x55);
    cpu.set_x(0xFF); // X has different value initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55); // X now equals A
    assert_eq!(cpu.a(), 0x55); // A unchanged
}

// ========== Zero Flag Tests ==========

#[test]
fn test_tax_zero_flag_set() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x00); // Transfer zero

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_tax_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x01); // Non-zero value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_tax_negative_flag_set() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x80); // 0b10000000 - bit 7 set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80);
    assert_eq!(cpu.a(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_tax_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x7F); // 0b01111111 - bit 7 clear

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is 0
}

#[test]
fn test_tax_negative_flag_with_0xff() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0xFF); // All bits set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Edge Case Tests ==========

#[test]
fn test_tax_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99); // Accumulator should be unchanged
    assert_eq!(cpu.x(), 0x99); // X should equal original A
}

#[test]
fn test_tax_preserves_y_register() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_y(0x88); // Set Y register

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert_eq!(cpu.y(), 0x88); // Y register should be unchanged
}

#[test]
fn test_tax_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_tax_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_tax_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

#[test]
fn test_tax_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

// ========== Multiple Transfer Tests ==========

#[test]
fn test_tax_sequence() {
    let mut cpu = setup_cpu();

    // Set up two TAX instructions
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.memory_mut().write(0x8001, 0xAA);

    cpu.set_a(0x10);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x10);
    assert_eq!(cpu.pc(), 0x8001);

    // Change A and execute another TAX
    cpu.set_a(0x20);
    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x20); // X updated with new A value
    assert_eq!(cpu.pc(), 0x8002);
}

#[test]
fn test_tax_overwrites_previous_x() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0xAB);
    cpu.set_x(0xCD); // X has a different value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xAB); // X is overwritten with A
    assert_eq!(cpu.a(), 0xAB);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_tax_cycle_count() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // TAX takes 2 cycles
}

#[test]
fn test_tax_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 TAX instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0xAA);
    }

    cpu.set_a(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each TAX takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_tax_program_counter_advance() {
    let mut cpu = setup_cpu();

    // TAX (0xAA) - single byte instruction
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}

// ========== Boundary Value Tests ==========

#[test]
fn test_tax_with_0x00() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tax_with_0x7f() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x7F); // Maximum positive signed value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tax_with_0x80() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x80); // Minimum negative signed value

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_tax_with_0xff() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Flag Combination Tests ==========

#[test]
fn test_tax_clears_previous_z_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_z(true); // Z flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_z()); // Z flag should be cleared
}

#[test]
fn test_tax_clears_previous_n_flag() {
    let mut cpu = setup_cpu();

    // TAX (0xAA)
    cpu.memory_mut().write(0x8000, 0xAA);
    cpu.set_a(0x42);
    cpu.set_flag_n(true); // N flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_n()); // N flag should be cleared
}
