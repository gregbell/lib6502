//! Memory-mapped device support for the 6502 emulator.
//!
//! This module provides a flexible memory mapping architecture that allows multiple
//! hardware devices (RAM, ROM, UART, future I/O) to be attached to the 6502 memory bus.
//!
//! # Architecture
//!
//! - **Device trait**: Abstract interface for memory-mapped hardware components
//! - **MappedMemory**: Routes read/write operations to registered devices based on address ranges
//! - **Device implementations**: RAM, ROM, UART (6551 ACIA), and future expansion
//!
//! # Example
//!
//! ```rust
//! use lib6502::{CPU, MappedMemory, RamDevice, RomDevice};
//!
//! // Create memory mapper
//! let mut memory = MappedMemory::new();
//!
//! // Add 16KB RAM at 0x0000-0x3FFF
//! memory.add_device(0x0000, Box::new(RamDevice::new(16384))).unwrap();
//!
//! // Add 16KB ROM at 0xC000-0xFFFF
//! let rom_data = vec![0xEA; 16384]; // NOP instructions
//! memory.add_device(0xC000, Box::new(RomDevice::new(rom_data))).unwrap();
//!
//! // Create CPU with mapped memory
//! let cpu = CPU::new(memory);
//! ```

use crate::MemoryBus;

// Device implementations
pub mod ram;
pub mod rom;

// Re-export device types
pub use ram::RamDevice;
pub use rom::RomDevice;

/// Abstract interface for memory-mapped hardware devices.
///
/// Devices implement this trait to provide read/write access to their internal
/// registers and state. The memory mapper calls these methods with offset-based
/// addressing (0 to size-1) to maintain device independence from mapped address.
///
/// # Design
///
/// - **Offset-based**: Device receives offset (0 to size-1), not absolute address
/// - **No panics**: All operations must succeed or fail gracefully
/// - **Immutable read**: Allows shared read access
/// - **Mutable write**: Explicit side effects (buffer updates, flag changes)
///
/// # Examples
///
/// ```rust
/// use lib6502::Device;
///
/// struct SimpleRam {
///     data: Vec<u8>,
/// }
///
/// impl Device for SimpleRam {
///     fn read(&self, offset: u16) -> u8 {
///         self.data[offset as usize]
///     }
///
///     fn write(&mut self, offset: u16, value: u8) {
///         self.data[offset as usize] = value;
///     }
///
///     fn size(&self) -> u16 {
///         self.data.len() as u16
///     }
/// }
/// ```
pub trait Device {
    /// Read byte from device at offset relative to device base address.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset within device's address space (0 to size-1)
    ///
    /// # Returns
    ///
    /// The byte value at the specified offset
    fn read(&self, offset: u16) -> u8;

    /// Write byte to device at offset relative to device base address.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset within device's address space (0 to size-1)
    /// * `value` - Byte value to write
    fn write(&mut self, offset: u16, value: u8);

    /// Return size of device's address space in bytes.
    ///
    /// # Returns
    ///
    /// Number of bytes in device's address range
    fn size(&self) -> u16;
}

/// Internal mapping of a device to a base address.
struct DeviceMapping {
    base_addr: u16,
    device: Box<dyn Device>,
}

/// Error returned when device registration fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
    /// Address range overlaps with an existing device.
    OverlapError {
        /// Base address of the new device
        new_base: u16,
        /// Size of the new device
        new_size: u16,
        /// Base address of the conflicting existing device
        existing_base: u16,
        /// Size of the conflicting existing device
        existing_size: u16,
    },
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeviceError::OverlapError {
                new_base,
                new_size,
                existing_base,
                existing_size,
            } => {
                write!(
                    f,
                    "Device address range overlap: new device at 0x{:04X}-0x{:04X} overlaps with existing device at 0x{:04X}-0x{:04X}",
                    new_base,
                    new_base.saturating_add(*new_size).saturating_sub(1),
                    existing_base,
                    existing_base.saturating_add(*existing_size).saturating_sub(1)
                )
            }
        }
    }
}

impl std::error::Error for DeviceError {}

