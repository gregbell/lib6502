//! Simple RAM example
//!
//! Demonstrates basic CPU initialization and execution with FlatMemory.
//!
//! This example shows:
//! - Creating a 64KB flat memory instance
//! - Setting up the reset vector
//! - Loading a simple program
//! - Initializing the CPU
//! - Executing instructions and inspecting state

use lib6502::{ExecutionError, FlatMemory, MemoryBus, CPU, OPCODE_TABLE};

fn main() {
    println!("6502 CPU Core Foundation - Simple RAM Example");
    println!("==============================================\n");

    // Create 64KB flat memory (all addresses mapped to RAM)
    let mut memory = FlatMemory::new();

    // Set reset vector to point to program start at 0x8000
    // Reset vector is stored at 0xFFFC/0xFFFD (little-endian)
    memory.write(0xFFFC, 0x00); // Low byte
    memory.write(0xFFFD, 0x80); // High byte (PC will be 0x8000)

    println!("Reset vector set to 0x8000");

    // Load a simple "program" (just placeholder opcodes for demonstration)
    // Note: No instructions are actually implemented in this foundational feature
    memory.write(0x8000, 0xEA); // NOP (Not implemented, but will show error)
    memory.write(0x8001, 0xA9); // LDA #$42 (Load immediate)
    memory.write(0x8002, 0x42); // Operand: 0x42
    memory.write(0x8003, 0x00); // BRK

    println!("Loaded placeholder program at 0x8000-0x8003\n");

    // Initialize CPU with the memory
    let mut cpu = CPU::new(memory);

    // Display initial CPU state
    println!("CPU Initial State:");
    println!("-----------------");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  SP: 0x{:02X} (Stack: 0x01{:02X})", cpu.sp(), cpu.sp());
    println!("  A:  0x{:02X}", cpu.a());
    println!("  X:  0x{:02X}", cpu.x());
    println!("  Y:  0x{:02X}", cpu.y());
    println!(
        "  Status: 0x{:02X} (NV-BDIZC: {:08b})",
        cpu.status(),
        cpu.status()
    );
    println!("  Flags:");
    println!("    N (Negative):         {}", cpu.flag_n());
    println!("    V (Overflow):         {}", cpu.flag_v());
    println!("    B (Break):            {}", cpu.flag_b());
    println!("    D (Decimal):          {}", cpu.flag_d());
    println!("    I (Interrupt Disable): {}", cpu.flag_i());
    println!("    Z (Zero):             {}", cpu.flag_z());
    println!("    C (Carry):            {}", cpu.flag_c());
    println!("  Cycles: {}\n", cpu.cycles());

    // Attempt to execute a few instructions
    println!("Attempting to execute instructions:");
    println!("-----------------------------------");

    for step in 1..=3 {
        let pc_before = cpu.pc();
        let cycles_before = cpu.cycles();

        match cpu.step() {
            Ok(()) => {
                println!(
                    "Step {}: Successfully executed instruction at 0x{:04X}",
                    step, pc_before
                );
            }
            Err(ExecutionError::UnimplementedOpcode(opcode)) => {
                let metadata = &OPCODE_TABLE[opcode as usize];
                let cycles_consumed = cpu.cycles() - cycles_before;
                let pc_after = cpu.pc();

                println!(
                    "Step {}: Opcode 0x{:02X} ({}) at 0x{:04X}",
                    step, opcode, metadata.mnemonic, pc_before
                );
                println!("        Status: NOT IMPLEMENTED (expected in this foundation feature)");
                println!(
                    "        Mode: {:?}, Cycles: {}, Size: {} bytes",
                    metadata.addressing_mode, metadata.base_cycles, metadata.size_bytes
                );
                println!(
                    "        PC advanced: 0x{:04X} -> 0x{:04X} (+{})",
                    pc_before,
                    pc_after,
                    pc_after.wrapping_sub(pc_before)
                );
                println!("        Cycles consumed: {}\n", cycles_consumed);
            }
        }
    }

    // Display final CPU state
    println!("CPU Final State:");
    println!("----------------");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  Cycles: {}", cpu.cycles());

    println!("\nExample complete!");
    println!("\nNote: This is the foundational feature - no instructions are");
    println!("actually implemented yet. All opcodes return UnimplementedOpcode");
    println!("errors. Future features will implement specific instructions.");
}
