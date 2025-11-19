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
use std::any::Any;

// Device implementations
pub mod interrupts;
pub mod ram;
pub mod rom;
pub mod uart;

// Re-export device types
pub use interrupts::InterruptDevice;
pub use ram::RamDevice;
pub use rom::RomDevice;
pub use uart::Uart6551;

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
/// use std::any::Any;
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
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn Any {
///         self
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

    /// Support for downcasting to concrete device types.
    ///
    /// This method enables safe downcasting from `&dyn Device` to `&T`
    /// where `T` is the concrete device type.
    fn as_any(&self) -> &dyn Any;

    /// Support for downcasting to concrete device types (mutable).
    ///
    /// This method enables safe downcasting from `&mut dyn Device` to `&mut T`
    /// where `T` is the concrete device type.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Cast this device to an `InterruptDevice` trait object if it supports interrupts.
    ///
    /// This method provides a clean way for the memory bus to check if a device
    /// is interrupt-capable without requiring all devices to implement interrupt
    /// functionality.
    ///
    /// # Default Implementation
    ///
    /// Returns `None` for devices that don't support interrupts (RAM, ROM, etc.).
    /// Interrupt-capable devices should override this to return `Some(self)`.
    ///
    /// # Returns
    ///
    /// - `Some(&dyn InterruptDevice)` if device supports interrupts
    /// - `None` if device doesn't support interrupts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::{Device, InterruptDevice};
    /// use std::any::Any;
    ///
    /// // Interrupt-capable device
    /// struct TimerDevice {
    ///     interrupt_pending: bool,
    /// }
    ///
    /// impl InterruptDevice for TimerDevice {
    ///     fn has_interrupt(&self) -> bool {
    ///         self.interrupt_pending
    ///     }
    /// }
    ///
    /// impl Device for TimerDevice {
    ///     fn read(&self, offset: u16) -> u8 { 0 }
    ///     fn write(&mut self, offset: u16, value: u8) { }
    ///     fn size(&self) -> u16 { 4 }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn as_any_mut(&mut self) -> &mut dyn Any { self }
    ///
    ///     // Override to expose interrupt capability
    ///     fn as_interrupt_device(&self) -> Option<&dyn InterruptDevice> {
    ///         Some(self)
    ///     }
    /// }
    /// ```
    fn as_interrupt_device(&self) -> Option<&dyn InterruptDevice> {
        None // Default: device doesn't support interrupts
    }
}

/// Helper for address range calculations and overlap detection.
///
/// Wraps a `RangeInclusive<u16>` to represent memory address ranges.
/// Uses inclusive ranges to properly handle devices that extend to 0xFFFF
/// without special overflow handling.
///
/// # Invariants
///
/// - The range is always non-empty (size >= 1)
/// - The range is inclusive on both ends: [start, end]
#[derive(Debug, Clone)]
struct AddressRange(std::ops::RangeInclusive<u16>);

impl AddressRange {
    /// Create a new address range from base address and size.
    ///
    /// # Arguments
    ///
    /// * `base` - Starting address of the range
    /// * `size` - Number of bytes in the range (must be > 0)
    ///
    /// # Returns
    ///
    /// An inclusive range [base, base+size-1]. Handles overflow correctly
    /// (e.g., base=0xE000, size=0x2000 creates range [0xE000, 0xFFFF]).
    fn new(base: u16, size: u16) -> Self {
        let (end_plus_one, overflowed) = base.overflowing_add(size);
        let end = if overflowed {
            // Device extends to end of address space
            0xFFFF
        } else {
            // Normal case: base+size-1
            end_plus_one.wrapping_sub(1)
        };
        Self(base..=end)
    }

    /// Check if an address falls within this range.
    #[inline]
    fn contains(&self, addr: u16) -> bool {
        self.0.contains(&addr)
    }

    /// Check if this range overlaps with another range.
    ///
    /// Two ranges overlap if they share any addresses. This is true when:
    /// - The start of one range is <= the end of the other, AND
    /// - The end of one range is >= the start of the other
    ///
    /// This works correctly for all cases, including ranges at 0xFFFF.
    fn overlaps(&self, other: &AddressRange) -> bool {
        // Standard interval overlap test: [a,b] overlaps [c,d] iff a <= d && b >= c
        self.0.start() <= other.0.end() && self.0.end() >= other.0.start()
    }
}

/// Internal mapping of a device to a base address.
struct DeviceMapping {
    base_addr: u16,
    device: Box<dyn Device>,
}

