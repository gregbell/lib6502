//! C64 memory system with bank switching.
//!
//! The C64 has a complex memory architecture where multiple ROMs, RAM,
//! and I/O devices are mapped to overlapping address ranges. The 6510
//! CPU's I/O port ($00-$01) controls which components are visible.
//!
//! Memory Map:
//! - $0000-$0001: 6510 I/O port
//! - $0002-$9FFF: RAM (always)
//! - $A000-$BFFF: BASIC ROM or RAM
//! - $C000-$CFFF: RAM (always)
//! - $D000-$DFFF: I/O, Character ROM, or RAM
//! - $E000-$FFFF: KERNAL ROM or RAM

use super::keyboard::Keyboard;
use crate::devices::{Cia6526, ColorRam, Port6510, Sid6581, VicII};
use lib6502::{Device, MemoryBus};

/// C64 I/O area start address.
const IO_START: u16 = 0xD000;
/// C64 I/O area end address (inclusive).
const IO_END: u16 = 0xDFFF;
/// BASIC ROM start address.
const BASIC_START: u16 = 0xA000;
/// BASIC ROM end address (inclusive).
const BASIC_END: u16 = 0xBFFF;
/// KERNAL ROM start address.
const KERNAL_START: u16 = 0xE000;

/// C64 memory system implementing bank switching.
pub struct C64Memory {
    /// 64KB main RAM.
    ram: Box<[u8; 65536]>,

    /// BASIC ROM (8KB at $A000-$BFFF).
    basic_rom: Box<[u8; 8192]>,
    /// KERNAL ROM (8KB at $E000-$FFFF).
    kernal_rom: Box<[u8; 8192]>,
    /// Character ROM (4KB at $D000-$DFFF when visible).
    char_rom: Box<[u8; 4096]>,

    /// 6510 I/O port (bank switching control).
    pub port: Port6510,

    /// VIC-II video chip.
    pub vic: VicII,
    /// SID audio chip.
    pub sid: Sid6581,
    /// CIA1 (keyboard, joystick, IRQ).
    pub cia1: Cia6526,
    /// CIA2 (IEC bus, VIC bank, NMI).
    pub cia2: Cia6526,
    /// Color RAM.
    pub color_ram: ColorRam,

    /// Keyboard matrix.
    pub keyboard: Keyboard,

    /// ROMs loaded flag.
    roms_loaded: bool,
}

impl C64Memory {
    /// Create a new C64 memory system with empty ROMs.
    pub fn new() -> Self {
        // Initialize RAM with a pattern (real C64 has random values)
        let mut ram = Box::new([0u8; 65536]);
        // Initialize zero page and stack with typical values
        ram[0x00] = 0x2F; // DDR default
        ram[0x01] = 0x37; // Port default

        Self {
            ram,
            basic_rom: Box::new([0; 8192]),
            kernal_rom: Box::new([0; 8192]),
            char_rom: Box::new([0; 4096]),
            port: Port6510::new(),
            vic: VicII::new(),
            sid: Sid6581::new(),
            cia1: Cia6526::new_cia1(),
            cia2: Cia6526::new_cia2(),
            color_ram: ColorRam::new(),
            keyboard: Keyboard::new(),
            roms_loaded: false,
        }
    }

    /// Load ROMs into memory.
    ///
    /// # Arguments
    /// - `basic`: BASIC ROM data (must be exactly 8192 bytes)
    /// - `kernal`: KERNAL ROM data (must be exactly 8192 bytes)
    /// - `charrom`: Character ROM data (must be exactly 4096 bytes)
    ///
    /// # Returns
    /// `Ok(())` if ROMs are valid sizes, `Err` with message otherwise.
    pub fn load_roms(&mut self, basic: &[u8], kernal: &[u8], charrom: &[u8]) -> Result<(), String> {
        if basic.len() != 8192 {
            return Err(format!(
                "BASIC ROM must be 8192 bytes, got {}",
                basic.len()
            ));
        }
        if kernal.len() != 8192 {
            return Err(format!(
                "KERNAL ROM must be 8192 bytes, got {}",
                kernal.len()
            ));
        }
        if charrom.len() != 4096 {
            return Err(format!(
                "Character ROM must be 4096 bytes, got {}",
                charrom.len()
            ));
        }

        self.basic_rom.copy_from_slice(basic);
        self.kernal_rom.copy_from_slice(kernal);
        self.char_rom.copy_from_slice(charrom);
        self.roms_loaded = true;

        Ok(())
    }

    /// Check if ROMs have been loaded.
    pub fn roms_loaded(&self) -> bool {
        self.roms_loaded
    }

