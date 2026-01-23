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

    /// Get the background color 1 (0-15) - used in multicolor modes.
    pub fn background_color_1(&self) -> u8 {
        self.registers[0x22] & 0x0F
    }

    /// Get the background color 2 (0-15) - used in multicolor modes.
    pub fn background_color_2(&self) -> u8 {
        self.registers[0x23] & 0x0F
    }

    /// Get the background color 3 (0-15) - used in ECM mode.
    pub fn background_color_3(&self) -> u8 {
        self.registers[0x24] & 0x0F
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
    /// - `char_rom`: The 4KB character ROM data (or 8KB bitmap data in bitmap modes)
    /// - `screen_ram`: The 1KB screen RAM containing character codes (or color info in bitmap modes)
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

        // Select rendering mode based on control bits
        // Note: Illegal mode combinations (BMM+ECM or all three) produce garbled output
        // We'll handle the common legal modes here
        match (is_bitmap_mode, is_ecm, is_multicolor) {
            // Standard text mode (BMM=0, ECM=0, MCM=0)
            (false, false, false) => {
                self.render_standard_text_scanline(display_line, char_rom, screen_ram, color_ram);
            }
            // Multicolor text mode (BMM=0, ECM=0, MCM=1)
            // In this mode, characters with color RAM bit 3 set use multicolor rendering
            (false, false, true) => {
                self.render_multicolor_text_scanline(display_line, char_rom, screen_ram, color_ram);
            }
            // Standard bitmap mode (BMM=1, ECM=0, MCM=0)
            // 320x200 pixels, 2 colors per 8x8 cell from screen RAM
            (true, false, false) => {
                self.render_standard_bitmap_scanline(display_line, char_rom, screen_ram);
            }
            // Multicolor bitmap mode (BMM=1, ECM=0, MCM=1)
            // 160x200 effective resolution, 4 colors per 8x8 cell
            (true, false, true) => {
                self.render_multicolor_bitmap_scanline(display_line, char_rom, screen_ram, color_ram);
            }
            // ECM (Extended Color Mode) text mode (BMM=0, ECM=1, MCM=0)
            // In this mode, bits 6-7 of the character code select one of 4 background colors
            // Only 64 characters are addressable (bits 0-5)
            (false, true, false) => {
                self.render_ecm_text_scanline(display_line, char_rom, screen_ram, color_ram);
            }
            // Illegal mode combinations (BMM+ECM or all three) produce garbled output
            // We fill with background as a safe default
            _ => {
                let bg_color = self.background_color();
                for x in 0..SCREEN_WIDTH {
                    self.framebuffer[display_line][x] = bg_color;
                }
            }
        }
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

    /// Render one scanline in multicolor text mode (40x25 characters).
    ///
    /// In multicolor text mode (MCM=1):
    /// - Characters with color RAM bit 3 CLEAR use standard hires rendering
    /// - Characters with color RAM bit 3 SET use multicolor rendering:
    ///   - Bit pairs determine pixel color (2 pixels at a time)
    ///   - 00 = Background color 0 ($D021)
    ///   - 01 = Background color 1 ($D022)
    ///   - 10 = Background color 2 ($D023)
    ///   - 11 = Foreground color (color RAM bits 0-2, only 8 colors available)
    fn render_multicolor_text_scanline(
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
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        let bg_color_0 = self.background_color();
        let bg_color_1 = self.background_color_1();
        let bg_color_2 = self.background_color_2();

        // Render each of the 40 character columns
        for char_col in 0..CHAR_COLUMNS {
            // Get the character code from screen RAM
            let screen_offset = char_row * CHAR_COLUMNS + char_col;
            let char_code = if screen_offset < screen_ram.len() {
                screen_ram[screen_offset]
            } else {
                0
            };

            // Get the color from color RAM
            let color_byte = if screen_offset < color_ram.len() {
                color_ram[screen_offset]
            } else {
                0
            };

            // Check if this character uses multicolor mode (bit 3 set)
            let use_multicolor = (color_byte & 0x08) != 0;

            // Get the character pattern byte for this line
            let char_offset = (char_code as usize) * 8 + char_line;
            let pattern = if char_offset < char_rom.len() {
                char_rom[char_offset]
            } else {
                0
            };

            let x_base = char_col * 8;

            if use_multicolor {
                // Multicolor mode: 2 pixels per bit pair, only 8 colors for foreground
                let fg_color = color_byte & 0x07; // Only bits 0-2 (8 colors)

                // Process 4 bit pairs (8 pixels displayed as 4 double-wide pixels)
                for bit_pair in 0..4 {
                    let shift = 6 - (bit_pair * 2);
                    let bits = (pattern >> shift) & 0x03;

                    let color = match bits {
                        0b00 => bg_color_0, // Background 0
                        0b01 => bg_color_1, // Background 1
                        0b10 => bg_color_2, // Background 2
                        0b11 => fg_color,   // Foreground (only 8 colors)
                        _ => unreachable!(),
                    };

                    // Each bit pair produces 2 pixels of the same color
                    let px = x_base + bit_pair * 2;
                    self.framebuffer[display_line][px] = color;
                    self.framebuffer[display_line][px + 1] = color;
                }
            } else {
                // Standard hires mode for this character (bit 3 clear)
                let fg_color = color_byte & 0x0F; // Full 16 colors

                for bit in 0..8 {
                    let pixel_set = (pattern & (0x80 >> bit)) != 0;
                    let color = if pixel_set { fg_color } else { bg_color_0 };
                    self.framebuffer[display_line][x_base + bit] = color;
                }
            }
        }
    }

    /// Render one scanline in standard bitmap mode (320x200).
    ///
    /// In standard bitmap mode (BMM=1, MCM=0):
    /// - The display is a 320x200 pixel bitmap (8000 bytes)
    /// - Each bit in the bitmap corresponds to one pixel
    /// - Screen RAM (video matrix) provides colors for each 8x8 cell:
    ///   - Upper nibble (bits 4-7) = foreground color (pixel set)
    ///   - Lower nibble (bits 0-3) = background color (pixel clear)
    /// - Bitmap data is organized as 40 columns × 25 rows of 8-byte cells
    /// - Each cell is 8 bytes (one byte per scanline within the cell)
    ///
    /// The `bitmap_data` parameter should contain 8000 bytes of bitmap data.
    /// The `screen_ram` parameter provides the 1000-byte color information.
    fn render_standard_bitmap_scanline(
        &mut self,
        display_line: usize,
        bitmap_data: &[u8],
        screen_ram: &[u8],
    ) {
        // Calculate which character row and which line within that row
        let char_row = display_line / CHAR_HEIGHT;
        let char_line = display_line % CHAR_HEIGHT;

        // Skip if we're past the character display area
        if char_row >= CHAR_ROWS {
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        // Render each of the 40 character columns
        for char_col in 0..CHAR_COLUMNS {
            // Get colors from screen RAM (video matrix)
            // Upper nibble = foreground, lower nibble = background
            let screen_offset = char_row * CHAR_COLUMNS + char_col;
            let color_byte = if screen_offset < screen_ram.len() {
                screen_ram[screen_offset]
            } else {
                0
            };

            let fg_color = (color_byte >> 4) & 0x0F;
            let bg_color = color_byte & 0x0F;

            // Calculate bitmap data offset
            // Bitmap is organized in a character-cell manner:
            // - Each 8x8 cell is stored as 8 consecutive bytes
            // - Cells are arranged row by row (40 cells per row)
            // - Within each cell, bytes are stored top to bottom (one per scanline)
            //
            // Offset calculation:
            // - char_row * 320: skip to the start of this character row (40 cells × 8 bytes)
            // - char_col * 8: skip to the start of this cell within the row
            // - char_line: select the specific byte within the 8-byte cell
            let bitmap_offset = char_row * 320 + char_col * 8 + char_line;
            let pattern = if bitmap_offset < bitmap_data.len() {
                bitmap_data[bitmap_offset]
            } else {
                0
            };

            // Render 8 pixels for this cell
            let x_base = char_col * 8;
            for bit in 0..8 {
                // Bit 7 is the leftmost pixel
                let pixel_set = (pattern & (0x80 >> bit)) != 0;
                let color = if pixel_set { fg_color } else { bg_color };
                self.framebuffer[display_line][x_base + bit] = color;
            }
        }
    }

    /// Render one scanline in multicolor bitmap mode (160x200 effective).
    ///
    /// In multicolor bitmap mode (BMM=1, MCM=1):
    /// - The display is a 160x200 effective pixel bitmap (double-wide pixels)
    /// - Each bit pair in the bitmap corresponds to one double-wide pixel
    /// - Colors are determined by bit pairs:
    ///   - 00 = Background color 0 ($D021)
    ///   - 01 = Upper nibble of screen RAM (bits 4-7)
    ///   - 10 = Lower nibble of screen RAM (bits 0-3)
    ///   - 11 = Color RAM (bits 0-3, full 16 colors)
    /// - Bitmap data is organized the same as standard bitmap mode (8 bytes per cell)
    ///
    /// The `bitmap_data` parameter should contain 8000 bytes of bitmap data.
    /// The `screen_ram` parameter provides the 1000-byte color info for bit pairs 01 and 10.
    /// The `color_ram` parameter provides the 1000-byte color info for bit pair 11.
    fn render_multicolor_bitmap_scanline(
        &mut self,
        display_line: usize,
        bitmap_data: &[u8],
        screen_ram: &[u8],
        color_ram: &[u8],
    ) {
        // Calculate which character row and which line within that row
        let char_row = display_line / CHAR_HEIGHT;
        let char_line = display_line % CHAR_HEIGHT;

        // Skip if we're past the character display area
        if char_row >= CHAR_ROWS {
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        let bg_color_0 = self.background_color();

        // Render each of the 40 character columns
        for char_col in 0..CHAR_COLUMNS {
            // Get colors from screen RAM and color RAM
            let screen_offset = char_row * CHAR_COLUMNS + char_col;

            let screen_byte = if screen_offset < screen_ram.len() {
                screen_ram[screen_offset]
            } else {
                0
            };

            let color_byte = if screen_offset < color_ram.len() {
                color_ram[screen_offset]
            } else {
                0
            };

            // Colors for each bit pair:
            // 00 = Background color 0 ($D021)
            // 01 = Screen RAM upper nibble (bits 4-7)
            // 10 = Screen RAM lower nibble (bits 0-3)
            // 11 = Color RAM (bits 0-3)
            let color_01 = (screen_byte >> 4) & 0x0F;
            let color_10 = screen_byte & 0x0F;
            let color_11 = color_byte & 0x0F;

            // Calculate bitmap data offset (same as standard bitmap mode)
            let bitmap_offset = char_row * 320 + char_col * 8 + char_line;
            let pattern = if bitmap_offset < bitmap_data.len() {
                bitmap_data[bitmap_offset]
            } else {
                0
            };

            let x_base = char_col * 8;

            // Process 4 bit pairs (8 pixels displayed as 4 double-wide pixels)
            for bit_pair in 0..4 {
                let shift = 6 - (bit_pair * 2);
                let bits = (pattern >> shift) & 0x03;

                let color = match bits {
                    0b00 => bg_color_0, // Background 0
                    0b01 => color_01,   // Screen RAM upper nibble
                    0b10 => color_10,   // Screen RAM lower nibble
                    0b11 => color_11,   // Color RAM
                    _ => unreachable!(),
                };

                // Each bit pair produces 2 pixels of the same color
                let px = x_base + bit_pair * 2;
                self.framebuffer[display_line][px] = color;
                self.framebuffer[display_line][px + 1] = color;
            }
        }
    }

    /// Render one scanline in ECM (Extended Color Mode) text mode (40x25 characters).
    ///
    /// In ECM text mode (ECM=1, BMM=0, MCM=0):
    /// - Character code bits 6-7 select one of 4 background colors ($D021-$D024)
    /// - Character code bits 0-5 address character patterns (only 64 characters available)
    /// - Foreground color comes from color RAM (full 16 colors)
    /// - Each character is still 8x8 pixels at full 320x200 resolution
    ///
    /// This mode allows different background colors per character while using standard
    /// hires rendering, useful for colorful text displays.
    fn render_ecm_text_scanline(
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
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[display_line][x] = bg_color;
            }
            return;
        }

        // Get all 4 background colors
        let bg_colors = [
            self.background_color(),   // $D021 - bits 00
            self.background_color_1(), // $D022 - bits 01
            self.background_color_2(), // $D023 - bits 10
            self.background_color_3(), // $D024 - bits 11
        ];

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

            // In ECM mode:
            // - Bits 6-7 of char code select background color (0-3)
            // - Bits 0-5 of char code select character pattern (0-63 only)
            let bg_select = (char_code >> 6) & 0x03;
            let char_index = char_code & 0x3F; // Only bits 0-5

            let bg_color = bg_colors[bg_select as usize];

            // Get the character pattern byte for this line
            // Character ROM: each character is 8 bytes (one per line)
            let char_offset = (char_index as usize) * 8 + char_line;
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

    #[test]
    fn test_multicolor_text_mode_bit_pairs() {
        let mut vic = VicII::new();

        // Enable multicolor mode (MCM bit in $D016)
        vic.registers[0x16] |= 0x10;
        assert!(vic.multicolor_mode());

        // Set up background colors
        vic.registers[0x21] = 0x00; // Background 0: black
        vic.registers[0x22] = 0x01; // Background 1: white
        vic.registers[0x23] = 0x02; // Background 2: red

        // Create character ROM with test pattern
        // Pattern 0b00_01_10_11 = 0x1B tests all four bit pairs
        let mut char_rom = vec![0u8; 2048];
        for i in 0..8 {
            char_rom[8 + i] = 0x1B; // Character 1 with pattern 00 01 10 11
        }

        // Screen RAM: character 1 at position 0
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 1;

        // Color RAM: bit 3 SET enables multicolor, bits 0-2 = foreground color (3 = cyan)
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x0B; // bit 3 set + color 3

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Pattern 0x1B = 0b00011011
        // Bit pair 0 (bits 7-6): 00 -> background 0 (black = 0)
        // Bit pair 1 (bits 5-4): 01 -> background 1 (white = 1)
        // Bit pair 2 (bits 3-2): 10 -> background 2 (red = 2)
        // Bit pair 3 (bits 1-0): 11 -> foreground (cyan = 3, from color RAM bits 0-2)

        // Each bit pair produces 2 identical pixels
        assert_eq!(vic.framebuffer[0][0], 0, "Pixel 0 should be background 0");
        assert_eq!(vic.framebuffer[0][1], 0, "Pixel 1 should be background 0");
        assert_eq!(vic.framebuffer[0][2], 1, "Pixel 2 should be background 1");
        assert_eq!(vic.framebuffer[0][3], 1, "Pixel 3 should be background 1");
        assert_eq!(vic.framebuffer[0][4], 2, "Pixel 4 should be background 2");
        assert_eq!(vic.framebuffer[0][5], 2, "Pixel 5 should be background 2");
        assert_eq!(vic.framebuffer[0][6], 3, "Pixel 6 should be foreground");
        assert_eq!(vic.framebuffer[0][7], 3, "Pixel 7 should be foreground");
    }

    #[test]
    fn test_multicolor_text_mode_hires_character() {
        let mut vic = VicII::new();

        // Enable multicolor mode globally
        vic.registers[0x16] |= 0x10;

        // Set up background colors
        vic.registers[0x21] = 0x00; // Background 0: black

        // Create character ROM
        let mut char_rom = vec![0u8; 2048];
        // Character 1: alternating pattern 0xAA (10101010)
        for i in 0..8 {
            char_rom[8 + i] = 0xAA;
        }

        // Screen RAM: character 1 at position 0
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 1;

        // Color RAM: bit 3 CLEAR means use hires mode for this character
        // Color = 5 (green), no multicolor bit
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x05; // bit 3 clear, full 16 colors available

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // In hires mode, pattern 0xAA = 10101010 means alternating pixels
        // Bit 7=1: foreground (green = 5)
        // Bit 6=0: background (black = 0)
        // etc.
        assert_eq!(
            vic.framebuffer[0][0], 5,
            "Pixel 0 should be foreground (hires)"
        );
        assert_eq!(
            vic.framebuffer[0][1], 0,
            "Pixel 1 should be background (hires)"
        );
        assert_eq!(
            vic.framebuffer[0][2], 5,
            "Pixel 2 should be foreground (hires)"
        );
        assert_eq!(
            vic.framebuffer[0][3], 0,
            "Pixel 3 should be background (hires)"
        );
    }

    #[test]
    fn test_multicolor_text_mode_mixed_characters() {
        let mut vic = VicII::new();

        // Enable multicolor mode globally
        vic.registers[0x16] |= 0x10;

        // Set up background colors
        vic.registers[0x21] = 0x00; // Background 0: black
        vic.registers[0x22] = 0x01; // Background 1: white
        vic.registers[0x23] = 0x02; // Background 2: red

        // Create character ROM
        let mut char_rom = vec![0u8; 2048];
        // Character 1: all bits set (0xFF)
        for i in 0..8 {
            char_rom[8 + i] = 0xFF;
        }
        // Character 2: all bits set (0xFF)
        for i in 0..8 {
            char_rom[16 + i] = 0xFF;
        }

        // Screen RAM: char 1 at position 0 (multicolor), char 2 at position 1 (hires)
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 1;
        screen_ram[1] = 2;

        // Color RAM:
        // Char 0: multicolor (bit 3 set), foreground = 3
        // Char 1: hires (bit 3 clear), foreground = 7
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x0B; // bit 3 set + color 3
        color_ram[1] = 0x07; // bit 3 clear + color 7

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // First character (multicolor, pattern 0xFF = 11_11_11_11)
        // All bit pairs are 11, so all pixels should be foreground color 3
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[0][x], 3,
                "Char 0 pixel {} should be multicolor foreground",
                x
            );
        }

        // Second character (hires, pattern 0xFF = all bits set)
        // All pixels should be foreground color 7
        for x in 8..16 {
            assert_eq!(
                vic.framebuffer[0][x], 7,
                "Char 1 pixel {} should be hires foreground",
                x
            );
        }
    }

    #[test]
    fn test_multicolor_text_mode_only_8_colors() {
        let mut vic = VicII::new();

        // Enable multicolor mode
        vic.registers[0x16] |= 0x10;

        // Create character ROM with foreground pattern (11 bit pairs)
        let mut char_rom = vec![0u8; 2048];
        for i in 0..8 {
            char_rom[8 + i] = 0xFF; // All 11 bit pairs
        }

        // Screen RAM
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 1;

        // Color RAM: multicolor with color 15 in bits 0-3
        // But in multicolor mode, only bits 0-2 are used (8 colors max)
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x0F; // bit 3 set (multicolor), bits 0-2 = 7

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Foreground should be 7 (bits 0-2 of 0x0F), not 15
        // (bit 3 is the multicolor enable flag, not part of color)
        assert_eq!(
            vic.framebuffer[0][0], 7,
            "Multicolor foreground should use only bits 0-2"
        );
    }

    // =========================================================================
    // Standard Bitmap Mode Tests (BMM=1, ECM=0, MCM=0)
    // =========================================================================

    #[test]
    fn test_standard_bitmap_mode_basic_rendering() {
        let mut vic = VicII::new();

        // Enable bitmap mode (BMM bit in $D011)
        vic.registers[0x11] |= 0x20;
        assert!(vic.bitmap_mode());
        assert!(!vic.multicolor_mode());

        // Create bitmap data (8000 bytes)
        // We'll set up a simple pattern: first cell has alternating pixels
        let mut bitmap_data = vec![0u8; 8000];
        // Cell at row 0, col 0, line 0: pattern 0xAA (10101010)
        bitmap_data[0] = 0xAA;

        // Screen RAM provides colors: upper nibble = foreground, lower nibble = background
        // Cell 0: foreground = white (1), background = black (0)
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x10; // Upper nibble = 1 (white), lower nibble = 0 (black)

        let color_ram = vec![0u8; 1000];

        // Render first visible scanline (line 51 = display_line 0)
        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Pattern 0xAA = 10101010
        // Bit 7=1: foreground (white = 1)
        // Bit 6=0: background (black = 0)
        // Bit 5=1: foreground
        // etc.
        assert_eq!(vic.framebuffer[0][0], 1, "Pixel 0 should be foreground (white)");
        assert_eq!(vic.framebuffer[0][1], 0, "Pixel 1 should be background (black)");
        assert_eq!(vic.framebuffer[0][2], 1, "Pixel 2 should be foreground");
        assert_eq!(vic.framebuffer[0][3], 0, "Pixel 3 should be background");
        assert_eq!(vic.framebuffer[0][4], 1, "Pixel 4 should be foreground");
        assert_eq!(vic.framebuffer[0][5], 0, "Pixel 5 should be background");
        assert_eq!(vic.framebuffer[0][6], 1, "Pixel 6 should be foreground");
        assert_eq!(vic.framebuffer[0][7], 0, "Pixel 7 should be background");
    }

    #[test]
    fn test_standard_bitmap_mode_multiple_cells() {
        let mut vic = VicII::new();

        // Enable bitmap mode
        vic.registers[0x11] |= 0x20;

        // Create bitmap data
        let mut bitmap_data = vec![0u8; 8000];
        // Cell 0, line 0: all pixels on (0xFF)
        bitmap_data[0] = 0xFF;
        // Cell 1, line 0: all pixels off (0x00)
        bitmap_data[8] = 0x00;

        // Screen RAM with different colors for each cell
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x23; // Cell 0: fg=red(2), bg=cyan(3)
        screen_ram[1] = 0x56; // Cell 1: fg=green(5), bg=blue(6)

        let color_ram = vec![0u8; 1000];

        // Render first visible scanline
        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // First cell (all pixels set) should be foreground (red = 2)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[0][x], 2,
                "Cell 0 pixel {} should be red (foreground)",
                x
            );
        }

        // Second cell (all pixels clear) should be background (blue = 6)
        for x in 8..16 {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "Cell 1 pixel {} should be blue (background)",
                x
            );
        }
    }

    #[test]
    fn test_standard_bitmap_mode_scanline_progression() {
        let mut vic = VicII::new();

        // Enable bitmap mode
        vic.registers[0x11] |= 0x20;

        // Create bitmap data with different patterns for each line within the cell
        let mut bitmap_data = vec![0u8; 8000];
        // Cell 0, lines 0-7: different patterns for each line
        bitmap_data[0] = 0xFF; // Line 0: all set
        bitmap_data[1] = 0x00; // Line 1: all clear
        bitmap_data[2] = 0xF0; // Line 2: left half set
        bitmap_data[3] = 0x0F; // Line 3: right half set
        bitmap_data[4] = 0xAA; // Line 4: alternating
        bitmap_data[5] = 0x55; // Line 5: alternating (opposite)
        bitmap_data[6] = 0xCC; // Line 6: pairs
        bitmap_data[7] = 0x33; // Line 7: pairs (opposite)

        // Screen RAM: white foreground, black background
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x10; // fg=white(1), bg=black(0)

        let color_ram = vec![0u8; 1000];

        // Render multiple scanlines
        for scanline_offset in 0..8 {
            vic.step_scanline(51 + scanline_offset, &bitmap_data, &screen_ram, &color_ram);
        }

        // Check line 0: all white (0xFF -> all foreground)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 1, "Line 0, pixel {} should be white", x);
        }

        // Check line 1: all black (0x00 -> all background)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[1][x], 0, "Line 1, pixel {} should be black", x);
        }

        // Check line 2: left half white (0xF0)
        for x in 0..4 {
            assert_eq!(vic.framebuffer[2][x], 1, "Line 2, pixel {} should be white", x);
        }
        for x in 4..8 {
            assert_eq!(vic.framebuffer[2][x], 0, "Line 2, pixel {} should be black", x);
        }

        // Check line 3: right half white (0x0F)
        for x in 0..4 {
            assert_eq!(vic.framebuffer[3][x], 0, "Line 3, pixel {} should be black", x);
        }
        for x in 4..8 {
            assert_eq!(vic.framebuffer[3][x], 1, "Line 3, pixel {} should be white", x);
        }
    }

    #[test]
    fn test_standard_bitmap_mode_multiple_rows() {
        let mut vic = VicII::new();

        // Enable bitmap mode
        vic.registers[0x11] |= 0x20;

        // Create bitmap data
        let mut bitmap_data = vec![0u8; 8000];

        // Row 0, cell 0: pattern 0xFF
        bitmap_data[0] = 0xFF;

        // Row 1, cell 0 (starts at offset 320): pattern 0xAA
        // Row offset = row * 320, then + cell * 8 + line
        bitmap_data[320] = 0xAA;

        // Screen RAM with different colors
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x12;  // Row 0, Cell 0: fg=1, bg=2
        screen_ram[40] = 0x34; // Row 1, Cell 0: fg=3, bg=4

        let color_ram = vec![0u8; 1000];

        // Render row 0, line 0
        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Row 0 cell 0 should be all foreground (1)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 1, "Row 0 pixel {} should be 1", x);
        }

        // Render row 1, line 0 (scanline 51 + 8 = 59)
        vic.step_scanline(59, &bitmap_data, &screen_ram, &color_ram);

        // Row 1 cell 0 should be alternating (pattern 0xAA with fg=3, bg=4)
        // 10101010: pixel 0=fg(3), pixel 1=bg(4), etc.
        assert_eq!(vic.framebuffer[8][0], 3, "Row 1 pixel 0 should be foreground");
        assert_eq!(vic.framebuffer[8][1], 4, "Row 1 pixel 1 should be background");
    }

    #[test]
    fn test_standard_bitmap_mode_full_row_cells() {
        let mut vic = VicII::new();

        // Enable bitmap mode
        vic.registers[0x11] |= 0x20;

        // Create bitmap data
        let mut bitmap_data = vec![0u8; 8000];

        // Set up cells 0, 10, 20, 30, 39 with different patterns
        bitmap_data[0] = 0xFF;      // Cell 0: all on
        bitmap_data[80] = 0x00;     // Cell 10: all off
        bitmap_data[160] = 0xF0;    // Cell 20: left half
        bitmap_data[240] = 0x0F;    // Cell 30: right half
        bitmap_data[312] = 0xAA;    // Cell 39: alternating

        // Screen RAM with distinct colors for each cell
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x10;   // Cell 0: white/black
        screen_ram[10] = 0x23;  // Cell 10: red/cyan
        screen_ram[20] = 0x45;  // Cell 20: purple/green
        screen_ram[30] = 0x67;  // Cell 30: blue/yellow
        screen_ram[39] = 0x89;  // Cell 39: orange/brown

        let color_ram = vec![0u8; 1000];

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Cell 0 (x=0-7): all foreground (white=1)
        assert_eq!(vic.framebuffer[0][0], 1, "Cell 0 pixel should be white");

        // Cell 10 (x=80-87): all background (cyan=3)
        assert_eq!(vic.framebuffer[0][80], 3, "Cell 10 pixel should be cyan (bg)");

        // Cell 20 (x=160-167): left half foreground (purple=4), right half background (green=5)
        assert_eq!(vic.framebuffer[0][160], 4, "Cell 20 left pixel should be purple");
        assert_eq!(vic.framebuffer[0][164], 5, "Cell 20 right pixel should be green");

        // Cell 30 (x=240-247): left half background (yellow=7), right half foreground (blue=6)
        assert_eq!(vic.framebuffer[0][240], 7, "Cell 30 left pixel should be yellow (bg)");
        assert_eq!(vic.framebuffer[0][244], 6, "Cell 30 right pixel should be blue (fg)");

        // Cell 39 (x=312-319): alternating orange(8)/brown(9)
        assert_eq!(vic.framebuffer[0][312], 8, "Cell 39 pixel 0 should be orange (fg)");
        assert_eq!(vic.framebuffer[0][313], 9, "Cell 39 pixel 1 should be brown (bg)");
    }

    #[test]
    fn test_standard_bitmap_mode_flag_detection() {
        let mut vic = VicII::new();

        // Initially not in bitmap mode
        assert!(!vic.bitmap_mode());

        // Enable bitmap mode (BMM bit = bit 5 of $D011)
        vic.registers[0x11] |= 0x20;
        assert!(vic.bitmap_mode());

        // Disable bitmap mode
        vic.registers[0x11] &= !0x20;
        assert!(!vic.bitmap_mode());
    }

    #[test]
    fn test_standard_bitmap_mode_disabled_display() {
        let mut vic = VicII::new();

        // Enable bitmap mode but disable display (DEN=0)
        vic.registers[0x11] = 0x20; // BMM=1, DEN=0 (bit 4)

        let bitmap_data = vec![0xFFu8; 8000]; // All pixels set
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x12;
        let color_ram = vec![0u8; 1000];

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // All pixels should be background color (6 = blue, default)
        for x in 0..SCREEN_WIDTH {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "Pixel {} should be background when display disabled",
                x
            );
        }
    }

    // =========================================================================
    // Multicolor Bitmap Mode Tests (BMM=1, ECM=0, MCM=1)
    // =========================================================================

    #[test]
    fn test_multicolor_bitmap_mode_basic_rendering() {
        let mut vic = VicII::new();

        // Enable bitmap mode (BMM bit in $D011) and multicolor mode (MCM in $D016)
        vic.registers[0x11] |= 0x20; // BMM=1
        vic.registers[0x16] |= 0x10; // MCM=1
        assert!(vic.bitmap_mode());
        assert!(vic.multicolor_mode());

        // Set background color 0
        vic.registers[0x21] = 0x00; // Black

        // Create bitmap data (8000 bytes)
        // Pattern 0x1B = 0b00011011 tests all four bit pairs
        let mut bitmap_data = vec![0u8; 8000];
        bitmap_data[0] = 0x1B; // Cell 0, line 0

        // Screen RAM provides colors for bit pairs 01 (upper nibble) and 10 (lower nibble)
        // Cell 0: upper nibble = 1 (white), lower nibble = 2 (red)
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x12;

        // Color RAM provides color for bit pair 11
        // Cell 0: color 3 (cyan)
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x03;

        // Render first visible scanline
        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Pattern 0x1B = 0b00011011
        // Bit pair 0 (bits 7-6): 00 -> background 0 (black = 0)
        // Bit pair 1 (bits 5-4): 01 -> screen RAM upper nibble (white = 1)
        // Bit pair 2 (bits 3-2): 10 -> screen RAM lower nibble (red = 2)
        // Bit pair 3 (bits 1-0): 11 -> color RAM (cyan = 3)

        // Each bit pair produces 2 identical pixels
        assert_eq!(vic.framebuffer[0][0], 0, "Pixel 0 should be background 0 (black)");
        assert_eq!(vic.framebuffer[0][1], 0, "Pixel 1 should be background 0 (black)");
        assert_eq!(vic.framebuffer[0][2], 1, "Pixel 2 should be screen upper (white)");
        assert_eq!(vic.framebuffer[0][3], 1, "Pixel 3 should be screen upper (white)");
        assert_eq!(vic.framebuffer[0][4], 2, "Pixel 4 should be screen lower (red)");
        assert_eq!(vic.framebuffer[0][5], 2, "Pixel 5 should be screen lower (red)");
        assert_eq!(vic.framebuffer[0][6], 3, "Pixel 6 should be color RAM (cyan)");
        assert_eq!(vic.framebuffer[0][7], 3, "Pixel 7 should be color RAM (cyan)");
    }

    #[test]
    fn test_multicolor_bitmap_mode_all_same_color() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20; // BMM=1
        vic.registers[0x16] |= 0x10; // MCM=1

        // Set background color 0
        vic.registers[0x21] = 0x05; // Green

        // Bitmap data: all 00 bit pairs (background color)
        let mut bitmap_data = vec![0u8; 8000];
        bitmap_data[0] = 0x00; // All 00 pairs

        let screen_ram = vec![0xFFu8; 1000];
        let color_ram = vec![0xFFu8; 1000];

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // All pixels should be background 0 (green = 5)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 5, "Pixel {} should be green (bg0)", x);
        }
    }

    #[test]
    fn test_multicolor_bitmap_mode_multiple_cells() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20;
        vic.registers[0x16] |= 0x10;

        vic.registers[0x21] = 0x00; // Background 0: black

        // Create bitmap data with different patterns for each cell
        let mut bitmap_data = vec![0u8; 8000];
        bitmap_data[0] = 0x00;  // Cell 0: all 00 (background)
        bitmap_data[8] = 0x55;  // Cell 1: all 01 (screen upper nibble)
        bitmap_data[16] = 0xAA; // Cell 2: all 10 (screen lower nibble)
        bitmap_data[24] = 0xFF; // Cell 3: all 11 (color RAM)

        // Screen RAM with different colors for each cell
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x00; // Cell 0: doesn't matter, all bg
        screen_ram[1] = 0x10; // Cell 1: upper=1 (white), lower=0
        screen_ram[2] = 0x02; // Cell 2: upper=0, lower=2 (red)
        screen_ram[3] = 0x00; // Cell 3: doesn't matter for 11 pattern

        // Color RAM
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x00;
        color_ram[1] = 0x00;
        color_ram[2] = 0x00;
        color_ram[3] = 0x03; // Cell 3: cyan

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Cell 0 (x=0-7): all background (black=0)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 0, "Cell 0 pixel {} should be black", x);
        }

        // Cell 1 (x=8-15): all 01 pattern -> screen upper nibble (white=1)
        for x in 8..16 {
            assert_eq!(vic.framebuffer[0][x], 1, "Cell 1 pixel {} should be white", x);
        }

        // Cell 2 (x=16-23): all 10 pattern -> screen lower nibble (red=2)
        for x in 16..24 {
            assert_eq!(vic.framebuffer[0][x], 2, "Cell 2 pixel {} should be red", x);
        }

        // Cell 3 (x=24-31): all 11 pattern -> color RAM (cyan=3)
        for x in 24..32 {
            assert_eq!(vic.framebuffer[0][x], 3, "Cell 3 pixel {} should be cyan", x);
        }
    }

    #[test]
    fn test_multicolor_bitmap_mode_scanline_progression() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20;
        vic.registers[0x16] |= 0x10;

        vic.registers[0x21] = 0x00; // Background 0: black

        // Create bitmap data with different patterns for each line within the cell
        let mut bitmap_data = vec![0u8; 8000];
        bitmap_data[0] = 0x00; // Line 0: all 00 (bg)
        bitmap_data[1] = 0x55; // Line 1: all 01
        bitmap_data[2] = 0xAA; // Line 2: all 10
        bitmap_data[3] = 0xFF; // Line 3: all 11
        bitmap_data[4] = 0x1B; // Line 4: 00 01 10 11

        // Screen RAM: upper=5 (green), lower=6 (blue)
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x56;

        // Color RAM: color 7 (yellow)
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x07;

        // Render multiple scanlines
        for scanline_offset in 0..5 {
            vic.step_scanline(51 + scanline_offset, &bitmap_data, &screen_ram, &color_ram);
        }

        // Line 0: all background (0)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 0, "Line 0, pixel {} should be bg", x);
        }

        // Line 1: all 01 -> green (5)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[1][x], 5, "Line 1, pixel {} should be green", x);
        }

        // Line 2: all 10 -> blue (6)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[2][x], 6, "Line 2, pixel {} should be blue", x);
        }

        // Line 3: all 11 -> yellow (7)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[3][x], 7, "Line 3, pixel {} should be yellow", x);
        }

        // Line 4: mixed pattern 00 01 10 11
        assert_eq!(vic.framebuffer[4][0], 0, "Line 4, pixels 0-1 should be bg");
        assert_eq!(vic.framebuffer[4][1], 0);
        assert_eq!(vic.framebuffer[4][2], 5, "Line 4, pixels 2-3 should be green");
        assert_eq!(vic.framebuffer[4][3], 5);
        assert_eq!(vic.framebuffer[4][4], 6, "Line 4, pixels 4-5 should be blue");
        assert_eq!(vic.framebuffer[4][5], 6);
        assert_eq!(vic.framebuffer[4][6], 7, "Line 4, pixels 6-7 should be yellow");
        assert_eq!(vic.framebuffer[4][7], 7);
    }

    #[test]
    fn test_multicolor_bitmap_mode_multiple_rows() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20;
        vic.registers[0x16] |= 0x10;

        vic.registers[0x21] = 0x00; // Background 0: black

        // Create bitmap data
        let mut bitmap_data = vec![0u8; 8000];
        // Row 0, cell 0, line 0: all 11
        bitmap_data[0] = 0xFF;
        // Row 1, cell 0, line 0: all 10
        bitmap_data[320] = 0xAA;

        // Screen RAM
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x12;  // Row 0, Cell 0: upper=1, lower=2
        screen_ram[40] = 0x34; // Row 1, Cell 0: upper=3, lower=4

        // Color RAM
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x05;  // Row 0, Cell 0: 5
        color_ram[40] = 0x06; // Row 1, Cell 0: 6

        // Render row 0, line 0
        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Row 0 cell 0 should be all color RAM (5)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 5, "Row 0 pixel {} should be 5", x);
        }

        // Render row 1, line 0 (scanline 51 + 8 = 59)
        vic.step_scanline(59, &bitmap_data, &screen_ram, &color_ram);

        // Row 1 cell 0 should be all screen lower nibble (4)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[8][x], 4, "Row 1 pixel {} should be 4", x);
        }
    }

    #[test]
    fn test_multicolor_bitmap_mode_flag_detection() {
        let mut vic = VicII::new();

        // Initially neither bitmap nor multicolor mode
        assert!(!vic.bitmap_mode());
        assert!(!vic.multicolor_mode());

        // Enable only bitmap mode
        vic.registers[0x11] |= 0x20;
        assert!(vic.bitmap_mode());
        assert!(!vic.multicolor_mode());

        // Enable multicolor mode too
        vic.registers[0x16] |= 0x10;
        assert!(vic.bitmap_mode());
        assert!(vic.multicolor_mode());

        // Disable bitmap mode, keep multicolor (this would be multicolor text mode)
        vic.registers[0x11] &= !0x20;
        assert!(!vic.bitmap_mode());
        assert!(vic.multicolor_mode());
    }

    #[test]
    fn test_multicolor_bitmap_mode_disabled_display() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode but disable display (DEN=0)
        vic.registers[0x11] = 0x20; // BMM=1, DEN=0
        vic.registers[0x16] |= 0x10; // MCM=1

        let bitmap_data = vec![0xFFu8; 8000]; // All pixels set to 11
        let screen_ram = vec![0xFFu8; 1000];
        let color_ram = vec![0x0Fu8; 1000]; // Should be white if displayed

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // All pixels should be background color (6 = blue, default)
        for x in 0..SCREEN_WIDTH {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "Pixel {} should be background when display disabled",
                x
            );
        }
    }

    #[test]
    fn test_multicolor_bitmap_mode_full_16_colors_color_ram() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20;
        vic.registers[0x16] |= 0x10;

        vic.registers[0x21] = 0x00; // Background 0: black

        // Create bitmap data: all 11 bit pairs
        let bitmap_data = vec![0xFFu8; 8000];

        // Screen RAM (doesn't matter for 11 pattern)
        let screen_ram = vec![0u8; 1000];

        // Color RAM with various colors to test full 16-color range
        let mut color_ram = vec![0u8; 1000];
        for i in 0..16 {
            color_ram[i] = i as u8;
        }

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Each cell should display its color RAM value
        for cell in 0..16 {
            let x = cell * 8;
            assert_eq!(
                vic.framebuffer[0][x],
                cell as u8,
                "Cell {} should have color {}",
                cell,
                cell
            );
        }
    }

    #[test]
    fn test_multicolor_bitmap_mode_full_row_coverage() {
        let mut vic = VicII::new();

        // Enable multicolor bitmap mode
        vic.registers[0x11] |= 0x20;
        vic.registers[0x16] |= 0x10;

        vic.registers[0x21] = 0x0E; // Background 0: light blue

        // Create bitmap data with alternating patterns across all 40 cells
        let mut bitmap_data = vec![0u8; 8000];
        for cell in 0..40 {
            // Alternate between all 4 bit pair patterns
            let pattern = match cell % 4 {
                0 => 0x00, // 00 00 00 00
                1 => 0x55, // 01 01 01 01
                2 => 0xAA, // 10 10 10 10
                _ => 0xFF, // 11 11 11 11
            };
            bitmap_data[cell * 8] = pattern;
        }

        // Screen RAM with distinct colors
        let mut screen_ram = vec![0u8; 1000];
        for cell in 0..40 {
            screen_ram[cell] = 0x12; // upper=1, lower=2
        }

        // Color RAM
        let mut color_ram = vec![0u8; 1000];
        for cell in 0..40 {
            color_ram[cell] = 0x03; // cyan
        }

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Verify the pattern across all 40 cells
        for cell in 0..40 {
            let x_base = cell * 8;
            let expected_color = match cell % 4 {
                0 => 0x0E, // bg0 (light blue)
                1 => 1,    // screen upper (white)
                2 => 2,    // screen lower (red)
                _ => 3,    // color RAM (cyan)
            };

            // All 8 pixels in the cell should have the same color
            for pixel in 0..8 {
                assert_eq!(
                    vic.framebuffer[0][x_base + pixel],
                    expected_color,
                    "Cell {} pixel {} should be color {}",
                    cell,
                    pixel,
                    expected_color
                );
            }
        }
    }

    // =========================================================================
    // ECM (Extended Color Mode) Text Mode Tests (BMM=0, ECM=1, MCM=0)
    // =========================================================================

    #[test]
    fn test_ecm_text_mode_basic_rendering() {
        let mut vic = VicII::new();

        // Enable ECM mode (ECM bit = bit 6 of $D011)
        vic.registers[0x11] |= 0x40;
        assert!(vic.extended_color_mode());
        assert!(!vic.bitmap_mode());
        assert!(!vic.multicolor_mode());

        // Set up 4 background colors
        vic.registers[0x21] = 0x00; // Background 0: black
        vic.registers[0x22] = 0x01; // Background 1: white
        vic.registers[0x23] = 0x02; // Background 2: red
        vic.registers[0x24] = 0x03; // Background 3: cyan

        // Create character ROM with a simple pattern
        // Character 1 (index 1): all bits set (0xFF) for full coverage
        let mut char_rom = vec![0u8; 2048];
        for i in 0..8 {
            char_rom[8 + i] = 0xFF; // Character 1: all pixels on
        }

        // Screen RAM with character codes selecting different background colors
        // Char code bits 6-7 select background, bits 0-5 select character pattern
        let mut screen_ram = vec![0u8; 1000];
        // Character index 1 with background 0 (bits 6-7 = 00)
        screen_ram[0] = 0x01; // 00_000001 -> bg0, char 1
        // Character index 1 with background 1 (bits 6-7 = 01)
        screen_ram[1] = 0x41; // 01_000001 -> bg1, char 1
        // Character index 1 with background 2 (bits 6-7 = 10)
        screen_ram[2] = 0x81; // 10_000001 -> bg2, char 1
        // Character index 1 with background 3 (bits 6-7 = 11)
        screen_ram[3] = 0xC1; // 11_000001 -> bg3, char 1

        // Color RAM: foreground color 5 (green) for all characters
        let mut color_ram = vec![0u8; 1000];
        for i in 0..4 {
            color_ram[i] = 0x05;
        }

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // All 4 cells have character 1 (all pixels set), so all should be foreground (green = 5)
        for cell in 0..4 {
            for pixel in 0..8 {
                let x = cell * 8 + pixel;
                assert_eq!(
                    vic.framebuffer[0][x], 5,
                    "Cell {} pixel {} should be foreground (green)",
                    cell, pixel
                );
            }
        }
    }

    #[test]
    fn test_ecm_text_mode_background_selection() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set up 4 distinct background colors
        vic.registers[0x21] = 0x00; // Background 0: black
        vic.registers[0x22] = 0x01; // Background 1: white
        vic.registers[0x23] = 0x02; // Background 2: red
        vic.registers[0x24] = 0x03; // Background 3: cyan

        // Create character ROM where character 0 has all pixels OFF
        // This way we'll see the background colors
        let char_rom = vec![0u8; 2048]; // All zeros = all pixels off

        // Screen RAM with character 0 (all pixels off) but different background selectors
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x00; // 00_000000 -> bg0, char 0
        screen_ram[1] = 0x40; // 01_000000 -> bg1, char 0
        screen_ram[2] = 0x80; // 10_000000 -> bg2, char 0
        screen_ram[3] = 0xC0; // 11_000000 -> bg3, char 0

        // Color RAM (doesn't matter since all pixels are off)
        let color_ram = vec![0x0Fu8; 1000];

        // Render first visible scanline
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Cell 0: background 0 (black = 0)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][pixel], 0,
                "Cell 0 pixel {} should be bg0 (black)",
                pixel
            );
        }

        // Cell 1: background 1 (white = 1)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][8 + pixel], 1,
                "Cell 1 pixel {} should be bg1 (white)",
                pixel
            );
        }

        // Cell 2: background 2 (red = 2)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][16 + pixel], 2,
                "Cell 2 pixel {} should be bg2 (red)",
                pixel
            );
        }

        // Cell 3: background 3 (cyan = 3)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][24 + pixel], 3,
                "Cell 3 pixel {} should be bg3 (cyan)",
                pixel
            );
        }
    }

    #[test]
    fn test_ecm_text_mode_64_character_limit() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set backgrounds
        vic.registers[0x21] = 0x00;

        // Create character ROM with distinct patterns for chars 0-63
        // We'll use a simple pattern: each char's first line equals char index
        let mut char_rom = vec![0u8; 2048];
        for i in 0..64 {
            // Make each character have a unique pattern
            char_rom[i * 8] = 0xFF; // First line all on
        }
        // Characters 64-255 should NOT be accessed in ECM mode

        // Screen RAM: Test that bits 6-7 select background, not character
        let mut screen_ram = vec![0u8; 1000];
        // Code 0x00 = bg0, char 0
        screen_ram[0] = 0x00;
        // Code 0x40 = bg1, char 0 (NOT char 64!)
        screen_ram[1] = 0x40;
        // Code 0x80 = bg2, char 0 (NOT char 128!)
        screen_ram[2] = 0x80;
        // Code 0xC0 = bg3, char 0 (NOT char 192!)
        screen_ram[3] = 0xC0;

        // All should use character 0's pattern, which has all pixels on
        let mut color_ram = vec![0u8; 1000];
        for i in 0..4 {
            color_ram[i] = 0x05; // green foreground
        }

        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // All cells should show foreground (green = 5) because char 0 has all pixels on
        for cell in 0..4 {
            assert_eq!(
                vic.framebuffer[0][cell * 8], 5,
                "Cell {} should use char 0 pattern (foreground)",
                cell
            );
        }
    }

    #[test]
    fn test_ecm_text_mode_full_16_foreground_colors() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set background colors
        vic.registers[0x21] = 0x00;

        // Character ROM: char 0 has all pixels ON
        let mut char_rom = vec![0u8; 2048];
        for i in 0..8 {
            char_rom[i] = 0xFF;
        }

        // Screen RAM: all char 0
        let screen_ram = vec![0u8; 1000];

        // Color RAM: each cell has a different foreground color (0-15)
        let mut color_ram = vec![0u8; 1000];
        for i in 0..16 {
            color_ram[i] = i as u8;
        }

        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Each cell should have its own foreground color
        for cell in 0..16 {
            assert_eq!(
                vic.framebuffer[0][cell * 8],
                cell as u8,
                "Cell {} should have foreground color {}",
                cell,
                cell
            );
        }
    }

    #[test]
    fn test_ecm_text_mode_mixed_foreground_background() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set up distinct background colors
        vic.registers[0x21] = 0x06; // bg0: blue
        vic.registers[0x22] = 0x07; // bg1: yellow
        vic.registers[0x23] = 0x08; // bg2: orange
        vic.registers[0x24] = 0x09; // bg3: brown

        // Character ROM: char 0 has alternating pattern (0xAA = 10101010)
        let mut char_rom = vec![0u8; 2048];
        for i in 0..8 {
            char_rom[i] = 0xAA;
        }

        // Screen RAM: char 0 with different backgrounds
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x00; // bg0
        screen_ram[1] = 0x40; // bg1
        screen_ram[2] = 0x80; // bg2
        screen_ram[3] = 0xC0; // bg3

        // Color RAM: foreground color 1 (white)
        let mut color_ram = vec![0u8; 1000];
        for i in 0..4 {
            color_ram[i] = 0x01;
        }

        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Pattern 0xAA = 10101010
        // Pixel 0 = 1 (foreground = white)
        // Pixel 1 = 0 (background)
        // etc.

        // Cell 0: foreground=white(1), background=blue(6)
        assert_eq!(vic.framebuffer[0][0], 1, "Cell 0 pixel 0 should be white (fg)");
        assert_eq!(vic.framebuffer[0][1], 6, "Cell 0 pixel 1 should be blue (bg0)");

        // Cell 1: foreground=white(1), background=yellow(7)
        assert_eq!(vic.framebuffer[0][8], 1, "Cell 1 pixel 0 should be white (fg)");
        assert_eq!(vic.framebuffer[0][9], 7, "Cell 1 pixel 1 should be yellow (bg1)");

        // Cell 2: foreground=white(1), background=orange(8)
        assert_eq!(vic.framebuffer[0][16], 1, "Cell 2 pixel 0 should be white (fg)");
        assert_eq!(vic.framebuffer[0][17], 8, "Cell 2 pixel 1 should be orange (bg2)");

        // Cell 3: foreground=white(1), background=brown(9)
        assert_eq!(vic.framebuffer[0][24], 1, "Cell 3 pixel 0 should be white (fg)");
        assert_eq!(vic.framebuffer[0][25], 9, "Cell 3 pixel 1 should be brown (bg3)");
    }

    #[test]
    fn test_ecm_text_mode_scanline_progression() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set backgrounds
        vic.registers[0x21] = 0x00; // black

        // Character ROM: char 0 has different patterns per line
        let mut char_rom = vec![0u8; 2048];
        char_rom[0] = 0xFF; // Line 0: all on
        char_rom[1] = 0x00; // Line 1: all off
        char_rom[2] = 0xF0; // Line 2: left half on
        char_rom[3] = 0x0F; // Line 3: right half on

        // Screen RAM: char 0 with bg0
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x00;

        // Color RAM: white foreground
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x01;

        // Render 4 scanlines
        for line in 0..4 {
            vic.step_scanline(51 + line as u16, &char_rom, &screen_ram, &color_ram);
        }

        // Line 0: all foreground (white)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[0][x], 1, "Line 0, pixel {} should be white", x);
        }

        // Line 1: all background (black)
        for x in 0..8 {
            assert_eq!(vic.framebuffer[1][x], 0, "Line 1, pixel {} should be black", x);
        }

        // Line 2: left half foreground, right half background
        for x in 0..4 {
            assert_eq!(vic.framebuffer[2][x], 1, "Line 2, pixel {} should be white", x);
        }
        for x in 4..8 {
            assert_eq!(vic.framebuffer[2][x], 0, "Line 2, pixel {} should be black", x);
        }

        // Line 3: left half background, right half foreground
        for x in 0..4 {
            assert_eq!(vic.framebuffer[3][x], 0, "Line 3, pixel {} should be black", x);
        }
        for x in 4..8 {
            assert_eq!(vic.framebuffer[3][x], 1, "Line 3, pixel {} should be white", x);
        }
    }

    #[test]
    fn test_ecm_text_mode_flag_detection() {
        let mut vic = VicII::new();

        // Initially ECM should be off
        assert!(!vic.extended_color_mode());

        // Enable ECM (bit 6 of $D011)
        vic.registers[0x11] |= 0x40;
        assert!(vic.extended_color_mode());

        // Disable ECM
        vic.registers[0x11] &= !0x40;
        assert!(!vic.extended_color_mode());
    }

    #[test]
    fn test_ecm_text_mode_disabled_display() {
        let mut vic = VicII::new();

        // Enable ECM but disable display (DEN=0)
        vic.registers[0x11] = 0x40; // ECM=1, DEN=0

        let char_rom = vec![0xFFu8; 2048];
        let screen_ram = vec![0u8; 1000];
        let color_ram = vec![0x0Fu8; 1000];

        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // All pixels should be background color (6 = blue, default)
        for x in 0..SCREEN_WIDTH {
            assert_eq!(
                vic.framebuffer[0][x], 6,
                "Pixel {} should be background when display disabled",
                x
            );
        }
    }

    #[test]
    fn test_ecm_text_mode_full_row_coverage() {
        let mut vic = VicII::new();

        // Enable ECM mode
        vic.registers[0x11] |= 0x40;

        // Set up 4 background colors
        vic.registers[0x21] = 0x00; // bg0: black
        vic.registers[0x22] = 0x01; // bg1: white
        vic.registers[0x23] = 0x02; // bg2: red
        vic.registers[0x24] = 0x03; // bg3: cyan

        // Character ROM: char 0 has all pixels off
        let char_rom = vec![0u8; 2048];

        // Screen RAM: cycle through all 4 backgrounds
        let mut screen_ram = vec![0u8; 1000];
        for cell in 0..40 {
            let bg_select = (cell % 4) << 6;
            screen_ram[cell] = bg_select as u8;
        }

        let color_ram = vec![0x0Fu8; 1000]; // doesn't matter since all pixels off

        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Verify all 40 cells have correct background colors
        for cell in 0..40 {
            let expected_bg = (cell % 4) as u8;
            for pixel in 0..8 {
                let x = cell * 8 + pixel;
                assert_eq!(
                    vic.framebuffer[0][x],
                    expected_bg,
                    "Cell {} pixel {} should be bg{}",
                    cell,
                    pixel,
                    expected_bg
                );
            }
        }
    }
}
