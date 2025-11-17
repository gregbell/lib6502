//! RAM device implementation.
//!
//! Provides readable and writable memory storage via the Device trait.

use super::Device;
use std::any::Any;

/// Simple RAM device with readable and writable storage.
///
/// `RamDevice` provides a straightforward memory implementation where all addresses
/// within the device's size are readable and writable.
///
/// # Examples
///
/// ```rust
/// use lib6502::{RamDevice, Device};
///
/// let mut ram = RamDevice::new(1024); // 1KB RAM
///
/// // Write and read
/// ram.write(0x42, 0xAA);
/// assert_eq!(ram.read(0x42), 0xAA);
/// ```
#[derive(Clone)]
pub struct RamDevice {
    data: Vec<u8>,
}

impl RamDevice {
    /// Create a new RAM device with the specified size.
    ///
    /// All bytes are initialized to zero.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes in the RAM device
    ///
    /// # Returns
    ///
    /// A new `RamDevice` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::RamDevice;
    ///
    /// let ram = RamDevice::new(16384); // 16KB RAM
    /// ```
    pub fn new(size: u16) -> Self {
        Self {
            data: vec![0; size as usize],
        }
    }

    /// Load bytes into RAM at the specified offset.
    ///
    /// This is useful for initializing RAM contents with program data or preloaded values.
    ///
    /// # Arguments
    ///
    /// * `offset` - Starting offset within the RAM device
    /// * `bytes` - Slice of bytes to load
    ///
    /// # Returns
    ///
    /// * `Ok(())` if bytes were loaded successfully
    /// * `Err(&str)` if offset or length would exceed device size
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::{RamDevice, Device};
    ///
    /// let mut ram = RamDevice::new(1024);
    /// ram.load_bytes(0x100, &[0x01, 0x02, 0x03]).unwrap();
    ///
    /// assert_eq!(ram.read(0x100), 0x01);
    /// assert_eq!(ram.read(0x101), 0x02);
    /// assert_eq!(ram.read(0x102), 0x03);
    ///
    /// // Out of bounds returns error
    /// assert!(ram.load_bytes(1023, &[0x01, 0x02]).is_err());
    /// ```
    pub fn load_bytes(&mut self, offset: u16, bytes: &[u8]) -> Result<(), &'static str> {
        let start = offset as usize;
        let end = start.checked_add(bytes.len()).ok_or("Offset overflow")?;

        if end > self.data.len() {
            return Err("Data exceeds device size");
        }

        self.data[start..end].copy_from_slice(bytes);
        Ok(())
    }
}

impl Device for RamDevice {
    fn read(&self, offset: u16) -> u8 {
        self.data.get(offset as usize).copied().unwrap_or(0xFF) // Safe fallback for out-of-bounds
    }

    fn write(&mut self, offset: u16, value: u8) {
        if let Some(cell) = self.data.get_mut(offset as usize) {
            *cell = value;
        }
        // Out-of-bounds writes are silently ignored
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_new() {
        let ram = RamDevice::new(256);
        assert_eq!(ram.size(), 256);

        // All bytes initially zero
        for i in 0..256 {
            assert_eq!(ram.read(i), 0x00);
        }
    }

    #[test]
    fn test_ram_read_write() {
        let mut ram = RamDevice::new(256);

        // Write some values
        ram.write(0, 0xAA);
        ram.write(100, 0xBB);
        ram.write(255, 0xCC);

        // Read them back
        assert_eq!(ram.read(0), 0xAA);
        assert_eq!(ram.read(100), 0xBB);
        assert_eq!(ram.read(255), 0xCC);

        // Verify other addresses still zero
        assert_eq!(ram.read(1), 0x00);
        assert_eq!(ram.read(99), 0x00);
    }

    #[test]
    fn test_ram_load_bytes() {
        let mut ram = RamDevice::new(256);

        let program = vec![0xA9, 0x42, 0x85, 0x10]; // LDA #$42, STA $10
        ram.load_bytes(0x200 - 0x200, &program).unwrap(); // Offset 0 within device

        assert_eq!(ram.read(0), 0xA9);
        assert_eq!(ram.read(1), 0x42);
        assert_eq!(ram.read(2), 0x85);
        assert_eq!(ram.read(3), 0x10);
    }

    #[test]
    fn test_ram_overwrite() {
        let mut ram = RamDevice::new(256);

        // Write, read, overwrite, read again
        ram.write(42, 0x11);
        assert_eq!(ram.read(42), 0x11);

        ram.write(42, 0x22);
        assert_eq!(ram.read(42), 0x22);
    }
}
