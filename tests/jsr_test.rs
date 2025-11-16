//! Comprehensive tests for the JSR (Jump to Subroutine) instruction.
//!
//! Tests cover:
//! - Basic JSR operation works correctly
//! - All addressing modes implemented:
//!   - Absolute (opcode 0x20)
//! - Correct cycle counts for all addressing modes
//! - Return address (PC+2) pushed to stack correctly
//! - Stack pointer updated correctly
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

// ========== Basic JSR Operation Tests ==========

#[test]
fn test_jsr_basic_operation() {
    let mut cpu = setup_cpu();

    // JSR $1234 (opcode 0x20)
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // PC should be set to $1234
    assert_eq!(cpu.pc(), 0x1234);

    // Stack pointer should have been decremented by 2 (PC high, PC low)
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));

    // Cycles should be 6
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jsr_to_zero_page() {
    let mut cpu = setup_cpu();

    // JSR $0042 - Jump to subroutine in zero page
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x8002, 0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x0042);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jsr_to_high_memory() {
    let mut cpu = setup_cpu();

    // JSR $FFFC - Jump to subroutine in high memory
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0xFC);
    cpu.memory_mut().write(0x8002, 0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0xFFFC);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jsr_same_page() {
    let mut cpu = setup_cpu();

    // JSR $8010 - Jump within same page
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x8002, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x8010);
    assert_eq!(cpu.cycles(), 6);
}

// ========== Stack Operation Tests ==========

#[test]
fn test_jsr_pushes_return_address_minus_one() {
    let mut cpu = setup_cpu();

    // JSR $1234 at $8000
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read the return address from stack
    // Stack grows downward: first push is at SP, then SP-1
    // Order: PC_high at SP, PC_low at SP-1
    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    let return_address = ((pc_high as u16) << 8) | (pc_low as u16);

    // JSR should push PC+2 (the address of the last byte of JSR)
    // This is 0x8000 + 2 = 0x8002
    assert_eq!(return_address, 0x8002);
}

#[test]
fn test_jsr_stack_push_order() {
    let mut cpu = setup_cpu();

    // JSR $1234 at $8000
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Stack grows downward, so:
    // SP+0: PC high byte (0x80)
    // SP-1: PC low byte (0x02)

    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));

    // Verify PC high byte
    assert_eq!(pc_high, 0x80, "High byte of PC+2 should be 0x80");

    // Verify PC low byte
    assert_eq!(pc_low, 0x02, "Low byte of PC+2 should be 0x02");
}

#[test]
fn test_jsr_stack_pointer_update() {
    let mut cpu = setup_cpu();

    // JSR $1234
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // SP should have decremented by 2 (one for each push)
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));
}

#[test]
fn test_jsr_stack_wrapping() {
    // Start with SP at 0x01, JSR will wrap SP to 0xFF
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    let mut cpu = CPU::new(memory);

    // JSR $1234
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    // Manually set SP to 0x01
    cpu.set_sp(0x01);

    cpu.step().unwrap();

    // SP should wrap: 0x01 - 2 = 0xFF (wrapping subtraction)
    assert_eq!(cpu.sp(), 0xFF);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_jsr_preserves_all_flags() {
    let mut cpu = setup_cpu();

    // JSR $1234
    cpu.memory_mut().write(0x8000, 0x20);
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
fn test_jsr_preserves_all_flags_clear() {
    let mut cpu = setup_cpu();

    // JSR $1234
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    // Clear all flags
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

// ========== Register Preservation Tests ==========

#[test]
fn test_jsr_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_jsr_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_x(0x55);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55);
}

#[test]
fn test_jsr_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    cpu.set_y(0x66);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x66);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_jsr_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // JSR should take exactly 6 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 6);
}

// ========== Complex Scenarios ==========

#[test]
fn test_jsr_chain() {
    let mut cpu = setup_cpu();

    // JSR $8003 at $8000
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x03);
    cpu.memory_mut().write(0x8002, 0x80);

    // JSR $9000 at $8003
    cpu.memory_mut().write(0x8003, 0x20);
    cpu.memory_mut().write(0x8004, 0x00);
    cpu.memory_mut().write(0x8005, 0x90);

    let initial_sp = cpu.sp();

    // First JSR
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));

    // Second JSR
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x9000);
    assert_eq!(cpu.cycles(), 12); // 6 + 6
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(4)); // 2 + 2
}

#[test]
fn test_jsr_nested_stack() {
    let mut cpu = setup_cpu();

    // JSR $1000 at $8000
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x10);

    // JSR $2000 at $1000
    cpu.memory_mut().write(0x1000, 0x20);
    cpu.memory_mut().write(0x1001, 0x00);
    cpu.memory_mut().write(0x1002, 0x20);

    let initial_sp = cpu.sp();

    // First JSR - pushes 0x8002
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x1000);

    // Check first return address on stack
    let ret1_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let ret1_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    assert_eq!(ret1_high, 0x80);
    assert_eq!(ret1_low, 0x02);

    // Second JSR - pushes 0x1002
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x2000);

    // Check second return address on stack
    let ret2_high = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(2) as u16));
    let ret2_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(3) as u16));
    assert_eq!(ret2_high, 0x10);
    assert_eq!(ret2_low, 0x02);

    // Stack should have 4 bytes pushed
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(4));
}

// ========== Edge Cases ==========

#[test]
fn test_jsr_to_same_address() {
    let mut cpu = setup_cpu();

    // JSR $8000 (jump to same instruction)
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x8000);
}

#[test]
fn test_jsr_across_page_boundaries() {
    let mut cpu = setup_cpu();

    // JSR $80FF -> $8100 (crosses page boundary)
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x81);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x81FF);
    assert_eq!(cpu.cycles(), 6); // No extra cycle for page crossing in JSR
}

#[test]
fn test_jsr_with_zero_address() {
    let mut cpu = setup_cpu();

    // JSR $0000 - jump to address 0
    cpu.memory_mut().write(0x8000, 0x20);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x0000);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_jsr_at_end_of_memory() {
    // JSR at 0xFFFD (last possible location for a 3-byte instruction)
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFD);
    memory.write(0xFFFD, 0xFF);

    let mut cpu = CPU::new(memory);

    // JSR $1234 at $FFFD
    cpu.memory_mut().write(0xFFFD, 0x20);
    cpu.memory_mut().write(0xFFFE, 0x34);
    cpu.memory_mut().write(0xFFFF, 0x12);

    cpu.set_pc(0xFFFD);
    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // PC should be $1234
    assert_eq!(cpu.pc(), 0x1234);

    // Return address should be $FFFF (0xFFFD + 2)
    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    let return_address = ((pc_high as u16) << 8) | (pc_low as u16);
    assert_eq!(return_address, 0xFFFF);
}

#[test]
fn test_jsr_return_address_for_various_locations() {
    let mut cpu = setup_cpu();

    // Test JSR at $1000
    cpu.set_pc(0x1000);
    cpu.memory_mut().write(0x1000, 0x20);
    cpu.memory_mut().write(0x1001, 0x00);
    cpu.memory_mut().write(0x1002, 0x20);

    let initial_sp = cpu.sp();

    cpu.step().unwrap();

    // Read return address
    let pc_high = cpu.memory_mut().read(0x0100 | (initial_sp as u16));
    let pc_low = cpu
        .memory_mut()
        .read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
    let return_address = ((pc_high as u16) << 8) | (pc_low as u16);

    // Should be $1002 (0x1000 + 2)
    assert_eq!(return_address, 0x1002);
}