impl DeviceMapping {
    /// Get the address range for this device mapping.
    fn range(&self) -> AddressRange {
        AddressRange::new(self.base_addr, self.device.size())
    }
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
    #[must_use = "ignoring device registration errors can lead to silent failures"]
    pub fn add_device(
        &mut self,
        base_addr: u16,
        device: Box<dyn Device>,
    ) -> Result<(), DeviceError> {
        let new_range = AddressRange::new(base_addr, device.size());

        // Check for overlaps with existing devices
        for mapping in &self.devices {
            let existing_range = mapping.range();

            if new_range.overlaps(&existing_range) {
                return Err(DeviceError::OverlapError {
                    new_base: base_addr,
                    new_size: device.size(),
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
            let range = mapping.range();

            if range.contains(addr) {
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
            let range = mapping.range();

            if range.contains(addr) {
                let offset = addr - mapping.base_addr;
                return Some((mapping.device.as_ref(), offset));
            }
        }
        None
    }

    /// Get a reference to a device at a specific address, downcast to a concrete type.
    ///
    /// This method allows accessing device-specific methods after registration.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The concrete device type to downcast to
    ///
    /// # Arguments
    ///
    /// * `addr` - Address within the device's mapped range
    ///
    /// # Returns
    ///
    /// * `Some(&T)` - Reference to the device if found and successfully downcast
    /// * `None` - If no device at address or downcast fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::{MappedMemory, Uart6551};
    ///
    /// let mut memory = MappedMemory::new();
    /// let mut uart = Uart6551::new();
    ///
    /// uart.set_transmit_callback(|byte| {
    ///     println!("TX: 0x{:02X}", byte);
    /// });
    ///
    /// memory.add_device(0x8000, Box::new(uart)).unwrap();
    ///
    /// // Later, get device reference to check status
    /// if let Some(uart) = memory.get_device_at::<Uart6551>(0x8000) {
    ///     println!("UART status: 0x{:02X}", uart.status());
    /// }
    /// ```
    pub fn get_device_at<T: Device + 'static>(&self, addr: u16) -> Option<&T> {
        if let Some((device, _)) = self.find_device_immut(addr) {
            device.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }

    /// Get a mutable reference to a device at a specific address, downcast to a concrete type.
    ///
    /// This method allows accessing device-specific methods after registration.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The concrete device type to downcast to
    ///
    /// # Arguments
    ///
    /// * `addr` - Address within the device's mapped range
    ///
    /// # Returns
    ///
    /// * `Some(&mut T)` - Mutable reference to the device if found and successfully downcast
    /// * `None` - If no device at address or downcast fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::{MappedMemory, Uart6551};
    ///
    /// let mut memory = MappedMemory::new();
    /// memory.add_device(0x8000, Box::new(Uart6551::new())).unwrap();
    ///
    /// // Set callback after registration
    /// if let Some(uart) = memory.get_device_at_mut::<Uart6551>(0x8000) {
    ///     uart.set_transmit_callback(|byte| {
    ///         println!("TX: 0x{:02X}", byte);
    ///     });
    /// }
    /// ```
    pub fn get_device_at_mut<T: Device + 'static>(&mut self, addr: u16) -> Option<&mut T> {
        if let Some((device, _)) = self.find_device(addr) {
            device.as_any_mut().downcast_mut::<T>()
        } else {
            None
        }
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

    fn irq_active(&self) -> bool {
        // Level-Sensitive IRQ Line Implementation
        //
        // This method implements the 6502's level-sensitive IRQ line behavior by
        // performing a logical OR of all device interrupt flags. The IRQ line is
        // active (high) if ANY device has a pending interrupt.
        //
        // Hardware Semantics:
        // - IRQ line is ACTIVE when at least one device has_interrupt() returns true
        // - IRQ line is INACTIVE only when ALL devices have cleared their interrupts
        // - This matches real 6502 hardware where multiple devices share the IRQ line
        //
        // Multi-Device Behavior:
        // - When multiple devices assert interrupts, the IRQ line stays active
        // - ISR must poll all devices to identify interrupt sources
        // - ISR must acknowledge each device individually (device-specific mechanism)
        // - IRQ line goes inactive only after ALL devices are acknowledged
        // - If new interrupt asserts during ISR, CPU re-enters ISR after RTI
        //
        // ISR Pattern for Multi-Device Systems:
        // 1. ISR polls each device's status register in priority order
        // 2. For each device with pending interrupt:
        //    a. Read device status to identify cause
        //    b. Handle the interrupt
        //    c. Acknowledge device (write to control register or read data)
        // 3. RTI instruction returns to interrupted code
        // 4. CPU immediately checks IRQ line - if still active, re-enters ISR
        //
        // Example (3 devices at 0xD000, 0xD100, 0xD200):
        //   irq_handler:
        //       lda $D000          ; Check timer status (highest priority)
        //       and #$80           ; Interrupt pending?
        //       beq check_uart
        //       ; handle timer...
        //       sta $D001          ; Acknowledge timer
        //   check_uart:
        //       lda $D100          ; Check UART status
        //       and #$80
        //       beq check_gpio
        //       ; handle UART...
        //       lda $D104          ; Acknowledge by reading data
        //   check_gpio:
        //       lda $D200          ; Check GPIO status
        //       and #$80
        //       beq done
        //       ; handle GPIO...
        //       sta $D201          ; Acknowledge GPIO
        //   done:
        //       rti                ; Return - CPU will re-check IRQ line
        //
        self.devices
            .iter()
            .filter_map(|mapping| mapping.device.as_interrupt_device())
            .any(|interrupt_device| interrupt_device.has_interrupt())
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

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
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

    // ========== Edge Case Tests for Address Ranges ==========

    #[test]
    fn test_device_at_address_ffff() {
        // Test a 1-byte device at the very end of address space
        let mut memory = MappedMemory::new();
        let result = memory.add_device(0xFFFF, Box::new(TestDevice::new(1)));
        assert!(result.is_ok(), "Should allow 1-byte device at 0xFFFF");

        // Verify it's accessible
        memory.write(0xFFFF, 0x42);
        assert_eq!(memory.read(0xFFFF), 0x42);
    }

    #[test]
    fn test_device_extending_to_ffff() {
        // Test device that extends exactly to end of address space
        let mut memory = MappedMemory::new();
        // Device at 0xE000 with size 0x2000 should extend to 0xFFFF
        let result = memory.add_device(0xE000, Box::new(TestDevice::new(0x2000)));
        assert!(result.is_ok(), "Should allow device extending to 0xFFFF");

        // Verify end addresses are accessible
        memory.write(0xFFFE, 0xAA);
        memory.write(0xFFFF, 0xBB);
        assert_eq!(memory.read(0xFFFE), 0xAA);
        assert_eq!(memory.read(0xFFFF), 0xBB);
    }

    #[test]
    fn test_overlapping_devices_rejected() {
        let mut memory = MappedMemory::new();

        // Add device at 0x8000-0xBFFF (16KB)
        memory
            .add_device(0x8000, Box::new(TestDevice::new(0x4000)))
            .unwrap();

        // Try to add overlapping device at 0xA000-0xDFFF
        let result = memory.add_device(0xA000, Box::new(TestDevice::new(0x4000)));
        assert!(
            result.is_err(),
            "Should reject device that overlaps existing device"
        );

        // Try to add device that contains existing device
        let result = memory.add_device(0x7000, Box::new(TestDevice::new(0x6000)));
        assert!(
            result.is_err(),
            "Should reject device that contains existing device"
        );

        // Try to add device contained by existing device
        let result = memory.add_device(0x9000, Box::new(TestDevice::new(0x1000)));
        assert!(
            result.is_err(),
            "Should reject device contained by existing device"
        );
    }

    #[test]
    fn test_device_at_overflow_boundary() {
        // Test device at 0xFFFF with size > 1 (would overflow)
        let mut memory = MappedMemory::new();

        // This should work but the device gets clamped to [0xFFFF, 0xFFFF]
        let result = memory.add_device(0xFFFF, Box::new(TestDevice::new(10)));
        assert!(
            result.is_ok(),
            "Should handle overflow by clamping to 0xFFFF"
        );

        // Only 0xFFFF should be accessible
        memory.write(0xFFFF, 0x42);
        assert_eq!(memory.read(0xFFFF), 0x42);
    }

    #[test]
    fn test_multiple_devices_with_overflow() {
        let mut memory = MappedMemory::new();

        // Add device extending to 0xFFFF
        memory
            .add_device(0xE000, Box::new(TestDevice::new(0x2000)))
            .unwrap();

        // Try to add another device at 0xF000 (would overlap)
        let result = memory.add_device(0xF000, Box::new(TestDevice::new(0x1000)));
        assert!(result.is_err(), "Should detect overlap near 0xFFFF");

        // Try to add device at 0xD000 (should succeed, adjacent)
        let result = memory.add_device(0xD000, Box::new(TestDevice::new(0x1000)));
        assert!(result.is_ok(), "Should allow adjacent device before 0xE000");
    }

    #[test]
    fn test_address_range_contains() {
        // Test the AddressRange::contains() method for edge cases
        let range1 = AddressRange::new(0x1000, 256);
        assert!(range1.contains(0x1000), "Should contain start address");
        assert!(range1.contains(0x10FF), "Should contain end address");
        assert!(
            !range1.contains(0x0FFF),
            "Should not contain address before"
        );
        assert!(!range1.contains(0x1100), "Should not contain address after");

        // Test range at 0xFFFF
        let range2 = AddressRange::new(0xFFFF, 1);
        assert!(range2.contains(0xFFFF), "Should contain 0xFFFF");
        assert!(!range2.contains(0xFFFE), "Should not contain 0xFFFE");

        // Test range extending to 0xFFFF
        let range3 = AddressRange::new(0xE000, 0x2000);
        assert!(range3.contains(0xE000), "Should contain start");
        assert!(range3.contains(0xFFFF), "Should contain 0xFFFF");
        assert!(!range3.contains(0xDFFF), "Should not contain before start");
    }

    #[test]
    fn test_address_range_overlaps_symmetric() {
        // Overlap should be symmetric: if A overlaps B, then B overlaps A
        let range1 = AddressRange::new(0x1000, 0x1000);
        let range2 = AddressRange::new(0x1800, 0x1000);

        assert_eq!(
            range1.overlaps(&range2),
            range2.overlaps(&range1),
            "Overlap should be symmetric"
        );

        // Test with ranges that don't overlap
        let range3 = AddressRange::new(0x3000, 0x1000);
        assert_eq!(
            range1.overlaps(&range3),
            range3.overlaps(&range1),
            "Non-overlap should be symmetric"
        );
    }
}
