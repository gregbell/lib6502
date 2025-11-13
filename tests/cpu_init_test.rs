//! CPU initialization tests
//!
//! Verifies that the CPU initializes correctly to 6502 reset state.

use cpu6502::{FlatMemory, MemoryBus, CPU};

#[test]
fn test_cpu_reset_values() {
    let mut memory = FlatMemory::new();

    // Set reset vector to 0x1234
    memory.write(0xFFFC, 0x34);
    memory.write(0xFFFD, 0x12);

    let cpu = CPU::new(memory);

    // Verify PC loaded from reset vector
    assert_eq!(cpu.pc(), 0x1234, "PC should be loaded from reset vector");

    // Verify initial register values
    assert_eq!(cpu.a(), 0x00, "Accumulator should be 0x00");
    assert_eq!(cpu.x(), 0x00, "X register should be 0x00");
    assert_eq!(cpu.y(), 0x00, "Y register should be 0x00");
    assert_eq!(cpu.sp(), 0xFD, "Stack pointer should be 0xFD");

    // Verify initial status flags
    assert_eq!(cpu.flag_i(), true, "Interrupt disable flag should be set");
    assert_eq!(cpu.flag_n(), false, "Negative flag should be clear");
    assert_eq!(cpu.flag_v(), false, "Overflow flag should be clear");
    assert_eq!(cpu.flag_b(), false, "Break flag should be clear");
    assert_eq!(cpu.flag_d(), false, "Decimal flag should be clear");
    assert_eq!(cpu.flag_z(), false, "Zero flag should be clear");
    assert_eq!(cpu.flag_c(), false, "Carry flag should be clear");

    // Verify cycle counter
    assert_eq!(cpu.cycles(), 0, "Cycle counter should start at 0");
}

#[test]
fn test_status_register_format() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    let cpu = CPU::new(memory);
    let status = cpu.status();

    // Verify bit 5 is always 1
    assert_eq!(status & 0b00100000, 0b00100000, "Bit 5 should always be 1");

    // Verify I flag is set (bit 2)
    assert_eq!(status & 0b00000100, 0b00000100, "I flag should be set on reset");

    // Status register should be 0x24 (00100100) on reset
    // Bit 5 = 1 (always), Bit 2 = 1 (I flag)
    assert_eq!(status, 0x24, "Status register should be 0x24 on reset");
}

#[test]
fn test_reset_vector_little_endian() {
    let mut memory = FlatMemory::new();

    // Test little-endian byte order
    // Reset vector 0xABCD = low byte 0xCD at 0xFFFC, high byte 0xAB at 0xFFFD
    memory.write(0xFFFC, 0xCD);
    memory.write(0xFFFD, 0xAB);

    let cpu = CPU::new(memory);
    assert_eq!(cpu.pc(), 0xABCD, "PC should correctly load little-endian reset vector");
}

#[test]
fn test_different_reset_vectors() {
    // Test various reset vector values
    let test_vectors = [0x0000, 0x8000, 0xC000, 0xFFFF];

    for &expected_pc in &test_vectors {
        let mut memory = FlatMemory::new();
        memory.write(0xFFFC, (expected_pc & 0xFF) as u8);
        memory.write(0xFFFD, (expected_pc >> 8) as u8);

        let cpu = CPU::new(memory);
        assert_eq!(
            cpu.pc(),
            expected_pc,
            "PC should be {} after reset",
            expected_pc
        );
    }
}
