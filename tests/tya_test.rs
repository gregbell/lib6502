//! Comprehensive tests for the TYA (Transfer Y to Accumulator) instruction.
//!
//! Tests cover:
//! - Basic TYA operation
//! - Flag updates (Z, N)
//! - Various Y register values (0, positive, negative, edge cases)
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

// ========== Basic TYA Operation Tests ==========

#[test]
fn test_tya_basic() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // A = Y
    assert_eq!(cpu.y(), 0x42); // Y unchanged
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_tya_transfers_y_to_accumulator() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x55);
    cpu.set_a(0xFF); // A has different value initially

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55); // A now equals Y
    assert_eq!(cpu.y(), 0x55); // Y unchanged
}

// ========== Zero Flag Tests ==========

#[test]
fn test_tya_zero_flag_set() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x00); // Transfer zero

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert_eq!(cpu.y(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_tya_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x01); // Non-zero value

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_tya_negative_flag_set() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x80); // 0b10000000 - bit 7 set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80);
    assert_eq!(cpu.y(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_tya_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x7F); // 0b01111111 - bit 7 clear

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is 0
}

#[test]
fn test_tya_negative_flag_with_0xff() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0xFF); // All bits set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Edge Case Tests ==========

#[test]
fn test_tya_preserves_y_register() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x99); // Y register should be unchanged
    assert_eq!(cpu.a(), 0x99); // A should equal original Y
}

#[test]
fn test_tya_preserves_x_register() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_x(0x88); // Set X register

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.x(), 0x88); // X register should be unchanged
}

#[test]
fn test_tya_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_tya_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_tya_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

#[test]
fn test_tya_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

// ========== Multiple Transfer Tests ==========

#[test]
fn test_tya_sequence() {
    let mut cpu = setup_cpu();

    // Set up two TYA instructions
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.memory_mut().write(0x8001, 0x98);

    cpu.set_y(0x10);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x10);
    assert_eq!(cpu.pc(), 0x8001);

    // Change Y and execute another TYA
    cpu.set_y(0x20);
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x20); // A updated with new Y value
    assert_eq!(cpu.pc(), 0x8002);
}

#[test]
fn test_tya_overwrites_previous_a() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0xAB);
    cpu.set_a(0xCD); // A has a different value

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAB); // A is overwritten with Y
    assert_eq!(cpu.y(), 0xAB);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_tya_cycle_count() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // TYA takes 2 cycles
}

#[test]
fn test_tya_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 TYA instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x98);
    }

    cpu.set_y(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each TYA takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_tya_program_counter_advance() {
    let mut cpu = setup_cpu();

    // TYA (0x98) - single byte instruction
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}

// ========== Boundary Value Tests ==========

#[test]
fn test_tya_with_0x00() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tya_with_0x7f() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x7F); // Maximum positive signed value

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_tya_with_0x80() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x80); // Minimum negative signed value

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_tya_with_0xff() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Flag Combination Tests ==========

#[test]
fn test_tya_clears_previous_z_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_z(true); // Z flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_z()); // Z flag should be cleared
}

#[test]
fn test_tya_clears_previous_n_flag() {
    let mut cpu = setup_cpu();

    // TYA (0x98)
    cpu.memory_mut().write(0x8000, 0x98);
    cpu.set_y(0x42);
    cpu.set_flag_n(true); // N flag is set initially

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_n()); // N flag should be cleared
}
