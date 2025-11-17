//! Integration tests for UART (6551 ACIA) device functionality.
//!
//! These tests verify the UART device works correctly when integrated with
//! MappedMemory and the CPU.

use lib6502::{MappedMemory, MemoryBus, RamDevice, Uart6551, CPU};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_uart_device_registration() {
    let mut memory = MappedMemory::new();

    // Add UART at 0x8000 (typical I/O region)
    let uart = Uart6551::new();
    memory
        .add_device(0x8000, Box::new(uart))
        .expect("Failed to register UART");

    // Verify UART is accessible (4 registers)
    // Status register should have TDRE (bit 4) set by default
    assert_eq!(memory.read(0x8001) & 0x10, 0x10);
}

#[test]
fn test_uart_transmit_via_mapped_memory() {
    let mut memory = MappedMemory::new();
    let mut uart = Uart6551::new();

    let transmitted = Rc::new(RefCell::new(Vec::new()));
    let transmitted_clone = Rc::clone(&transmitted);

    uart.set_transmit_callback(move |byte| {
        transmitted_clone.borrow_mut().push(byte);
    });

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Write to data register (offset 0)
    memory.write(0x8000, b'H');
    memory.write(0x8000, b'e');
    memory.write(0x8000, b'l');
    memory.write(0x8000, b'l');
    memory.write(0x8000, b'o');

    assert_eq!(*transmitted.borrow(), b"Hello");
}

#[test]
fn test_uart_receive_via_mapped_memory() {
    let mut memory = MappedMemory::new();
    let mut uart = Uart6551::new();

    // Inject received bytes before adding to memory map
    uart.receive_byte(b'A');
    uart.receive_byte(b'B');
    uart.receive_byte(b'C');

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Read status register - RDRF should be set (bit 3)
    let status = memory.read(0x8001);
    assert_eq!(status & 0x08, 0x08);

    // Read data register
    // Now correctly pops from buffer (FIXED: uses interior mutability)
    let data = memory.read(0x8000);
    assert_eq!(data, b'A'); // First byte (FIFO order)

    // Read again - should get B
    let data = memory.read(0x8000);
    assert_eq!(data, b'B');

    // Read again - should get C
    let data = memory.read(0x8000);
    assert_eq!(data, b'C');

    // Buffer now empty - RDRF should be clear
    let status = memory.read(0x8001);
    assert_eq!(status & 0x08, 0x00); // RDRF clear
}

#[test]
fn test_uart_with_cpu_integration() {
    let mut memory = MappedMemory::new();

    // Add RAM at 0x0000-0x7FFF for program and stack
    let mut ram = RamDevice::new(32768);

    // Simple program to test UART:
    // LDA #$42    ; Load 0x42
    // STA $8000   ; Write to UART data register
    // LDA #$FF    ; Load 0xFF
    // STA $8000   ; Write to UART data register
    // BRK         ; Stop (will cause UnimplementedOpcode)
    let program = vec![
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x80, // STA $8000
        0xA9, 0xFF, // LDA #$FF
        0x8D, 0x00, 0x80, // STA $8000
        0x00, // BRK (stop execution)
    ];

    ram.load_bytes(0x0200, &program).unwrap();

    memory.add_device(0x0000, Box::new(ram)).unwrap();

    // Add UART at 0x8000
    let mut uart = Uart6551::new();

    let transmitted = Rc::new(RefCell::new(Vec::new()));
    let transmitted_clone = Rc::clone(&transmitted);

    uart.set_transmit_callback(move |byte| {
        transmitted_clone.borrow_mut().push(byte);
    });

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Add ROM at 0xC000 with reset vector
    let mut rom = vec![0; 16384];
    // Reset vector points to 0x0200 (where program is loaded in RAM)
    rom[0x3FFC] = 0x00; // Low byte of 0x0200
    rom[0x3FFD] = 0x02; // High byte of 0x0200
    memory
        .add_device(0xC000, Box::new(lib6502::RomDevice::new(rom)))
        .unwrap();

    // Create CPU
    let mut cpu = CPU::new(memory);

    // Verify PC initialized to 0x0200 from reset vector
    assert_eq!(cpu.pc(), 0x0200);

    // Execute program
    cpu.step().unwrap(); // LDA #$42
    cpu.step().unwrap(); // STA $8000
    cpu.step().unwrap(); // LDA #$FF
    cpu.step().unwrap(); // STA $8000

    // Verify transmitted data
    assert_eq!(*transmitted.borrow(), vec![0x42, 0xFF]);
}

