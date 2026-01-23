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

/// Number of hardware sprites.
pub const SPRITE_COUNT: usize = 8;

/// Height of each sprite in pixels (21 scanlines).
pub const SPRITE_HEIGHT: usize = 21;

/// Width of each sprite in pixels (24 pixels = 3 bytes).
pub const SPRITE_WIDTH: usize = 24;

/// Bytes per sprite line (3 bytes = 24 pixels).
const SPRITE_BYTES_PER_LINE: usize = 3;

/// Total bytes per sprite data block (63 bytes).
pub const SPRITE_DATA_SIZE: usize = SPRITE_HEIGHT * SPRITE_BYTES_PER_LINE;

/// Offset within screen RAM where sprite pointers are located ($3F8).
/// This is Screen RAM base + $3F8 in VIC address space.
const SPRITE_POINTER_OFFSET: u16 = 0x03F8;

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

    /// Foreground mask: tracks which pixels have foreground graphics (for sprite priority).
    /// When a pixel is set, it means there's foreground content (not background color).
    /// Used by sprite-to-background priority (register $1B).
    foreground_mask: Box<[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]>,

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
            foreground_mask: Box::new([[false; SCREEN_WIDTH]; SCREEN_HEIGHT]),
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

    /// Set a pixel in the framebuffer and update the foreground mask.
    ///
    /// # Arguments
    /// * `line` - The display line (0-199)
    /// * `x` - The x coordinate (0-319)
    /// * `color` - The color index (0-15)
    /// * `is_foreground` - True if this is foreground graphics (not background color)
    #[inline(always)]
    fn set_pixel(&mut self, line: usize, x: usize, color: u8, is_foreground: bool) {
        self.framebuffer[line][x] = color;
        self.foreground_mask[line][x] = is_foreground;
    }

    /// Clear a scanline's foreground mask (called at start of each scanline).
    #[inline(always)]
    fn clear_foreground_mask_line(&mut self, line: usize) {
        for x in 0..SCREEN_WIDTH {
            self.foreground_mask[line][x] = false;
        }
    }

    /// Check if a pixel has foreground content (for sprite priority).
    #[inline(always)]
    fn is_foreground_pixel(&self, line: usize, x: usize) -> bool {
        self.foreground_mask[line][x]
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

        // Clear foreground mask for this scanline (for sprite priority)
        self.clear_foreground_mask_line(display_line);

        // If display is disabled (DEN=0), fill with background color
        if !self.display_enabled() {
            let bg_color = self.background_color();
            for x in 0..SCREEN_WIDTH {
                self.set_pixel(display_line, x, bg_color, false);
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
                self.render_multicolor_bitmap_scanline(
                    display_line,
                    char_rom,
                    screen_ram,
                    color_ram,
                );
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
                    self.set_pixel(display_line, x, bg_color, false);
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
                self.set_pixel(display_line, x, bg_color, false);
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
                // In text mode, foreground pixels (pixel_set) are graphics
                self.set_pixel(display_line, x_base + bit, color, pixel_set);
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
                self.set_pixel(display_line, x, bg_color, false);
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

                    // In multicolor text mode, bit pattern 00 is background (not foreground)
                    // Other patterns (01, 10, 11) are considered foreground for sprite priority
                    let is_foreground = bits != 0b00;

                    let color = match bits {
                        0b00 => bg_color_0, // Background 0
                        0b01 => bg_color_1, // Background 1
                        0b10 => bg_color_2, // Background 2
                        0b11 => fg_color,   // Foreground (only 8 colors)
                        _ => unreachable!(),
                    };

                    // Each bit pair produces 2 pixels of the same color
                    let px = x_base + bit_pair * 2;
                    self.set_pixel(display_line, px, color, is_foreground);
                    self.set_pixel(display_line, px + 1, color, is_foreground);
                }
            } else {
                // Standard hires mode for this character (bit 3 clear)
                let fg_color = color_byte & 0x0F; // Full 16 colors

                for bit in 0..8 {
                    let pixel_set = (pattern & (0x80 >> bit)) != 0;
                    let color = if pixel_set { fg_color } else { bg_color_0 };
                    self.set_pixel(display_line, x_base + bit, color, pixel_set);
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
                self.set_pixel(display_line, x, bg_color, false);
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
                // In bitmap mode, set pixels are foreground graphics
                self.set_pixel(display_line, x_base + bit, color, pixel_set);
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
                self.set_pixel(display_line, x, bg_color, false);
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

                // In multicolor bitmap mode, bit pattern 00 is background
                // Other patterns (01, 10, 11) are foreground graphics
                let is_foreground = bits != 0b00;

                let color = match bits {
                    0b00 => bg_color_0, // Background 0
                    0b01 => color_01,   // Screen RAM upper nibble
                    0b10 => color_10,   // Screen RAM lower nibble
                    0b11 => color_11,   // Color RAM
                    _ => unreachable!(),
                };

                // Each bit pair produces 2 pixels of the same color
                let px = x_base + bit_pair * 2;
                self.set_pixel(display_line, px, color, is_foreground);
                self.set_pixel(display_line, px + 1, color, is_foreground);
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
                self.set_pixel(display_line, x, bg_color, false);
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
                // In ECM text mode, foreground pixels (pixel_set) are graphics
                self.set_pixel(display_line, x_base + bit, color, pixel_set);
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

    // =========================================================================
    // Sprite Registers and Data Access
    // =========================================================================

    /// Get sprite X position (9-bit value).
    ///
    /// The X position uses the low 8 bits from register $D000+sprite*2
    /// and bit 8 from register $D010 (MSB register).
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_x(&self, sprite: usize) -> u16 {
        if sprite >= SPRITE_COUNT {
            return 0;
        }
        let low = self.registers[sprite * 2] as u16;
        let msb_mask = 1 << sprite;
        let high = if self.registers[0x10] & msb_mask != 0 {
            0x100
        } else {
            0
        };
        high | low
    }

    /// Get sprite Y position (8-bit value).
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_y(&self, sprite: usize) -> u8 {
        if sprite >= SPRITE_COUNT {
            return 0;
        }
        self.registers[sprite * 2 + 1]
    }

    /// Check if a sprite is enabled.
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_enabled(&self, sprite: usize) -> bool {
        if sprite >= SPRITE_COUNT {
            return false;
        }
        self.registers[0x15] & (1 << sprite) != 0
    }

    /// Get the sprite enable register value (all 8 sprites).
    pub fn sprite_enable_bits(&self) -> u8 {
        self.registers[0x15]
    }

    /// Check if sprite has priority over background (appears behind).
    ///
    /// When the priority bit is set, the sprite appears behind background graphics.
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_behind_background(&self, sprite: usize) -> bool {
        if sprite >= SPRITE_COUNT {
            return false;
        }
        self.registers[0x1B] & (1 << sprite) != 0
    }

    /// Check if sprite is in multicolor mode.
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_multicolor(&self, sprite: usize) -> bool {
        if sprite >= SPRITE_COUNT {
            return false;
        }
        self.registers[0x1C] & (1 << sprite) != 0
    }

    /// Check if sprite has X expansion (double width).
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_x_expand(&self, sprite: usize) -> bool {
        if sprite >= SPRITE_COUNT {
            return false;
        }
        self.registers[0x1D] & (1 << sprite) != 0
    }

    /// Check if sprite has Y expansion (double height).
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_y_expand(&self, sprite: usize) -> bool {
        if sprite >= SPRITE_COUNT {
            return false;
        }
        self.registers[0x17] & (1 << sprite) != 0
    }

    /// Get sprite color (0-15).
    ///
    /// # Arguments
    /// * `sprite` - Sprite number (0-7)
    pub fn sprite_color(&self, sprite: usize) -> u8 {
        if sprite >= SPRITE_COUNT {
            return 0;
        }
        self.registers[0x27 + sprite] & 0x0F
    }

    /// Get sprite multicolor 0 (shared color for all sprites, register $D025).
    pub fn sprite_multicolor_0(&self) -> u8 {
        self.registers[0x25] & 0x0F
    }

    /// Get sprite multicolor 1 (shared color for all sprites, register $D026).
    pub fn sprite_multicolor_1(&self) -> u8 {
        self.registers[0x26] & 0x0F
    }

    /// Calculate the address offset within screen RAM where sprite pointers are stored.
    ///
    /// Sprite pointers are located at Screen RAM base + $3F8.
    /// Each of the 8 sprites has a 1-byte pointer at $3F8+sprite_number.
    ///
    /// # Returns
    /// The offset (0x3F8) to add to screen RAM base address.
    pub fn sprite_pointer_offset() -> u16 {
        SPRITE_POINTER_OFFSET
    }

    /// Get the sprite data pointer value for a sprite.
    ///
    /// This reads from screen RAM at offset $3F8+sprite.
    /// The pointer value * 64 = address of sprite data within VIC bank.
    ///
    /// # Arguments
    /// * `screen_ram` - 1KB screen RAM data
    /// * `sprite` - Sprite number (0-7)
    ///
    /// # Returns
    /// The pointer value (0-255), which when multiplied by 64 gives the
    /// sprite data address within the current VIC bank.
    pub fn get_sprite_pointer(&self, screen_ram: &[u8], sprite: usize) -> u8 {
        if sprite >= SPRITE_COUNT {
            return 0;
        }
        let offset = (SPRITE_POINTER_OFFSET as usize) + sprite;
        if offset < screen_ram.len() {
            screen_ram[offset]
        } else {
            0
        }
    }

    /// Calculate the sprite data address within VIC bank.
    ///
    /// The pointer value from screen RAM is multiplied by 64 to get
    /// the actual address of the 63-byte sprite data block.
    ///
    /// # Arguments
    /// * `pointer` - The sprite pointer value (0-255) from screen RAM
    ///
    /// # Returns
    /// The address offset within the VIC bank (0-16320).
    pub fn sprite_data_address(pointer: u8) -> u16 {
        (pointer as u16) * 64
    }

    /// Fetch sprite data from VIC memory.
    ///
    /// This fetches the 63 bytes of sprite data for a single sprite.
    /// The sprite data is organized as 21 lines of 3 bytes each
    /// (24 pixels per line in hires mode, 12 double-wide pixels in multicolor).
    ///
    /// # Arguments
    /// * `vic_memory` - Closure that reads a byte from VIC address space
    /// * `pointer` - Sprite data pointer value (from screen RAM + $3F8)
    ///
    /// # Returns
    /// Array of 63 bytes containing the sprite pattern data.
    ///
    /// # Example
    /// ```ignore
    /// let pointer = vic.get_sprite_pointer(&screen_ram, 0);
    /// let sprite_data = VicII::fetch_sprite_data(|addr| memory.vic_read(addr), pointer);
    /// ```
    pub fn fetch_sprite_data<F>(vic_memory: F, pointer: u8) -> [u8; SPRITE_DATA_SIZE]
    where
        F: Fn(u16) -> u8,
    {
        let base_addr = Self::sprite_data_address(pointer);
        let mut data = [0u8; SPRITE_DATA_SIZE];

        for (i, byte) in data.iter_mut().enumerate() {
            *byte = vic_memory(base_addr + i as u16);
        }

        data
    }

    /// Fetch all enabled sprites' data.
    ///
    /// This is a convenience method that fetches sprite data for all 8 sprites.
    /// For disabled sprites, the data array will contain zeros.
    ///
    /// # Arguments
    /// * `vic_memory` - Closure that reads a byte from VIC address space
    /// * `screen_ram` - 1KB screen RAM data (for reading sprite pointers)
    ///
    /// # Returns
    /// Array of 8 sprite data blocks (63 bytes each).
    pub fn fetch_all_sprite_data<F>(
        &self,
        vic_memory: F,
        screen_ram: &[u8],
    ) -> [[u8; SPRITE_DATA_SIZE]; SPRITE_COUNT]
    where
        F: Fn(u16) -> u8,
    {
        let mut all_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        let enabled = self.sprite_enable_bits();

        for (sprite, sprite_data) in all_data.iter_mut().enumerate() {
            if enabled & (1 << sprite) != 0 {
                let pointer = self.get_sprite_pointer(screen_ram, sprite);
                *sprite_data = Self::fetch_sprite_data(&vic_memory, pointer);
            }
        }

        all_data
    }

    // =========================================================================
    // Sprite Rendering (T072)
    // =========================================================================

    /// Render sprites for a single scanline.
    ///
    /// This renders all enabled sprites onto the current scanline of the framebuffer.
    /// Sprites are rendered in priority order (sprite 0 has highest priority).
    ///
    /// # Arguments
    /// * `scanline` - The current raster line (0-311 PAL, 0-262 NTSC)
    /// * `sprite_data` - Array of 8 sprite data blocks (63 bytes each)
    ///
    /// This method handles:
    /// - Standard hires sprites (24x21 pixels, 1 color)
    /// - Sprite enable/disable via register $D015
    /// - Sprite X/Y positioning (9-bit X, 8-bit Y)
    /// - Sprite colors from registers $D027-$D02E
    ///
    /// NOTE: This is the basic implementation (T072). Additional features are in:
    /// - T073: Multicolor sprite mode
    /// - T074: X/Y expansion (double size)
    /// - T075: Sprite priority (sprite-to-sprite, sprite-to-background)
    /// - T076: Collision detection
    pub fn render_sprites_scanline(
        &mut self,
        scanline: u16,
        sprite_data: &[[u8; SPRITE_DATA_SIZE]; SPRITE_COUNT],
    ) {
        // Calculate the display line (relative to the visible area)
        let display_start = DISPLAY_START_LINE_PAL;
        let display_end = display_start + SCREEN_HEIGHT as u16;

        // Check if we're in the visible display area
        if scanline < display_start || scanline >= display_end {
            return;
        }

        let display_line = (scanline - display_start) as usize;
        let enabled = self.sprite_enable_bits();

        // Nothing to do if no sprites are enabled
        if enabled == 0 {
            return;
        }

        // T076: Collision detection tracking
        // Track which sprites have drawn non-transparent pixels at each X position
        // on this scanline. Used for sprite-sprite collision detection.
        // Each element contains a bitmask of sprites that have drawn at that X position.
        let mut sprite_pixel_mask: [u8; SCREEN_WIDTH] = [0; SCREEN_WIDTH];

        // Render sprites in reverse order so that sprite 0 has highest priority
        // (drawn last, appearing on top)
        for sprite_num in (0..SPRITE_COUNT).rev() {
            // Skip if sprite is not enabled
            if enabled & (1 << sprite_num) == 0 {
                continue;
            }

            // Get sprite position
            // Sprite Y coordinate is the raster line where sprite starts
            // Note: C64 sprites have a 50-pixel Y offset (first visible line)
            let sprite_y = self.sprite_y(sprite_num) as u16;

            // Check Y expansion (T074 - basic support for visibility check)
            let y_expand = self.sprite_y_expand(sprite_num);
            let effective_height = if y_expand {
                SPRITE_HEIGHT * 2
            } else {
                SPRITE_HEIGHT
            };

            // Calculate which lines the sprite occupies (in absolute raster coordinates)
            // On the C64, a sprite's Y register value specifies the first raster line
            // where the sprite appears. So sprite_y is an absolute raster line number.
            let sprite_start_raster = sprite_y;
            let sprite_end_raster = sprite_start_raster.wrapping_add(effective_height as u16);

            // Check if this scanline intersects the sprite
            if scanline < sprite_start_raster || scanline >= sprite_end_raster {
                continue;
            }

            // Calculate which line of the sprite data to use
            let sprite_line_offset = scanline.wrapping_sub(sprite_start_raster) as usize;
            let sprite_data_line = if y_expand {
                sprite_line_offset / 2
            } else {
                sprite_line_offset
            };

            // Safety check - don't read past sprite data
            if sprite_data_line >= SPRITE_HEIGHT {
                continue;
            }

            // Get sprite X position (9-bit)
            let sprite_x = self.sprite_x(sprite_num);

            // Get sprite color
            let sprite_color = self.sprite_color(sprite_num);

            // Check X expansion (T074 - basic support)
            let x_expand = self.sprite_x_expand(sprite_num);
            let _effective_width = if x_expand {
                SPRITE_WIDTH * 2
            } else {
                SPRITE_WIDTH
            };

            // Check sprite-to-background priority (T075)
            // When the bit is set in $1B, the sprite appears BEHIND foreground graphics
            let sprite_behind_bg = self.sprite_behind_background(sprite_num);

            // Get the 3 bytes of sprite data for this line
            let data_offset = sprite_data_line * SPRITE_BYTES_PER_LINE;
            let line_data =
                &sprite_data[sprite_num][data_offset..data_offset + SPRITE_BYTES_PER_LINE];

            // Check if sprite is in multicolor mode (T073)
            let multicolor = self.sprite_multicolor(sprite_num);

            if multicolor {
                // Multicolor mode: 12 double-width pixels per line (2 bits per pixel)
                // Bit pairs map to colors:
                // - 00: Transparent
                // - 01: Sprite multicolor 0 ($D025)
                // - 10: Sprite individual color ($D027+n)
                // - 11: Sprite multicolor 1 ($D026)
                let mc0 = self.sprite_multicolor_0();
                let mc1 = self.sprite_multicolor_1();

                for byte_idx in 0..SPRITE_BYTES_PER_LINE {
                    let byte = line_data[byte_idx];

                    // Process 4 bit pairs per byte (8 bits / 2 bits per pair)
                    for pair_idx in 0..4 {
                        // Extract bit pair (high bits are leftmost)
                        let shift = 6 - (pair_idx * 2);
                        let bit_pair = (byte >> shift) & 0x03;

                        // Get color for this bit pair
                        let color = match bit_pair {
                            0b00 => continue, // Transparent
                            0b01 => mc0,
                            0b10 => sprite_color,
                            0b11 => mc1,
                            _ => unreachable!(),
                        };

                        // Each multicolor pixel is 2 screen pixels wide
                        // Calculate base pixel position (12 pixels per line, each 2 wide = 24 pixels)
                        let mc_pixel_idx = byte_idx * 4 + pair_idx;

                        // Base width is 2 pixels per multicolor pixel
                        // With X expansion, each becomes 4 pixels
                        let base_width = 2;
                        let pixel_width = if x_expand { base_width * 2 } else { base_width };
                        let pixel_offset = mc_pixel_idx * pixel_width;

                        // Draw the double-width pixel
                        for sub_pixel in 0..pixel_width {
                            let screen_x = sprite_x
                                .wrapping_sub(24) // Convert to screen coordinates
                                .wrapping_add((pixel_offset + sub_pixel) as u16);

                            // Check if pixel is within visible screen area
                            if screen_x >= SCREEN_WIDTH as u16 {
                                continue;
                            }

                            let x = screen_x as usize;

                            // T076: Sprite-background collision detection
                            // Collision is detected even if sprite is behind background
                            if self.is_foreground_pixel(display_line, x) {
                                self.sprite_collision_sb |= 1 << sprite_num;
                            }

                            // T076: Sprite-sprite collision detection
                            // Check if any other sprite has already drawn at this position
                            let existing_sprites = sprite_pixel_mask[x];
                            if existing_sprites != 0 {
                                // Collision! Mark both this sprite and all sprites already at this position
                                self.sprite_collision_ss |= (1 << sprite_num) | existing_sprites;
                            }
                            // Mark this sprite as having drawn at this position
                            sprite_pixel_mask[x] |= 1 << sprite_num;

                            // T075: Sprite-to-background priority
                            // If sprite is behind background and there's foreground at this pixel,
                            // don't draw the sprite pixel (but collision was still detected above)
                            if sprite_behind_bg && self.is_foreground_pixel(display_line, x) {
                                continue;
                            }

                            self.framebuffer[display_line][x] = color;
                        }
                    }
                }
            } else {
                // Standard hires mode: 24 pixels from 3 bytes (8 pixels per byte)
                // Bit 7 of each byte is the leftmost pixel
                for byte_idx in 0..SPRITE_BYTES_PER_LINE {
                    let byte = line_data[byte_idx];

                    for bit_idx in 0..8 {
                        // Check if this pixel is set
                        let pixel_set = (byte & (0x80 >> bit_idx)) != 0;

                        if !pixel_set {
                            continue; // Transparent pixel
                        }

                        // Calculate X position on screen
                        // Sprite X position 24 ($18) aligns with left edge of display
                        // Subtract 24 to convert sprite coordinates to screen coordinates
                        let pixel_offset = if x_expand {
                            (byte_idx * 8 + bit_idx) * 2
                        } else {
                            byte_idx * 8 + bit_idx
                        };

                        // Handle X expansion - draw each pixel twice
                        let pixels_to_draw = if x_expand { 2 } else { 1 };

                        for expand_idx in 0..pixels_to_draw {
                            let screen_x = sprite_x
                                .wrapping_sub(24) // Convert to screen coordinates
                                .wrapping_add(pixel_offset as u16)
                                .wrapping_add(if x_expand { expand_idx } else { 0 });

                            // Check if pixel is within visible screen area
                            if screen_x >= SCREEN_WIDTH as u16 {
                                continue;
                            }

                            let x = screen_x as usize;

                            // T076: Sprite-background collision detection
                            // Collision is detected even if sprite is behind background
                            if self.is_foreground_pixel(display_line, x) {
                                self.sprite_collision_sb |= 1 << sprite_num;
                            }

                            // T076: Sprite-sprite collision detection
                            // Check if any other sprite has already drawn at this position
                            let existing_sprites = sprite_pixel_mask[x];
                            if existing_sprites != 0 {
                                // Collision! Mark both this sprite and all sprites already at this position
                                self.sprite_collision_ss |= (1 << sprite_num) | existing_sprites;
                            }
                            // Mark this sprite as having drawn at this position
                            sprite_pixel_mask[x] |= 1 << sprite_num;

                            // T075: Sprite-to-background priority
                            // If sprite is behind background and there's foreground at this pixel,
                            // don't draw the sprite pixel (but collision was still detected above)
                            if sprite_behind_bg && self.is_foreground_pixel(display_line, x) {
                                continue;
                            }

                            self.framebuffer[display_line][x] = sprite_color;
                        }
                    }
                }
            }
        }
    }

    /// Set sprite-sprite collision flags.
    ///
    /// Called during sprite rendering to record collisions.
    /// Flags are OR'd with existing flags (cleared on read by CPU).
    pub fn set_sprite_collision_ss(&mut self, flags: u8) {
        self.sprite_collision_ss |= flags;
    }

    /// Set sprite-background collision flags.
    ///
    /// Called during sprite rendering to record collisions.
    /// Flags are OR'd with existing flags (cleared on read by CPU).
    pub fn set_sprite_collision_sb(&mut self, flags: u8) {
        self.sprite_collision_sb |= flags;
    }

    /// Clear sprite-sprite collision register.
    ///
    /// Called by memory handler after CPU reads register $D01E.
    pub fn clear_sprite_collision_ss(&mut self) {
        self.sprite_collision_ss = 0;
    }

    /// Clear sprite-background collision register.
    ///
    /// Called by memory handler after CPU reads register $D01F.
    pub fn clear_sprite_collision_sb(&mut self) {
        self.sprite_collision_sb = 0;
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
        assert_eq!(
            vic.framebuffer[0][0], 1,
            "Pixel 0 should be foreground (white)"
        );
        assert_eq!(
            vic.framebuffer[0][1], 0,
            "Pixel 1 should be background (black)"
        );
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
            assert_eq!(
                vic.framebuffer[0][x], 1,
                "Line 0, pixel {} should be white",
                x
            );
        }

        // Check line 1: all black (0x00 -> all background)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[1][x], 0,
                "Line 1, pixel {} should be black",
                x
            );
        }

        // Check line 2: left half white (0xF0)
        for x in 0..4 {
            assert_eq!(
                vic.framebuffer[2][x], 1,
                "Line 2, pixel {} should be white",
                x
            );
        }
        for x in 4..8 {
            assert_eq!(
                vic.framebuffer[2][x], 0,
                "Line 2, pixel {} should be black",
                x
            );
        }

        // Check line 3: right half white (0x0F)
        for x in 0..4 {
            assert_eq!(
                vic.framebuffer[3][x], 0,
                "Line 3, pixel {} should be black",
                x
            );
        }
        for x in 4..8 {
            assert_eq!(
                vic.framebuffer[3][x], 1,
                "Line 3, pixel {} should be white",
                x
            );
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
        screen_ram[0] = 0x12; // Row 0, Cell 0: fg=1, bg=2
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
        assert_eq!(
            vic.framebuffer[8][0], 3,
            "Row 1 pixel 0 should be foreground"
        );
        assert_eq!(
            vic.framebuffer[8][1], 4,
            "Row 1 pixel 1 should be background"
        );
    }

    #[test]
    fn test_standard_bitmap_mode_full_row_cells() {
        let mut vic = VicII::new();

        // Enable bitmap mode
        vic.registers[0x11] |= 0x20;

        // Create bitmap data
        let mut bitmap_data = vec![0u8; 8000];

        // Set up cells 0, 10, 20, 30, 39 with different patterns
        bitmap_data[0] = 0xFF; // Cell 0: all on
        bitmap_data[80] = 0x00; // Cell 10: all off
        bitmap_data[160] = 0xF0; // Cell 20: left half
        bitmap_data[240] = 0x0F; // Cell 30: right half
        bitmap_data[312] = 0xAA; // Cell 39: alternating

        // Screen RAM with distinct colors for each cell
        let mut screen_ram = vec![0u8; 1000];
        screen_ram[0] = 0x10; // Cell 0: white/black
        screen_ram[10] = 0x23; // Cell 10: red/cyan
        screen_ram[20] = 0x45; // Cell 20: purple/green
        screen_ram[30] = 0x67; // Cell 30: blue/yellow
        screen_ram[39] = 0x89; // Cell 39: orange/brown

        let color_ram = vec![0u8; 1000];

        vic.step_scanline(51, &bitmap_data, &screen_ram, &color_ram);

        // Cell 0 (x=0-7): all foreground (white=1)
        assert_eq!(vic.framebuffer[0][0], 1, "Cell 0 pixel should be white");

        // Cell 10 (x=80-87): all background (cyan=3)
        assert_eq!(
            vic.framebuffer[0][80], 3,
            "Cell 10 pixel should be cyan (bg)"
        );

        // Cell 20 (x=160-167): left half foreground (purple=4), right half background (green=5)
        assert_eq!(
            vic.framebuffer[0][160], 4,
            "Cell 20 left pixel should be purple"
        );
        assert_eq!(
            vic.framebuffer[0][164], 5,
            "Cell 20 right pixel should be green"
        );

        // Cell 30 (x=240-247): left half background (yellow=7), right half foreground (blue=6)
        assert_eq!(
            vic.framebuffer[0][240], 7,
            "Cell 30 left pixel should be yellow (bg)"
        );
        assert_eq!(
            vic.framebuffer[0][244], 6,
            "Cell 30 right pixel should be blue (fg)"
        );

        // Cell 39 (x=312-319): alternating orange(8)/brown(9)
        assert_eq!(
            vic.framebuffer[0][312], 8,
            "Cell 39 pixel 0 should be orange (fg)"
        );
        assert_eq!(
            vic.framebuffer[0][313], 9,
            "Cell 39 pixel 1 should be brown (bg)"
        );
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
        assert_eq!(
            vic.framebuffer[0][0], 0,
            "Pixel 0 should be background 0 (black)"
        );
        assert_eq!(
            vic.framebuffer[0][1], 0,
            "Pixel 1 should be background 0 (black)"
        );
        assert_eq!(
            vic.framebuffer[0][2], 1,
            "Pixel 2 should be screen upper (white)"
        );
        assert_eq!(
            vic.framebuffer[0][3], 1,
            "Pixel 3 should be screen upper (white)"
        );
        assert_eq!(
            vic.framebuffer[0][4], 2,
            "Pixel 4 should be screen lower (red)"
        );
        assert_eq!(
            vic.framebuffer[0][5], 2,
            "Pixel 5 should be screen lower (red)"
        );
        assert_eq!(
            vic.framebuffer[0][6], 3,
            "Pixel 6 should be color RAM (cyan)"
        );
        assert_eq!(
            vic.framebuffer[0][7], 3,
            "Pixel 7 should be color RAM (cyan)"
        );
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
            assert_eq!(
                vic.framebuffer[0][x], 5,
                "Pixel {} should be green (bg0)",
                x
            );
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
        bitmap_data[0] = 0x00; // Cell 0: all 00 (background)
        bitmap_data[8] = 0x55; // Cell 1: all 01 (screen upper nibble)
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
            assert_eq!(
                vic.framebuffer[0][x], 0,
                "Cell 0 pixel {} should be black",
                x
            );
        }

        // Cell 1 (x=8-15): all 01 pattern -> screen upper nibble (white=1)
        for x in 8..16 {
            assert_eq!(
                vic.framebuffer[0][x], 1,
                "Cell 1 pixel {} should be white",
                x
            );
        }

        // Cell 2 (x=16-23): all 10 pattern -> screen lower nibble (red=2)
        for x in 16..24 {
            assert_eq!(vic.framebuffer[0][x], 2, "Cell 2 pixel {} should be red", x);
        }

        // Cell 3 (x=24-31): all 11 pattern -> color RAM (cyan=3)
        for x in 24..32 {
            assert_eq!(
                vic.framebuffer[0][x], 3,
                "Cell 3 pixel {} should be cyan",
                x
            );
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
            assert_eq!(
                vic.framebuffer[1][x], 5,
                "Line 1, pixel {} should be green",
                x
            );
        }

        // Line 2: all 10 -> blue (6)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[2][x], 6,
                "Line 2, pixel {} should be blue",
                x
            );
        }

        // Line 3: all 11 -> yellow (7)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[3][x], 7,
                "Line 3, pixel {} should be yellow",
                x
            );
        }

        // Line 4: mixed pattern 00 01 10 11
        assert_eq!(vic.framebuffer[4][0], 0, "Line 4, pixels 0-1 should be bg");
        assert_eq!(vic.framebuffer[4][1], 0);
        assert_eq!(
            vic.framebuffer[4][2], 5,
            "Line 4, pixels 2-3 should be green"
        );
        assert_eq!(vic.framebuffer[4][3], 5);
        assert_eq!(
            vic.framebuffer[4][4], 6,
            "Line 4, pixels 4-5 should be blue"
        );
        assert_eq!(vic.framebuffer[4][5], 6);
        assert_eq!(
            vic.framebuffer[4][6], 7,
            "Line 4, pixels 6-7 should be yellow"
        );
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
        screen_ram[0] = 0x12; // Row 0, Cell 0: upper=1, lower=2
        screen_ram[40] = 0x34; // Row 1, Cell 0: upper=3, lower=4

        // Color RAM
        let mut color_ram = vec![0u8; 1000];
        color_ram[0] = 0x05; // Row 0, Cell 0: 5
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
                vic.framebuffer[0][x], cell as u8,
                "Cell {} should have color {}",
                cell, cell
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
                vic.framebuffer[0][8 + pixel],
                1,
                "Cell 1 pixel {} should be bg1 (white)",
                pixel
            );
        }

        // Cell 2: background 2 (red = 2)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][16 + pixel],
                2,
                "Cell 2 pixel {} should be bg2 (red)",
                pixel
            );
        }

        // Cell 3: background 3 (cyan = 3)
        for pixel in 0..8 {
            assert_eq!(
                vic.framebuffer[0][24 + pixel],
                3,
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
                vic.framebuffer[0][cell * 8],
                5,
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
        assert_eq!(
            vic.framebuffer[0][0], 1,
            "Cell 0 pixel 0 should be white (fg)"
        );
        assert_eq!(
            vic.framebuffer[0][1], 6,
            "Cell 0 pixel 1 should be blue (bg0)"
        );

        // Cell 1: foreground=white(1), background=yellow(7)
        assert_eq!(
            vic.framebuffer[0][8], 1,
            "Cell 1 pixel 0 should be white (fg)"
        );
        assert_eq!(
            vic.framebuffer[0][9], 7,
            "Cell 1 pixel 1 should be yellow (bg1)"
        );

        // Cell 2: foreground=white(1), background=orange(8)
        assert_eq!(
            vic.framebuffer[0][16], 1,
            "Cell 2 pixel 0 should be white (fg)"
        );
        assert_eq!(
            vic.framebuffer[0][17], 8,
            "Cell 2 pixel 1 should be orange (bg2)"
        );

        // Cell 3: foreground=white(1), background=brown(9)
        assert_eq!(
            vic.framebuffer[0][24], 1,
            "Cell 3 pixel 0 should be white (fg)"
        );
        assert_eq!(
            vic.framebuffer[0][25], 9,
            "Cell 3 pixel 1 should be brown (bg3)"
        );
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
            assert_eq!(
                vic.framebuffer[0][x], 1,
                "Line 0, pixel {} should be white",
                x
            );
        }

        // Line 1: all background (black)
        for x in 0..8 {
            assert_eq!(
                vic.framebuffer[1][x], 0,
                "Line 1, pixel {} should be black",
                x
            );
        }

        // Line 2: left half foreground, right half background
        for x in 0..4 {
            assert_eq!(
                vic.framebuffer[2][x], 1,
                "Line 2, pixel {} should be white",
                x
            );
        }
        for x in 4..8 {
            assert_eq!(
                vic.framebuffer[2][x], 0,
                "Line 2, pixel {} should be black",
                x
            );
        }

        // Line 3: left half background, right half foreground
        for x in 0..4 {
            assert_eq!(
                vic.framebuffer[3][x], 0,
                "Line 3, pixel {} should be black",
                x
            );
        }
        for x in 4..8 {
            assert_eq!(
                vic.framebuffer[3][x], 1,
                "Line 3, pixel {} should be white",
                x
            );
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
                    vic.framebuffer[0][x], expected_bg,
                    "Cell {} pixel {} should be bg{}",
                    cell, pixel, expected_bg
                );
            }
        }
    }

    // =========================================================================
    // Sprite Data Fetching Tests (T071)
    // =========================================================================

    #[test]
    fn test_sprite_pointer_offset() {
        // Sprite pointers are at Screen RAM + $3F8
        assert_eq!(VicII::sprite_pointer_offset(), 0x03F8);
    }

    #[test]
    fn test_sprite_data_address_calculation() {
        // Sprite data address = pointer * 64
        assert_eq!(VicII::sprite_data_address(0), 0);
        assert_eq!(VicII::sprite_data_address(1), 64);
        assert_eq!(VicII::sprite_data_address(13), 13 * 64);
        assert_eq!(VicII::sprite_data_address(255), 255 * 64);
    }

    #[test]
    fn test_get_sprite_pointer() {
        let vic = VicII::new();

        // Create screen RAM with sprite pointers at $3F8
        let mut screen_ram = vec![0u8; 1024]; // 1KB

        // Set sprite pointers (at offset $3F8 = 1016)
        screen_ram[0x3F8] = 13; // Sprite 0 pointer
        screen_ram[0x3F9] = 14; // Sprite 1 pointer
        screen_ram[0x3FA] = 15; // Sprite 2 pointer
        screen_ram[0x3FB] = 0; // Sprite 3 pointer
        screen_ram[0x3FC] = 128; // Sprite 4 pointer
        screen_ram[0x3FD] = 200; // Sprite 5 pointer
        screen_ram[0x3FE] = 255; // Sprite 6 pointer
        screen_ram[0x3FF] = 100; // Sprite 7 pointer

        assert_eq!(vic.get_sprite_pointer(&screen_ram, 0), 13);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 1), 14);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 2), 15);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 3), 0);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 4), 128);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 5), 200);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 6), 255);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 7), 100);

        // Invalid sprite number should return 0
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 8), 0);
        assert_eq!(vic.get_sprite_pointer(&screen_ram, 255), 0);
    }

    #[test]
    fn test_fetch_sprite_data() {
        // Create test VIC memory (16KB)
        let mut vic_mem = vec![0u8; 16384];

        // Put test pattern at sprite data address (pointer 13 = address 832)
        let sprite_addr = 13 * 64;
        for i in 0..SPRITE_DATA_SIZE {
            vic_mem[sprite_addr + i] = i as u8;
        }

        // Fetch sprite data using closure
        let data = VicII::fetch_sprite_data(|addr| vic_mem[addr as usize], 13);

        // Verify all 63 bytes were fetched correctly
        for i in 0..SPRITE_DATA_SIZE {
            assert_eq!(data[i], i as u8, "Byte {} mismatch", i);
        }
    }

    #[test]
    fn test_sprite_position_registers() {
        let mut vic = VicII::new();

        // Set sprite 0 position to (256, 100) - X uses MSB bit
        vic.registers[0x00] = 0x00; // X low byte (0)
        vic.registers[0x01] = 100; // Y
        vic.registers[0x10] = 0x01; // MSB for sprite 0 set

        assert_eq!(vic.sprite_x(0), 256);
        assert_eq!(vic.sprite_y(0), 100);

        // Set sprite 1 position to (50, 200)
        vic.registers[0x02] = 50; // X low byte
        vic.registers[0x03] = 200; // Y
                                   // MSB bit 1 is clear (from above)

        assert_eq!(vic.sprite_x(1), 50);
        assert_eq!(vic.sprite_y(1), 200);

        // Set sprite 7 position to (320, 255) - max values
        vic.registers[0x0E] = 0x40; // 320 & 0xFF = 64
        vic.registers[0x0F] = 255; // Y
        vic.registers[0x10] |= 0x80; // MSB for sprite 7

        assert_eq!(vic.sprite_x(7), 0x140); // 256 + 64 = 320
        assert_eq!(vic.sprite_y(7), 255);
    }

    #[test]
    fn test_sprite_enable_register() {
        let mut vic = VicII::new();

        // No sprites enabled initially
        vic.registers[0x15] = 0;
        for i in 0..8 {
            assert!(!vic.sprite_enabled(i), "Sprite {} should be disabled", i);
        }

        // Enable sprites 0, 3, 7
        vic.registers[0x15] = 0b10001001;
        assert!(vic.sprite_enabled(0));
        assert!(!vic.sprite_enabled(1));
        assert!(!vic.sprite_enabled(2));
        assert!(vic.sprite_enabled(3));
        assert!(!vic.sprite_enabled(4));
        assert!(!vic.sprite_enabled(5));
        assert!(!vic.sprite_enabled(6));
        assert!(vic.sprite_enabled(7));

        assert_eq!(vic.sprite_enable_bits(), 0b10001001);
    }

    #[test]
    fn test_sprite_priority_register() {
        let mut vic = VicII::new();

        // All sprites in front of background by default
        vic.registers[0x1B] = 0;
        for i in 0..8 {
            assert!(
                !vic.sprite_behind_background(i),
                "Sprite {} should be in front",
                i
            );
        }

        // Set sprites 2 and 5 behind background
        vic.registers[0x1B] = 0b00100100;
        assert!(!vic.sprite_behind_background(0));
        assert!(!vic.sprite_behind_background(1));
        assert!(vic.sprite_behind_background(2));
        assert!(!vic.sprite_behind_background(3));
        assert!(!vic.sprite_behind_background(4));
        assert!(vic.sprite_behind_background(5));
        assert!(!vic.sprite_behind_background(6));
        assert!(!vic.sprite_behind_background(7));
    }

    #[test]
    fn test_sprite_multicolor_register() {
        let mut vic = VicII::new();

        vic.registers[0x1C] = 0b11110000; // Sprites 4-7 multicolor

        assert!(!vic.sprite_multicolor(0));
        assert!(!vic.sprite_multicolor(1));
        assert!(!vic.sprite_multicolor(2));
        assert!(!vic.sprite_multicolor(3));
        assert!(vic.sprite_multicolor(4));
        assert!(vic.sprite_multicolor(5));
        assert!(vic.sprite_multicolor(6));
        assert!(vic.sprite_multicolor(7));
    }

    #[test]
    fn test_sprite_expansion_registers() {
        let mut vic = VicII::new();

        // X expansion register
        vic.registers[0x1D] = 0b01010101; // Sprites 0, 2, 4, 6 expanded X
        assert!(vic.sprite_x_expand(0));
        assert!(!vic.sprite_x_expand(1));
        assert!(vic.sprite_x_expand(2));
        assert!(!vic.sprite_x_expand(3));

        // Y expansion register
        vic.registers[0x17] = 0b10101010; // Sprites 1, 3, 5, 7 expanded Y
        assert!(!vic.sprite_y_expand(0));
        assert!(vic.sprite_y_expand(1));
        assert!(!vic.sprite_y_expand(2));
        assert!(vic.sprite_y_expand(3));
    }

    #[test]
    fn test_sprite_colors() {
        let mut vic = VicII::new();

        // Set individual sprite colors
        vic.registers[0x27] = 0x01; // Sprite 0: white
        vic.registers[0x28] = 0x02; // Sprite 1: red
        vic.registers[0x29] = 0x03; // Sprite 2: cyan
        vic.registers[0x2A] = 0x04; // Sprite 3: purple
        vic.registers[0x2B] = 0x05; // Sprite 4: green
        vic.registers[0x2C] = 0x06; // Sprite 5: blue
        vic.registers[0x2D] = 0x07; // Sprite 6: yellow
        vic.registers[0x2E] = 0x08; // Sprite 7: orange

        assert_eq!(vic.sprite_color(0), 0x01);
        assert_eq!(vic.sprite_color(1), 0x02);
        assert_eq!(vic.sprite_color(2), 0x03);
        assert_eq!(vic.sprite_color(3), 0x04);
        assert_eq!(vic.sprite_color(4), 0x05);
        assert_eq!(vic.sprite_color(5), 0x06);
        assert_eq!(vic.sprite_color(6), 0x07);
        assert_eq!(vic.sprite_color(7), 0x08);

        // Test multicolor shared colors
        vic.registers[0x25] = 0x09; // Multicolor 0
        vic.registers[0x26] = 0x0A; // Multicolor 1

        assert_eq!(vic.sprite_multicolor_0(), 0x09);
        assert_eq!(vic.sprite_multicolor_1(), 0x0A);
    }

    #[test]
    fn test_sprite_collision_registers() {
        let mut vic = VicII::new();

        // Set collision flags
        vic.set_sprite_collision_ss(0x03); // Sprites 0 and 1 collided
        assert_eq!(vic.sprite_collision_ss, 0x03);

        // OR with more collisions
        vic.set_sprite_collision_ss(0x0C); // Sprites 2 and 3 collided
        assert_eq!(vic.sprite_collision_ss, 0x0F);

        vic.set_sprite_collision_sb(0x55);
        assert_eq!(vic.sprite_collision_sb, 0x55);

        // Clear collision registers
        vic.clear_sprite_collision_ss();
        vic.clear_sprite_collision_sb();

        assert_eq!(vic.sprite_collision_ss, 0);
        assert_eq!(vic.sprite_collision_sb, 0);
    }

    #[test]
    fn test_fetch_all_sprite_data() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 2 only
        vic.registers[0x15] = 0b00000101;

        // Create VIC memory with test patterns
        let mut vic_mem = vec![0u8; 16384];

        // Sprite 0 data at pointer 10 = address 640
        let addr0 = 10 * 64;
        for i in 0..SPRITE_DATA_SIZE {
            vic_mem[addr0 + i] = 0xAA; // Pattern for sprite 0
        }

        // Sprite 2 data at pointer 20 = address 1280
        let addr2 = 20 * 64;
        for i in 0..SPRITE_DATA_SIZE {
            vic_mem[addr2 + i] = 0x55; // Pattern for sprite 2
        }

        // Create screen RAM with pointers
        let mut screen_ram = vec![0u8; 1024];
        screen_ram[0x3F8] = 10; // Sprite 0 pointer
        screen_ram[0x3F9] = 11; // Sprite 1 pointer (disabled)
        screen_ram[0x3FA] = 20; // Sprite 2 pointer

        // Fetch all sprite data
        let all_data = vic.fetch_all_sprite_data(|addr| vic_mem[addr as usize], &screen_ram);

        // Check sprite 0 data
        for i in 0..SPRITE_DATA_SIZE {
            assert_eq!(all_data[0][i], 0xAA, "Sprite 0 byte {} mismatch", i);
        }

        // Sprite 1 should be zeros (disabled)
        for i in 0..SPRITE_DATA_SIZE {
            assert_eq!(all_data[1][i], 0, "Sprite 1 should be zeros (disabled)");
        }

        // Check sprite 2 data
        for i in 0..SPRITE_DATA_SIZE {
            assert_eq!(all_data[2][i], 0x55, "Sprite 2 byte {} mismatch", i);
        }

        // Sprites 3-7 should be zeros (disabled)
        for sprite in 3..8 {
            for i in 0..SPRITE_DATA_SIZE {
                assert_eq!(all_data[sprite][i], 0, "Sprite {} should be zeros", sprite);
            }
        }
    }

    #[test]
    fn test_sprite_data_size_constant() {
        // Verify constant matches spec: 21 lines × 3 bytes = 63 bytes
        assert_eq!(SPRITE_DATA_SIZE, 63);
        assert_eq!(SPRITE_HEIGHT, 21);
        assert_eq!(SPRITE_WIDTH, 24);
        assert_eq!(SPRITE_COUNT, 8);
    }

    // =========================================================================
    // Sprite Rendering Tests (T072)
    // =========================================================================

    #[test]
    fn test_render_sprites_scanline_disabled_sprites() {
        let mut vic = VicII::new();

        // No sprites enabled (register $D015 = 0)
        vic.registers[0x15] = 0;

        // Create empty sprite data
        let sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06; // Blue background
            }
        }

        // Render sprites at scanline 100 (visible area)
        vic.render_sprites_scanline(100, &sprite_data);

        // Framebuffer should be unchanged (no sprites to render)
        assert_eq!(vic.framebuffer[49][0], 0x06);
        assert_eq!(vic.framebuffer[49][160], 0x06);
    }

    #[test]
    fn test_render_sprites_scanline_basic_sprite() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;

        // Set sprite 0 position to (24+50, 51+10) = screen position (50, 10)
        // X position 24 is the left edge of the display
        vic.registers[0x00] = 24 + 50; // X low byte
        vic.registers[0x01] = 51 + 10; // Y position (sprite appears at Y-1+display_start)
        vic.registers[0x10] = 0; // X MSB = 0

        // Set sprite 0 color to white (1)
        vic.registers[0x27] = 0x01;

        // Create sprite data with first pixel set
        // First byte = 0x80 (bit 7 set = leftmost pixel)
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // First pixel of first line

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06; // Blue background
            }
        }

        // Render sprites at scanline 61 (display line 10, matches sprite Y)
        // scanline 61 = display_start(51) + 10
        vic.render_sprites_scanline(61, &sprite_data);

        // The sprite should have rendered one pixel at X=50, Y=10
        assert_eq!(
            vic.framebuffer[10][50], 0x01,
            "Sprite pixel should be white at (50, 10)"
        );

        // Adjacent pixels should still be background
        assert_eq!(
            vic.framebuffer[10][49], 0x06,
            "Left of sprite should be background"
        );
        assert_eq!(
            vic.framebuffer[10][51], 0x06,
            "Right of sprite should be background"
        );
    }

    #[test]
    fn test_render_sprites_scanline_full_line() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;

        // Set sprite 0 position to left edge
        // Sprite Y coordinate specifies the raster line where sprite first appears
        vic.registers[0x00] = 24; // X = 24 (screen X = 0)
        vic.registers[0x01] = 51; // Y = 51 (display line 0, since display_start = 51)
        vic.registers[0x10] = 0;

        // Set sprite 0 color to red (2)
        vic.registers[0x27] = 0x02;

        // Create sprite data with all pixels set for first line
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0xFF; // First 8 pixels
        sprite_data[0][1] = 0xFF; // Next 8 pixels
        sprite_data[0][2] = 0xFF; // Last 8 pixels (24 total)

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        // Render sprites at scanline 51 (display line 0)
        vic.render_sprites_scanline(51, &sprite_data);

        // First 24 pixels of line 0 should be red
        for x in 0..24 {
            assert_eq!(
                vic.framebuffer[0][x], 0x02,
                "Sprite pixel at X={} should be red",
                x
            );
        }

        // Pixel 24 should be background
        assert_eq!(vic.framebuffer[0][24], 0x06);
    }

    #[test]
    fn test_render_sprites_priority_order() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1
        vic.registers[0x15] = 0x03;

        // Set both sprites to same position
        // Sprite Y = raster line where sprite first appears
        vic.registers[0x00] = 50; // Sprite 0 X
        vic.registers[0x01] = 60; // Sprite 0 Y (raster 60 = display line 9)
        vic.registers[0x02] = 50; // Sprite 1 X
        vic.registers[0x03] = 60; // Sprite 1 Y
        vic.registers[0x10] = 0;

        // Sprite 0 = white (1), Sprite 1 = red (2)
        vic.registers[0x27] = 0x01;
        vic.registers[0x28] = 0x02;

        // Both sprites have first pixel set
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;
        sprite_data[1][0] = 0x80;

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        // Render at the sprite Y position
        // Y=60 means sprite starts at raster 60, display line = 60 - 51 = 9
        vic.render_sprites_scanline(60, &sprite_data);

        // Screen X = 50 - 24 = 26
        // Raster 60 - display_start 51 = display line 9
        // Sprite 0 has higher priority, so it should appear on top (white)
        assert_eq!(
            vic.framebuffer[9][26], 0x01,
            "Sprite 0 should have priority over sprite 1"
        );
    }

    #[test]
    fn test_render_sprites_outside_visible_area() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;
        vic.registers[0x00] = 50;
        vic.registers[0x01] = 60;
        vic.registers[0x27] = 0x01;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0xFF;

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        // Render at scanline outside visible area (before display start)
        vic.render_sprites_scanline(10, &sprite_data);

        // Framebuffer should be unchanged
        for row in vic.framebuffer.iter() {
            for pixel in row.iter() {
                assert_eq!(*pixel, 0x06);
            }
        }

        // Also test scanline after visible area
        vic.render_sprites_scanline(300, &sprite_data);

        for row in vic.framebuffer.iter() {
            for pixel in row.iter() {
                assert_eq!(*pixel, 0x06);
            }
        }
    }

    #[test]
    fn test_render_sprites_x_position_9bit() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;

        // Set sprite 0 X position to 300 (requires MSB)
        // 300 = 256 + 44
        vic.registers[0x00] = 44; // Low byte
        vic.registers[0x10] = 0x01; // MSB for sprite 0
        vic.registers[0x01] = 51; // Y position (raster 51 = display line 0)

        vic.registers[0x27] = 0x01; // White

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // One pixel

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // Screen X = 300 - 24 = 276
        assert_eq!(
            vic.framebuffer[0][276], 0x01,
            "Sprite should render at X=276"
        );
    }

    #[test]
    fn test_render_sprites_y_expand() {
        let mut vic = VicII::new();

        // Enable sprite 0 with Y expansion
        vic.registers[0x15] = 0x01;
        vic.registers[0x17] = 0x01; // Y expand for sprite 0

        vic.registers[0x00] = 24; // X at left edge
        vic.registers[0x01] = 51; // Y position (raster 51 = display line 0)
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x01; // White

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // First line has one pixel

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        // With Y expansion, first line of sprite data should appear on two display lines
        vic.render_sprites_scanline(51, &sprite_data);
        vic.render_sprites_scanline(52, &sprite_data);

        // Both lines should have the sprite pixel
        assert_eq!(vic.framebuffer[0][0], 0x01, "First line should have sprite");
        assert_eq!(
            vic.framebuffer[1][0], 0x01,
            "Second line should also have sprite (Y expand)"
        );
    }

    #[test]
    fn test_render_sprites_x_expand() {
        let mut vic = VicII::new();

        // Enable sprite 0 with X expansion
        vic.registers[0x15] = 0x01;
        vic.registers[0x1D] = 0x01; // X expand for sprite 0

        vic.registers[0x00] = 24; // X at left edge
        vic.registers[0x01] = 51; // Y position (raster 51 = display line 0)
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x01; // White

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // One pixel in first position

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // With X expansion, one pixel should become two pixels
        assert_eq!(
            vic.framebuffer[0][0], 0x01,
            "First pixel of expanded sprite"
        );
        assert_eq!(
            vic.framebuffer[0][1], 0x01,
            "Second pixel of expanded sprite"
        );
        assert_eq!(
            vic.framebuffer[0][2], 0x06,
            "Third pixel should be background"
        );
    }

    #[test]
    fn test_render_sprites_multiple_sprites() {
        let mut vic = VicII::new();

        // Enable sprites 0, 3, and 7
        vic.registers[0x15] = 0b10001001;

        // Set different positions (all at same Y, different X)
        vic.registers[0x00] = 24 + 0; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y (raster 51 = display line 0)
        vic.registers[0x06] = 24 + 50; // Sprite 3 X
        vic.registers[0x07] = 51; // Sprite 3 Y
        vic.registers[0x0E] = 24 + 100; // Sprite 7 X
        vic.registers[0x0F] = 51; // Sprite 7 Y
        vic.registers[0x10] = 0;

        // Different colors
        vic.registers[0x27] = 0x01; // Sprite 0: white
        vic.registers[0x2A] = 0x02; // Sprite 3: red
        vic.registers[0x2E] = 0x03; // Sprite 7: cyan

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;
        sprite_data[3][0] = 0x80;
        sprite_data[7][0] = 0x80;

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // Check each sprite rendered at its position
        assert_eq!(vic.framebuffer[0][0], 0x01, "Sprite 0 at X=0");
        assert_eq!(vic.framebuffer[0][50], 0x02, "Sprite 3 at X=50");
        assert_eq!(vic.framebuffer[0][100], 0x03, "Sprite 7 at X=100");
    }

    // =========================================================================
    // Sprite Multicolor Mode Tests (T073)
    // =========================================================================

    #[test]
    fn test_render_sprites_multicolor_basic() {
        let mut vic = VicII::new();

        // Enable sprite 0 in multicolor mode
        vic.registers[0x15] = 0x01; // Enable sprite 0
        vic.registers[0x1C] = 0x01; // Multicolor mode for sprite 0

        // Position sprite at left edge
        vic.registers[0x00] = 24; // X at left edge
        vic.registers[0x01] = 51; // Y position (raster 51 = display line 0)
        vic.registers[0x10] = 0; // X MSB

        // Set multicolor colors
        vic.registers[0x25] = 0x05; // Multicolor 0 (green)
        vic.registers[0x26] = 0x07; // Multicolor 1 (yellow)
        vic.registers[0x27] = 0x02; // Sprite 0 individual color (red)

        // Create sprite data with all bit pair patterns:
        // Byte 0: 01 10 11 00 = 0x6C
        // This should produce: MC0, sprite color, MC1, transparent
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0b01_10_11_00; // 0x6C

        // Clear framebuffer
        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06; // Blue background
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // In multicolor mode, each bit pair produces a 2-pixel wide "pixel"
        // Pattern 0b01_10_11_00:
        // - Bits 01 (MC0 = 0x05 green) at pixels 0-1
        // - Bits 10 (sprite color = 0x02 red) at pixels 2-3
        // - Bits 11 (MC1 = 0x07 yellow) at pixels 4-5
        // - Bits 00 (transparent) at pixels 6-7

        assert_eq!(vic.framebuffer[0][0], 0x05, "Pixel 0 should be MC0 (green)");
        assert_eq!(vic.framebuffer[0][1], 0x05, "Pixel 1 should be MC0 (green)");
        assert_eq!(
            vic.framebuffer[0][2], 0x02,
            "Pixel 2 should be sprite color (red)"
        );
        assert_eq!(
            vic.framebuffer[0][3], 0x02,
            "Pixel 3 should be sprite color (red)"
        );
        assert_eq!(
            vic.framebuffer[0][4], 0x07,
            "Pixel 4 should be MC1 (yellow)"
        );
        assert_eq!(
            vic.framebuffer[0][5], 0x07,
            "Pixel 5 should be MC1 (yellow)"
        );
        assert_eq!(
            vic.framebuffer[0][6], 0x06,
            "Pixel 6 should be background (transparent)"
        );
        assert_eq!(
            vic.framebuffer[0][7], 0x06,
            "Pixel 7 should be background (transparent)"
        );
    }

    #[test]
    fn test_render_sprites_multicolor_full_line() {
        let mut vic = VicII::new();

        // Enable sprite 0 in multicolor mode
        vic.registers[0x15] = 0x01;
        vic.registers[0x1C] = 0x01;

        vic.registers[0x00] = 24;
        vic.registers[0x01] = 51;
        vic.registers[0x10] = 0;

        vic.registers[0x25] = 0x05; // MC0
        vic.registers[0x26] = 0x07; // MC1
        vic.registers[0x27] = 0x02; // Sprite color

        // Fill all 3 bytes with bit pair 10 (sprite color)
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0b10_10_10_10; // 0xAA
        sprite_data[0][1] = 0b10_10_10_10; // 0xAA
        sprite_data[0][2] = 0b10_10_10_10; // 0xAA

        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // All 24 pixels should be sprite color (red)
        // 3 bytes * 4 bit pairs * 2 pixels per pair = 24 pixels
        for x in 0..24 {
            assert_eq!(
                vic.framebuffer[0][x], 0x02,
                "Pixel {} should be sprite color",
                x
            );
        }
        // Next pixel should be background
        assert_eq!(
            vic.framebuffer[0][24], 0x06,
            "Pixel 24 should be background"
        );
    }

    #[test]
    fn test_render_sprites_multicolor_with_x_expansion() {
        let mut vic = VicII::new();

        // Enable sprite 0 in multicolor mode with X expansion
        vic.registers[0x15] = 0x01;
        vic.registers[0x1C] = 0x01; // Multicolor
        vic.registers[0x1D] = 0x01; // X expand

        vic.registers[0x00] = 24;
        vic.registers[0x01] = 51;
        vic.registers[0x10] = 0;

        vic.registers[0x25] = 0x05; // MC0
        vic.registers[0x26] = 0x07; // MC1
        vic.registers[0x27] = 0x02; // Sprite color

        // First bit pair is 01 (MC0)
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0b01_00_00_00; // Only first multicolor pixel set

        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // With X expansion, a multicolor pixel (normally 2 wide) becomes 4 wide
        assert_eq!(vic.framebuffer[0][0], 0x05, "Pixel 0 (MC0 expanded)");
        assert_eq!(vic.framebuffer[0][1], 0x05, "Pixel 1 (MC0 expanded)");
        assert_eq!(vic.framebuffer[0][2], 0x05, "Pixel 2 (MC0 expanded)");
        assert_eq!(vic.framebuffer[0][3], 0x05, "Pixel 3 (MC0 expanded)");
        assert_eq!(vic.framebuffer[0][4], 0x06, "Pixel 4 (background)");
    }

    #[test]
    fn test_render_sprites_multicolor_disabled_sprite_stays_hires() {
        let mut vic = VicII::new();

        // Enable sprite 0, but NOT in multicolor mode
        vic.registers[0x15] = 0x01;
        vic.registers[0x1C] = 0x00; // No multicolor

        vic.registers[0x00] = 24;
        vic.registers[0x01] = 51;
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x02; // Sprite color

        // In hires mode, each bit is one pixel
        // 0x80 = 10000000 = first pixel set
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;

        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // In hires mode, only pixel 0 should be set
        assert_eq!(vic.framebuffer[0][0], 0x02, "Pixel 0 (hires sprite)");
        assert_eq!(vic.framebuffer[0][1], 0x06, "Pixel 1 (background in hires)");
    }

    #[test]
    fn test_render_sprites_mixed_multicolor_and_hires() {
        let mut vic = VicII::new();

        // Enable sprites 0 (hires) and 1 (multicolor)
        vic.registers[0x15] = 0b00000011; // Sprites 0 and 1
        vic.registers[0x1C] = 0b00000010; // Only sprite 1 multicolor

        // Sprite 0 at X=24 (hires)
        vic.registers[0x00] = 24;
        vic.registers[0x01] = 51;

        // Sprite 1 at X=24+50 (multicolor)
        vic.registers[0x02] = 24 + 50;
        vic.registers[0x03] = 51;

        vic.registers[0x10] = 0;

        vic.registers[0x25] = 0x05; // MC0
        vic.registers[0x26] = 0x07; // MC1
        vic.registers[0x27] = 0x01; // Sprite 0 color (white)
        vic.registers[0x28] = 0x02; // Sprite 1 color (red)

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Hires: first pixel
        sprite_data[1][0] = 0b10_00_00_00; // Multicolor: first bit pair = sprite color

        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06;
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite 0 (hires): single pixel at X=0
        assert_eq!(vic.framebuffer[0][0], 0x01, "Sprite 0 hires pixel");
        assert_eq!(vic.framebuffer[0][1], 0x06, "Background after sprite 0");

        // Sprite 1 (multicolor): 2 pixels at X=50
        assert_eq!(vic.framebuffer[0][50], 0x02, "Sprite 1 multicolor pixel 0");
        assert_eq!(vic.framebuffer[0][51], 0x02, "Sprite 1 multicolor pixel 1");
        assert_eq!(vic.framebuffer[0][52], 0x06, "Background after sprite 1");
    }

    #[test]
    fn test_render_sprites_multicolor_all_patterns() {
        let mut vic = VicII::new();

        // Enable sprite 0 in multicolor mode
        vic.registers[0x15] = 0x01;
        vic.registers[0x1C] = 0x01;

        vic.registers[0x00] = 24;
        vic.registers[0x01] = 51;
        vic.registers[0x10] = 0;

        vic.registers[0x25] = 0x05; // MC0 (green)
        vic.registers[0x26] = 0x07; // MC1 (yellow)
        vic.registers[0x27] = 0x02; // Sprite color (red)

        // Test all four bit patterns
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        // Byte 0: 00 01 10 11 = patterns 0, 1, 2, 3
        sprite_data[0][0] = 0b00_01_10_11; // 0x1B

        for row in vic.framebuffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0x06; // Blue background
            }
        }

        vic.render_sprites_scanline(51, &sprite_data);

        // Pattern 00: Transparent (pixels 0-1)
        assert_eq!(
            vic.framebuffer[0][0], 0x06,
            "Pattern 00 pixel 0 = transparent"
        );
        assert_eq!(
            vic.framebuffer[0][1], 0x06,
            "Pattern 00 pixel 1 = transparent"
        );

        // Pattern 01: MC0 (pixels 2-3)
        assert_eq!(vic.framebuffer[0][2], 0x05, "Pattern 01 pixel 0 = MC0");
        assert_eq!(vic.framebuffer[0][3], 0x05, "Pattern 01 pixel 1 = MC0");

        // Pattern 10: Sprite color (pixels 4-5)
        assert_eq!(
            vic.framebuffer[0][4], 0x02,
            "Pattern 10 pixel 0 = sprite color"
        );
        assert_eq!(
            vic.framebuffer[0][5], 0x02,
            "Pattern 10 pixel 1 = sprite color"
        );

        // Pattern 11: MC1 (pixels 6-7)
        assert_eq!(vic.framebuffer[0][6], 0x07, "Pattern 11 pixel 0 = MC1");
        assert_eq!(vic.framebuffer[0][7], 0x07, "Pattern 11 pixel 1 = MC1");
    }

    #[test]
    fn test_sprite_to_background_priority_sprite_in_front() {
        let mut vic = VicII::new();

        // Enable sprite 0 and place it at a visible position
        vic.registers[0x15] = 0x01; // Enable sprite 0
        vic.registers[0x00] = 24; // Sprite 0 X = 24 (left edge of display)
        vic.registers[0x01] = 51; // Sprite 0 Y = 51 (first visible line)
        vic.registers[0x10] = 0; // X MSB = 0 for all sprites
        vic.registers[0x27] = 0x02; // Sprite 0 color = red
        vic.registers[0x1B] = 0x00; // All sprites in FRONT of background

        // Create character ROM with a pattern that has foreground pixels
        let mut char_rom = vec![0u8; 2048];
        char_rom[0] = 0xFF; // Character 0, line 0: all pixels set (foreground)

        // Screen RAM: character 0 at position 0
        let screen_ram = vec![0u8; 1000];

        // Color RAM: foreground color = 1 (white)
        let color_ram = vec![0x01u8; 1000];

        // First render the text (creates foreground mask)
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Now render sprites
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        // Sprite 0: first line has pixel set at leftmost position
        sprite_data[0][0] = 0x80; // Bit 7 set = leftmost pixel

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite is in front, so sprite color should win over text foreground
        assert_eq!(
            vic.framebuffer[0][0], 0x02,
            "Sprite in front should override foreground text"
        );
    }

    #[test]
    fn test_sprite_to_background_priority_sprite_behind() {
        let mut vic = VicII::new();

        // Enable sprite 0 and place it at a visible position
        vic.registers[0x15] = 0x01; // Enable sprite 0
        vic.registers[0x00] = 24; // Sprite 0 X = 24 (left edge of display)
        vic.registers[0x01] = 51; // Sprite 0 Y = 51 (first visible line)
        vic.registers[0x10] = 0; // X MSB = 0 for all sprites
        vic.registers[0x27] = 0x02; // Sprite 0 color = red
        vic.registers[0x1B] = 0x01; // Sprite 0 BEHIND background

        // Create character ROM with a pattern that has foreground pixels
        let mut char_rom = vec![0u8; 2048];
        char_rom[0] = 0xFF; // Character 0, line 0: all pixels set (foreground)

        // Screen RAM: character 0 at position 0
        let screen_ram = vec![0u8; 1000];

        // Color RAM: foreground color = 1 (white)
        let color_ram = vec![0x01u8; 1000];

        // First render the text (creates foreground mask)
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Now render sprites
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        // Sprite 0: first line has pixel set at leftmost position
        sprite_data[0][0] = 0x80; // Bit 7 set = leftmost pixel

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite is behind foreground, so text foreground color should be preserved
        assert_eq!(
            vic.framebuffer[0][0], 0x01,
            "Sprite behind should not override foreground text"
        );
    }

    #[test]
    fn test_sprite_to_background_priority_sprite_behind_shows_in_background_area() {
        let mut vic = VicII::new();

        // Enable sprite 0 and place it at a visible position
        vic.registers[0x15] = 0x01; // Enable sprite 0
        vic.registers[0x00] = 24; // Sprite 0 X = 24 (left edge of display)
        vic.registers[0x01] = 51; // Sprite 0 Y = 51 (first visible line)
        vic.registers[0x10] = 0; // X MSB = 0 for all sprites
        vic.registers[0x27] = 0x02; // Sprite 0 color = red
        vic.registers[0x21] = 0x06; // Background color = blue
        vic.registers[0x1B] = 0x01; // Sprite 0 BEHIND background

        // Create character ROM with a pattern that has some background pixels
        let mut char_rom = vec![0u8; 2048];
        char_rom[0] = 0x00; // Character 0, line 0: all pixels clear (background)

        // Screen RAM: character 0 at position 0
        let screen_ram = vec![0u8; 1000];

        // Color RAM: foreground color = 1 (white)
        let color_ram = vec![0x01u8; 1000];

        // First render the text (creates foreground mask)
        vic.step_scanline(51, &char_rom, &screen_ram, &color_ram);

        // Now render sprites
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        // Sprite 0: first line has pixel set at leftmost position
        sprite_data[0][0] = 0x80; // Bit 7 set = leftmost pixel

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite is behind, but this area is background (not foreground),
        // so sprite should still be visible
        assert_eq!(
            vic.framebuffer[0][0], 0x02,
            "Sprite behind should show through background color areas"
        );
    }

    #[test]
    fn test_sprite_to_sprite_priority() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1 at the same position
        vic.registers[0x15] = 0x03; // Enable sprites 0 and 1
        vic.registers[0x00] = 24; // Sprite 0 X = 24
        vic.registers[0x01] = 51; // Sprite 0 Y = 51
        vic.registers[0x02] = 24; // Sprite 1 X = 24 (same as sprite 0)
        vic.registers[0x03] = 51; // Sprite 1 Y = 51 (same as sprite 0)
        vic.registers[0x10] = 0; // X MSB = 0 for all sprites
        vic.registers[0x27] = 0x02; // Sprite 0 color = red
        vic.registers[0x28] = 0x05; // Sprite 1 color = green

        // Clear foreground mask for this line
        vic.clear_foreground_mask_line(0);

        // Both sprites have the same pixel set
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Sprite 0: leftmost pixel
        sprite_data[1][0] = 0x80; // Sprite 1: leftmost pixel (same position)

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite 0 has higher priority than sprite 1, so sprite 0 color should win
        assert_eq!(
            vic.framebuffer[0][0], 0x02,
            "Sprite 0 (higher priority) should appear on top of sprite 1"
        );
    }

    // T076: Sprite Collision Detection Tests

    #[test]
    fn test_sprite_sprite_collision_two_overlapping() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1 at the same position
        vic.registers[0x15] = 0x03; // Enable sprites 0 and 1
        vic.registers[0x00] = 24; // Sprite 0 X = 24
        vic.registers[0x01] = 51; // Sprite 0 Y = 51
        vic.registers[0x02] = 24; // Sprite 1 X = 24 (same as sprite 0)
        vic.registers[0x03] = 51; // Sprite 1 Y = 51 (same as sprite 0)
        vic.registers[0x10] = 0; // X MSB = 0 for all sprites
        vic.registers[0x27] = 0x02; // Sprite 0 color
        vic.registers[0x28] = 0x05; // Sprite 1 color

        // Clear foreground mask and collision registers
        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_ss = 0;
        vic.sprite_collision_sb = 0;

        // Both sprites have the same pixel set
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Sprite 0: leftmost pixel
        sprite_data[1][0] = 0x80; // Sprite 1: leftmost pixel (same position)

        vic.render_sprites_scanline(51, &sprite_data);

        // Both sprites should be marked as colliding (bits 0 and 1 set)
        assert_eq!(
            vic.sprite_collision_ss, 0x03,
            "Sprite-sprite collision should mark sprites 0 and 1"
        );
    }

    #[test]
    fn test_sprite_sprite_collision_no_overlap() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1 at different positions (no overlap)
        vic.registers[0x15] = 0x03; // Enable sprites 0 and 1
        vic.registers[0x00] = 24; // Sprite 0 X = 24
        vic.registers[0x01] = 51; // Sprite 0 Y = 51
        vic.registers[0x02] = 100; // Sprite 1 X = 100 (far from sprite 0)
        vic.registers[0x03] = 51; // Sprite 1 Y = 51
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x02;
        vic.registers[0x28] = 0x05;

        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_ss = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Sprite 0: leftmost pixel
        sprite_data[1][0] = 0x80; // Sprite 1: leftmost pixel (but at different X)

        vic.render_sprites_scanline(51, &sprite_data);

        // No collision because sprites don't overlap
        assert_eq!(
            vic.sprite_collision_ss, 0,
            "No sprite-sprite collision when sprites don't overlap"
        );
    }

    #[test]
    fn test_sprite_sprite_collision_three_sprites() {
        let mut vic = VicII::new();

        // Enable sprites 0, 1, and 2 at the same position
        vic.registers[0x15] = 0x07; // Enable sprites 0, 1, 2
        vic.registers[0x00] = 24; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y
        vic.registers[0x02] = 24; // Sprite 1 X
        vic.registers[0x03] = 51; // Sprite 1 Y
        vic.registers[0x04] = 24; // Sprite 2 X
        vic.registers[0x05] = 51; // Sprite 2 Y
        vic.registers[0x10] = 0;

        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_ss = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;
        sprite_data[1][0] = 0x80;
        sprite_data[2][0] = 0x80;

        vic.render_sprites_scanline(51, &sprite_data);

        // All three sprites should be marked as colliding (bits 0, 1, 2 set)
        assert_eq!(
            vic.sprite_collision_ss, 0x07,
            "Sprite-sprite collision should mark sprites 0, 1, and 2"
        );
    }

    #[test]
    fn test_sprite_background_collision() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;
        vic.registers[0x00] = 24; // Sprite 0 X = 24
        vic.registers[0x01] = 51; // Sprite 0 Y = 51
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x02; // Sprite color

        // Set foreground at position (0, 0)
        vic.foreground_mask[0][0] = true;
        vic.sprite_collision_sb = 0;
        vic.sprite_collision_ss = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Sprite pixel at position that has foreground

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprite 0 should be marked as colliding with background
        assert_eq!(
            vic.sprite_collision_sb, 0x01,
            "Sprite-background collision should mark sprite 0"
        );
    }

    #[test]
    fn test_sprite_background_collision_no_foreground() {
        let mut vic = VicII::new();

        // Enable sprite 0
        vic.registers[0x15] = 0x01;
        vic.registers[0x00] = 24; // Sprite 0 X = 24
        vic.registers[0x01] = 51; // Sprite 0 Y = 51
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x02;

        // Ensure no foreground at sprite position
        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_sb = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;

        vic.render_sprites_scanline(51, &sprite_data);

        // No collision because no foreground at sprite position
        assert_eq!(
            vic.sprite_collision_sb, 0,
            "No sprite-background collision when no foreground"
        );
    }

    #[test]
    fn test_sprite_collision_with_priority_behind_background() {
        let mut vic = VicII::new();

        // Enable sprite 0 and set it behind background
        vic.registers[0x15] = 0x01; // Enable sprite 0
        vic.registers[0x1B] = 0x01; // Sprite 0 behind background
        vic.registers[0x00] = 24; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y
        vic.registers[0x10] = 0;
        vic.registers[0x27] = 0x02;

        // Set foreground at sprite position
        vic.foreground_mask[0][0] = true;
        vic.framebuffer[0][0] = 0x05; // Some foreground color
        vic.sprite_collision_sb = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80;

        vic.render_sprites_scanline(51, &sprite_data);

        // Collision should still be detected even though sprite is behind background
        assert_eq!(
            vic.sprite_collision_sb, 0x01,
            "Sprite-background collision should be detected even when sprite is behind"
        );

        // But the sprite should NOT be visible (foreground color preserved)
        assert_eq!(
            vic.framebuffer[0][0], 0x05,
            "Sprite behind background should not overwrite foreground"
        );
    }

    #[test]
    fn test_collision_registers_cleared_on_read() {
        let mut vic = VicII::new();

        // Set collision flags
        vic.sprite_collision_ss = 0x55;
        vic.sprite_collision_sb = 0xAA;

        // Read collision registers (this returns the value)
        let ss = vic.read(0x1E);
        let sb = vic.read(0x1F);

        assert_eq!(ss, 0x55);
        assert_eq!(sb, 0xAA);

        // Note: In the real implementation, clearing is handled by C64Memory
        // after reading. Here we test that the clear methods work.
        vic.clear_sprite_collision_ss();
        vic.clear_sprite_collision_sb();

        assert_eq!(vic.sprite_collision_ss, 0);
        assert_eq!(vic.sprite_collision_sb, 0);
    }

    #[test]
    fn test_collision_accumulates_across_scanlines() {
        let mut vic = VicII::new();

        // Enable sprites 0, 1, 2 - all at the same position
        vic.registers[0x15] = 0x07;
        vic.registers[0x10] = 0;

        // All sprites at line 51, same X position
        vic.registers[0x00] = 24; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y
        vic.registers[0x02] = 24; // Sprite 1 X
        vic.registers[0x03] = 51; // Sprite 1 Y
        vic.registers[0x04] = 24; // Sprite 2 X
        vic.registers[0x05] = 51; // Sprite 2 Y

        vic.sprite_collision_ss = 0;

        // Set pixels in first two lines for all sprites
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        // First line (data offset 0)
        sprite_data[0][0] = 0x80;
        sprite_data[1][0] = 0x80;
        sprite_data[2][0] = 0x80;
        // Second line (data offset 3, since 3 bytes per line)
        sprite_data[0][3] = 0x80;
        sprite_data[1][3] = 0x80;
        sprite_data[2][3] = 0x80;

        // First scanline: all three sprites collide
        vic.clear_foreground_mask_line(0);
        vic.render_sprites_scanline(51, &sprite_data);
        assert_eq!(
            vic.sprite_collision_ss, 0x07,
            "Line 51: sprites 0, 1, and 2 collide"
        );

        // Clear collision register to test that second scanline re-detects
        vic.sprite_collision_ss = 0;

        // Second scanline: collisions still detected
        vic.clear_foreground_mask_line(1);
        vic.render_sprites_scanline(52, &sprite_data);
        assert_eq!(
            vic.sprite_collision_ss, 0x07,
            "Line 52: sprites 0, 1, and 2 collide again"
        );
    }

    #[test]
    fn test_multicolor_sprite_collision() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1 in multicolor mode at same position
        vic.registers[0x15] = 0x03; // Enable sprites 0 and 1
        vic.registers[0x1C] = 0x03; // Multicolor mode for sprites 0 and 1
        vic.registers[0x00] = 24; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y
        vic.registers[0x02] = 24; // Sprite 1 X
        vic.registers[0x03] = 51; // Sprite 1 Y
        vic.registers[0x10] = 0;
        vic.registers[0x25] = 0x03; // Multicolor 0
        vic.registers[0x26] = 0x04; // Multicolor 1
        vic.registers[0x27] = 0x02; // Sprite 0 color
        vic.registers[0x28] = 0x05; // Sprite 1 color

        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_ss = 0;

        // Both sprites have non-transparent pixels at same position
        // Bit pair 10 = sprite color (non-transparent)
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0b10_00_00_00; // First multicolor pixel set
        sprite_data[1][0] = 0b10_00_00_00; // Same position

        vic.render_sprites_scanline(51, &sprite_data);

        // Both sprites should collide
        assert_eq!(
            vic.sprite_collision_ss, 0x03,
            "Multicolor sprites should detect collision"
        );
    }

    #[test]
    fn test_expanded_sprite_collision() {
        let mut vic = VicII::new();

        // Enable sprites 0 and 1 with X expansion
        vic.registers[0x15] = 0x03;
        vic.registers[0x1D] = 0x03; // X expansion for sprites 0 and 1
        vic.registers[0x00] = 24; // Sprite 0 X
        vic.registers[0x01] = 51; // Sprite 0 Y
        // Sprite 1 positioned so expanded pixel overlaps with sprite 0
        vic.registers[0x02] = 25; // Sprite 1 X = 25 (1 pixel offset)
        vic.registers[0x03] = 51; // Sprite 1 Y
        vic.registers[0x10] = 0;

        vic.clear_foreground_mask_line(0);
        vic.sprite_collision_ss = 0;

        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];
        sprite_data[0][0] = 0x80; // Sprite 0: leftmost pixel (expands to 2 pixels)
        sprite_data[1][0] = 0x80; // Sprite 1: leftmost pixel

        vic.render_sprites_scanline(51, &sprite_data);

        // Sprites should collide because expanded pixels overlap
        assert_eq!(
            vic.sprite_collision_ss, 0x03,
            "Expanded sprites should detect collision when overlapping"
        );
    }
}
