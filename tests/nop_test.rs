//! Comprehensive tests for the NOP (No Operation) instruction.
//!
//! Tests cover:
//! - Basic NOP operation works correctly
//! - All addressing modes (Implicit only for NOP)
//! - Processor flags remain unchanged
//! - Registers remain unchanged
//! - Correct cycle counts
//! - PC advancement

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic NOP Operation Tests ==========

#[test]
fn test_nop_basic_operation() {
    let mut cpu = setup_cpu();

    // NOP (0xEA)
    cpu.memory_mut().write(0x8000, 0xEA);

    cpu.step().unwrap();

    // NOP should only advance PC and consume cycles
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 2); // 2 cycles
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_nop_preserves_all_flags_when_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Clear all flags (except I which is set by default on reset)
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_b());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
}

#[test]
fn test_nop_preserves_all_flags_when_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set all flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_nop_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set carry flag
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    // Carry flag should be unchanged
    assert!(cpu.flag_c());
}

#[test]
fn test_nop_preserves_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set zero flag
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    // Zero flag should be unchanged
    assert!(cpu.flag_z());
}

#[test]
fn test_nop_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set interrupt disable flag (already set by default)
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    // Interrupt disable flag should be unchanged
    assert!(cpu.flag_i());
}

#[test]
fn test_nop_preserves_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set decimal mode flag
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    // Decimal mode flag should be unchanged
    assert!(cpu.flag_d());
}

#[test]
fn test_nop_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set break flag
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    // Break flag should be unchanged
    assert!(cpu.flag_b());
}

#[test]
fn test_nop_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set overflow flag
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Overflow flag should be unchanged
    assert!(cpu.flag_v());
}

#[test]
fn test_nop_preserves_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set negative flag
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // Negative flag should be unchanged
    assert!(cpu.flag_n());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_nop_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set accumulator to a known value
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should be unchanged
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_nop_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set X register to a known value
    cpu.set_x(0x33);

    cpu.step().unwrap();

    // X register should be unchanged
    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_nop_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set Y register to a known value
    cpu.set_y(0x44);

    cpu.step().unwrap();

    // Y register should be unchanged
    assert_eq!(cpu.y(), 0x44);
}

#[test]
fn test_nop_preserves_all_registers() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Set all registers to known values
    cpu.set_a(0x11);
    cpu.set_x(0x22);
    cpu.set_y(0x33);

    cpu.step().unwrap();

    // All registers should be unchanged
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.x(), 0x22);
    assert_eq!(cpu.y(), 0x33);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_nop_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // NOP should take exactly 2 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

// ========== PC Advancement Tests ==========

#[test]
fn test_nop_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of NOP instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_nop_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0xEA); // NOP at page boundary

    let mut cpu = CPU::new(memory);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_nop_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // NOP uses implicit addressing mode (opcode 0xEA)
    cpu.memory_mut().write(0x8000, 0xEA);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_nop_multiple_executions() {
    let mut cpu = setup_cpu();

    // Write multiple NOP instructions
    cpu.memory_mut().write(0x8000, 0xEA);
    cpu.memory_mut().write(0x8001, 0xEA);
    cpu.memory_mut().write(0x8002, 0xEA);

    // First execution
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);

    // Second execution
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);

    // Third execution
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_nop_with_other_instructions() {
    let mut cpu = setup_cpu();

    // Write NOP followed by other instructions
    cpu.memory_mut().write(0x8000, 0xEA); // NOP
    cpu.memory_mut().write(0x8001, 0x18); // CLC
    cpu.memory_mut().write(0x8002, 0xEA); // NOP

    // Set carry flag
    cpu.set_flag_c(true);

    // Execute NOP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    assert!(cpu.flag_c()); // Carry flag still set

    // Execute CLC
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_c()); // Carry flag cleared

    // Execute NOP again
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert!(!cpu.flag_c()); // Carry flag still cleared
}

// ========== Memory Preservation Tests ==========

#[test]
fn test_nop_does_not_modify_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Write some values to memory
    cpu.memory_mut().write(0x0000, 0xAA);
    cpu.memory_mut().write(0x0100, 0xBB);
    cpu.memory_mut().write(0x1000, 0xCC);

    cpu.step().unwrap();

    // Memory values should remain unchanged
    assert_eq!(cpu.memory_mut().read(0x0000), 0xAA);
    assert_eq!(cpu.memory_mut().read(0x0100), 0xBB);
    assert_eq!(cpu.memory_mut().read(0x1000), 0xCC);
}

// ========== Stack Preservation Tests ==========

#[test]
fn test_nop_preserves_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xEA);

    // Note initial stack pointer (0xFD by default)
    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Stack pointer should be unchanged
    assert_eq!(cpu.sp(), initial_sp);
}