/// Memory mapper that routes read/write operations to registered devices.
///
/// `MappedMemory` implements the `MemoryBus` trait and dispatches memory accesses
/// to the appropriate device based on address ranges. Unmapped addresses return
/// a default value (0xFF by default, mimicking 6502 floating bus behavior).
///
/// # Address Routing
///
/// When the CPU reads or writes to an address:
/// 1. Iterate through registered devices
/// 2. Check if address falls within device's range (base_addr to base_addr+size-1)
/// 3. If found, call device's read/write with offset (addr - base_addr)
/// 4. If not found, return unmapped_value (reads) or ignore (writes)
///
/// # Examples
///
/// ```rust
/// use lib6502::{MappedMemory, RamDevice, MemoryBus};
///
/// let mut memory = MappedMemory::new();
///
/// // Add 16KB RAM at 0x0000
/// memory.add_device(0x0000, Box::new(RamDevice::new(16384))).unwrap();
///
/// // Access memory through MemoryBus trait
/// memory.write(0x1234, 0x42);
/// assert_eq!(memory.read(0x1234), 0x42);
///
/// // Unmapped address returns 0xFF
/// assert_eq!(memory.read(0x8000), 0xFF);
/// ```
pub struct MappedMemory {
    devices: Vec<DeviceMapping>,
    unmapped_value: u8,
}

impl MappedMemory {
    /// Create a new empty memory mapper.
    ///
    /// # Returns
    ///
    /// A new `MappedMemory` instance with no devices and unmapped reads returning 0xFF.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::MappedMemory;
    ///
    /// let memory = MappedMemory::new();
    /// ```
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            unmapped_value: 0xFF, // Classic 6502 floating bus behavior
        }
    }

    /// Register a device at the specified base address.
    ///
    /// The device will occupy addresses from `base_addr` to `base_addr + device.size() - 1`.
    /// Registration fails if the new device's address range overlaps with any existing device.
    ///
    /// # Arguments
    ///
    /// * `base_addr` - Starting address for the device in the memory map
    /// * `device` - Boxed device implementation
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Device registered successfully
    /// * `Err(DeviceError::OverlapError)` - Address range overlaps with existing device
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::{MappedMemory, RamDevice};
    ///
    /// let mut memory = MappedMemory::new();
    ///
    /// // Add 16KB RAM at 0x0000-0x3FFF
    /// memory.add_device(0x0000, Box::new(RamDevice::new(16384))).unwrap();
    ///
    /// // This will fail (overlap)
    /// let result = memory.add_device(0x1000, Box::new(RamDevice::new(1024)));
    /// assert!(result.is_err());
    /// ```
    pub fn add_device(
        &mut self,
        base_addr: u16,
        device: Box<dyn Device>,
    ) -> Result<(), DeviceError> {
        let new_size = device.size();
        let new_end = base_addr.saturating_add(new_size);

        // Check for overlaps with existing devices
        for mapping in &self.devices {
            let existing_end = mapping.base_addr.saturating_add(mapping.device.size());

            // Check if ranges overlap
            // Range 1: [base_addr, new_end)
            // Range 2: [mapping.base_addr, existing_end)
            // Overlap if: base_addr < existing_end AND new_end > mapping.base_addr
            if base_addr < existing_end && new_end > mapping.base_addr {
                return Err(DeviceError::OverlapError {
                    new_base: base_addr,
                    new_size,
                    existing_base: mapping.base_addr,
                    existing_size: mapping.device.size(),
                });
            }
        }

        // No overlap, add the device
        self.devices.push(DeviceMapping { base_addr, device });
        Ok(())
    }

    /// Find device that handles the given address and return mutable reference with offset.
    ///
    /// # Arguments
    ///
    /// * `addr` - Absolute memory address (0x0000-0xFFFF)
    ///
    /// # Returns
    ///
    /// * `Some((&mut dyn Device, offset))` - Device and offset if address is mapped
    /// * `None` - If address is not mapped to any device
    fn find_device(&mut self, addr: u16) -> Option<(&mut dyn Device, u16)> {
        for mapping in &mut self.devices {
            let size = mapping.device.size();
            let (end_addr, overflow) = mapping.base_addr.overflowing_add(size);

            // Check if address is in device range
            // If overflow, device extends to 0xFFFF (inclusive)
            // Otherwise, device extends to end_addr (exclusive)
            let in_range = if overflow {
                addr >= mapping.base_addr
            } else {
                addr >= mapping.base_addr && addr < end_addr
            };

            if in_range {
                let offset = addr - mapping.base_addr;
                return Some((mapping.device.as_mut(), offset));
            }
        }
        None
    }

    /// Find device that handles the given address and return immutable reference with offset.
    ///
    /// # Arguments
    ///
    /// * `addr` - Absolute memory address (0x0000-0xFFFF)
    ///
    /// # Returns
    ///
    /// * `Some((&dyn Device, offset))` - Device and offset if address is mapped
    /// * `None` - If address is not mapped to any device
    fn find_device_immut(&self, addr: u16) -> Option<(&dyn Device, u16)> {
        for mapping in &self.devices {
            let size = mapping.device.size();
            let (end_addr, overflow) = mapping.base_addr.overflowing_add(size);

            // Check if address is in device range
            // If overflow, device extends to 0xFFFF (inclusive)
            // Otherwise, device extends to end_addr (exclusive)
            let in_range = if overflow {
                addr >= mapping.base_addr
            } else {
                addr >= mapping.base_addr && addr < end_addr
            };

            if in_range {
                let offset = addr - mapping.base_addr;
                return Some((mapping.device.as_ref(), offset));
            }
        }
        None
    }
}

