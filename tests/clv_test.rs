//! Comprehensive tests for the CLV (Clear Overflow Flag) instruction.
//!
//! Tests cover:
//! - Basic CLV operation clears overflow flag
//! - Overflow flag cleared when already clear
//! - Overflow flag cleared when set
//! - All addressing modes (Implied only for CLV)
//! - Processor flags updated correctly (V set to 0, others unchanged)
//! - Correct cycle counts
//! - PC advancement

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic CLV Operation Tests ==========

#[test]
fn test_clv_clears_overflow_flag_when_set() {
    let mut cpu = setup_cpu();

    // CLV (0xB8)
    cpu.memory_mut().write(0x8000, 0xB8);

    // Set overflow flag
    cpu.set_flag_v(true);
    assert!(cpu.flag_v());

    cpu.step().unwrap();

    // Overflow flag should be cleared
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 2); // 2 cycles
}

#[test]
fn test_clv_clears_overflow_flag_when_already_clear() {
    let mut cpu = setup_cpu();

    // CLV (0xB8)
    cpu.memory_mut().write(0x8000, 0xB8);

    // Ensure overflow flag is already clear
    cpu.set_flag_v(false);

    cpu.step().unwrap();

    // Overflow flag should remain cleared
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_clv_preserves_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set zero flag and overflow flag
    cpu.set_flag_z(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Zero flag should be unchanged
    assert!(cpu.flag_z());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set negative flag and overflow flag
    cpu.set_flag_n(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Negative flag should be unchanged
    assert!(cpu.flag_n());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set interrupt disable flag (already set by default) and overflow flag
    cpu.set_flag_i(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Interrupt disable flag should be unchanged
    assert!(cpu.flag_i());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set decimal mode flag and overflow flag
    cpu.set_flag_d(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Decimal mode flag should be unchanged
    assert!(cpu.flag_d());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set carry flag and overflow flag
    cpu.set_flag_c(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Carry flag should be unchanged
    assert!(cpu.flag_c());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set break flag and overflow flag
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Break flag should be unchanged
    assert!(cpu.flag_b());
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_all_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set all flags to known values
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // Only overflow flag should be cleared, all others unchanged
    assert!(cpu.flag_c()); // Unchanged
    assert!(cpu.flag_z()); // Unchanged
    assert!(cpu.flag_i()); // Unchanged
    assert!(cpu.flag_d()); // Unchanged
    assert!(cpu.flag_b()); // Unchanged
    assert!(!cpu.flag_v()); // Cleared
    assert!(cpu.flag_n()); // Unchanged
}

// ========== Register Preservation Tests ==========

#[test]
fn test_clv_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set accumulator to a known value
    cpu.set_a(0x42);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Accumulator should be unchanged
    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set X register to a known value
    cpu.set_x(0x33);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // X register should be unchanged
    assert_eq!(cpu.x(), 0x33);
    assert!(!cpu.flag_v());
}

#[test]
fn test_clv_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);

    // Set Y register to a known value
    cpu.set_y(0x44);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Y register should be unchanged
    assert_eq!(cpu.y(), 0x44);
    assert!(!cpu.flag_v());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_clv_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);
    cpu.set_flag_v(true);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // CLV should take exactly 2 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

#[test]
fn test_clv_cycle_count_when_already_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);
    cpu.set_flag_v(false);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // CLV should take exactly 2 cycles even when overflow is already clear
    assert_eq!(cpu.cycles() - initial_cycles, 2);
}

// ========== PC Advancement Tests ==========

#[test]
fn test_clv_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xB8);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of CLV instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_clv_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0xB8); // CLV at page boundary

    let mut cpu = CPU::new(memory);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
    assert!(!cpu.flag_v());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_clv_implied_addressing_mode() {
    let mut cpu = setup_cpu();

    // CLV uses implied addressing mode (opcode 0xB8)
    cpu.memory_mut().write(0x8000, 0xB8);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_clv_multiple_executions() {
    let mut cpu = setup_cpu();

    // Write multiple CLV instructions
    cpu.memory_mut().write(0x8000, 0xB8);
    cpu.memory_mut().write(0x8001, 0xB8);
    cpu.memory_mut().write(0x8002, 0xB8);

    cpu.set_flag_v(true);

    // First execution
    cpu.step().unwrap();
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);

    // Second execution (overflow already clear)
    cpu.step().unwrap();
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);

    // Third execution
    cpu.step().unwrap();
    assert!(!cpu.flag_v());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}
