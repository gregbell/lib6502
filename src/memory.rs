//! # Memory Bus Abstraction
//!
//! This module provides the `MemoryBus` trait that decouples the CPU from specific
//! memory implementations. This enables flexible memory configurations including:
//!
//! - Flat 64KB RAM (FlatMemory implementation provided)
//! - Memory-mapped I/O
//! - ROM/RAM splits
//! - Banked memory systems
//! - Debugging wrappers with logging
//!
//! ## Design Principles
//!
//! The MemoryBus trait follows 6502 hardware behavior:
//! - No bus errors - reads/writes always succeed
//! - Unmapped reads may return garbage
//! - Writes to ROM/unmapped regions may be ignored
//! - Simple signatures for WASM compatibility

/// Memory bus trait for CPU to read/write bytes.
///
/// Implementations of this trait provide the memory backend for the CPU.
/// The CPU accesses all memory (RAM, ROM, I/O) through this abstraction.
///
/// # Design
///
/// - `read(&self)`: Immutable reference allows shared reads
/// - `write(&mut self)`: Mutable reference makes side effects explicit
/// - No error types: 6502 hardware has no bus error mechanism
///
/// # Examples
///
/// ```
/// use lib6502::{MemoryBus, FlatMemory};
///
/// let mut mem = FlatMemory::new();
///
/// // Write a value
/// mem.write(0x1234, 0x42);
///
/// // Read it back
/// assert_eq!(mem.read(0x1234), 0x42);
/// ```
///
/// ## Implementing Custom Memory
///
/// ```
/// use lib6502::MemoryBus;
///
/// struct RomRamMemory {
///     ram: [u8; 0x8000],  // 32KB RAM (0x0000-0x7FFF)
///     rom: [u8; 0x8000],  // 32KB ROM (0x8000-0xFFFF)
/// }
///
/// impl RomRamMemory {
///     pub fn new() -> Self {
///         Self {
///             ram: [0; 0x8000],
///             rom: [0; 0x8000],
///         }
///     }
/// }
///
/// impl MemoryBus for RomRamMemory {
///     fn read(&self, addr: u16) -> u8 {
///         if addr < 0x8000 {
///             self.ram[addr as usize]
///         } else {
///             self.rom[(addr - 0x8000) as usize]
///         }
///     }
///
///     fn write(&mut self, addr: u16, value: u8) {
///         if addr < 0x8000 {
///             // Writes to RAM succeed
///             self.ram[addr as usize] = value;
///         }
///         // Writes to ROM (0x8000+) are silently ignored
///     }
/// }
/// ```
pub trait MemoryBus {
    /// Reads a byte from the specified 16-bit address.
    ///
    /// This method must never panic. If the address is unmapped or invalid,
    /// implementations may return garbage data (matching 6502 hardware behavior).
    ///
    /// # Arguments
    ///
    /// * `addr` - 16-bit memory address (0x0000-0xFFFF)
    ///
    /// # Returns
    ///
    /// The byte value at the specified address
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::{MemoryBus, FlatMemory};
    ///
    /// let mem = FlatMemory::new();
    /// let value = mem.read(0x1234);
    /// ```
    fn read(&self, addr: u16) -> u8;