    /// Load KERNAL ROM separately.
    pub fn load_kernal(&mut self, data: &[u8]) {
        self.kernal_rom.copy_from_slice(data);
        self.update_roms_loaded();
    }

    /// Load BASIC ROM separately.
    pub fn load_basic(&mut self, data: &[u8]) {
        self.basic_rom.copy_from_slice(data);
        self.update_roms_loaded();
    }

    /// Load Character ROM separately.
    pub fn load_charrom(&mut self, data: &[u8]) {
        self.char_rom.copy_from_slice(data);
        self.update_roms_loaded();
    }

    /// Check if all ROMs are loaded (non-zero data).
    fn update_roms_loaded(&mut self) {
        // Check if any byte is non-zero in each ROM
        let kernal_loaded = self.kernal_rom.iter().any(|&b| b != 0);
        let basic_loaded = self.basic_rom.iter().any(|&b| b != 0);
        let char_loaded = self.char_rom.iter().any(|&b| b != 0);
        self.roms_loaded = kernal_loaded && basic_loaded && char_loaded;
    }

    /// Get direct access to RAM for DMA-like operations.
    pub fn ram(&self) -> &[u8] {
        &*self.ram
    }

    /// Get mutable access to RAM.
    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut *self.ram
    }

    /// Get the character ROM data (for VIC-II rendering).
    pub fn char_rom(&self) -> &[u8] {
        &*self.char_rom
    }

    /// Get the current VIC-II bank (0-3).
    ///
    /// The VIC-II sees memory in 16KB banks selected by CIA2 port A.
    pub fn vic_bank(&self) -> u8 {
        self.cia2.vic_bank()
    }

    /// Read a byte as the VIC-II would see it.
    ///
    /// VIC-II has different memory mapping than the CPU:
    /// - Never sees BASIC, KERNAL, or I/O
    /// - Sees Character ROM at $1000-$1FFF in banks 0 and 2
    pub fn vic_read(&self, addr: u16) -> u8 {
        let bank = self.vic_bank() as u16;
        let physical_addr = (bank << 14) | (addr & 0x3FFF);

        // VIC sees Character ROM at $1000-$1FFF in banks 0 and 2
        if (bank == 0 || bank == 2) && (0x1000..0x2000).contains(&(addr & 0x3FFF)) {
            let char_offset = (addr & 0x0FFF) as usize;
            return self.char_rom[char_offset];
        }

        self.ram[physical_addr as usize]
    }

    /// Reset memory to power-on state (preserving ROMs).
    pub fn reset(&mut self) {
        // Clear RAM (but keep ROMs)
        self.ram.fill(0);
        self.ram[0x00] = 0x2F;
        self.ram[0x01] = 0x37;

        // Reset devices
        self.port = Port6510::new();
        self.vic.reset();
        self.sid.reset();
        self.cia1.reset();
        self.cia2.reset();
        self.color_ram.reset();
        self.keyboard.release_all();
    }

    /// Get the offset within an I/O device for a given address.
    #[inline]
    fn io_offset(&self, addr: u16) -> u16 {
        addr & 0x00FF
    }
}

impl Default for C64Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBus for C64Memory {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // 6510 I/O port
            0x0000..=0x0001 => self.port.read(addr),

            // RAM (always visible for CPU reads in $0002-$9FFF and $C000-$CFFF)
            0x0002..=0x9FFF | 0xC000..=0xCFFF => self.ram[addr as usize],

            // BASIC ROM area ($A000-$BFFF)
            BASIC_START..=BASIC_END => {
                if self.port.basic_visible() {
                    self.basic_rom[(addr - BASIC_START) as usize]
                } else {
                    self.ram[addr as usize]
                }
            }

