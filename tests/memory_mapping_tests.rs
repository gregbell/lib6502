//! Integration tests for memory mapping functionality.
//!
//! These tests verify that the memory mapping system works correctly with RAM,
//! ROM, and the CPU integration.

use lib6502::{DeviceError, MappedMemory, MemoryBus, RamDevice, RomDevice, CPU};

#[test]
fn test_ram_device_basic_read_write() {
    let mut memory = MappedMemory::new();

    // Add 1KB RAM at 0x0000
    memory
        .add_device(0x0000, Box::new(RamDevice::new(1024)))
        .unwrap();

    // Write through MemoryBus trait
    memory.write(0x0000, 0x42);
    memory.write(0x0100, 0xAA);
    memory.write(0x03FF, 0xFF);

    // Read back
    assert_eq!(memory.read(0x0000), 0x42);
    assert_eq!(memory.read(0x0100), 0xAA);
    assert_eq!(memory.read(0x03FF), 0xFF);

    // Verify unmapped addresses return 0xFF
    assert_eq!(memory.read(0x0400), 0xFF);
    assert_eq!(memory.read(0x1000), 0xFF);
}

#[test]
fn test_rom_device_read_only() {
    let mut memory = MappedMemory::new();

    // Create ROM with specific data
    let rom_data = vec![0x01, 0x02, 0x03, 0x04];
    memory
        .add_device(0x8000, Box::new(RomDevice::new(rom_data)))
        .unwrap();

    // Read works
    assert_eq!(memory.read(0x8000), 0x01);
    assert_eq!(memory.read(0x8001), 0x02);
    assert_eq!(memory.read(0x8002), 0x03);
    assert_eq!(memory.read(0x8003), 0x04);

    // Try to write (should be ignored)
    memory.write(0x8000, 0xFF);
    memory.write(0x8001, 0xFF);

    // Verify writes were ignored
    assert_eq!(memory.read(0x8000), 0x01);
    assert_eq!(memory.read(0x8001), 0x02);
}

#[test]
fn test_mapped_memory_routing() {
    let mut memory = MappedMemory::new();

    // Add RAM at 0x0000-0x3FFF (16KB)
    memory
        .add_device(0x0000, Box::new(RamDevice::new(16384)))
        .unwrap();

    // Add ROM at 0xC000-0xFFFF (16KB)
    let rom_data = vec![0xEA; 16384]; // NOP instructions
    memory
        .add_device(0xC000, Box::new(RomDevice::new(rom_data)))
        .unwrap();

    // Write to RAM
    memory.write(0x0000, 0x11);
    memory.write(0x1234, 0x22);
    memory.write(0x3FFF, 0x33);

    // Read from RAM
    assert_eq!(memory.read(0x0000), 0x11);
    assert_eq!(memory.read(0x1234), 0x22);
    assert_eq!(memory.read(0x3FFF), 0x33);

    // Read from ROM
    assert_eq!(memory.read(0xC000), 0xEA);
    assert_eq!(memory.read(0xD000), 0xEA);
    assert_eq!(memory.read(0xFFFF), 0xEA); // Last byte of address space

    // Unmapped region (0x4000-0xBFFF) returns 0xFF
    assert_eq!(memory.read(0x4000), 0xFF);
    assert_eq!(memory.read(0x8000), 0xFF);
    assert_eq!(memory.read(0xBFFF), 0xFF);

    // Verify ROM is read-only
    memory.write(0xC000, 0x42);
    assert_eq!(memory.read(0xC000), 0xEA); // Still original value
}

#[test]
fn test_unmapped_address_returns_ff() {
    let memory = MappedMemory::new();

    // No devices registered, all reads should return 0xFF
    assert_eq!(memory.read(0x0000), 0xFF);
    assert_eq!(memory.read(0x1234), 0xFF);
    assert_eq!(memory.read(0x8000), 0xFF);
    assert_eq!(memory.read(0xFFFF), 0xFF);
}

