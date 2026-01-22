//! C64 Color RAM implementation.
//!
//! The Color RAM is a 1KB area of 4-bit memory at $D800-$DBFF that stores
//! the foreground color for each character cell in text mode. Each byte
//! stores one color value (0-15) in the lower 4 bits; the upper 4 bits
//! are not connected and read as random values (typically the last value
//! on the data bus).

use lib6502::Device;
use std::any::Any;

/// Size of color RAM in bytes.
pub const COLOR_RAM_SIZE: usize = 1024;

/// C64 Color RAM device.
///
/// Maps to $D800-$DBFF in the C64 memory map (when I/O is visible).
#[derive(Debug, Clone)]
pub struct ColorRam {
    /// Color data (4-bit values, only lower nibble is valid).
    data: Box<[u8; COLOR_RAM_SIZE]>,
}

impl ColorRam {
    /// Create a new color RAM initialized to a default color.
    ///
    /// The default color is light blue (14) to match C64 boot state.
    pub fn new() -> Self {
        Self {
            data: Box::new([14; COLOR_RAM_SIZE]), // Light blue
        }
    }

    /// Create color RAM initialized with specific data.
    pub fn with_data(data: [u8; COLOR_RAM_SIZE]) -> Self {
        Self {
            data: Box::new(data),
        }
    }

    /// Get the color value at an offset (0-1023).
    #[inline]
    pub fn get(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0) & 0x0F
    }

    /// Set the color value at an offset.
    #[inline]
    pub fn set(&mut self, offset: usize, color: u8) {
        if let Some(cell) = self.data.get_mut(offset) {
            *cell = color & 0x0F;
        }
    }

    /// Get a slice of the color RAM data.
    pub fn as_slice(&self) -> &[u8] {
        &*self.data
    }

    /// Clear color RAM to a specific color.
    pub fn clear(&mut self, color: u8) {
        let color = color & 0x0F;
        for cell in self.data.iter_mut() {
            *cell = color;
        }
    }

    /// Reset to default state (light blue).
    pub fn reset(&mut self) {
        self.clear(14);
    }
}

impl Default for ColorRam {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for ColorRam {
    fn read(&self, offset: u16) -> u8 {
        // Upper nibble is "floating" - in reality it reads whatever was
        // last on the data bus. We simulate this by returning 0xF0 | color.
        // Some software depends on reading back what was written.
        let offset = offset as usize;
        if offset < COLOR_RAM_SIZE {
            self.data[offset] | 0xF0
        } else {
            0xFF
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        let offset = offset as usize;
        if offset < COLOR_RAM_SIZE {
            // Only lower 4 bits are stored
            self.data[offset] = value & 0x0F;
        }
    }

    fn size(&self) -> u16 {
        COLOR_RAM_SIZE as u16
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
    fn test_new_color_ram() {
        let ram = ColorRam::new();
        // Should be initialized to light blue (14)
        assert_eq!(ram.get(0), 14);
        assert_eq!(ram.get(999), 14);
    }

    #[test]
    fn test_read_write() {
        let mut ram = ColorRam::new();

        // Write a color
        ram.write(100, 0x05);
        // Read back - upper nibble should be set
        assert_eq!(ram.read(100), 0xF5);
        // get() should return only lower nibble
        assert_eq!(ram.get(100), 0x05);
    }

    #[test]
    fn test_only_lower_nibble_stored() {
        let mut ram = ColorRam::new();

        // Write with upper nibble set
        ram.write(50, 0xAB);
        // Only lower nibble (0x0B) should be stored
        assert_eq!(ram.get(50), 0x0B);
        // Read back includes upper nibble from floating bus sim
        assert_eq!(ram.read(50), 0xFB);
    }

    #[test]
    fn test_clear() {
        let mut ram = ColorRam::new();

        // Set some values
        ram.write(0, 0x01);
        ram.write(100, 0x02);
        ram.write(999, 0x03);

        // Clear to black
        ram.clear(0);

        assert_eq!(ram.get(0), 0);
        assert_eq!(ram.get(100), 0);
        assert_eq!(ram.get(999), 0);
    }

    #[test]
    fn test_bounds() {
        let mut ram = ColorRam::new();

        // Write beyond bounds should be ignored
        ram.write(2000, 0x05);

        // Read beyond bounds returns 0xFF
        assert_eq!(ram.read(2000), 0xFF);
    }

    #[test]
    fn test_size() {
        let ram = ColorRam::new();
        assert_eq!(ram.size(), 1024);
    }
}
