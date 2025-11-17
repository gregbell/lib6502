//! Example demonstrating a memory-mapped system with RAM and ROM.
//!
//! This example shows how to:
//! - Create a MappedMemory instance
//! - Add RAM and ROM devices at specific addresses
//! - Run a simple 6502 program that accesses both regions
//!
//! Memory layout:
//! - 0x0000-0x7FFF: 32KB RAM (writable)
//! - 0x8000-0xFFFF: 32KB ROM (read-only, contains program)

use lib6502::{RamDevice, RomDevice, MappedMemory, MemoryBus, CPU, ExecutionError};

fn main() {
    println!("6502 Memory-Mapped System Example");
    println!("==================================\n");

    // Create memory mapper
    let mut memory = MappedMemory::new();

    // Add 32KB RAM at 0x0000-0x7FFF
    println!("Adding 32KB RAM at 0x0000-0x7FFF");
    memory
        .add_device(0x0000, Box::new(RamDevice::new(32768)))
        .expect("Failed to add RAM");

    // Create ROM with a simple program
    let mut rom_data = vec![0; 32768]; // 32KB ROM

    // Program at 0x8000 (offset 0 in ROM):
    // LDA #$42    ; Load 0x42 into accumulator
    // STA $10     ; Store accumulator to address 0x10 (in RAM)
    // LDA #$FF    ; Load 0xFF into accumulator
    // STA $11     ; Store accumulator to address 0x11 (in RAM)
    // BRK         ; Stop
    let program = vec![
        0xA9, 0x42, // LDA #$42
        0x85, 0x10, // STA $10
        0xA9, 0xFF, // LDA #$FF
        0x85, 0x11, // STA $11
        0x00,       // BRK
    ];

    // Load program at beginning of ROM (will be at 0x8000 in memory map)
    rom_data[0..program.len()].copy_from_slice(&program);

    // Set reset vector to point to 0x8000 (offset 0x7FFC-0x7FFD in ROM)
    rom_data[0x7FFC] = 0x00; // Low byte of 0x8000
    rom_data[0x7FFD] = 0x80; // High byte of 0x8000

    println!("Adding 32KB ROM at 0x8000-0xFFFF");
    println!("Program loaded at 0x8000:\n  LDA #$42\n  STA $10\n  LDA #$FF\n  STA $11\n  BRK\n");

    memory
        .add_device(0x8000, Box::new(RomDevice::new(rom_data)))
        .expect("Failed to add ROM");

    // Create CPU with mapped memory
    let mut cpu = CPU::new(memory);

    println!("CPU initialized:");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  A:  0x{:02X}", cpu.a());
    println!("  SP: 0x{:02X}\n", cpu.sp());

    // Run program (step through each instruction)
    println!("Executing program...\n");
    let mut instruction_count = 0;
    loop {
        let pc_before = cpu.pc();
        match cpu.step() {
            Ok(()) => {
                instruction_count += 1;
                println!(
                    "Instruction {}: PC=0x{:04X}, A=0x{:02X}",
                    instruction_count,
                    pc_before,
                    cpu.a()
                );

                // Simple halt detection: if PC didn't advance (BRK), stop
                if cpu.pc() == pc_before {
                    break;
                }

                // Safety limit
                if instruction_count > 100 {
                    break;
                }
            }
            Err(ExecutionError::UnimplementedOpcode(_)) => {
                // BRK (0x00) is not implemented, this is expected
                break;
            }
        }
    }

    println!("\nProgram completed after {} instructions", instruction_count);
    println!("\nFinal CPU state:");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  A:  0x{:02X}", cpu.a());
    println!("  SP: 0x{:02X}", cpu.sp());

    // Verify memory writes
    println!("\nMemory contents:");
    println!("  0x0010: 0x{:02X} (expected 0x42)", cpu.memory_mut().read(0x10));
    println!("  0x0011: 0x{:02X} (expected 0xFF)", cpu.memory_mut().read(0x11));

    // Verify RAM is writable and ROM is read-only
    println!("\nVerifying device behavior:");
    cpu.memory_mut().write(0x0010, 0xAA);
    println!(
        "  After writing 0xAA to RAM (0x0010): 0x{:02X} (should be 0xAA)",
        cpu.memory_mut().read(0x10)
    );

    cpu.memory_mut().write(0x8000, 0x00);
    println!(
        "  After writing 0x00 to ROM (0x8000): 0x{:02X} (should still be 0xA9)",
        cpu.memory_mut().read(0x8000)
    );

    // Test unmapped address (there is no gap between RAM and ROM in this config)
    // All addresses 0x0000-0xFFFF are mapped, but let's verify ROM boundary
    println!(
        "  Reading last byte of RAM (0x7FFF): 0x{:02X}",
        cpu.memory_mut().read(0x7FFF)
    );
    println!(
        "  Reading first byte of ROM (0x8000): 0x{:02X} (program start)",
        cpu.memory_mut().read(0x8000)
    );

    println!("\nExample completed successfully!");
}
