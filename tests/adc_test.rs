//! Comprehensive tests for the ADC (Add with Carry) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (C, Z, V, N)
//! - Various operand values (0, positive, negative)
//! - Overflow/underflow conditions
//! - Cycle counts including page crossing penalties

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic ADC Operation Tests ==========

#[test]
fn test_adc_immediate_basic() {
    let mut cpu = setup_cpu();

    // ADC #$05 (0x69 0x05)
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x10);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x15); // 0x10 + 0x05 = 0x15
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_adc_with_carry_in() {
    let mut cpu = setup_cpu();

    // ADC #$05
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x10);
    cpu.set_flag_c(true); // Carry flag set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x16); // 0x10 + 0x05 + 1 = 0x16
}

// ========== Flag Tests ==========

#[test]
fn test_adc_carry_flag() {
    let mut cpu = setup_cpu();

    // ADC #$FF
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x01);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0x01 + 0xFF = 0x100 (wrapped to 0x00)
    assert!(cpu.flag_c()); // Carry should be set
    assert!(cpu.flag_z()); // Result is zero
}

#[test]
fn test_adc_zero_flag() {
    let mut cpu = setup_cpu();

    // ADC #$00
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
}

#[test]
fn test_adc_negative_flag() {
    let mut cpu = setup_cpu();

    // ADC #$80
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80);
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_adc_overflow_positive_to_negative() {
    let mut cpu = setup_cpu();

    // Adding two positive numbers that overflow to negative
    // 0x50 (80) + 0x50 (80) = 0xA0 (160 unsigned, -96 signed)
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_a(0x50);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xA0);
    assert!(cpu.flag_v()); // Overflow occurred
    assert!(cpu.flag_n()); // Result is negative in signed interpretation
}

#[test]
fn test_adc_overflow_negative_to_positive() {
    let mut cpu = setup_cpu();

    // Adding two negative numbers that overflow to positive
    // 0x80 (-128) + 0xFF (-1) = 0x7F (127) with carry
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x80);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(cpu.flag_v()); // Overflow occurred
    assert!(cpu.flag_c()); // Carry occurred
}

#[test]
fn test_adc_no_overflow_positive_positive() {
    let mut cpu = setup_cpu();

    // Adding two positive numbers without overflow
    // 0x20 (32) + 0x30 (48) = 0x50 (80)
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_a(0x20);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x50);
    assert!(!cpu.flag_v()); // No overflow
}

#[test]
fn test_adc_no_overflow_negative_negative() {
    let mut cpu = setup_cpu();

    // Adding two negative numbers without signed overflow
    // 0xFF (-1) + 0xFE (-2) = 0x1FD -> 0xFD (-3) with carry
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0xFE);

    cpu.set_a(0xFF);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFD);
    assert!(!cpu.flag_v()); // No signed overflow
    assert!(cpu.flag_c()); // Unsigned carry
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_adc_zero_page() {
    let mut cpu = setup_cpu();

    // ADC $42 (0x65 0x42)
    cpu.memory_mut().write(0x8000, 0x65);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33); // Value at zero page address

    cpu.set_a(0x11);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x44); // 0x11 + 0x33 = 0x44
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_adc_zero_page_x() {
    let mut cpu = setup_cpu();

    // ADC $40,X (0x75 0x40)
    cpu.memory_mut().write(0x8000, 0x75);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x22); // Value at 0x40 + 0x05

    cpu.set_a(0x11);
    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x33); // 0x11 + 0x22 = 0x33
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_adc_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // ADC $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x75);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x42); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_a(0x10);
    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x52); // 0x10 + 0x42 = 0x52
}

#[test]
fn test_adc_absolute() {
    let mut cpu = setup_cpu();

    // ADC $1234 (0x6D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x6D);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x55);

    cpu.set_a(0x10);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x65); // 0x10 + 0x55 = 0x65
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_adc_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // ADC $1200,X (0x7D 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x7D);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x33); // Value at 0x1200 + 0x05

    cpu.set_a(0x11);
    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x44);
    assert_eq!(cpu.cycles(), 4); // No page cross, base cycles only
}