#[test]
fn test_overlapping_devices_rejected() {
    let mut memory = MappedMemory::new();

    // Add first device at 0x1000-0x10FF (256 bytes)
    memory
        .add_device(0x1000, Box::new(RamDevice::new(256)))
        .unwrap();

    // Try to add overlapping device (should fail)
    let result = memory.add_device(0x1080, Box::new(RamDevice::new(256)));
    assert!(result.is_err());

    match result {
        Err(DeviceError::OverlapError { .. }) => {
            // Expected error
        }
        _ => panic!("Expected OverlapError"),
    }

    // Try to add device that overlaps from before
    let result = memory.add_device(0x0F80, Box::new(RamDevice::new(256)));
    assert!(result.is_err());

    // Non-overlapping device should succeed
    let result = memory.add_device(0x2000, Box::new(RamDevice::new(256)));
    assert!(result.is_ok());

    // Adjacent device should succeed
    let result = memory.add_device(0x0F00, Box::new(RamDevice::new(256)));
    assert!(result.is_ok());
}

#[test]
fn test_cpu_with_mapped_memory() {
    let mut memory = MappedMemory::new();

    // Add lower 32KB RAM
    let mut ram_low = RamDevice::new(32768);
    // Load a simple program at 0x200: LDA #$42, STA $10
    ram_low
        .load_bytes(
            0x0200,
            &[
                0xA9, 0x42, // LDA #$42
                0x85, 0x10, // STA $10
            ],
        )
        .unwrap();
    memory.add_device(0x0000, Box::new(ram_low)).unwrap();

    // Add upper 32KB RAM
    let mut ram_high = RamDevice::new(32768);
    // Set reset vector to 0x0200 (0x7FFC-0x7FFD within device)
    ram_high.load_bytes(0x7FFC, &[0x00, 0x02]).unwrap();
    memory.add_device(0x8000, Box::new(ram_high)).unwrap();

    // Create CPU
    let mut cpu = CPU::new(memory);

    // Verify CPU initialized correctly (reset vector points to 0x0200)
    assert_eq!(cpu.pc(), 0x0200);

    // Step through LDA #$42
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);

    // Step through STA $10
    cpu.step().unwrap();

    // Verify memory was written
    assert_eq!(cpu.memory_mut().read(0x10), 0x42);
}

#[test]
fn test_ram_load_bytes_integration() {
    let mut memory = MappedMemory::new();

    // Create RAM and load program data
    let mut ram = RamDevice::new(16384);
    let program = vec![0xA9, 0xFF, 0x85, 0x00]; // LDA #$FF, STA $00
    ram.load_bytes(0x200, &program).unwrap();

    memory.add_device(0x0000, Box::new(ram)).unwrap();

    // Verify program was loaded correctly
    assert_eq!(memory.read(0x0200), 0xA9);
    assert_eq!(memory.read(0x0201), 0xFF);
    assert_eq!(memory.read(0x0202), 0x85);
    assert_eq!(memory.read(0x0203), 0x00);

    // Other addresses should be zero
    assert_eq!(memory.read(0x0000), 0x00);
    assert_eq!(memory.read(0x0204), 0x00);
}

#[test]
fn test_multiple_ram_regions() {
    let mut memory = MappedMemory::new();

    // Add zero page RAM (256 bytes)
    memory
        .add_device(0x0000, Box::new(RamDevice::new(256)))
        .unwrap();

    // Add stack RAM (256 bytes)
    memory
        .add_device(0x0100, Box::new(RamDevice::new(256)))
        .unwrap();

    // Add general RAM (8KB)
    memory
        .add_device(0x0200, Box::new(RamDevice::new(8192)))
        .unwrap();

    // Write to each region
    memory.write(0x0042, 0xAA); // Zero page
    memory.write(0x0142, 0xBB); // Stack
    memory.write(0x0242, 0xCC); // General

    // Verify reads
    assert_eq!(memory.read(0x0042), 0xAA);
    assert_eq!(memory.read(0x0142), 0xBB);
    assert_eq!(memory.read(0x0242), 0xCC);
}
