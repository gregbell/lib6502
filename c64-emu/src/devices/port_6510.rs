//! 6510 CPU I/O Port implementation.
//!
//! The MOS 6510 CPU has a built-in 8-bit I/O port at addresses $00-$01:
//! - $00: Data Direction Register (DDR) - 0=input, 1=output per bit
//! - $01: Data Register - actual port value
//!
//! Bits 0-2 of the data register control C64 memory banking:
//! - Bit 0 (LORAM): 1 = BASIC ROM visible at $A000-$BFFF
//! - Bit 1 (HIRAM): 1 = KERNAL ROM visible at $E000-$FFFF
//! - Bit 2 (CHAREN): 0 = Character ROM, 1 = I/O at $D000-$DFFF
//!
//! The default configuration ($37) makes BASIC, KERNAL, and I/O visible.

use lib6502::Device;
use std::any::Any;

/// 6510 CPU I/O port device.
///
/// Maps to addresses $00-$01 in the C64 memory map.
#[derive(Debug, Clone)]
pub struct Port6510 {
    /// Data Direction Register ($00)
    /// Each bit: 0 = input (read external), 1 = output (drive value)
    ddr: u8,

    /// Data Register ($01)
    /// Bits 0-2 control memory banking configuration
    data: u8,

    /// External input value (bits where DDR=0 read this value)
    /// On a real C64, this includes cassette sense and motor control
    external: u8,
}

impl Port6510 {
    /// Create a new 6510 I/O port with default C64 configuration.
    ///
    /// Default values:
    /// - DDR: $2F (bits 0-2, 5 as outputs; bits 3-4, 6-7 as inputs)
    /// - Data: $37 (BASIC + KERNAL + I/O visible)
    pub fn new() -> Self {
        Self {
            ddr: 0x2F,   // Standard C64 DDR configuration
            data: 0x37,  // LORAM=1, HIRAM=1, CHAREN=1
            external: 0, // No external signals (cassette not present)
        }
    }

    /// Get the current bank configuration (bits 0-2 of data register).
    ///
    /// Returns a value 0-7 determining which memory regions are visible:
    ///
    /// | Value | $A000-$BFFF | $D000-$DFFF | $E000-$FFFF |
    /// |-------|-------------|-------------|-------------|
    /// | 0-1   | RAM         | RAM         | RAM         |
    /// | 2     | RAM         | CHAR ROM    | KERNAL      |
    /// | 3     | BASIC       | CHAR ROM    | KERNAL      |
    /// | 4     | RAM         | RAM         | RAM         |
    /// | 5     | RAM         | I/O         | RAM         |
    /// | 6     | RAM         | I/O         | KERNAL      |
    /// | 7     | BASIC       | I/O         | KERNAL      |
    #[inline]
    pub fn bank_config(&self) -> u8 {
        self.effective_data() & 0x07
    }

    /// Check if BASIC ROM is visible at $A000-$BFFF.
    #[inline]
    pub fn basic_visible(&self) -> bool {
        self.effective_data() & 0x03 == 0x03 // LORAM=1 AND HIRAM=1
    }

    /// Check if KERNAL ROM is visible at $E000-$FFFF.
    #[inline]
    pub fn kernal_visible(&self) -> bool {
        self.effective_data() & 0x02 != 0 // HIRAM=1
    }

    /// Check if I/O area is visible at $D000-$DFFF (vs Character ROM or RAM).
    #[inline]
    pub fn io_visible(&self) -> bool {
        let cfg = self.effective_data() & 0x07;
        cfg == 5 || cfg == 6 || cfg == 7
    }

    /// Check if Character ROM is visible at $D000-$DFFF.
    #[inline]
    pub fn char_rom_visible(&self) -> bool {
        let cfg = self.effective_data() & 0x07;
        cfg == 2 || cfg == 3
    }

    /// Get the effective data value considering DDR.
    ///
    /// For output bits (DDR=1), returns data register value.
    /// For input bits (DDR=0), returns external input value.
    #[inline]
    fn effective_data(&self) -> u8 {
        (self.data & self.ddr) | (self.external & !self.ddr)
    }

    /// Set external input bits (e.g., cassette sense).
    #[inline]
    pub fn set_external(&mut self, value: u8) {
        self.external = value;
    }

    /// Get the data direction register value.
    #[inline]
    pub fn ddr(&self) -> u8 {
        self.ddr
    }

    /// Get the raw data register value.
    #[inline]
    pub fn data(&self) -> u8 {
        self.data
    }

    /// Set the DDR register (for save state restoration).
    #[inline]
    pub fn set_ddr(&mut self, ddr: u8) {
        self.ddr = ddr;
    }

    /// Set the data register (for save state restoration).
    #[inline]
    pub fn set_data(&mut self, data: u8) {
        self.data = data;
    }
}

impl Default for Port6510 {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Port6510 {
    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => self.ddr,
            1 => self.effective_data(),
            _ => 0xFF, // Should not happen (size is 2)
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset {
            0 => self.ddr = value,
            1 => self.data = value,
            _ => {} // Ignore writes outside range
        }
    }

    fn size(&self) -> u16 {
        2 // $00-$01
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
    fn test_default_configuration() {
        let port = Port6510::new();
        assert_eq!(port.ddr, 0x2F);
        assert_eq!(port.data, 0x37);
        assert_eq!(port.bank_config(), 7);
    }

    #[test]
    fn test_bank_config() {
        let mut port = Port6510::new();

        // Default: BASIC + I/O + KERNAL visible
        assert!(port.basic_visible());
        assert!(port.kernal_visible());
        assert!(port.io_visible());
        assert!(!port.char_rom_visible());

        // Set to config 0: all RAM
        port.data = 0x30; // Keep other bits, clear bits 0-2
        assert!(!port.basic_visible());
        assert!(!port.kernal_visible());
        assert!(!port.io_visible());

        // Set to config 3: BASIC + CHAR ROM + KERNAL
        port.data = 0x33;
        assert!(port.basic_visible());
        assert!(port.kernal_visible());
        assert!(!port.io_visible());
        assert!(port.char_rom_visible());
    }

    #[test]
    fn test_device_read_write() {
        let mut port = Port6510::new();

        // Read DDR
        assert_eq!(port.read(0), 0x2F);

        // Read data - returns effective value considering DDR and external
        // DDR=0x2F, data=0x37, external=0
        // effective = (0x37 & 0x2F) | (0 & 0xD0) = 0x27
        assert_eq!(port.read(1), 0x27);

        // Write DDR to all outputs
        port.write(0, 0xFF);
        assert_eq!(port.read(0), 0xFF);

        // Now with DDR=FF, data register is returned directly
        port.write(1, 0x37);
        assert_eq!(port.read(1), 0x37);

        // Write data to 0
        port.write(1, 0x00);
        assert_eq!(port.read(1), 0x00);
        assert_eq!(port.bank_config(), 0);
    }

    #[test]
    fn test_external_input() {
        let mut port = Port6510::new();

        // Set some bits as input (clear DDR bits)
        port.ddr = 0x07; // Only bits 0-2 are outputs

        // Set external input
        port.set_external(0xF0);

        // Data register has 0x37, external has 0xF0
        // Effective: (0x37 & 0x07) | (0xF0 & 0xF8) = 0x07 | 0xF0 = 0xF7
        assert_eq!(port.read(1), 0xF7);

        // Bank config only uses bits 0-2 which are outputs
        assert_eq!(port.bank_config(), 7);
    }

    #[test]
    fn test_size() {
        let port = Port6510::new();
        assert_eq!(port.size(), 2);
    }
}
