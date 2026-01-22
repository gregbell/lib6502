//! Test for demo/examples/uart-interrupt-advanced.asm program

use lib6502::assembler::assemble;
use lib6502::{Device, MappedMemory, MemoryBus, RamDevice, RomDevice, Uart6551, CPU};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_uart_irq_advanced_program() {
    let source = include_str!("../demo/examples/uart-interrupt-advanced.asm");
    let rom_base = 0xC000u16;
    let source_with_org = format!(".org ${:04X}\n{}", rom_base, source);

    let result = assemble(&source_with_org);
    assert!(result.is_ok(), "Assembly should succeed: {:?}", result.err());

    let output = result.unwrap();
    println!("Assembled {} bytes", output.bytes.len());
    println!("Symbol table: {:?}", output.symbol_table);

    // Create memory map
    let mut memory = MappedMemory::new();
    memory.add_device(0x0000, Box::new(RamDevice::new(32768))).unwrap();

    let mut uart = Uart6551::new();
    let transmitted = Rc::new(RefCell::new(Vec::new()));
    let transmitted_clone = Rc::clone(&transmitted);
    uart.set_transmit_callback(move |byte| {
        transmitted_clone.borrow_mut().push(byte);
    });
    memory.add_device(0xA000, Box::new(uart)).unwrap();

    let mut rom_data = output.bytes.clone();
    rom_data.resize(15872, 0x00);
    memory.add_device(rom_base, Box::new(RomDevice::new(rom_data))).unwrap();

    let mut vector_ram = RamDevice::new(256);
    vector_ram.write(0xFC, 0x00);
    vector_ram.write(0xFD, 0xC0);
    memory.add_device(0xFF00, Box::new(vector_ram)).unwrap();

    let mut cpu = CPU::new(memory);
    println!("CPU initialized at PC=0x{:04X}", cpu.pc());

    // Run initialization until we hit idle loop
    let mut found_idle = false;
    for i in 0..100 {
        let pc = cpu.pc();
        let opcode = cpu.memory_mut().read(pc);
        if i < 20 {
            println!("Init step {}: PC=0x{:04X} opcode=0x{:02X}", i, pc, opcode);
        }
        if opcode == 0xEA {
            println!("Found idle loop at step {}, PC=0x{:04X}", i, pc);
            found_idle = true;
            cpu.step().unwrap();
            break;
        }
        cpu.step().unwrap();
    }
    assert!(found_idle, "Should reach idle loop");
    assert!(!cpu.flag_i(), "I flag should be clear");

    println!("\n=== Sending character 'A' ===");
    
    let uart = cpu.memory_mut().get_device_at_mut::<Uart6551>(0xA000).unwrap();
    uart.receive_byte(b'A');
    
    let irq_active = cpu.memory_mut().irq_active();
    println!("IRQ active: {}", irq_active);
    assert!(irq_active, "IRQ should be active");

    // Run until character is echoed
    for i in 0..200 {
        let pc = cpu.pc();
        if i < 50 {
            let opcode = cpu.memory_mut().read(pc);
            println!("Step {}: PC=0x{:04X} opcode=0x{:02X} A=0x{:02X}", 
                     i, pc, opcode, cpu.a());
        }
        
        if let Err(e) = cpu.step() {
            panic!("CPU error at step {}, PC=0x{:04X}: {:?}", i, pc, e);
        }
        
        if !transmitted.borrow().is_empty() {
            println!("Character transmitted at step {}", i);
            break;
        }
    }

    let output = transmitted.borrow().clone();
    println!("Transmitted: {:?}", output);
    assert_eq!(output, vec![b'A'], "Should echo 'A'");
    
    println!("\n=== Test passed! ===");
}
