//! Memory bus trait tests
//!
//! Verifies that the MemoryBus trait implementation works correctly.

use cpu6502::{FlatMemory, MemoryBus};

#[test]
fn test_flat_memory_initialization() {
    let memory = FlatMemory::new();

    // All memory should be initialized to zero
    for addr in [0x0000, 0x1234, 0x8000, 0xFFFF].iter() {
        assert_eq!(
            memory.read(*addr),
            0x00,
            "Memory at 0x{:04X} should be initialized to 0",
            addr
        );
    }
}

#[test]
fn test_flat_memory_read_write_round_trip() {
    let mut memory = FlatMemory::new();

    // Test various addresses and values
    let test_data = [
        (0x0000, 0x01),
        (0x00FF, 0xFF),
        (0x0100, 0x7F),
        (0x1234, 0x42),
        (0x8000, 0xAB),
        (0xFFFF, 0xCD),
    ];

    for &(addr, value) in &test_data {
        memory.write(addr, value);
        assert_eq!(
            memory.read(addr),
            value,
            "Memory at 0x{:04X} should contain 0x{:02X}",
            addr,
            value
        );
    }
}

#[test]
fn test_flat_memory_independence() {
    let mut memory = FlatMemory::new();

    // Write to different addresses
    memory.write(0x1000, 0xAA);
    memory.write(0x2000, 0xBB);
    memory.write(0x3000, 0xCC);

    // Verify each address maintains its own value
    assert_eq!(memory.read(0x1000), 0xAA);
    assert_eq!(memory.read(0x2000), 0xBB);
    assert_eq!(memory.read(0x3000), 0xCC);

    // Verify adjacent addresses are unaffected
    assert_eq!(memory.read(0x0FFF), 0x00);
    assert_eq!(memory.read(0x1001), 0x00);
    assert_eq!(memory.read(0x1FFF), 0x00);
    assert_eq!(memory.read(0x2001), 0x00);
}

#[test]
fn test_flat_memory_overwrites() {
    let mut memory = FlatMemory::new();

    // Write initial value
    memory.write(0x5000, 0x11);
    assert_eq!(memory.read(0x5000), 0x11);

    // Overwrite with new value
    memory.write(0x5000, 0x22);
    assert_eq!(memory.read(0x5000), 0x22);

    // Overwrite again
    memory.write(0x5000, 0x33);
    assert_eq!(memory.read(0x5000), 0x33);
}

#[test]
fn test_flat_memory_full_address_space() {
    let mut memory = FlatMemory::new();

    // Test boundary addresses
    memory.write(0x0000, 0x00);
    memory.write(0x7FFF, 0x7F);
    memory.write(0x8000, 0x80);
    memory.write(0xFFFF, 0xFF);

    assert_eq!(memory.read(0x0000), 0x00);
    assert_eq!(memory.read(0x7FFF), 0x7F);
    assert_eq!(memory.read(0x8000), 0x80);
    assert_eq!(memory.read(0xFFFF), 0xFF);
}

#[test]
fn test_memory_bus_with_cpu() {
    use cpu6502::CPU;

    let mut memory = FlatMemory::new();

    // Set up reset vector and a test value
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x10);
    memory.write(0x1000, 0x42);

    let cpu = CPU::new(memory);

    // CPU should have loaded PC from reset vector
    assert_eq!(cpu.pc(), 0x1000);
}
