//! VIC-II (MOS 6569/6567) Video Interface Chip emulation.
//!
//! The VIC-II is the C64's graphics chip, responsible for:
//! - Text and bitmap display modes
//! - 8 hardware sprites
//! - Raster interrupts
//! - Border and background colors
//!
//! This implementation provides frame-accurate emulation suitable for games,
//! but not cycle-exact timing required for some advanced demos.

use lib6502::Device;
use std::any::Any;

/// VIC-II register count (47 registers at $D000-$D02E).
pub const VIC_REGISTER_COUNT: usize = 47;

/// Screen width in pixels (active area).
pub const SCREEN_WIDTH: usize = 320;

/// Screen height in pixels (active area).
pub const SCREEN_HEIGHT: usize = 200;

/// MOS 6569 (PAL) / 6567 (NTSC) Video Interface Chip.
#[derive(Debug)]
pub struct VicII {
    /// Hardware registers ($D000-$D02E).
    registers: [u8; VIC_REGISTER_COUNT],

    /// Current raster line (0-311 PAL, 0-262 NTSC).
    current_raster: u16,

    /// Cycle within current scanline.
    #[allow(dead_code)]
    cycle_in_line: u8,

    /// Sprite-sprite collision flags (cleared on read).
    sprite_collision_ss: u8,

    /// Sprite-background collision flags (cleared on read).
    sprite_collision_sb: u8,

    /// Framebuffer: indexed color values (0-15) for each pixel.
    framebuffer: Box<[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]>,

    /// IRQ pending flag.
    irq_pending: bool,
}

impl VicII {
    /// Create a new VIC-II with default register values.
    pub fn new() -> Self {
        let mut vic = Self {
            registers: [0; VIC_REGISTER_COUNT],
            current_raster: 0,
            cycle_in_line: 0,
            sprite_collision_ss: 0,
            sprite_collision_sb: 0,
            framebuffer: Box::new([[0; SCREEN_WIDTH]; SCREEN_HEIGHT]),
            irq_pending: false,
        };

        // Set default register values (typical C64 boot state)
        vic.registers[0x11] = 0x1B; // Control register 1: DEN=1, RSEL=1
        vic.registers[0x16] = 0xC8; // Control register 2: CSEL=1
        vic.registers[0x18] = 0x15; // Memory pointers
        vic.registers[0x20] = 0x0E; // Border color: light blue
        vic.registers[0x21] = 0x06; // Background color: blue

        vic
    }