impl Default for MappedMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBus for MappedMemory {
    fn read(&self, addr: u16) -> u8 {
        if let Some((device, offset)) = self.find_device_immut(addr) {
            device.read(offset)
        } else {
            self.unmapped_value
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if let Some((device, offset)) = self.find_device(addr) {
            device.write(offset, value);
        }
        // Unmapped writes are silently ignored (matching 6502 hardware behavior)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test device for unit testing
    struct TestDevice {
        data: Vec<u8>,
    }

    impl TestDevice {
        fn new(size: u16) -> Self {
            Self {
                data: vec![0; size as usize],
            }
        }
    }

    impl Device for TestDevice {
        fn read(&self, offset: u16) -> u8 {
            self.data[offset as usize]
        }

        fn write(&mut self, offset: u16, value: u8) {
            self.data[offset as usize] = value;
        }

        fn size(&self) -> u16 {
            self.data.len() as u16
        }
    }

    #[test]
    fn test_mapped_memory_empty() {
        let memory = MappedMemory::new();
        // Unmapped reads return 0xFF
        assert_eq!(memory.read(0x0000), 0xFF);
        assert_eq!(memory.read(0x1234), 0xFF);
        assert_eq!(memory.read(0xFFFF), 0xFF);
    }

    #[test]
    fn test_mapped_memory_single_device() {
        let mut memory = MappedMemory::new();
        memory
            .add_device(0x1000, Box::new(TestDevice::new(256)))
            .unwrap();

        // Write and read from mapped device
        memory.write(0x1000, 0x42);
        assert_eq!(memory.read(0x1000), 0x42);

        memory.write(0x10FF, 0x99);
        assert_eq!(memory.read(0x10FF), 0x99);

        // Unmapped addresses still return 0xFF
        assert_eq!(memory.read(0x0FFF), 0xFF);
        assert_eq!(memory.read(0x1100), 0xFF);
    }

    #[test]
    fn test_mapped_memory_multiple_devices() {
        let mut memory = MappedMemory::new();

        // Add device 1 at 0x0000 (size 256)
        memory
            .add_device(0x0000, Box::new(TestDevice::new(256)))
            .unwrap();

        // Add device 2 at 0x1000 (size 256)
        memory
            .add_device(0x1000, Box::new(TestDevice::new(256)))
            .unwrap();

        // Write to both devices
        memory.write(0x0042, 0xAA);
        memory.write(0x1042, 0xBB);

        // Verify routing
        assert_eq!(memory.read(0x0042), 0xAA);
        assert_eq!(memory.read(0x1042), 0xBB);

        // Unmapped region
        assert_eq!(memory.read(0x0500), 0xFF);
    }

    #[test]
    fn test_overlap_detection() {
        let mut memory = MappedMemory::new();

        // Add device at 0x1000-0x10FF (256 bytes)
        memory
            .add_device(0x1000, Box::new(TestDevice::new(256)))
            .unwrap();

        // Try to add overlapping device at 0x1080 (overlaps middle)
        let result = memory.add_device(0x1080, Box::new(TestDevice::new(256)));
        assert!(result.is_err());

        // Try to add device that starts before and overlaps (0x0F80 + 256 = 0x1080)
        let result = memory.add_device(0x0F80, Box::new(TestDevice::new(256)));
        assert!(result.is_err());

        // Adjacent device should succeed (0x0F00 + 256 = 0x1000, exactly adjacent)
        let result = memory.add_device(0x0F00, Box::new(TestDevice::new(256)));
        assert!(result.is_ok());

        // Non-overlapping device should succeed
        let result = memory.add_device(0x2000, Box::new(TestDevice::new(256)));
        assert!(result.is_ok());
    }

    #[test]
    fn test_unmapped_write_ignored() {
        let mut memory = MappedMemory::new();

        // Writing to unmapped address should not panic
        memory.write(0x1234, 0x42);

        // Should still read as 0xFF (unmapped)
        assert_eq!(memory.read(0x1234), 0xFF);
    }
}
