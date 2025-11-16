//! Comprehensive tests for the STX (Store X Register) instruction.
//!
//! Tests cover:
//! - All 3 addressing modes (Zero Page, Zero Page,Y, Absolute)
//! - No flag updates (STX does not affect flags)
//! - Various operand values (0x00, 0xFF, positive, negative)
//! - Cycle counts (no page crossing penalties for stores)

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic STX Operation Tests ==========

#[test]
fn test_stx_zero_page_basic() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x33);

    cpu.step().unwrap();

    // Verify the X register value was stored at 0x0042
    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_stx_stores_value() {
    let mut cpu = setup_cpu();

    // STX $1234 (0x8E 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8E);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_x(0xFF);

    cpu.step().unwrap();

    // Verify the X register value was stored at 0x1234
    assert_eq!(cpu.memory_mut().read(0x1234), 0xFF);
}

// ========== Flag Tests ==========

#[test]
fn test_stx_does_not_affect_zero_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x00);
    cpu.set_flag_z(false); // Start with zero flag clear

    cpu.step().unwrap();

    // Zero flag should remain unchanged
    assert!(!cpu.flag_z());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_stx_does_not_affect_negative_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x80); // Negative value
    cpu.set_flag_n(false); // Start with negative flag clear

    cpu.step().unwrap();

    // Negative flag should remain unchanged
    assert!(!cpu.flag_n());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x80);
}

#[test]
fn test_stx_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_stx_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_stx_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

#[test]
fn test_stx_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

// ========== Edge Case Tests ==========

#[test]
fn test_stx_store_0x00() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_stx_store_0xff() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_stx_zero_page() {
    let mut cpu = setup_cpu();

    // STX $42 (0x86 0x42)
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_stx_zero_page_y() {
    let mut cpu = setup_cpu();

    // STX $42,Y (0x96 0x42)
    cpu.memory_mut().write(0x8000, 0x96);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_x(0x55);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    // Should store at 0x42 + 0x05 = 0x47
    assert_eq!(cpu.memory_mut().read(0x0047), 0x55);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_stx_zero_page_y_wraps() {
    let mut cpu = setup_cpu();

    // STX $FF,Y (0x96 0xFF) - should wrap around within zero page
    cpu.memory_mut().write(0x8000, 0x96);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_x(0x77);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    // Should store at 0xFF + 0x05 = 0x04 (wrapped)
    assert_eq!(cpu.memory_mut().read(0x0004), 0x77);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_stx_absolute() {
    let mut cpu = setup_cpu();

    // STX $1234 (0x8E 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8E);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_x(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x99);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Comprehensive Cycle Count Tests ==========

#[test]
fn test_stx_zero_page_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x86);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_x(0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_stx_zero_page_y_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x96);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_x(0x55);
    cpu.set_y(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_stx_absolute_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x8E);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_x(0x99);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}
