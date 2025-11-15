//! Execution loop tests
//!
//! Verifies fetch-decode-execute cycle, error handling, and cycle counting.

use cpu6502::{ExecutionError, FlatMemory, MemoryBus, CPU};

#[test]
fn test_step_returns_unimplemented_error() {
    let mut memory = FlatMemory::new();

    // Set reset vector and place a SBC immediate instruction
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xE9); // SBC immediate opcode (not implemented)

    let mut cpu = CPU::new(memory);

    // Attempting to execute should return UnimplementedOpcode
    match cpu.step() {
        Err(ExecutionError::UnimplementedOpcode(0xE9)) => {
            // Expected error
        }
        Ok(()) => panic!("Expected UnimplementedOpcode error, got Ok"),
        Err(e) => panic!("Expected UnimplementedOpcode(0xE9), got {:?}", e),
    }
}

#[test]
fn test_step_increments_cycle_counter() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xEA); // NOP - 2 cycles

    let mut cpu = CPU::new(memory);
    let initial_cycles = cpu.cycles();

    let _ = cpu.step();

    // Cycles should have incremented even though instruction isn't implemented
    assert!(
        cpu.cycles() > initial_cycles,
        "Cycle counter should increment after step()"
    );
    assert_eq!(cpu.cycles(), initial_cycles + 2, "NOP should add 2 cycles");
}

#[test]
fn test_step_advances_program_counter() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xEA); // NOP - 1 byte instruction

    let mut cpu = CPU::new(memory);
    let initial_pc = cpu.pc();

    let _ = cpu.step();

    // PC should have advanced by instruction size
    assert_eq!(
        cpu.pc(),
        initial_pc + 1,
        "PC should advance by instruction size"
    );
}

#[test]
fn test_step_with_multi_byte_instruction() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xA9); // LDA immediate - 2 bytes
    memory.write(0x8001, 0x42); // Operand

    let mut cpu = CPU::new(memory);
    let initial_pc = cpu.pc();

    let _ = cpu.step();

    // PC should have advanced by 2 (opcode + operand)
    assert_eq!(
        cpu.pc(),
        initial_pc + 2,
        "PC should advance by 2 for LDA immediate"
    );
}

#[test]
fn test_run_for_cycles_executes_multiple_instructions() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    // Fill with NOP instructions (2 cycles each)
    for addr in 0x8000..=0x8010 {
        memory.write(addr, 0xEA);
    }

    let mut cpu = CPU::new(memory);
    let initial_cycles = cpu.cycles();

    // Try to run for 10 cycles
    // Note: In this foundation feature, all instructions are unimplemented,
    // so run_for_cycles will stop on the first UnimplementedOpcode error
    let _ = cpu.run_for_cycles(10);

    // Should have executed at least one instruction
    assert!(
        cpu.cycles() >= initial_cycles + 2,
        "Should have executed at least one instruction (2 cycles)"
    );
}

#[test]
fn test_pc_wraps_at_boundary() {
    let mut memory = FlatMemory::new();

    // Set reset vector to near the end of address space
    memory.write(0xFFFC, 0xFE);
    memory.write(0xFFFD, 0xFF);
    memory.write(0xFFFE, 0xEA); // NOP at 0xFFFE
    memory.write(0xFFFF, 0xEA); // NOP at 0xFFFF
    memory.write(0x0000, 0xEA); // NOP at 0x0000 (after wrap)

    let mut cpu = CPU::new(memory);
    assert_eq!(cpu.pc(), 0xFFFE);

    // Execute instruction at 0xFFFE
    let _ = cpu.step();
    assert_eq!(cpu.pc(), 0xFFFF);

    // Execute instruction at 0xFFFF
    let _ = cpu.step();

    // PC should wrap to 0x0000
    assert_eq!(cpu.pc(), 0x0000, "PC should wrap from 0xFFFF to 0x0000");
}

#[test]
fn test_different_opcode_cycle_costs() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xEA); // NOP - 2 cycles
    memory.write(0x8001, 0x00); // BRK - 7 cycles

    let mut cpu = CPU::new(memory);

    // Execute NOP
    let _ = cpu.step();
    assert_eq!(cpu.cycles(), 2, "NOP should cost 2 cycles");

    // Execute BRK
    let _ = cpu.step();
    assert_eq!(cpu.cycles(), 9, "BRK should cost 7 cycles (total 9)");
}

#[test]
fn test_error_contains_opcode_value() {
    let mut memory = FlatMemory::new();

    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    memory.write(0x8000, 0xE9); // SBC immediate (not yet implemented)

    let mut cpu = CPU::new(memory);

    match cpu.step() {
        Err(ExecutionError::UnimplementedOpcode(opcode)) => {
            assert_eq!(opcode, 0xE9, "Error should contain the opcode value");
        }
        _ => panic!("Expected UnimplementedOpcode error"),
    }
}
