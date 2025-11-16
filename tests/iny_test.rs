//! Comprehensive tests for the INY (Increment Y Register) instruction.
//!
//! Tests cover:
//! - Basic INY operation
//! - Flag updates (Z, N)
//! - Various Y register values (0, positive, negative, edge cases)
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

// ========== Basic INY Operation Tests ==========

#[test]
fn test_iny_basic() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x06); // 5 + 1 = 6
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_iny_increments_by_one() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x43); // 66 + 1 = 67
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_iny_zero_flag_set() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0xFF); // 255 + 1 = 0 (wraps)

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_iny_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_iny_negative_flag_set() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_iny_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x01); // 0b00000001

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x02); // 0b00000010
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

#[test]
fn test_iny_negative_flag_remains_set() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x81); // 0b10000001
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

// ========== Overflow/Wrap Tests ==========

#[test]
fn test_iny_wraps_from_ff_to_zero() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0xFF); // 255 + 1 wraps to 0

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n()); // 0x00 has bit 7 clear
}

#[test]
fn test_iny_from_0x7f_to_0x80() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 set
}

// ========== Edge Case Tests ==========

#[test]
fn test_iny_from_zero() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x01);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_iny_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x05);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x06);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_iny_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x05);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x06);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_iny_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x05);
    cpu.set_a(0x42); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x06);
    assert_eq!(cpu.a(), 0x42); // Accumulator should be unchanged
}

#[test]
fn test_iny_preserves_x_register() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x05);
    cpu.set_x(0x99); // Set X register

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x06);
    assert_eq!(cpu.x(), 0x99); // X register should be unchanged
}

// ========== Multiple Increments Test ==========

#[test]
fn test_iny_sequence() {
    let mut cpu = setup_cpu();

    // Set up three INY instructions
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.memory_mut().write(0x8001, 0xC8);
    cpu.memory_mut().write(0x8002, 0xC8);

    cpu.set_y(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x01);
    assert_eq!(cpu.pc(), 0x8001);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x02);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x03);
    assert_eq!(cpu.pc(), 0x8003);
    assert!(!cpu.flag_z());
}

#[test]
fn test_iny_countup_from_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 INY instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + i, 0xC8);
    }

    cpu.set_y(0);

    for i in 0..10 {
        cpu.step().unwrap();
        assert_eq!(cpu.y(), i + 1);
    }

    // Last value should not have Z flag set
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_iny_overflow_sequence() {
    let mut cpu = setup_cpu();

    // INY from 0xFE to 0xFF
    cpu.memory_mut().write(0x8000, 0xC8);
    // INY from 0xFF to 0x00
    cpu.memory_mut().write(0x8001, 0xC8);
    // INY from 0x00 to 0x01
    cpu.memory_mut().write(0x8002, 0xC8);

    cpu.set_y(0xFE);

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x01);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_iny_cycle_count() {
    let mut cpu = setup_cpu();

    // INY (0xC8)
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // INY takes 2 cycles
}

#[test]
fn test_iny_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 INY instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0xC8);
    }

    cpu.set_y(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each INY takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_iny_program_counter_advance() {
    let mut cpu = setup_cpu();

    // INY (0xC8) - single byte instruction
    cpu.memory_mut().write(0x8000, 0xC8);
    cpu.set_y(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}
