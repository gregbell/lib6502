//! Comprehensive tests for the STA (Store Accumulator) instruction.
//!
//! Tests cover:
//! - All 7 addressing modes
//! - No flag updates (STA does not affect flags)
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

// ========== Basic STA Operation Tests ==========

#[test]
fn test_sta_zero_page_basic() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x33);

    cpu.step().unwrap();

    // Verify the accumulator value was stored at 0x0042
    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sta_stores_value() {
    let mut cpu = setup_cpu();

    // STA $1234 (0x8D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8D);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Verify the accumulator value was stored at 0x1234
    assert_eq!(cpu.memory_mut().read(0x1234), 0xFF);
}

// ========== Flag Tests ==========

#[test]
fn test_sta_does_not_affect_zero_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x00);
    cpu.set_flag_z(false); // Start with zero flag clear

    cpu.step().unwrap();

    // Zero flag should remain unchanged
    assert!(!cpu.flag_z());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_sta_does_not_affect_negative_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x80); // Negative value
    cpu.set_flag_n(false); // Start with negative flag clear

    cpu.step().unwrap();

    // Negative flag should remain unchanged
    assert!(!cpu.flag_n());
    assert_eq!(cpu.memory_mut().read(0x0042), 0x80);
}

#[test]
fn test_sta_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_sta_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_sta_preserves_interrupt_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_i(false); // Clear interrupt flag

    cpu.step().unwrap();

    assert!(!cpu.flag_i()); // Interrupt flag should be unchanged
}

#[test]
fn test_sta_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    assert!(cpu.flag_d()); // Decimal flag should be unchanged
}

// ========== Edge Case Tests ==========

#[test]
fn test_sta_store_0x00() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
}

#[test]
fn test_sta_store_0xff() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_sta_zero_page() {
    let mut cpu = setup_cpu();

    // STA $42 (0x85 0x42)
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x33);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sta_zero_page_x() {
    let mut cpu = setup_cpu();

    // STA $42,X (0x95 0x42)
    cpu.memory_mut().write(0x8000, 0x95);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x55);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0x42 + 0x05 = 0x47
    assert_eq!(cpu.memory_mut().read(0x0047), 0x55);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sta_zero_page_x_wraps() {
    let mut cpu = setup_cpu();

    // STA $FF,X (0x95 0xFF) - should wrap around within zero page
    cpu.memory_mut().write(0x8000, 0x95);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x77);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0xFF + 0x05 = 0x04 (wrapped)
    assert_eq!(cpu.memory_mut().read(0x0004), 0x77);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sta_absolute() {
    let mut cpu = setup_cpu();

    // STA $1234 (0x8D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x8D);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0x99);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x99);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sta_absolute_x_no_page_crossing() {
    let mut cpu = setup_cpu();

    // STA $1234,X (0x9D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x9D);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0xAA);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0x1234 + 0x05 = 0x1239
    assert_eq!(cpu.memory_mut().read(0x1239), 0xAA);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Store instructions always take base cycles
}

#[test]
fn test_sta_absolute_x_with_page_crossing() {
    let mut cpu = setup_cpu();

    // STA $12FF,X (0x9D 0xFF 0x12) - crosses page boundary
    cpu.memory_mut().write(0x8000, 0x9D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0xBB);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // Should store at 0x12FF + 0x05 = 0x1304
    assert_eq!(cpu.memory_mut().read(0x1304), 0xBB);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Store instructions do NOT have page crossing penalty
}

#[test]
fn test_sta_absolute_y_no_page_crossing() {
    let mut cpu = setup_cpu();

    // STA $1234,Y (0x99 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x99);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0xCC);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    // Should store at 0x1234 + 0x03 = 0x1237
    assert_eq!(cpu.memory_mut().read(0x1237), 0xCC);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Store instructions always take base cycles
}

#[test]
fn test_sta_absolute_y_with_page_crossing() {
    let mut cpu = setup_cpu();

    // STA $12FE,Y (0x99 0xFE 0x12) - crosses page boundary
    cpu.memory_mut().write(0x8000, 0x99);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0xDD);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    // Should store at 0x12FE + 0x05 = 0x1303
    assert_eq!(cpu.memory_mut().read(0x1303), 0xDD);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // Store instructions do NOT have page crossing penalty
}

