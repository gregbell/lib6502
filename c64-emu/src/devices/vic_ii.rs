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

/// Number of character columns on screen.
const CHAR_COLUMNS: usize = 40;

/// Number of character rows on screen.
const CHAR_ROWS: usize = 25;

/// Height of each character in pixels.
const CHAR_HEIGHT: usize = 8;

/// First visible scanline for the display area (PAL).
/// The C64 display starts at raster line 51 (approx) for PAL.
const DISPLAY_START_LINE_PAL: u16 = 51;

/// First visible scanline for the display area (NTSC).
#[allow(dead_code)]
const DISPLAY_START_LINE_NTSC: u16 = 51;

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

    /// Get a raw pointer to the framebuffer data.
    ///
    /// This is useful for WASM bindings where JavaScript can directly access
    /// the framebuffer memory without copying. The framebuffer is a contiguous
    /// 320×200 array of indexed colors (0-15).
    ///
    /// # Safety
    /// The returned pointer is valid as long as the VicII instance exists.
    pub fn framebuffer_ptr(&self) -> *const u8 {
        self.framebuffer.as_ptr() as *const u8
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
    /// This renders one scanline of the display into the framebuffer.
    /// For standard text mode (40x25 characters, 8x8 pixels each):
    /// - `scanline`: The current raster line (0-311 PAL, 0-262 NTSC)
    /// - `char_rom`: The 4KB character ROM data
    /// - `screen_ram`: The 1KB screen RAM containing character codes
    /// - `color_ram`: The 1KB color RAM containing character colors
    pub fn step_scanline(
        &mut self,
        scanline: u16,
        char_rom: &[u8],
        screen_ram: &[u8],
        color_ram: &[u8],
    ) {
        // Calculate the display line (relative to the visible area)
        // PAL: Display area is roughly lines 51-250 (200 visible lines)
        let display_start = DISPLAY_START_LINE_PAL;
        let display_end = display_start + SCREEN_HEIGHT as u16;

        // Check if we're in the visible display area
        if scanline < display_start || scanline >= display_end {
            // Outside visible area - nothing to render to framebuffer
            return;
        }

        let display_line = (scanline - display_start) as usize;

        // If display is disabled (DEN=0), fill with background color
        if !self.display_enabled() {
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        // Determine rendering mode based on BMM, ECM, MCM bits
        let is_bitmap_mode = self.bitmap_mode();
        let is_multicolor = self.multicolor_mode();
        let is_ecm = self.extended_color_mode();

        // For now, only implement standard text mode (BMM=0, ECM=0, MCM=0)
        // Other modes will be implemented in T067-T070
        if is_bitmap_mode || is_ecm || is_multicolor {
            // Fill with background for unsupported modes (placeholder)
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        // Standard text mode rendering
        self.render_standard_text_scanline(display_line, char_rom, screen_ram, color_ram);
    }

    /// Render one scanline in standard text mode (40x25 characters).
    ///
    /// In standard text mode:
    /// - Each character is 8x8 pixels
    /// - Screen RAM provides the character code (0-255)
    /// - Character ROM provides the 8-byte pattern for each character
    /// - Color RAM provides the foreground color (0-15)
    /// - Background color comes from register $D021
    fn render_standard_text_scanline(
        &mut self,
        display_line: usize,
        char_rom: &[u8],
        screen_ram: &[u8],
        color_ram: &[u8],
    ) {
        // Calculate which character row and which line within that character
        let char_row = display_line / CHAR_HEIGHT;
        let char_line = display_line % CHAR_HEIGHT;

        // Skip if we're past the character display area
        if char_row >= CHAR_ROWS {
            // Fill remaining lines with background
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        let bg_color = self.background_color();

        // Render each of the 40 character columns
        for char_col in 0..CHAR_COLUMNS {
            // Get the character code from screen RAM
            let screen_offset = char_row * CHAR_COLUMNS + char_col;
            let char_code = if screen_offset < screen_ram.len() {
                screen_ram[screen_offset]
            } else {
                0
            };

            // Get the foreground color from color RAM (lower 4 bits)
            let fg_color = if screen_offset < color_ram.len() {
                color_ram[screen_offset] & 0x0F
            } else {
                0x0E // Default to light blue
            };

            // Get the character pattern byte for this line
            // Character ROM: each character is 8 bytes (one per line)
            let char_offset = (char_code as usize) * 8 + char_line;
            let pattern = if char_offset < char_rom.len() {
                char_rom[char_offset]
            } else {
                0
            };

            // Render 8 pixels for this character cell
            let x_base = char_col * 8;
            for bit in 0..8 {
                // Bit 7 is the leftmost pixel
                let pixel_set = (pattern & (0x80 >> bit)) != 0;
                let color = if pixel_set { fg_color } else { bg_color };
                self.framebuffer[display_line][x_base + bit] = color;
            }
        }
    }

    /// Get the Y scroll value (0-7).
    pub fn y_scroll(&self) -> u8 {
        self.registers[0x11] & 0x07
    }

    /// Get the X scroll value (0-7).
    pub fn x_scroll(&self) -> u8 {
        self.registers[0x16] & 0x07
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

    #[test]
    fn test_standard_text_mode_rendering() {
        let mut vic = VicII::new();

        // Create a simple character ROM with a test pattern
        // Character 'A' (code 1) will have a simple pattern
        let mut char_rom = vec![0u8; 2048];
        // Character 0: empty (all zeros)
        // Character 1: vertical line pattern (0x80 = leftmost pixel set)
        for i in 0..8 {
            char_rom[8 + i] = 0x80; // Character 1, all 8 lines have leftmost pixel
        }

        // Create screen RAM with character code 1 at position 0
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 1; // First character is code 1

        // Create color RAM with foreground color 1 (white)
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 1; // White foreground

        // Render scanline 51 (first visible line, which is char row 0, line 0)
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Check the framebuffer
        // First pixel (x=0) should be white (color 1) because bit 7 is set
        assert_eq!(
            vic.framebuffer[0][0], 1,
            "First pixel should be foreground color"
        );

        // Second pixel (x=1) should be background (blue = 6) because bit 6 is 0
        assert_eq!(
            vic.framebuffer[0][1], 6,
            "Second pixel should be background color"
        );
    }

    #[test]
    fn test_text_mode_character_boundary() {
        let mut vic = VicII::new();

        // Character ROM with different patterns for characters 0 and 1
        let mut char_rom = vec![0u8; 2048];
        // Character 0: all pixels off (0x00)
        // Character 1: all pixels on (0xFF)
        for i in 0..8 {
            char_rom[8 + i] = 0xFF; // Character 1, all pixels set
        }

        // Screen RAM: character 0 at position 0, character 1 at position 1
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0; // First cell: character 0 (empty)
        screen_ram[1] = 1; // Second cell: character 1 (full)

        // Color RAM: different colors for each cell
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 2; // Red (won't show, char is empty)
        color_ram[1] = 3; // Cyan

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // First 8 pixels (character 0) should all be background color (6 = blue)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "Char 0 pixel {} should be background",
                x
            );
        }

        // Next 8 pixels (character 1) should all be foreground color (3 = cyan)
        for x in 8..16 {
            assert_eq!(
                vic.framebuffer[0][x], 3,
                "Char 1 pixel {} should be foreground",
                x
            );
        }
    }

    #[test]
    fn test_text_mode_multiple_rows() {
        let mut vic = VicII::new();

        // Create character ROM
        let mut char_rom = vec![0u8; 2048];
        // Character 1 has pattern 0xAA (alternating pixels: 10101010)
        for i in 0..8 {
            char_rom[8 + i] = 0xAA;
        }

        // Screen RAM: all character 1
        let screen_ram = vec![1u8; 1000];

        // Color RAM: all white
        let color_ram = vec![1u8; 1000];

        // Render multiple scanlines to test character row progression
        // Line 51 = row 0, line 0 of char
        // Line 52 = row 0, line 1 of char
        // ...
        // Line 58 = row 0, line 7 of char
        // Line 59 = row 1, line 0 of char

        for scanline in 51..=59 {
            vic.step_scanline(scanline, &char_rom, &screen_ram, &color_ram);
        }

        // Check first row, first line (display_line 0)
        assert_eq!(
            vic.framebuffer[0][0], 1,
            "Pixel 0,0 should be foreground (1 bit in 0xAA)"
        );
        assert_eq!(
            vic.framebuffer[0][1], 6,
            "Pixel 0,1 should be background (0 bit in 0xAA)"
        );

        // Check second row (display_line 8)
        assert_eq!(vic.framebuffer[8][0], 1, "Pixel 8,0 should be foreground");
        assert_eq!(vic.framebuffer[8][1], 6, "Pixel 8,1 should be background");
    }

    #[test]
    fn test_display_disabled() {
        let mut vic = VicII::new();

        // Disable display by clearing DEN bit
        vic.registers[0x11] &= !0x10;
        assert!(!vic.display_enabled());

        // Any character data
        let char_rom = vec![0xFFu8; 2048];
        let screen_ram = vec![1u8; 1000];
        let color_ram = vec![1u8; 1000];

        // Render a scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // All pixels should be background color (6) when display is disabled
        for x in 0..SCREEN_WIDTH {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "All pixels should be background when DEN=0"
            );
        }
    }

    #[test]
    fn test_border_area_not_rendered() {
        let mut vic = VicII::new();

        // Set framebuffer to non-zero to verify it's not touched
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0xFF;
            }
        }

        let char_rom = vec![0u8; 2048];
        let screen_ram = vec![0u8; 1000];
        let color_ram = vec![0u8; 1000];

        // Render a scanline in the top border area (before line 51)
        vic.step_scanline(0, &char_rom, &screen_ram, &color_ram);

        // Framebuffer should remain unchanged (all 0xFF)
        assert_eq!(
            vic.framebuffer[0][0], 0xFF,
            "Border scanline should not modify framebuffer"
        );
    }
}
