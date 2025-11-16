//! Comprehensive tests for the INX (Increment X Register) instruction.
//!
//! Tests cover:
//! - Basic INX operation
//! - Flag updates (Z, N)
//! - Various X register values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Wrapping behavior

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic INX Operation Tests ==========

#[test]
fn test_inx_basic() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x06); // 5 + 1 = 6
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_inx_increments_by_one() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x43); // 66 + 1 = 67
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_inx_zero_flag_set() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0xFF); // 255 + 1 = 0 (wraps)

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_inx_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_inx_negative_flag_set() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_inx_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x01); // 0b00000001

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x02); // 0b00000010
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

#[test]
fn test_inx_negative_flag_remains_set() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x81); // 0b10000001
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

// ========== Overflow/Wrap Tests ==========

#[test]
fn test_inx_wraps_from_ff_to_zero() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0xFF); // 255 + 1 wraps to 0

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n()); // 0x00 has bit 7 clear
}

#[test]
fn test_inx_from_0x7f_to_0x80() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 set
}

// ========== Edge Case Tests ==========

#[test]
fn test_inx_from_zero() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_inx_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x05);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x06);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_inx_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x05);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x06);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_inx_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x05);
    cpu.set_a(0x42); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x06);
    assert_eq!(cpu.a(), 0x42); // Accumulator should be unchanged
}

#[test]
fn test_inx_preserves_y_register() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x05);
    cpu.set_y(0x99); // Set Y register

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x06);
    assert_eq!(cpu.y(), 0x99); // Y register should be unchanged
}

// ========== Multiple Increments Test ==========

#[test]
fn test_inx_sequence() {
    let mut cpu = setup_cpu();

    // Set up three INX instructions
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.memory_mut().write(0x8001, 0xE8);
    cpu.memory_mut().write(0x8002, 0xE8);

    cpu.set_x(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x01);
    assert_eq!(cpu.pc(), 0x8001);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x02);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x03);
    assert_eq!(cpu.pc(), 0x8003);
    assert!(!cpu.flag_z());
}

#[test]
fn test_inx_countup_from_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 INX instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + i, 0xE8);
    }

    cpu.set_x(0);

    for i in 0..10 {
        cpu.step().unwrap();
        assert_eq!(cpu.x(), i + 1);
    }

    // Last value should not have Z flag set
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_inx_overflow_sequence() {
    let mut cpu = setup_cpu();

    // INX from 0xFE to 0xFF
    cpu.memory_mut().write(0x8000, 0xE8);
    // INX from 0xFF to 0x00
    cpu.memory_mut().write(0x8001, 0xE8);
    // INX from 0x00 to 0x01
    cpu.memory_mut().write(0x8002, 0xE8);

    cpu.set_x(0xFE);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_inx_cycle_count() {
    let mut cpu = setup_cpu();

    // INX (0xE8)
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // INX takes 2 cycles
}

#[test]
fn test_inx_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 INX instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0xE8);
    }

    cpu.set_x(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each INX takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_inx_program_counter_advance() {
    let mut cpu = setup_cpu();

    // INX (0xE8) - single byte instruction
    cpu.memory_mut().write(0x8000, 0xE8);
    cpu.set_x(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}