#[test]
fn test_sta_indirect_x() {
    let mut cpu = setup_cpu();

    // STA ($40,X) (0x81 0x40)
    cpu.memory_mut().write(0x8000, 0x81);
    cpu.memory_mut().write(0x8001, 0x40);

    // X = 0x05, so effective zero page address is 0x45
    cpu.set_x(0x05);

    // Store pointer at 0x0045: points to 0x1234
    cpu.memory_mut().write(0x0045, 0x34); // Low byte
    cpu.memory_mut().write(0x0046, 0x12); // High byte

    cpu.set_a(0xEE);

    cpu.step().unwrap();

    // Should store at 0x1234
    assert_eq!(cpu.memory_mut().read(0x1234), 0xEE);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_sta_indirect_x_wraps_in_zero_page() {
    let mut cpu = setup_cpu();

    // STA ($FF,X) (0x81 0xFF)
    cpu.memory_mut().write(0x8000, 0x81);
    cpu.memory_mut().write(0x8001, 0xFF);

    // X = 0x05, so effective zero page address is 0x04 (wrapped)
    cpu.set_x(0x05);

    // Store pointer at 0x0004: points to 0x5678
    cpu.memory_mut().write(0x0004, 0x78); // Low byte
    cpu.memory_mut().write(0x0005, 0x56); // High byte

    cpu.set_a(0x11);

    cpu.step().unwrap();

    // Should store at 0x5678
    assert_eq!(cpu.memory_mut().read(0x5678), 0x11);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_sta_indirect_y_no_page_crossing() {
    let mut cpu = setup_cpu();

    // STA ($40),Y (0x91 0x40)
    cpu.memory_mut().write(0x8000, 0x91);
    cpu.memory_mut().write(0x8001, 0x40);

    // Store pointer at 0x0040: points to 0x1234
    cpu.memory_mut().write(0x0040, 0x34); // Low byte
    cpu.memory_mut().write(0x0041, 0x12); // High byte

    // Y = 0x05, so effective address is 0x1234 + 0x05 = 0x1239
    cpu.set_y(0x05);

    cpu.set_a(0x22);

    cpu.step().unwrap();

    // Should store at 0x1239
    assert_eq!(cpu.memory_mut().read(0x1239), 0x22);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6); // Store instructions do NOT have page crossing penalty
}

#[test]
fn test_sta_indirect_y_with_page_crossing() {
    let mut cpu = setup_cpu();

    // STA ($40),Y (0x91 0x40)
    cpu.memory_mut().write(0x8000, 0x91);
    cpu.memory_mut().write(0x8001, 0x40);

    // Store pointer at 0x0040: points to 0x12FF
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte
    cpu.memory_mut().write(0x0041, 0x12); // High byte

    // Y = 0x05, so effective address is 0x12FF + 0x05 = 0x1304 (page crossing)
    cpu.set_y(0x05);

    cpu.set_a(0x44);

    cpu.step().unwrap();

    // Should store at 0x1304
    assert_eq!(cpu.memory_mut().read(0x1304), 0x44);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6); // Store instructions do NOT have page crossing penalty
}

// ========== Comprehensive Cycle Count Tests ==========

#[test]
fn test_sta_zero_page_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x85);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_a(0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sta_zero_page_x_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x95);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.set_a(0x55);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sta_absolute_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x8D);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_a(0x99);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sta_absolute_x_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x9D);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_a(0xAA);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_sta_absolute_x_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x9D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_a(0xBB);
    cpu.set_x(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5); // No page crossing penalty
}

#[test]
fn test_sta_absolute_y_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x99);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_a(0xCC);
    cpu.set_y(0x03);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_sta_absolute_y_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x99);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.set_a(0xDD);
    cpu.set_y(0x05);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 5); // No page crossing penalty
}

#[test]
fn test_sta_indirect_x_cycles() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x81);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.set_x(0x05);
    cpu.memory_mut().write(0x0045, 0x34);
    cpu.memory_mut().write(0x0046, 0x12);
    cpu.set_a(0xEE);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_sta_indirect_y_cycles_no_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x91);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x34);
    cpu.memory_mut().write(0x0041, 0x12);
    cpu.set_y(0x05);
    cpu.set_a(0x22);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_sta_indirect_y_cycles_with_crossing() {
    let mut cpu = setup_cpu();
    cpu.memory_mut().write(0x8000, 0x91);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF);
    cpu.memory_mut().write(0x0041, 0x12);
    cpu.set_y(0x05);
    cpu.set_a(0x44);
    cpu.step().unwrap();
    assert_eq!(cpu.cycles(), 6); // No page crossing penalty
}
