//! Comprehensive tests for the LDA (Load Accumulator) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (Z, N)
//! - Various operand values (0x00, 0xFF, positive, negative)
//! - Cycle counts including page crossing penalties

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic LDA Operation Tests ==========

#[test]
fn test_lda_immediate_basic() {
    let mut cpu = setup_cpu();

    // LDA #$42 (0xA9 0x42)
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_lda_loads_value() {
    let mut cpu = setup_cpu();

    // LDA #$FF
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x00); // Start with zero

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // Accumulator loaded with 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Flag Tests ==========

#[test]
fn test_lda_zero_flag() {
    let mut cpu = setup_cpu();

    // LDA #$00
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0xFF); // Start with non-zero

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Zero flag set
    assert!(!cpu.flag_n());
}

#[test]
fn test_lda_negative_flag() {
    let mut cpu = setup_cpu();

    // LDA #$80 (0b10000000)
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80);
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_z());
}

#[test]
fn test_lda_clears_negative_flag() {
    let mut cpu = setup_cpu();

    // LDA #$7F (0b01111111)
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_flag_n(true); // Start with negative flag set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(!cpu.flag_n()); // Bit 7 is clear
    assert!(!cpu.flag_z());
}

#[test]
fn test_lda_clears_zero_flag() {
    let mut cpu = setup_cpu();

    // LDA #$01
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_flag_z(true); // Start with zero flag set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01);
    assert!(!cpu.flag_z()); // Zero flag cleared
    assert!(!cpu.flag_n());
}

#[test]
fn test_lda_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // LDA #$42
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_lda_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // LDA #$42
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_lda_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // LDA #$42
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

#[test]
fn test_lda_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // LDA #$42
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

// ========== Edge Case Tests ==========

#[test]
fn test_lda_load_0x00() {
    let mut cpu = setup_cpu();

    // LDA #$00
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_lda_load_0xff() {
    let mut cpu = setup_cpu();

    // LDA #$FF
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_lda_zero_page() {
    let mut cpu = setup_cpu();

    // LDA $42 (0xA5 0x42)
    cpu.memory_mut().write(0x8000, 0xA5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33); // Value at zero page address

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x33);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_lda_zero_page_x() {
    let mut cpu = setup_cpu();

    // LDA $42,X (0xB5 0x42)
    cpu.memory_mut().write(0x8000, 0xB5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0047, 0x55); // Value at 0x42 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_zero_page_x_wraps() {
    let mut cpu = setup_cpu();

    // LDA $FF,X (0xB5 0xFF) - should wrap around within zero page
    cpu.memory_mut().write(0x8000, 0xB5);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0004, 0x77); // Value at 0xFF + 0x05 = 0x04 (wrapped)

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x77);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute() {
    let mut cpu = setup_cpu();

    // LDA $1234 (0xAD 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xAD);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0x99 has bit 7 set
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute_x_no_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA $1234,X (0xBD 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xBD);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1239, 0xAA); // Value at 0x1234 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAA);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4); // No page crossing
}

#[test]
fn test_lda_absolute_x_with_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA $12FF,X (0xBD 0xFF 0x12) - crosses page boundary
    cpu.memory_mut().write(0x8000, 0xBD);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1304, 0xBB); // Value at 0x12FF + 0x05 = 0x1304

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xBB);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Page crossing adds 1 cycle
}

#[test]
fn test_lda_absolute_y_no_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA $1234,Y (0xB9 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xB9);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1237, 0xCC); // Value at 0x1234 + 0x03

    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xCC);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4); // No page crossing
}

#[test]
fn test_lda_absolute_y_with_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA $12FE,Y (0xB9 0xFE 0x12) - crosses page boundary
    cpu.memory_mut().write(0x8000, 0xB9);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1303, 0xDD); // Value at 0x12FE + 0x05 = 0x1303

    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xDD);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Page crossing adds 1 cycle
}

#[test]
fn test_lda_indirect_x() {
    let mut cpu = setup_cpu();

    // LDA ($40,X) (0xA1 0x40)
    cpu.memory_mut().write(0x8000, 0xA1);
    cpu.memory_mut().write(0x8001, 0x40);

    // X = 0x05, so effective zero page address is 0x45
    cpu.set_x(0x05);

    // Store pointer at 0x0045: points to 0x1234
    cpu.memory_mut().write(0x0045, 0x34); // Low byte
    cpu.memory_mut().write(0x0046, 0x12); // High byte

    // Store value at target address
    cpu.memory_mut().write(0x1234, 0xEE);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xEE);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_lda_indirect_x_wraps_in_zero_page() {
    let mut cpu = setup_cpu();

    // LDA ($FF,X) (0xA1 0xFF)
    cpu.memory_mut().write(0x8000, 0xA1);
    cpu.memory_mut().write(0x8001, 0xFF);

    // X = 0x05, so effective zero page address is 0x04 (wrapped)
    cpu.set_x(0x05);

    // Store pointer at 0x0004: points to 0x5678
    cpu.memory_mut().write(0x0004, 0x78); // Low byte
    cpu.memory_mut().write(0x0005, 0x56); // High byte

    // Store value at target address
    cpu.memory_mut().write(0x5678, 0x11);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x11);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_lda_indirect_y_no_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA ($40),Y (0xB1 0x40)
    cpu.memory_mut().write(0x8000, 0xB1);
    cpu.memory_mut().write(0x8001, 0x40);

    // Store pointer at 0x0040: points to 0x1234
    cpu.memory_mut().write(0x0040, 0x34); // Low byte
    cpu.memory_mut().write(0x0041, 0x12); // High byte

    // Y = 0x05, so effective address is 0x1234 + 0x05 = 0x1239
    cpu.set_y(0x05);

    // Store value at target address
    cpu.memory_mut().write(0x1239, 0x22);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x22);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5); // No page crossing
}

#[test]
fn test_lda_indirect_y_with_page_crossing() {
    let mut cpu = setup_cpu();

    // LDA ($40),Y (0xB1 0x40)
    cpu.memory_mut().write(0x8000, 0xB1);
    cpu.memory_mut().write(0x8001, 0x40);

    // Store pointer at 0x0040: points to 0x12FF
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte
    cpu.memory_mut().write(0x0041, 0x12); // High byte

    // Y = 0x05, so effective address is 0x12FF + 0x05 = 0x1304 (page crossing)
    cpu.set_y(0x05);

    // Store value at target address
    cpu.memory_mut().write(0x1304, 0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x44);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6); // Page crossing adds 1 cycle
}

// ========== Comprehensive Cycle Count Tests ==========

#[test]
fn test_lda_immediate_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xA9);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_lda_zero_page_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xA5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_lda_zero_page_x_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0047, 0x55);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xAD);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x99);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute_x_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xBD);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1239, 0xAA);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute_x_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xBD);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1304, 0xBB);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_lda_absolute_y_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB9);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1237, 0xCC);
    cpu.set_y(0x03);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_lda_absolute_y_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB9);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1303, 0xDD);
    cpu.set_y(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_lda_indirect_x_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xA1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.set_x(0x05);
    cpu.memory_mut().write(0x0045, 0x34);
    cpu.memory_mut().write(0x0046, 0x12);
    cpu.memory_mut().write(0x1234, 0xEE);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_lda_indirect_y_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x34);
    cpu.memory_mut().write(0x0041, 0x12);
    cpu.set_y(0x05);
    cpu.memory_mut().write(0x1239, 0x22);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_lda_indirect_y_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0xB1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF);
    cpu.memory_mut().write(0x0041, 0x12);
    cpu.set_y(0x05);
    cpu.memory_mut().write(0x1304, 0x44);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 6);
}
