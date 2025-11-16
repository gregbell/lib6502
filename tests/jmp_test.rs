//! Comprehensive tests for the JMP (Jump) instruction.
//!
//! Tests cover:
//! - Basic JMP operation works correctly
//! - All addressing modes implemented:
//!   - Absolute (opcode 0x4C)
//!   - Indirect (opcode 0x6C)
//! - Correct cycle counts for all addressing modes
//! - Hardware bug in Indirect mode (page boundary wrapping)
//! - No flags affected
//! - Register preservation

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic JMP Absolute Operation Tests ==========

#[test]
fn test_jmp_absolute_basic() {
    let mut cpu = setup_cpu();

    // JMP $1234 (opcode 0x4C)
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte

    cpu.step().unwrap();

    // PC should be set to $1234
    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.cycles(), 3); // 3 cycles for absolute JMP
}

#[test]
fn test_jmp_absolute_to_zero_page() {
    let mut cpu = setup_cpu();

    // JMP $0042 - Jump to zero page
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x8002, 0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x0042);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_jmp_absolute_to_high_memory() {
    let mut cpu = setup_cpu();

    // JMP $FFFC - Jump to high memory
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0xFC);
    cpu.memory_mut().write(0x8002, 0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0xFFFC);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_jmp_absolute_same_page() {
    let mut cpu = setup_cpu();

    // JMP $8010 - Jump within same page
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x8002, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x8010);
    assert_eq!(cpu.cycles(), 3);
}

// ========== JMP Indirect Operation Tests ==========

#[test]
fn test_jmp_indirect_basic() {
    let mut cpu = setup_cpu();

    // JMP ($2000) - pointer at $2000 points to $3456
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x00); // Pointer low byte
    cpu.memory_mut().write(0x8002, 0x20); // Pointer high byte

    // Set up the target address at $2000
    cpu.memory_mut().write(0x2000, 0x56); // Target low byte
    cpu.memory_mut().write(0x2001, 0x34); // Target high byte

    cpu.step().unwrap();

    // PC should be set to $3456
    assert_eq!(cpu.pc(), 0x3456);
    assert_eq!(cpu.cycles(), 5); // 5 cycles for indirect JMP
}

#[test]
fn test_jmp_indirect_page_boundary_bug() {
    let mut cpu = setup_cpu();

    // JMP ($10FF) - This triggers the famous 6502 page boundary bug
    // The high byte should be read from $1000, not $1100
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0xFF); // Pointer low byte
    cpu.memory_mut().write(0x8002, 0x10); // Pointer high byte

    // Set up target address
    cpu.memory_mut().write(0x10FF, 0x34); // Target low byte at $10FF
    cpu.memory_mut().write(0x1000, 0x12); // Target high byte at $1000 (same page!)
    cpu.memory_mut().write(0x1100, 0x99); // This should NOT be used

    cpu.step().unwrap();

    // Due to the bug, PC should be $1234, not $9934
    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_jmp_indirect_no_bug_when_not_at_boundary() {
    let mut cpu = setup_cpu();

    // JMP ($10FE) - No bug, normal behavior
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);

    // Set up target address
    cpu.memory_mut().write(0x10FE, 0x78);
    cpu.memory_mut().write(0x10FF, 0x56);

    cpu.step().unwrap();

    // Normal behavior - PC should be $5678
    assert_eq!(cpu.pc(), 0x5678);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_jmp_indirect_zero_page_pointer() {
    let mut cpu = setup_cpu();

    // JMP ($0080) - Pointer in zero page
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x80);
    cpu.memory_mut().write(0x8002, 0x00);

    // Set up target address at $0080
    cpu.memory_mut().write(0x0080, 0xAB);
    cpu.memory_mut().write(0x0081, 0xCD);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0xCDAB);
    assert_eq!(cpu.cycles(), 5);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_jmp_preserves_all_flags() {
    let mut cpu = setup_cpu();

    // JMP $1234
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    // Set all flags to known values
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(false);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_jmp_indirect_preserves_all_flags() {
    let mut cpu = setup_cpu();

    // JMP ($2000)
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x20);
    cpu.memory_mut().write(0x2000, 0x34);
    cpu.memory_mut().write(0x2001, 0x12);

    // Set all flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(true);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_b());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_jmp_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_jmp_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_x(0x55);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55);
}

#[test]
fn test_jmp_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_y(0x66);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x66);
}

#[test]
fn test_jmp_preserves_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_sp(0xA0);

    cpu.step().unwrap();

    assert_eq!(cpu.sp(), 0xA0);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_jmp_absolute_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    assert_eq!(cpu.cycles() - initial_cycles, 3);
}

#[test]
fn test_jmp_indirect_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x20);
    cpu.memory_mut().write(0x2000, 0x34);
    cpu.memory_mut().write(0x2001, 0x12);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    assert_eq!(cpu.cycles() - initial_cycles, 5);
}

// ========== Complex Scenarios ==========

#[test]
fn test_jmp_chain() {
    let mut cpu = setup_cpu();

    // JMP $8003 at $8000
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x03);
    cpu.memory_mut().write(0x8002, 0x80);

    // JMP $9000 at $8003
    cpu.memory_mut().write(0x8003, 0x4C);
    cpu.memory_mut().write(0x8004, 0x00);
    cpu.memory_mut().write(0x8005, 0x90);

    // First jump
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 3);

    // Second jump
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x9000);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jmp_infinite_loop() {
    let mut cpu = setup_cpu();

    // JMP $8000 - infinite loop to itself
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x80);

    // First iteration
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8000);
    assert_eq!(cpu.cycles(), 3);

    // Second iteration
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8000);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jmp_indirect_through_indirect() {
    let mut cpu = setup_cpu();

    // JMP ($1000) where $1000 contains $2000
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x10);

    // Pointer at $1000 points to $2000
    cpu.memory_mut().write(0x1000, 0x00);
    cpu.memory_mut().write(0x1001, 0x20);

    cpu.step().unwrap();

    // PC should be $2000
    assert_eq!(cpu.pc(), 0x2000);
    assert_eq!(cpu.cycles(), 5);
}

// ========== Edge Cases ==========

#[test]
fn test_jmp_to_same_address() {
    let mut cpu = setup_cpu();

    // JMP $8000 (jump to same instruction)
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x8000);
}

#[test]
fn test_jmp_across_page_boundaries() {
    let mut cpu = setup_cpu();

    // JMP $80FF -> $8100 (crosses page boundary)
    cpu.memory_mut().write(0x8000, 0x4C);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x81);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x81FF);
    assert_eq!(cpu.cycles(), 3); // No extra cycle for page crossing in JMP
}

#[test]
fn test_jmp_indirect_with_zero_address() {
    let mut cpu = setup_cpu();

    // JMP ($0000) - pointer at address 0
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x00);

    cpu.memory_mut().write(0x0000, 0x34);
    cpu.memory_mut().write(0x0001, 0x12);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_jmp_indirect_page_boundary_at_0xff() {
    let mut cpu = setup_cpu();

    // JMP ($00FF) - page boundary bug at zero page boundary
    cpu.memory_mut().write(0x8000, 0x6C);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x00);

    // Due to the bug, high byte is read from $0000, not $0100
    cpu.memory_mut().write(0x00FF, 0x34);
    cpu.memory_mut().write(0x0000, 0x12); // Wraps to $0000
    cpu.memory_mut().write(0x0100, 0x99); // Should NOT be used

    cpu.step().unwrap();

    // PC should be $1234, not $9934
    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.cycles(), 5);
}