    /// Get a reference to the framebuffer.
    pub fn framebuffer(&self) -> &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.framebuffer
    }

    /// Get the current raster line.
    pub fn raster(&self) -> u16 {
        self.current_raster
    }

    /// Get the border color (0-15).
    pub fn border_color(&self) -> u8 {
        self.registers[0x20] & 0x0F
    }

    /// Get the background color 0 (0-15).
    pub fn background_color(&self) -> u8 {
        self.registers[0x21] & 0x0F
    }

    /// Check if display is enabled (DEN bit).
    pub fn display_enabled(&self) -> bool {
        self.registers[0x11] & 0x10 != 0
    }

    /// Check if bitmap mode is enabled (BMM bit).
    pub fn bitmap_mode(&self) -> bool {
        self.registers[0x11] & 0x20 != 0
    }

    /// Check if multicolor mode is enabled (MCM bit).
    pub fn multicolor_mode(&self) -> bool {
        self.registers[0x16] & 0x10 != 0
    }

    /// Check if extended color mode is enabled (ECM bit).
    pub fn extended_color_mode(&self) -> bool {
        self.registers[0x11] & 0x40 != 0
    }

    /// Step one scanline of VIC-II emulation.
    ///
    /// This is called by the C64 system during frame stepping.
    pub fn step_scanline(&mut self, _scanline: u16, _char_rom: &[u8], _screen_ram: &[u8], _color_ram: &[u8]) {
        // TODO: Implement scanline rendering
        // This will be implemented in a later task (T029-T033)
    }

    /// Check and potentially trigger raster interrupt.
    pub fn check_raster_irq(&mut self) {
        let raster_compare = self.get_raster_compare();
        if self.current_raster == raster_compare {
            self.registers[0x19] |= 0x01; // Set raster interrupt flag
            if self.registers[0x1A] & 0x01 != 0 {
                self.irq_pending = true;
            }
        }
    }

    /// Get the raster compare value from registers.
    fn get_raster_compare(&self) -> u16 {
        let low = self.registers[0x12] as u16;
        let high = ((self.registers[0x11] & 0x80) as u16) << 1;
        high | low
    }

    /// Advance to next scanline, wrapping at end of frame.
    pub fn advance_scanline(&mut self, max_scanlines: u16) {
        self.current_raster += 1;
        if self.current_raster >= max_scanlines {
            self.current_raster = 0;
        }
    }

    /// Reset the VIC-II to power-on state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for VicII {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for VicII {
    fn read(&self, offset: u16) -> u8 {
        match offset as usize {
            // Sprite-sprite collision: cleared on read
            0x1E => {
                // Note: We can't clear in immutable read, so collision
                // clearing will be handled specially by C64Memory
                self.sprite_collision_ss
            }
            // Sprite-background collision: cleared on read
            0x1F => self.sprite_collision_sb,
            // Current raster position
            0x11 => {
                let raster_bit8 = if self.current_raster > 255 { 0x80 } else { 0 };
                (self.registers[0x11] & 0x7F) | raster_bit8
            }
            0x12 => (self.current_raster & 0xFF) as u8,
            // Interrupt register
            0x19 => self.registers[0x19] | 0x70, // Unused bits read as 1
            // Normal register read
            n if n < VIC_REGISTER_COUNT => self.registers[n],
            // Registers mirror every 64 bytes in the $40 range
            _ => 0xFF,
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        let offset = offset as usize;
        if offset >= VIC_REGISTER_COUNT {
            return; // Ignore writes to mirrored/invalid addresses
        }

        match offset {
            // Collision registers are read-only
            0x1E | 0x1F => {}
            // Interrupt register: writing 1 clears the flag
            0x19 => {
                self.registers[0x19] &= !(value & 0x0F);
                // Clear IRQ if no interrupt flags remain
                if self.registers[0x19] & self.registers[0x1A] & 0x0F == 0 {
                    self.irq_pending = false;
                }
            }
            // Normal register write
            _ => self.registers[offset] = value,
        }
    }

    fn size(&self) -> u16 {
        64 // VIC-II registers occupy 64 bytes ($D000-$D03F), mirrored
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn has_interrupt(&self) -> bool {
        self.irq_pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vic_defaults() {
        let vic = VicII::new();
        assert_eq!(vic.border_color(), 0x0E); // Light blue
        assert_eq!(vic.background_color(), 0x06); // Blue
        assert!(vic.display_enabled());
    }

    #[test]
    fn test_raster_read() {
        let mut vic = VicII::new();

        // Raster line 100
        vic.current_raster = 100;
        assert_eq!(vic.read(0x12), 100);
        assert_eq!(vic.read(0x11) & 0x80, 0);

        // Raster line 300 (high bit set)
        vic.current_raster = 300;
        assert_eq!(vic.read(0x12), 44); // 300 & 0xFF
        assert_eq!(vic.read(0x11) & 0x80, 0x80);
    }

    #[test]
    fn test_interrupt_clear() {
        let mut vic = VicII::new();

        // Set interrupt flag
        vic.registers[0x19] = 0x01;
        vic.irq_pending = true;

        // Clear by writing 1
        vic.write(0x19, 0x01);
        assert_eq!(vic.registers[0x19] & 0x01, 0);
    }

    #[test]
    fn test_collision_read_only() {
        let mut vic = VicII::new();

        // Set collision values
        vic.sprite_collision_ss = 0x55;
        vic.sprite_collision_sb = 0xAA;

        // Writing should have no effect
        vic.write(0x1E, 0xFF);
        vic.write(0x1F, 0xFF);

        // Values should remain unchanged
        assert_eq!(vic.read(0x1E), 0x55);
        assert_eq!(vic.read(0x1F), 0xAA);
    }

    #[test]
    fn test_size() {
        let vic = VicII::new();
        assert_eq!(vic.size(), 64);
    }
}
