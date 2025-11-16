//! Comprehensive tests for the LDX (Load X Register) instruction.
//!
//! Tests cover:
//! - All 5 addressing modes
//! - Flag updates (Z, N)
//! - Various operand values (0x00, 0xFF, positive, negative)
//! - Cycle counts including page crossing penalties

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic LDX Operation Tests ==========

#[test]
fn test_ldx_immediate_basic() {
    let mut cpu = setup_cpu();

    // LDX #$42 (0xA2 0x42)
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_ldx_loads_value() {
    let mut cpu = setup_cpu();

    // LDX #$FF
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_x(0x00); // Start with zero

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF); // X register loaded with 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Flag Tests ==========

#[test]
fn test_ldx_zero_flag() {
    let mut cpu = setup_cpu();

    // LDX #$00
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_x(0xFF); // Start with non-zero

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z()); // Zero flag set
    assert!(!cpu.flag_n());
}

#[test]
fn test_ldx_negative_flag() {
    let mut cpu = setup_cpu();

    // LDX #$80 (0b10000000)
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80);
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_z());
}

#[test]
fn test_ldx_clears_negative_flag() {
    let mut cpu = setup_cpu();

    // LDX #$7F (0b01111111)
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_flag_n(true); // Start with negative flag set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F);
    assert!(!cpu.flag_n()); // Bit 7 is clear
    assert!(!cpu.flag_z());
}

#[test]
fn test_ldx_clears_zero_flag() {
    let mut cpu = setup_cpu();

    // LDX #$01
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_flag_z(true); // Start with zero flag set

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z()); // Zero flag cleared
    assert!(!cpu.flag_n());
}

#[test]
fn test_ldx_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // LDX #$42
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_ldx_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // LDX #$42
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_ldx_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // LDX #$42
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

#[test]
fn test_ldx_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // LDX #$42
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

// ========== Edge Case Tests ==========

#[test]
fn test_ldx_load_0x00() {
    let mut cpu = setup_cpu();

    // LDX #$00
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_ldx_load_0xff() {
    let mut cpu = setup_cpu();

    // LDX #$FF
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_ldx_zero_page() {
    let mut cpu = setup_cpu();

    // LDX $42 (0xA6 0x42)
    cpu.memory_mut().write(0x8000, 0xA6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33); // Value at zero page address

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_ldx_zero_page_y() {
    let mut cpu = setup_cpu();

    // LDX $42,Y (0xB6 0x42)
    cpu.memory_mut().write(0x8000, 0xB6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0047, 0x55); // Value at 0x42 + 0x05

    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_zero_page_y_wraps() {
    let mut cpu = setup_cpu();

    // LDX $FF,Y (0xB6 0xFF) - should wrap around within zero page
    cpu.memory_mut().write(0x8000, 0xB6);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0004, 0x77); // Value at 0xFF + 0x05 = 0x04 (wrapped)

    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x77);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_absolute() {
    let mut cpu = setup_cpu();

    // LDX $1234 (0xAE 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xAE);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x99);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0x99 has bit 7 set
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_absolute_y_no_page_crossing() {
    let mut cpu = setup_cpu();

    // LDX $1234,Y (0xBE 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xBE);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1239, 0xAA); // Value at 0x1234 + 0x05

    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xAA);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4); // No page crossing
}

#[test]
fn test_ldx_absolute_y_with_page_crossing() {
    let mut cpu = setup_cpu();

    // LDX $12FF,Y (0xBE 0xFF 0x12) - crosses page boundary
    cpu.memory_mut().write(0x8000, 0xBE);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1304, 0xBB); // Value at 0x12FF + 0x05 = 0x1304

    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xBB);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Page crossing adds 1 cycle
}

// ========== Comprehensive Cycle Count Tests ==========

#[test]
fn test_ldx_immediate_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xA2);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_ldx_zero_page_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xA6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_ldx_zero_page_y_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0047, 0x55);
    cpu.set_y(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_absolute_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xAE);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x99);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_absolute_y_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xBE);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1237, 0xCC);
    cpu.set_y(0x03);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ldx_absolute_y_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xBE);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1303, 0xDD);
    cpu.set_y(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}