            // I/O / Character ROM / RAM area ($D000-$DFFF)
            IO_START..=IO_END => {
                if self.port.io_visible() {
                    // I/O devices
                    match addr {
                        // VIC-II ($D000-$D3FF, mirrored)
                        0xD000..=0xD3FF => self.vic.read(self.io_offset(addr) & 0x3F),
                        // SID ($D400-$D7FF, mirrored)
                        0xD400..=0xD7FF => self.sid.read(self.io_offset(addr) & 0x1F),
                        // Color RAM ($D800-$DBFF)
                        0xD800..=0xDBFF => self.color_ram.read(addr - 0xD800),
                        // CIA1 ($DC00-$DCFF)
                        0xDC00..=0xDCFF => {
                            let offset = self.io_offset(addr) & 0x0F;
                            if offset == 0x01 {
                                // Reading Port B - combine keyboard scan with joystick
                                // Get column select from Port A output
                                let col_select = self.cia1.port_a.output();
                                // Scan keyboard matrix
                                let kb_rows = self.keyboard.scan(col_select);
                                // Combine with existing external_b (joystick)
                                let combined = self.cia1.external_b & kb_rows;
                                // Return combined value through CIA port logic
                                self.cia1.port_b.read(combined)
                            } else {
                                self.cia1.read(self.io_offset(addr))
                            }
                        }
                        // CIA2 ($DD00-$DDFF)
                        0xDD00..=0xDDFF => self.cia2.read(self.io_offset(addr)),
                        // Unmapped I/O ($DE00-$DFFF) - typically expansion port
                        0xDE00..=0xDFFF => 0xFF,
                        _ => unreachable!(),
                    }
                } else if self.port.char_rom_visible() {
                    // Character ROM
                    self.char_rom[(addr - IO_START) as usize]
                } else {
                    // RAM
                    self.ram[addr as usize]
                }
            }

