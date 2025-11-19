//! Test for demo/examples/uart-hello.asm program
//!
//! This test verifies that the uart-hello.asm program assembles correctly
//! and outputs "Hello, 6502!" followed by a newline to the UART.

use lib6502::assembler::assemble;
use lib6502::{MappedMemory, MemoryBus, RamDevice, RomDevice, Uart6551, CPU};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_uart_hello_program() {
    // Read the uart-hello.asm file
    let source = include_str!("../demo/examples/uart-hello.asm");

    // Prepend .org directive to assemble for ROM location
    let rom_base = 0xC000u16;
    let source_with_org = format!(".org ${:04X}\n{}", rom_base, source);

    // Assemble the program
    let result = assemble(&source_with_org);
    assert!(result.is_ok(), "Assembly should succeed");

    let output = result.unwrap();

    println!("Assembled {} bytes", output.bytes.len());
    println!("Assembly segments: {:?}", output.segments);
    println!("Symbol table: {:?}", output.symbol_table);

    // Create memory map
    let mut memory = MappedMemory::new();

    // Add RAM (32KB at 0x0000-0x7FFF) for zero-page, stack, and variables
    memory
        .add_device(0x0000, Box::new(RamDevice::new(32768)))
        .expect("Failed to add RAM");

    // Add UART at 0xA000 (as specified in uart-hello.asm)
    let mut uart = Uart6551::new();

    // Capture transmitted bytes
    let transmitted = Rc::new(RefCell::new(Vec::new()));
    let transmitted_clone = Rc::clone(&transmitted);

    uart.set_transmit_callback(move |byte| {
        transmitted_clone.borrow_mut().push(byte);
    });

    memory
        .add_device(0xA000, Box::new(uart))
        .expect("Failed to add UART");

    // Create ROM using the assembled bytes directly
    // The program is now assembled with .org $C000, so bytes can be loaded directly
    let mut rom_data = output.bytes.clone();

    // Expand ROM to 16KB and add reset vector at the end
    rom_data.resize(16384, 0x00); // Pad with zeros to 16KB

    // Set reset vector to point to ROM start (0xC000)
    // Vector is at 0xFFFC-0xFFFD, which is offset 0x3FFC-0x3FFD in our ROM
    rom_data[0x3FFC] = 0x00; // Low byte
    rom_data[0x3FFD] = 0xC0; // High byte

    // Set IRQ/BRK vector to point to itself (causes infinite BRK loop we can detect)
    // Vector is at 0xFFFE-0xFFFF, which is offset 0x3FFE-0x3FFF in our ROM
    rom_data[0x3FFE] = 0x00; // Low byte (points to 0x0000 where there's a BRK in RAM)
    rom_data[0x3FFF] = 0x00; // High byte

    memory
        .add_device(rom_base, Box::new(RomDevice::new(rom_data)))
        .expect("Failed to add ROM");

    // Create CPU
    let mut cpu = CPU::new(memory);

    println!("CPU initialized, PC = 0x{:04X}", cpu.pc());

    // Run the program - it should end with BRK
    let max_steps = 1000; // Safety limit
    let mut steps = 0;
    let mut brk_count = 0;

    loop {
        let pc = cpu.pc();
        let opcode = cpu.memory_mut().read(pc);

        // Detect BRK loop at 0x0000 (program finished)
        if pc == 0x0000 && opcode == 0x00 {
            brk_count += 1;
            if brk_count > 5 {
                println!("Detected BRK loop at 0x0000 after {} steps - program finished", steps);
                break;
            }
        }

        // Print first few steps for debugging
        if steps < 20 {
            println!(
                "Step {}: PC=0x{:04X} opcode=0x{:02X} A=0x{:02X} X=0x{:02X} Y=0x{:02X}",
                steps, pc, opcode, cpu.a(), cpu.x(), cpu.y()
            );
        }

        match cpu.step() {
            Ok(_) => {
                steps += 1;
                if steps >= max_steps {
                    println!("Transmitted so far: {:?}", transmitted.borrow());
                    panic!("Program exceeded maximum steps (possible infinite loop)");
                }
            }
            Err(e) => {
                println!("CPU stopped after {} steps: {:?}", steps, e);
                break;
            }
        }
    }

    // Get transmitted bytes
    let output_bytes = transmitted.borrow().clone();

    println!("Transmitted {} bytes: {:?}", output_bytes.len(), output_bytes);

    // Convert to string for easier verification
    let output_string = String::from_utf8_lossy(&output_bytes);
    println!("Output string: {:?}", output_string);

    // Verify the output
    let expected = b"Hello, 6502!\r\n";
    assert_eq!(
        output_bytes.as_slice(),
        expected,
        "UART output should be 'Hello, 6502!' followed by CR LF"
    );
}