    /// Writes a byte to the specified 16-bit address.
    ///
    /// This method must never panic. If the address is read-only or unmapped,
    /// implementations may ignore the write (matching 6502 hardware behavior).
    ///
    /// # Arguments
    ///
    /// * `addr` - 16-bit memory address (0x0000-0xFFFF)
    /// * `value` - Byte value to write
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::{MemoryBus, FlatMemory};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0x1234, 0xFF);
    /// assert_eq!(mem.read(0x1234), 0xFF);
    /// ```
    fn write(&mut self, addr: u16, value: u8);

    /// Checks if the IRQ (Interrupt Request) line is active.
    ///
    /// This method returns `true` if any memory-mapped device has a pending
    /// interrupt request. The CPU calls this method after each instruction
    /// to determine if an interrupt should be serviced.
    ///
    /// # Hardware Semantics
    ///
    /// The IRQ line on the 6502 is **level-sensitive** and **shared** among
    /// all devices:
    ///
    /// - **Level-sensitive**: IRQ reflects current device state, not edges
    /// - **Logical OR**: IRQ is active if ANY device has pending interrupt
    /// - **Active until cleared**: IRQ remains active until ALL devices clear
    ///
    /// # Default Implementation
    ///
    /// Returns `false` (no interrupts) for simple memory implementations
    /// (like FlatMemory) that don't support interrupt-capable devices.
    ///
    /// Memory mappers with interrupt-capable devices should override this
    /// to check all registered devices.
    ///
    /// # Returns
    ///
    /// - `true` if at least one device has a pending interrupt
    /// - `false` if no devices have pending interrupts
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::{MemoryBus, FlatMemory};
    ///
    /// let mem = FlatMemory::new();
    /// // FlatMemory has no interrupt-capable devices
    /// assert_eq!(mem.irq_active(), false);
    /// ```
    ///
    /// # Performance
    ///
    /// This method is called after EVERY instruction, so implementations
    /// should be efficient (typically O(1) or O(n) where n = device count).
    fn irq_active(&self) -> bool {
        false // Default: no interrupts
    }
}

/// Simple 64KB flat memory implementation.
///
/// This is a straightforward memory implementation where all 65536 addresses
/// (0x0000-0xFFFF) are mapped to a single contiguous RAM array.
///
/// Useful for:
/// - Testing and development
/// - Simple programs that don't need ROM/RAM distinction
/// - Fantasy console applications
///
/// # Memory Layout
///
/// All addresses (0x0000-0xFFFF) are writable RAM initialized to 0x00.
///
/// # Examples
///
/// ```
/// use lib6502::{CPU, FlatMemory, MemoryBus};
///
/// // Create memory and set up reset vector
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00); // Reset vector low byte
/// memory.write(0xFFFD, 0x80); // Reset vector high byte (PC = 0x8000)
///
/// // Load a simple program at 0x8000
/// memory.write(0x8000, 0xEA); // NOP instruction (if it were implemented)
///
/// // Create CPU with this memory
/// let mut cpu = CPU::new(memory);
/// assert_eq!(cpu.pc(), 0x8000);
/// ```
pub struct FlatMemory {
    /// 64KB contiguous memory array
    data: Box<[u8; 65536]>,
}

impl FlatMemory {
    /// Creates a new FlatMemory instance with all bytes initialized to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::{FlatMemory, MemoryBus};
    ///
    /// let mem = FlatMemory::new();
    /// // All memory initially zero
    /// assert_eq!(mem.read(0x0000), 0x00);
    /// assert_eq!(mem.read(0xFFFF), 0x00);
    /// ```
    pub fn new() -> Self {
        Self {
            data: Box::new([0; 65536]),
        }
    }
}

impl Default for FlatMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBus for FlatMemory {
    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_memory_read_write() {
        let mut mem = FlatMemory::new();

        // Initially all zeros
        assert_eq!(mem.read(0x0000), 0x00);
        assert_eq!(mem.read(0xFFFF), 0x00);

        // Write and read back
        mem.write(0x1234, 0x42);
        assert_eq!(mem.read(0x1234), 0x42);

        // Verify other addresses unchanged
        assert_eq!(mem.read(0x1233), 0x00);
        assert_eq!(mem.read(0x1235), 0x00);
    }

    #[test]
    fn test_flat_memory_full_range() {
        let mut mem = FlatMemory::new();

        // Test boundary addresses
        mem.write(0x0000, 0x01);
        mem.write(0x7FFF, 0x7F);
        mem.write(0x8000, 0x80);
        mem.write(0xFFFF, 0xFF);

        assert_eq!(mem.read(0x0000), 0x01);
        assert_eq!(mem.read(0x7FFF), 0x7F);
        assert_eq!(mem.read(0x8000), 0x80);
        assert_eq!(mem.read(0xFFFF), 0xFF);
    }
}