            // KERNAL ROM area ($E000-$FFFF)
            KERNAL_START..=0xFFFF => {
                if self.port.kernal_visible() {
                    self.kernal_rom[(addr - KERNAL_START) as usize]
                } else {
                    self.ram[addr as usize]
                }
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // 6510 I/O port
            0x0000..=0x0001 => {
                self.port.write(addr, value);
                // Also write to underlying RAM for compatibility
                self.ram[addr as usize] = value;
            }

            // RAM always receives writes (ROM write-through)
            0x0002..=0x9FFF | 0xC000..=0xCFFF => {
                self.ram[addr as usize] = value;
            }

            // BASIC ROM area - writes go to RAM
            BASIC_START..=BASIC_END => {
                self.ram[addr as usize] = value;
            }

            // I/O / Character ROM / RAM area
            IO_START..=IO_END => {
                if self.port.io_visible() {
                    // I/O devices
                    match addr {
                        // VIC-II
                        0xD000..=0xD3FF => self.vic.write(self.io_offset(addr) & 0x3F, value),
                        // SID
                        0xD400..=0xD7FF => self.sid.write(self.io_offset(addr) & 0x1F, value),
                        // Color RAM
                        0xD800..=0xDBFF => self.color_ram.write(addr - 0xD800, value),
                        // CIA1
                        0xDC00..=0xDCFF => self.cia1.write(self.io_offset(addr), value),
                        // CIA2
                        0xDD00..=0xDDFF => self.cia2.write(self.io_offset(addr), value),
                        // Unmapped I/O - ignored
                        0xDE00..=0xDFFF => {}
                        _ => unreachable!(),
                    }
                } else {
                    // Character ROM area or RAM - writes always go to RAM
                    self.ram[addr as usize] = value;
                }
            }

            // KERNAL ROM area - writes go to RAM
            KERNAL_START..=0xFFFF => {
                self.ram[addr as usize] = value;
            }
        }
    }

    fn irq_active(&self) -> bool {
        // CIA1 generates IRQ, VIC-II can also generate IRQ
        self.cia1.has_interrupt() || self.vic.has_interrupt()
    }

    fn nmi_active(&self) -> bool {
        // CIA2 generates NMI (not IRQ)
        // The RESTORE key also directly triggers NMI but that's handled
        // separately via CPU::trigger_nmi()
        self.cia2.has_interrupt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory() {
        let mem = C64Memory::new();
        assert!(!mem.roms_loaded());
        assert_eq!(mem.read(0x00), 0x2F); // DDR default
        // Port data register returns effective value considering DDR and external input
        // DDR=0x2F, data=0x37, external=0 => effective = (0x37 & 0x2F) | 0 = 0x27
        assert_eq!(mem.read(0x01), 0x27);
    }

    #[test]
    fn test_basic_ram_access() {
        let mut mem = C64Memory::new();

        // Write to low RAM
        mem.write(0x1000, 0x42);
        assert_eq!(mem.read(0x1000), 0x42);

        // Write to high RAM (below BASIC)
        mem.write(0x9000, 0x55);
        assert_eq!(mem.read(0x9000), 0x55);
    }

    #[test]
    fn test_rom_loading() {
        let mut mem = C64Memory::new();

        let basic = vec![0xAA; 8192];
        let kernal = vec![0xBB; 8192];
        let charrom = vec![0xCC; 4096];

        assert!(mem.load_roms(&basic, &kernal, &charrom).is_ok());
        assert!(mem.roms_loaded());
    }

    #[test]
    fn test_rom_validation() {
        let mut mem = C64Memory::new();

        // Wrong BASIC size
        assert!(mem.load_roms(&[0; 100], &[0; 8192], &[0; 4096]).is_err());

        // Wrong KERNAL size
        assert!(mem.load_roms(&[0; 8192], &[0; 100], &[0; 4096]).is_err());

        // Wrong character ROM size
        assert!(mem.load_roms(&[0; 8192], &[0; 8192], &[0; 100]).is_err());
    }

    #[test]
    fn test_bank_switching() {
        let mut mem = C64Memory::new();

        // Load ROMs
        let basic = vec![0xAA; 8192];
        let kernal = vec![0xBB; 8192];
        let charrom = vec![0xCC; 4096];
        mem.load_roms(&basic, &kernal, &charrom).unwrap();

        // Default config (7): BASIC visible at $A000
        assert_eq!(mem.read(0xA000), 0xAA);

        // Write to $A000 - goes to RAM under BASIC
        mem.write(0xA000, 0x55);
        assert_eq!(mem.read(0xA000), 0xAA); // Still reads BASIC

        // Switch to all-RAM mode (write 0 to $01)
        mem.write(0x01, 0x30); // Clear bits 0-2
        assert_eq!(mem.read(0xA000), 0x55); // Now reads RAM
    }

    #[test]
    fn test_io_area() {
        let mut mem = C64Memory::new();

        // Default: I/O visible at $D000
        // Write to VIC-II border color
        mem.write(0xD020, 0x05);
        assert_eq!(mem.vic.border_color(), 0x05);

        // Write to SID volume
        mem.write(0xD418, 0x0F);
        assert_eq!(mem.sid.volume(), 0x0F);
    }

    #[test]
    fn test_color_ram() {
        let mut mem = C64Memory::new();

        // Write color value
        mem.write(0xD800, 0x03);
        // Read back (upper nibble is "floating")
        let val = mem.read(0xD800);
        assert_eq!(val & 0x0F, 0x03);
    }

    #[test]
    fn test_keyboard_matrix_via_cia1() {
        let mut mem = C64Memory::new();

        // Configure CIA1 Port A as all outputs (for column select)
        mem.write(0xDC02, 0xFF); // DDRA = all outputs

        // Configure CIA1 Port B as all inputs (for row read)
        mem.write(0xDC03, 0x00); // DDRB = all inputs

        // No keys pressed: reading Port B should return 0xFF
        mem.write(0xDC00, 0x00); // Select all columns (active low)
        assert_eq!(mem.read(0xDC01), 0xFF); // No rows pulled low

        // Press the 'A' key (row 1, col 2)
        mem.keyboard.key_down(1, 2);

        // Select column 2 (bit 2 = 0)
        mem.write(0xDC00, 0xFB); // 0b11111011 = select column 2
        let port_b = mem.read(0xDC01);
        // Row 1 should be low (bit 1 = 0)
        assert_eq!(port_b & 0x02, 0x00, "Row 1 should be low when A is pressed");
        // Other rows should be high
        assert_eq!(port_b & 0xFD, 0xFD, "Other rows should be high");

        // Select column 0 only - 'A' is not in this column
        mem.write(0xDC00, 0xFE); // 0b11111110 = select column 0
        assert_eq!(mem.read(0xDC01), 0xFF); // No rows pulled low

        // Release the key
        mem.keyboard.key_up(1, 2);
        mem.write(0xDC00, 0xFB); // Select column 2 again
        assert_eq!(mem.read(0xDC01), 0xFF); // Now all rows are high
    }

    #[test]
    fn test_keyboard_multiple_keys() {
        let mut mem = C64Memory::new();

        // Configure CIA1 ports
        mem.write(0xDC02, 0xFF); // DDRA = all outputs
        mem.write(0xDC03, 0x00); // DDRB = all inputs

        // Press 'A' (row 1, col 2) and 'S' (row 1, col 5)
        mem.keyboard.key_down(1, 2);
        mem.keyboard.key_down(1, 5);

        // Select all columns
        mem.write(0xDC00, 0x00);
        let port_b = mem.read(0xDC01);
        // Row 1 should be low
        assert_eq!(port_b & 0x02, 0x00);

        // Press 'D' (row 2, col 2)
        mem.keyboard.key_down(2, 2);

        // Select column 2 - should see rows 1 and 2 low
        mem.write(0xDC00, 0xFB);
        let port_b = mem.read(0xDC01);
        assert_eq!(port_b & 0x02, 0x00, "Row 1 should be low");
        assert_eq!(port_b & 0x04, 0x00, "Row 2 should be low");
        assert_eq!(port_b & 0xF9, 0xF9, "Other rows should be high");
    }
}
