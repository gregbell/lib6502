//! Test for demo/examples/uart-echo.asm program
//!
//! This test verifies that the uart-echo.asm program assembles correctly
//! and properly echoes received characters using interrupt-driven I/O.

use lib6502::assembler::assemble;
use lib6502::{Device, MappedMemory, MemoryBus, RamDevice, RomDevice, Uart6551, CPU};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_uart_echo_program() {
    // Read the uart-echo.asm file
    let source = include_str!("../demo/examples/uart-echo.asm");

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

    // Add UART at 0xA000 (as specified in uart-echo.asm)
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

    // Expand ROM to 15872 bytes (0xC000-0xFEFF, excludes vector page)
    rom_data.resize(15872, 0x00); // Pad with zeros

    memory
        .add_device(rom_base, Box::new(RomDevice::new(rom_data)))
        .expect("Failed to add ROM");

    // Add RAM at 0xFF00-0xFFFF (256 bytes) for writable vector page
    // This allows programs to write their interrupt vectors
    let mut vector_ram = RamDevice::new(256);
    // Set reset vector to point to ROM start (0xC000)
    // Vector is at 0xFFFC-0xFFFD, which is offset 0xFC-0xFD in this RAM device
    vector_ram.write(0xFC, 0x00); // Low byte
    vector_ram.write(0xFD, 0xC0); // High byte
    // IRQ vector at 0xFFFE-0xFFFF (offset 0xFE-0xFF) will be written by the program

    memory
        .add_device(0xFF00, Box::new(vector_ram))
        .expect("Failed to add vector RAM");

    // Create CPU
    let mut cpu = CPU::new(memory);

    println!("CPU initialized, PC = 0x{:04X}", cpu.pc());
    println!("Initial I flag = {}", cpu.flag_i());

    // Run the initialization part of the program
    // The program:
    // 1. Sets IRQ vector at 0xFFFE-0xFFFF
    // 2. Enables UART interrupts
    // 3. Clears I flag with CLI
    // 4. Enters idle loop

    let max_init_steps = 100;
    let mut steps = 0;
    let mut idle_loop_pc = None;

    // Run until we hit the idle loop (NOP, JMP idle_loop pattern)
    while steps < max_init_steps {
        let pc = cpu.pc();
        let opcode = cpu.memory_mut().read(pc);

        if steps < 20 {
            println!(
                "Init step {}: PC=0x{:04X} opcode=0x{:02X} A=0x{:02X} X=0x{:02X} I={}",
                steps,
                pc,
                opcode,
                cpu.a(),
                cpu.x(),
                cpu.flag_i()
            );
        }

        // Detect idle loop: NOP (0xEA) followed by JMP
        if opcode == 0xEA {
            idle_loop_pc = Some(pc);
            println!("Found idle loop at PC=0x{:04X}", pc);
            cpu.step().unwrap(); // Execute the NOP
            steps += 1;
            break;
        }

        match cpu.step() {
            Ok(_) => {
                steps += 1;
            }
            Err(e) => {
                panic!("CPU error during initialization: {:?}", e);
            }
        }
    }

    assert!(
        idle_loop_pc.is_some(),
        "Should have reached idle loop during initialization"
    );
    assert!(
        !cpu.flag_i(),
        "I flag should be clear (interrupts enabled)"
    );

    println!("\n=== Initialization complete, now testing echo functionality ===\n");

    // Get mutable access to UART to inject received bytes
    let uart = cpu
        .memory_mut()
        .get_device_at_mut::<Uart6551>(0xA000)
        .expect("UART should exist");

    // Inject a test character 'A'
    println!("Injecting byte 'A' (0x41) into UART...");
    uart.receive_byte(b'A');

    // Check that interrupt is pending
    let has_interrupt = cpu.memory_mut().irq_active();
    println!("IRQ active after receive: {}", has_interrupt);
    assert!(has_interrupt, "UART should signal interrupt");

    // Now run the CPU - it should handle the interrupt
    let mut interrupt_handled = false;
    let max_interrupt_steps = 100;

    for step in 0..max_interrupt_steps {
        let pc = cpu.pc();
        let opcode = cpu.memory_mut().read(pc);

        if step < 30 {
            println!(
                "Step {}: PC=0x{:04X} opcode=0x{:02X} A=0x{:02X} cycles={}",
                step,
                pc,
                opcode,
                cpu.a(),
                cpu.cycles()
            );
        }

        // Check if we've transmitted anything
        if !transmitted.borrow().is_empty() {
            println!("Character echoed! Breaking execution loop.");
            interrupt_handled = true;
            break;
        }

        match cpu.step() {
            Ok(_) => {}
            Err(e) => {
                panic!("CPU error during interrupt handling: {:?}", e);
            }
        }
    }

    assert!(
        interrupt_handled,
        "Interrupt should have been handled and character echoed"
    );

    // Verify the echoed output
    let output_bytes = transmitted.borrow().clone();
    println!("Transmitted bytes: {:?}", output_bytes);

    assert_eq!(
        output_bytes.as_slice(),
        &[b'A'],
        "Should have echoed the character 'A'"
    );

    println!("\n=== Testing multiple characters ===\n");

    // Clear transmitted buffer
    transmitted.borrow_mut().clear();

    // Inject more characters
    let test_string = b"Hello!";
    for &byte in test_string {
        // Get mutable access to UART
        let uart = cpu
            .memory_mut()
            .get_device_at_mut::<Uart6551>(0xA000)
            .expect("UART should exist");

        println!("Injecting byte '{}'", byte as char);
        uart.receive_byte(byte);

        // Run until this character is echoed
        let initial_count = transmitted.borrow().len();
        for _ in 0..200 {
            cpu.step().unwrap();
            if transmitted.borrow().len() > initial_count {
                break;
            }
        }
    }

    let final_output = transmitted.borrow().clone();
    println!("Final transmitted bytes: {:?}", final_output);
    println!(
        "Final transmitted string: {:?}",
        String::from_utf8_lossy(&final_output)
    );

    assert_eq!(
        final_output.as_slice(),
        test_string,
        "Should have echoed all characters in order"
    );

    println!("\n=== Test passed! ===");
}
