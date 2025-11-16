//! Comprehensive tests for the STY (Store Y Register) instruction.
//!
//! Tests cover:
//! - All 3 addressing modes (Zero Page, Zero Page,X, Absolute)
//! - No flag updates (STY does not affect flags)
//! - Various operand values (0x00, 0xFF, positive, negative)
//! - Cycle counts (no page crossing penalties for stores)

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic STY Operation Tests ==========

#[test]
fn test_sty_zero_page_basic() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x33);

    cpu.step().unwrap();

    // Verify the Y register value was stored at 0x0042
    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sty_stores_value() {
    let mut cpu = setup_cpu();

    // STY $1234 (0x8C 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_y(0xFF);

    cpu.step().unwrap();

    // Verify the Y register value was stored at 0x1234
    assert_eq!(cpu.memory_mut().read(0x1234), 0xFF);
}

// ========== Flag Tests ==========

#[test]
fn test_sty_does_not_affect_zero_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x00);
    cpu.set_flag_z(false); // Start with zero flag clear

    cpu.step().unwrap();

    // Zero flag should remain unchanged
    assert!(!cpu.flag_z());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_sty_does_not_affect_negative_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x80); // Negative value
    cpu.set_flag_n(false); // Start with negative flag clear

    cpu.step().unwrap();

    // Negative flag should remain unchanged
    assert!(!cpu.flag_n());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x80);
}

#[test]
fn test_sty_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_sty_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_sty_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

#[test]
fn test_sty_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

// ========== Edge Case Tests ==========

#[test]
fn test_sty_store_0x00() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_sty_store_0xff() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_sty_zero_page() {
    let mut cpu = setup_cpu();

    // STY $42 (0x84 0x42)
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sty_zero_page_x() {
    let mut cpu = setup_cpu();

    // STY $42,X (0x94 0x42)
    cpu.memory_mut().write(0x8000, 0x94);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x55);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0x42 + 0x05 = 0x47
    assert_eq!(cpu.memory_mut().read(0x0047), 0x55);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sty_zero_page_x_wraps() {
    let mut cpu = setup_cpu();

    // STY $FF,X (0x94 0xFF) - should wrap around within zero page
    cpu.memory_mut().write(0x8000, 0x94);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_y(0x77);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0xFF + 0x05 = 0x04 (wrapped)
    assert_eq!(cpu.memory_mut().read(0x0004), 0x77);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sty_absolute() {
    let mut cpu = setup_cpu();

    // STY $1234 (0x8C 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_y(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x99);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Comprehensive Cycle Count Tests ==========

#[test]
fn test_sty_zero_page_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x84);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_y(0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sty_zero_page_x_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x94);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_y(0x55);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sty_absolute_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x8C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_y(0x99);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}