#[test]
fn test_uart_register_read_write() {
    let mut memory = MappedMemory::new();
    let uart = Uart6551::new();

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Write to command register (offset 2)
    memory.write(0x8002, 0xAA);
    assert_eq!(memory.read(0x8002), 0xAA);

    // Write to control register (offset 3)
    memory.write(0x8003, 0x55);
    assert_eq!(memory.read(0x8003), 0x55);

    // Try to write to status register (offset 1) - should be ignored
    let initial_status = memory.read(0x8001);
    memory.write(0x8001, 0xFF);
    assert_eq!(memory.read(0x8001), initial_status);
}

#[test]
fn test_uart_status_flags() {
    let mut memory = MappedMemory::new();
    let mut uart = Uart6551::new();

    // Check initial status (TDRE should be set)
    let initial_status = uart.status();
    assert_eq!(initial_status & 0x10, 0x10); // TDRE (bit 4)
    assert_eq!(initial_status & 0x08, 0x00); // RDRF (bit 3) not set

    // Inject received byte
    uart.receive_byte(0x42);

    // RDRF should now be set
    let status_after_rx = uart.status();
    assert_eq!(status_after_rx & 0x08, 0x08); // RDRF set

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // Verify status via memory read
    assert_eq!(memory.read(0x8001) & 0x08, 0x08);
}

#[test]
fn test_uart_multiple_devices() {
    let mut memory = MappedMemory::new();

    // Add first UART at 0x8000
    let uart1 = Uart6551::new();
    memory.add_device(0x8000, Box::new(uart1)).unwrap();

    // Add second UART at 0x8004 (right after first)
    let uart2 = Uart6551::new();
    memory.add_device(0x8004, Box::new(uart2)).unwrap();

    // Both should be accessible
    // Write to first UART's command register
    memory.write(0x8002, 0x11);
    assert_eq!(memory.read(0x8002), 0x11);

    // Write to second UART's command register
    memory.write(0x8006, 0x22);
    assert_eq!(memory.read(0x8006), 0x22);

    // Verify they're independent
    assert_eq!(memory.read(0x8002), 0x11);
    assert_eq!(memory.read(0x8006), 0x22);
}

#[test]
fn test_uart_in_realistic_memory_layout() {
    let mut memory = MappedMemory::new();

    // Realistic layout:
    // 0x0000-0x7FFF: RAM (32KB)
    // 0x8000-0x8003: UART (4 bytes)
    // 0xC000-0xFFFF: ROM (16KB)

    memory
        .add_device(0x0000, Box::new(RamDevice::new(32768)))
        .unwrap();

    let mut uart = Uart6551::new();
    let transmitted = Rc::new(RefCell::new(Vec::new()));
    let transmitted_clone = Rc::clone(&transmitted);

    uart.set_transmit_callback(move |byte| {
        transmitted_clone.borrow_mut().push(byte);
    });

    memory.add_device(0x8000, Box::new(uart)).unwrap();

    // ROM with program
    let mut rom_data = vec![0; 16384];

    // Program: LDA #$42, STA $8000 (transmit via UART)
    rom_data[0] = 0xA9;
    rom_data[1] = 0x42;
    rom_data[2] = 0x8D;
    rom_data[3] = 0x00;
    rom_data[4] = 0x80;

    // Reset vector points to 0xC000
    rom_data[0x3FFC] = 0x00;
    rom_data[0x3FFD] = 0xC0;

    memory
        .add_device(0xC000, Box::new(lib6502::RomDevice::new(rom_data)))
        .unwrap();

    // Create CPU and execute
    let mut cpu = CPU::new(memory);

    assert_eq!(cpu.pc(), 0xC000);

    cpu.step().unwrap(); // LDA #$42
    cpu.step().unwrap(); // STA $8000

    assert_eq!(*transmitted.borrow(), vec![0x42]);
}
