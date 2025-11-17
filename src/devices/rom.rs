//! ROM device implementation.
//!
//! Provides read-only memory storage via the Device trait.

use super::Device;

/// Read-only memory device.
///
/// `RomDevice` stores immutable data that can be read but not written.
/// Writes are silently ignored (no-op), matching typical ROM hardware behavior.
///
/// # Examples
///
/// ```rust
/// use lib6502::{RomDevice, Device};
///
/// let rom_data = vec![0xEA, 0xEA, 0xEA]; // Three NOP instructions
/// let mut rom = RomDevice::new(rom_data);
///
/// // Reads work
/// assert_eq!(rom.read(0), 0xEA);
///
/// // Writes are ignored
/// rom.write(0, 0xFF);
/// assert_eq!(rom.read(0), 0xEA); // Still original value
/// ```
pub struct RomDevice {
    data: Vec<u8>,
}

impl RomDevice {
    /// Create a new ROM device with the specified data.
    ///
    /// The data is immutable after construction - writes will be ignored.
    ///
    /// # Arguments
    ///
    /// * `data` - Initial ROM contents
    ///
    /// # Returns
    ///
    /// A new `RomDevice` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib6502::RomDevice;
    ///
    /// // Create ROM with reset vector and program
    /// let mut rom_data = vec![0; 16384]; // 16KB ROM
    ///
    /// // Set reset vector at end of ROM (0x3FFC-0x3FFD within device)
    /// rom_data[0x3FFC] = 0x00; // Low byte of reset vector
    /// rom_data[0x3FFD] = 0xC0; // High byte (PC = 0xC000)
    ///
    /// let rom = RomDevice::new(rom_data);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Device for RomDevice {
    fn read(&self, offset: u16) -> u8 {
        self.data[offset as usize]
    }

    fn write(&mut self, _offset: u16, _value: u8) {
        // Writes to ROM are silently ignored (no-op)
    }

    fn size(&self) -> u16 {
        self.data.len() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_new() {
        let data = vec![0xEA; 256];
        let rom = RomDevice::new(data);

        assert_eq!(rom.size(), 256);
        assert_eq!(rom.read(0), 0xEA);
        assert_eq!(rom.read(255), 0xEA);
    }

    #[test]
    fn test_rom_read() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let rom = RomDevice::new(data);

        assert_eq!(rom.read(0), 0x01);
        assert_eq!(rom.read(1), 0x02);
        assert_eq!(rom.read(2), 0x03);
        assert_eq!(rom.read(3), 0x04);
    }

    #[test]
    fn test_rom_write_ignored() {
        let data = vec![0xAA; 256];
        let mut rom = RomDevice::new(data);

        // Try to write
        rom.write(0, 0xFF);
        rom.write(100, 0xFF);

        // Verify writes were ignored
        assert_eq!(rom.read(0), 0xAA);
        assert_eq!(rom.read(100), 0xAA);
    }

    #[test]
    fn test_rom_with_reset_vector() {
        let mut data = vec![0; 16384]; // 16KB ROM

        // Set reset vector at end (0x3FFC-0x3FFD within device)
        data[0x3FFC] = 0x00; // Low byte
        data[0x3FFD] = 0xC0; // High byte (points to 0xC000)

        let rom = RomDevice::new(data);

        assert_eq!(rom.read(0x3FFC), 0x00);
        assert_eq!(rom.read(0x3FFD), 0xC0);
    }
}
