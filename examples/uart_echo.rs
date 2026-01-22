//! Example demonstrating UART (6551 ACIA) device with echo mode.
//!
//! This example shows how to:
//! - Create a UART device and add it to the memory map
//! - Set up transmit callbacks for output handling
//! - Inject received bytes from an external source (e.g., terminal)
//! - Use echo mode to automatically retransmit received bytes
//! - Integrate UART with CPU for program-driven communication
//!
//! Memory layout:
//! - 0x0000-0x7FFF: 32KB RAM (program and data)
//! - 0x8000-0x8003: UART (4 registers)
//! - 0xC000-0xFFFF: 16KB ROM (reset vector)

use lib6502::{Device, MappedMemory, MemoryBus, RamDevice, RomDevice, Uart6551, CPU};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("6502 UART Echo Mode Example");
    println!("============================\n");

    // Create memory mapper
    let mut memory = MappedMemory::new();

    // Add 32KB RAM at 0x0000-0x7FFF
    println!("Adding 32KB RAM at 0x0000-0x7FFF");
    let mut ram = RamDevice::new(32768);

    // Program at 0x0200: Read from UART and write back (simple echo)
    // LDA $8000   ; Read from UART data register
    // STA $8000   ; Write back to UART data register
    // JMP $0200   ; Loop forever
    let program = vec![
        0xAD, 0x00, 0x80, // LDA $8000 (read UART)
        0x8D, 0x00, 0x80, // STA $8000 (write UART)
        0x4C, 0x00, 0x02, // JMP $0200 (loop)
    ];

    ram.load_bytes(0x0200, &program).unwrap();
    memory.add_device(0x0000, Box::new(ram)).unwrap();

    // Add UART at 0x8000
    println!("Adding UART at 0x8000-0x8003");
    let mut uart = Uart6551::new();

    // Set transmit callback to print to console
    uart.set_transmit_callback(|byte| {
        print!("{}", byte as char);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    });

    // Inject some received bytes (simulating terminal input)
    println!("Simulating terminal input: \"Hello UART!\"");
    let input = "Hello UART!\n";
    for &byte in input.as_bytes() {
        uart.receive_byte(byte);
    }

    // Check UART status
    println!("UART status register: 0x{:02X}", uart.status());
    println!(
        "  TDRE (bit 4): {}",
        if uart.status() & 0x10 != 0 {
            "1 (ready)"
        } else {
            "0"
        }
    );
    println!(
        "  RDRF (bit 3): {}",
        if uart.status() & 0x08 != 0 {
            "1 (data available)"
        } else {
            "0"
        }
    );
    println!("  Receive buffer length: {}\n", uart.rx_buffer_len());

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Add 16KB ROM at 0xC000-0xFFFF with reset vector
    println!("Adding 16KB ROM at 0xC000-0xFFFF");
    let mut rom_data = vec![0xEA; 16384]; // Fill with NOP

    // Set reset vector to point to 0x0200 (offset 0x3FFC-0x3FFD in ROM)
    rom_data[0x3FFC] = 0x00; // Low byte of 0x0200
    rom_data[0x3FFD] = 0x02; // High byte of 0x0200

    memory
        .add_device(0xC000, Box::new(RomDevice::new(rom_data)))
        .unwrap();

    // Create CPU
    let mut cpu = CPU::new(memory);

    println!("CPU initialized:");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  A:  0x{:02X}", cpu.a());
    println!("  SP: 0x{:02X}\n", cpu.sp());

    // Run program for a few iterations to demonstrate echo
    println!("Running UART echo program (reading and echoing buffered input)...\n");
    println!("Output: ");

    let max_iterations = 50; // Limit to prevent infinite loop in example
    for i in 0..max_iterations {
        match cpu.step() {
            Ok(()) => {
                // Check if we've read all the input
                // (In real usage, this would run continuously)
                if i > 0 && cpu.a() == 0x00 {
                    // No more data to read
                    break;
                }
            }
            Err(e) => {
                println!("\nError: {}", e);
                break;
            }
        }
    }

    println!("\n\nProgram statistics:");
    println!("  Total cycles: {}", cpu.cycles());
    println!("  Final PC: 0x{:04X}", cpu.pc());
    println!("  Final A: 0x{:02X}\n", cpu.a());

    // Demonstrate echo mode feature
    println!("Demonstrating UART echo mode...");
    println!("===============================\n");

    let mut memory2 = MappedMemory::new();
    memory2
        .add_device(0x0000, Box::new(RamDevice::new(32768)))
        .unwrap();

    let uart2 = Rc::new(RefCell::new(Uart6551::new()));

    // Set transmit callback
    uart2.borrow_mut().set_transmit_callback(|byte| {
        print!("{}", byte as char);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    });

    // Enable echo mode (bit 3 of command register)
    println!("Enabling echo mode (command register bit 3 = 1)");
    memory2
        .add_shared_device(0x8000, Rc::clone(&uart2))
        .unwrap();

    // Write to command register to enable echo
    memory2.write(0x8002, 0x08); // Set bit 3

    println!("Command register: 0x{:02X}\n", memory2.read(0x8002));

    // Now when we receive bytes, they should be automatically echoed
    println!("Receiving bytes with echo mode enabled:");
    println!("Input:  \"Echo test\"");
    print!("Output: ");

    // Receive bytes via the shared UART handle - they should automatically echo
    let test_input = "Echo test\n";
    for &byte in test_input.as_bytes() {
        uart2.borrow_mut().receive_byte(byte);
    }

    println!("\n\nEcho mode demonstration complete!");
    println!("\nKey UART features demonstrated:");
    println!("  ✓ Memory-mapped register access (0x8000-0x8003)");
    println!("  ✓ Transmit callback for output handling");
    println!("  ✓ Receive buffer for input queuing");
    println!("  ✓ Status register flags (TDRE, RDRF)");
    println!("  ✓ Echo mode for automatic retransmission");
    println!("  ✓ CPU integration for program-driven I/O");
}
