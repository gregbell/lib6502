//! Comprehensive tests for the BMI (Branch if Minus) instruction.
//!
//! Tests cover:
//! - Branch taken when negative flag is set
//! - Branch not taken when negative flag is clear
//! - Forward and backward branches
//! - Page crossing detection and cycle penalties
//! - Correct cycle counts for all scenarios
//! - No flag modifications

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic BMI Operation Tests ==========

#[test]
fn test_bmi_branch_taken_forward() {
    let mut cpu = setup_cpu();

    // BMI +5 (0x30 0x05)
    // If negative flag set, branch forward 5 bytes
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x05);

    // Ensure negative flag is set
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 (instruction size) + 5 (offset) = 0x8007
    assert_eq!(cpu.pc(), 0x8007);
    assert_eq!(cpu.cycles(), 3); // 2 base + 1 for branch taken
}

#[test]
fn test_bmi_branch_not_taken() {
    let mut cpu = setup_cpu();

    // BMI +5 (0x30 0x05)
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x05);

    // Clear negative flag (branch should NOT be taken)
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // PC should just advance to next instruction (0x8000 + 2)
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2); // 2 base cycles only
}

#[test]
fn test_bmi_branch_backward() {
    let mut cpu = setup_cpu();

    // BMI -5 (0x30 0xFB) - 0xFB is -5 in signed 8-bit
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0xFB);

    // Ensure negative flag is set
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 (instruction size) - 5 (offset) = 0x7FFD
    // This crosses from page 0x80 to page 0x7F
    assert_eq!(cpu.pc(), 0x7FFD);
    assert_eq!(cpu.cycles(), 4); // 2 base + 1 for branch taken + 1 for page cross
}

#[test]
fn test_bmi_zero_offset() {
    let mut cpu = setup_cpu();

    // BMI +0 (0x30 0x00) - branch to next instruction
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 + 0 = 0x8002
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3); // 2 base + 1 for branch taken, no page cross
}

// ========== Page Crossing Tests ==========

#[test]
fn test_bmi_page_cross_forward() {
    // Position at 0x80FE, branch forward by 5 bytes
    // PC after instruction: 0x8100, target: 0x8105 (both on page 0x81)
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFE);
    memory.write(0xFFFD, 0x80);

    // BMI +5 (0x30 0x05)
    memory.write(0x80FE, 0x30);
    memory.write(0x80FF, 0x05);

    let mut cpu = CPU::new(memory);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x80FE + 2 + 5 = 0x8105 (no page cross, both on 0x81)
    assert_eq!(cpu.pc(), 0x8105);
    assert_eq!(cpu.cycles(), 3); // 2 base + 1 branch, no page cross
}

#[test]
fn test_bmi_page_cross_backward() {
    // Position at 0x8100, branch backward by 5 bytes
    // This should cross from page 0x81 to page 0x80
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x81);

    // BMI -5 (0x30 0xFB)
    memory.write(0x8100, 0x30);
    memory.write(0x8101, 0xFB);

    let mut cpu = CPU::new(memory);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8100 + 2 - 5 = 0x80FD (crosses page boundary)
    assert_eq!(cpu.pc(), 0x80FD);
    assert_eq!(cpu.cycles(), 4); // 2 base + 1 branch + 1 page cross
}

#[test]
fn test_bmi_no_page_cross_same_page() {
    let mut cpu = setup_cpu();

    // BMI +10 (0x30 0x0A) - stays within same page
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x0A);

    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 + 10 = 0x800C (same page)
    assert_eq!(cpu.pc(), 0x800C);
    assert_eq!(cpu.cycles(), 3); // 2 base + 1 branch, no page cross
}

#[test]
fn test_bmi_page_cross_boundary_exact() {
    // Position at 0x80FD, branch forward by 2
    // Result: 0x80FD + 2 + 2 = 0x8101 (crosses to next page)
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFD);
    memory.write(0xFFFD, 0x80);

    // BMI +2 (0x30 0x02)
    memory.write(0x80FD, 0x30);
    memory.write(0x80FE, 0x02);

    let mut cpu = CPU::new(memory);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x8101);
    assert_eq!(cpu.cycles(), 4); // 2 base + 1 branch + 1 page cross
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_bmi_preserves_all_flags() {
    let mut cpu = setup_cpu();

    // BMI +5 (0x30 0x05)
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x05);

    // Set various flags (negative must be set for branch)
    cpu.set_flag_n(true);
    cpu.set_flag_z(true);
    cpu.set_flag_c(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(cpu.flag_n());
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_v());
}

#[test]
fn test_bmi_preserves_flags_branch_not_taken() {
    let mut cpu = setup_cpu();

    // BMI +5 (0x30 0x05)
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x05);

    // Set flags with negative clear (branch not taken)
    cpu.set_flag_n(false);
    cpu.set_flag_z(true);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(!cpu.flag_n());
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
}

// ========== Edge Case Tests ==========

#[test]
fn test_bmi_max_forward_offset() {
    let mut cpu = setup_cpu();

    // BMI +127 (0x30 0x7F) - maximum positive offset
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 + 127 = 0x8081
    assert_eq!(cpu.pc(), 0x8081);
}

#[test]
fn test_bmi_max_backward_offset() {
    let mut cpu = setup_cpu();

    // BMI -128 (0x30 0x80) - maximum negative offset
    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should be 0x8000 + 2 - 128 = 0x7F82
    assert_eq!(cpu.pc(), 0x7F82);
}

#[test]
fn test_bmi_wrapping_behavior() {
    // Start near top of memory
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFE);
    memory.write(0xFFFD, 0xFF);

    // BMI +5 (0x30 0x05)
    memory.write(0xFFFE, 0x30);
    memory.write(0xFFFF, 0x05);

    let mut cpu = CPU::new(memory);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // PC should wrap: 0xFFFE + 2 + 5 = 0x10005 -> wraps to 0x0005
    assert_eq!(cpu.pc(), 0x0005);
}

// ========== Cycle Count Verification ==========

#[test]
fn test_bmi_cycles_branch_not_taken() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_flag_n(false); // Branch not taken

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

#[test]
fn test_bmi_cycles_branch_taken_no_page_cross() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x30);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_flag_n(true); // Branch taken

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    assert_eq!(cpu.cycles() - initial_cycles, 3);
}

#[test]
fn test_bmi_cycles_branch_taken_with_page_cross() {
    // Set up to actually cross page boundary
    // Position at 0x80FD, branch forward by 4
    // PC after instruction: 0x80FF, target: 0x8103 (crosses from 0x80 to 0x81)
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFD);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FD, 0x30);
    memory.write(0x80FE, 0x04);

    let mut cpu = CPU::new(memory);
    cpu.set_flag_n(true); // Branch taken with page cross

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    assert_eq!(cpu.cycles() - initial_cycles, 4);
}