#[test]
fn test_adc_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // ADC $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x7D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x77); // Value at 0x12FF + 0x02

    cpu.set_a(0x11);
    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x88);
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_adc_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // ADC $1200,Y (0x79 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x79);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1203, 0x44); // Value at 0x1200 + 0x03

    cpu.set_a(0x22);
    cpu.set_y(0x03);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x66);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_adc_absolute_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // ADC $10FE,Y with Y=3 crosses page boundary (0x10FE -> 0x1101)
    cpu.memory_mut().write(0x8000, 0x79);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);
    cpu.memory_mut().write(0x1101, 0x88); // Value at 0x10FE + 0x03

    cpu.set_a(0x11);
    cpu.set_y(0x03);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99);
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_adc_indirect_x() {
    let mut cpu = setup_cpu();

    // ADC ($40,X) (0x61 0x40)
    // With X=5, reads address from zero page 0x45/0x46
    cpu.memory_mut().write(0x8000, 0x61);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x00); // Low byte of target address
    cpu.memory_mut().write(0x0046, 0x20); // High byte of target address (0x2000)
    cpu.memory_mut().write(0x2000, 0x99); // Value at target address

    cpu.set_a(0x11);
    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAA); // 0x11 + 0x99 = 0xAA
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_adc_indirect_x_zero_page_wrap() {
    let mut cpu = setup_cpu();

    // ADC ($FF,X) with X=1
    // Address pointer wraps: 0xFF + 1 = 0x00 in zero page
    cpu.memory_mut().write(0x8000, 0x61);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0000, 0x34); // Low byte at 0x00
    cpu.memory_mut().write(0x0001, 0x12); // High byte at 0x01 (0x1234)
    cpu.memory_mut().write(0x1234, 0x55);

    cpu.set_a(0x10);
    cpu.set_x(0x01);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x65); // 0x10 + 0x55 = 0x65
}

#[test]
fn test_adc_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // ADC ($40),Y (0x71 0x40)
    // Reads base address from 0x40/0x41, then adds Y
    cpu.memory_mut().write(0x8000, 0x71);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x00); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte of base address (0x2000)
    cpu.memory_mut().write(0x2003, 0x77); // Value at 0x2000 + Y(3)

    cpu.set_a(0x11);
    cpu.set_y(0x03);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x88); // 0x11 + 0x77 = 0x88
    assert_eq!(cpu.cycles(), 5); // No page cross
}

#[test]
fn test_adc_indirect_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // ADC ($40),Y with page crossing
    cpu.memory_mut().write(0x8000, 0x71);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte (0x20FF)
    cpu.memory_mut().write(0x2101, 0x44); // Value at 0x20FF + Y(2) = 0x2101

    cpu.set_a(0x22);
    cpu.set_y(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x66); // 0x22 + 0x44 = 0x66
    assert_eq!(cpu.cycles(), 6); // Page cross adds 1 cycle
}

// ========== Edge Case Tests ==========

#[test]
fn test_adc_all_ones() {
    let mut cpu = setup_cpu();

    // ADC #$FF with A=$FF and C=1
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xFF + 0xFF + 1 = 0x1FF -> 0xFF
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_adc_all_zeros() {
    let mut cpu = setup_cpu();

    // ADC #$00 with A=$00 and C=0
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(!cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_adc_max_unsigned_addition() {
    let mut cpu = setup_cpu();

    // Test maximum unsigned addition
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFE); // 0xFF + 0xFF = 0x1FE -> 0xFE
    assert!(cpu.flag_c());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_adc_sequence() {
    let mut cpu = setup_cpu();

    // First ADC: 0x10 + 0x20 = 0x30
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x20);

    // Second ADC: 0x30 + 0x30 = 0x60
    cpu.memory_mut().write(0x8002, 0x69);
    cpu.memory_mut().write(0x8003, 0x30);

    cpu.set_a(0x10);
    cpu.set_flag_c(false);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x30);
    assert_eq!(cpu.pc(), 0x8002);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x60);
    assert_eq!(cpu.pc(), 0x8004);
}

#[test]
fn test_adc_carry_chain() {
    let mut cpu = setup_cpu();

    // ADC sequence that carries through multiple operations
    // 0xFF + 0x01 = 0x00 (carry=1)
    cpu.memory_mut().write(0x8000, 0x69);
    cpu.memory_mut().write(0x8001, 0x01);

    // 0x00 + 0x00 + carry(1) = 0x01
    cpu.memory_mut().write(0x8002, 0x69);
    cpu.memory_mut().write(0x8003, 0x00);

    cpu.set_a(0xFF);
    cpu.set_flag_c(false);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x01);
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}
