//! C64 system orchestration and timing.
//!
//! This module provides the top-level `C64System` struct that coordinates
//! CPU execution, VIC-II rendering, SID audio, and CIA timing.

use super::iec_bus::IecBus;
use super::joystick::JoystickPorts;
use super::C64Memory;
use crate::devices::{SPRITE_COUNT, SPRITE_DATA_SIZE};
use lib6502::{Device, MemoryBus, CPU, OPCODE_TABLE};

/// C64 region (PAL or NTSC) affecting timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Region {
    /// PAL (European) timing: 985,248 Hz, 50 Hz, 312 scanlines
    #[default]
    PAL,
    /// NTSC (American) timing: 1,022,727 Hz, 60 Hz, 263 scanlines
    NTSC,
}

impl Region {
    /// Get the CPU clock frequency in Hz.
    pub fn clock_hz(&self) -> u32 {
        match self {
            Region::PAL => 985_248,
            Region::NTSC => 1_022_727,
        }
    }

    /// Get the number of scanlines per frame.
    pub fn scanlines(&self) -> u16 {
        match self {
            Region::PAL => 312,
            Region::NTSC => 263,
        }
    }

    /// Get the cycles per scanline.
    pub fn cycles_per_line(&self) -> u16 {
        match self {
            Region::PAL => 63,
            Region::NTSC => 65,
        }
    }

    /// Get the total cycles per frame.
    pub fn cycles_per_frame(&self) -> u32 {
        self.scanlines() as u32 * self.cycles_per_line() as u32
    }

    /// Get the frame rate in Hz.
    pub fn frame_rate(&self) -> f32 {
        match self {
            Region::PAL => 50.125,  // Exact: 985248 / (312 * 63)
            Region::NTSC => 59.826, // Exact: 1022727 / (263 * 65)
        }
    }
}

/// Commodore 64 emulator system.
///
/// This is the main entry point for C64 emulation. It coordinates the CPU,
/// memory, and timing to produce accurate frame-by-frame emulation.
pub struct C64System {
    /// The 6502/6510 CPU.
    cpu: CPU<C64Memory>,

    /// Current region (PAL/NTSC).
    region: Region,

    /// Current scanline within the frame.
    current_scanline: u16,

    /// Cycle count within current scanline.
    cycle_in_scanline: u16,

    /// Total frame count since reset.
    frame_count: u64,

    /// Whether emulation is running.
    running: bool,

    /// IEC bus for disk drive communication.
    iec_bus: IecBus,

    /// Joystick ports manager.
    joystick_ports: JoystickPorts,
}

impl C64System {
    /// Create a new C64 system with the specified region.
    pub fn new(region: Region) -> Self {
        let memory = C64Memory::new();
        let cpu = CPU::new(memory);

        Self {
            cpu,
            region,
            current_scanline: 0,
            cycle_in_scanline: 0,
            frame_count: 0,
            running: false,
            iec_bus: IecBus::new(),
            joystick_ports: JoystickPorts::new(),
        }
    }

    /// Load ROMs into the C64 memory.
    ///
    /// ROMs must be loaded before the system can boot properly.
    pub fn load_roms(&mut self, basic: &[u8], kernal: &[u8], charrom: &[u8]) -> Result<(), String> {
        self.cpu.memory_mut().load_roms(basic, kernal, charrom)
    }

    /// Check if ROMs have been loaded.
    pub fn roms_loaded(&self) -> bool {
        // Note: We need mutable access because lib6502 doesn't have memory() getter
        // This is a workaround - the underlying data doesn't change
        false // Will be properly implemented when we have immutable memory access
    }

    /// Check if ROMs have been loaded (mutable version).
    pub fn roms_loaded_mut(&mut self) -> bool {
        self.cpu.memory_mut().roms_loaded()
    }

    /// Get the current region.
    pub fn region(&self) -> Region {
        self.region
    }

    /// Set the region (PAL/NTSC).
    pub fn set_region(&mut self, region: Region) {
        self.region = region;
        // Update SID sample rate for new clock speed
        self.cpu
            .memory_mut()
            .sid
            .set_sample_rate(44100, region.clock_hz());
    }

    /// Reset the C64 to power-on state.
    pub fn reset(&mut self) {
        self.cpu.memory_mut().reset();
        // Reset IEC bus (but keep mounted disk)
        self.iec_bus.reset();
        // Reinitialize CPU by reading reset vector
        // lib6502 CPU doesn't have a reset() method, so we manually reset state
        let pc_low = self.cpu.memory_mut().read(0xFFFC) as u16;
        let pc_high = self.cpu.memory_mut().read(0xFFFD) as u16;
        let pc = (pc_high << 8) | pc_low;
        self.cpu.set_pc(pc);
        self.cpu.set_sp(0xFD);
        self.cpu.set_a(0);
        self.cpu.set_x(0);
        self.cpu.set_y(0);
        self.cpu.set_flag_i(true);
        self.cpu.set_flag_n(false);
        self.cpu.set_flag_v(false);
        self.cpu.set_flag_b(false);
        self.cpu.set_flag_d(false);
        self.cpu.set_flag_z(false);
        self.cpu.set_flag_c(false);
        self.current_scanline = 0;
        self.cycle_in_scanline = 0;
        self.running = true;
    }

    /// Hard reset (cold boot) - clears all memory.
    pub fn hard_reset(&mut self) {
        // Reset preserves ROMs
        self.reset();
    }

    /// Render one scanline of the display.
    ///
    /// This extracts the necessary memory regions (screen RAM, color RAM,
    /// character ROM/bitmap data) and calls the VIC-II scanline renderer.
    fn render_scanline(&mut self, scanline: u16) {
        let mem = self.cpu.memory_mut();

        // Get VIC-II memory pointers from register $D018
        // Bits 4-7: Screen memory base address (× $0400)
        // Bits 1-3: Character/Bitmap memory base address (× $0800 for char, bit 3 for bitmap)
        let mem_pointers = mem.vic.read(0x18);

        // Calculate screen RAM base address within VIC bank
        // Default: $0400 (screen RAM at $0400-$07E7)
        let screen_offset = ((mem_pointers >> 4) & 0x0F) as u16 * 0x0400;

        // Check if we're in bitmap mode (BMM bit in $D011)
        let is_bitmap_mode = mem.vic.bitmap_mode();

        // Get VIC bank (0-3) from CIA2 port A
        let vic_bank = mem.vic_bank();
        let bank_base = (vic_bank as u16) << 14; // 0, $4000, $8000, $C000

        // Build screen RAM slice (1000 bytes for 40x25 screen)
        // VIC-II reads from its own address space (with bank offset)
        let mut screen_ram = [0u8; 1000];
        for (i, byte) in screen_ram.iter_mut().enumerate() {
            let addr = screen_offset + i as u16;
            *byte = mem.vic_read(addr);
        }

        // Get color RAM directly (always at $D800, not banked)
        let mut color_ram = [0u8; 1000];
        for (i, byte) in color_ram.iter_mut().enumerate() {
            *byte = mem.color_ram.read(i as u16) & 0x0F;
        }

        if is_bitmap_mode {
            // Bitmap mode: fetch 8000 bytes of bitmap data
            // In bitmap mode, bit 3 of $D018 selects base address:
            // - Bit 3 = 0: $0000 within VIC bank
            // - Bit 3 = 1: $2000 within VIC bank
            let bitmap_offset = if (mem_pointers & 0x08) != 0 {
                0x2000u16
            } else {
                0x0000u16
            };

            let mut bitmap_data = [0u8; 8000];
            for (i, byte) in bitmap_data.iter_mut().enumerate() {
                let addr = bitmap_offset + i as u16;
                *byte = mem.vic_read(addr);
            }

            // Call VIC-II scanline renderer with bitmap data
            mem.vic
                .step_scanline(scanline, &bitmap_data, &screen_ram, &color_ram);
        } else {
            // Text mode: fetch character ROM/RAM (2048 bytes)
            // Bits 1-3 of $D018: character base × $0800
            // Default: $1000 (character ROM at $1000-$1FFF in bank 0)
            let char_offset = ((mem_pointers >> 1) & 0x07) as u16 * 0x0800;

            let mut char_data = [0u8; 2048];
            for (i, byte) in char_data.iter_mut().enumerate() {
                let addr = char_offset + i as u16;
                *byte = mem.vic_read(addr);
            }

            // Call VIC-II scanline renderer with character data
            mem.vic
                .step_scanline(scanline, &char_data, &screen_ram, &color_ram);
        }

        // Suppress unused warning for bank_base (will be used when VIC bank selection is refined)
        let _ = bank_base;

        // Render sprites on top of the background (T072)
        // Sprites are rendered after background so they appear on top
        // (T075 will add proper priority handling)
        let sprite_data = self.fetch_sprite_data_for_rendering(&screen_ram);
        self.cpu
            .memory_mut()
            .vic
            .render_sprites_scanline(scanline, &sprite_data);
    }

    /// Fetch sprite data for rendering the current scanline.
    ///
    /// This method fetches the 63-byte data block for each enabled sprite
    /// from VIC memory. The data is used by `render_sprites_scanline`.
    fn fetch_sprite_data_for_rendering(
        &mut self,
        screen_ram: &[u8],
    ) -> [[u8; SPRITE_DATA_SIZE]; SPRITE_COUNT] {
        let mem = self.cpu.memory_mut();

        // Check which sprites are enabled
        let enabled = mem.vic.sprite_enable_bits();
        let mut sprite_data = [[0u8; SPRITE_DATA_SIZE]; SPRITE_COUNT];

        // Only fetch data for enabled sprites
        for (sprite_num, data) in sprite_data.iter_mut().enumerate() {
            if enabled & (1 << sprite_num) != 0 {
                // Get sprite pointer from screen RAM + $3F8
                let pointer = mem.vic.get_sprite_pointer(screen_ram, sprite_num);

                // Fetch 63 bytes of sprite data
                // Sprite data address = pointer * 64
                let base_addr = (pointer as u16) * 64;
                for (i, byte) in data.iter_mut().enumerate() {
                    *byte = mem.vic_read(base_addr + i as u16);
                }
            }
        }

        sprite_data
    }

    /// Execute one full frame of emulation.
    ///
    /// This is the main emulation loop entry point. Call this at the frame rate
    /// (50 Hz PAL, 60 Hz NTSC) for real-time emulation.
    ///
    /// Returns the number of CPU cycles executed.
    pub fn step_frame(&mut self) -> u32 {
        if !self.running {
            return 0;
        }

        let cycles_per_frame = self.region.cycles_per_frame();
        let mut cycles_remaining = cycles_per_frame as i32;
        let mut total_cycles = 0u32;

        while cycles_remaining > 0 {
            // Get cycle count from opcode table before executing
            let pc = self.cpu.pc();
            let opcode = self.cpu.memory_mut().read(pc);
            let metadata = &OPCODE_TABLE[opcode as usize];
            let base_cycles = metadata.base_cycles as i32;

            // Execute one CPU instruction
            let cycles = match self.cpu.step() {
                Ok(()) => base_cycles,
                Err(_) => {
                    // Unimplemented opcode - use base cycles anyway
                    base_cycles
                }
            };

            cycles_remaining -= cycles;
            total_cycles += cycles as u32;

            // Clock CIA timers
            for _ in 0..cycles {
                self.cpu.memory_mut().cia1.clock();
                self.cpu.memory_mut().cia2.clock();
                self.cpu.memory_mut().sid.clock();
            }

            // Update scanline tracking
            self.cycle_in_scanline += cycles as u16;
            while self.cycle_in_scanline >= self.region.cycles_per_line() {
                self.cycle_in_scanline -= self.region.cycles_per_line();

                // Render the current scanline before advancing
                self.render_scanline(self.current_scanline);

                self.current_scanline += 1;

                // Check for raster interrupt
                self.cpu.memory_mut().vic.check_raster_irq();

                if self.current_scanline >= self.region.scanlines() {
                    self.current_scanline = 0;
                }

                // Update VIC-II raster position
                self.cpu
                    .memory_mut()
                    .vic
                    .advance_scanline(self.region.scanlines());
            }
        }

        self.frame_count += 1;
        total_cycles
    }

    /// Get the frame count since reset.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Check if the emulator is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Pause emulation.
    pub fn pause(&mut self) {
        self.running = false;
    }

    /// Resume emulation.
    pub fn resume(&mut self) {
        self.running = true;
    }

    /// Get a reference to the framebuffer for display.
    ///
    /// Note: This requires mutable access due to lib6502 API limitations.
    pub fn framebuffer(&mut self) -> &[[u8; 320]; 200] {
        self.cpu.memory_mut().vic.framebuffer()
    }

    /// Get audio samples from the SID and clear the buffer.
    pub fn take_audio_samples(&mut self) -> Vec<f32> {
        self.cpu.memory_mut().sid.take_samples()
    }

    /// Handle a key press on the C64 keyboard matrix.
    ///
    /// Row and col are 0-7 corresponding to the C64 8x8 keyboard matrix.
    /// See `keyboard::keys` module for key constants.
    ///
    /// # Example
    /// ```ignore
    /// use c64_emu::system::keys;
    ///
    /// // Press the 'A' key
    /// c64.key_down(keys::A.0, keys::A.1);
    ///
    /// // Or directly with coordinates
    /// c64.key_down(1, 2); // 'A' is at row 1, col 2
    /// ```
    pub fn key_down(&mut self, row: u8, col: u8) {
        if row < 8 && col < 8 {
            self.cpu.memory_mut().keyboard.key_down(row, col);
        }
    }

    /// Handle a key release on the C64 keyboard matrix.
    ///
    /// Row and col are 0-7 corresponding to the C64 8x8 keyboard matrix.
    /// See `keyboard::keys` module for key constants.
    pub fn key_up(&mut self, row: u8, col: u8) {
        if row < 8 && col < 8 {
            self.cpu.memory_mut().keyboard.key_up(row, col);
        }
    }

    /// Release all keys on the keyboard.
    ///
    /// Useful when the browser tab loses focus or before loading a new program.
    pub fn release_all_keys(&mut self) {
        self.cpu.memory_mut().keyboard.release_all();
    }

    /// Check if a specific key is pressed.
    pub fn is_key_pressed(&mut self, row: u8, col: u8) -> bool {
        self.cpu.memory_mut().keyboard.is_key_pressed(row, col)
    }

    /// Set joystick state for a logical port.
    ///
    /// This is the main API for joystick input. It respects port swapping,
    /// so if ports are swapped, port 2 input goes to physical port 1.
    ///
    /// Port 1 or 2, state is a bitmask (active-high):
    /// - Bit 0: Up
    /// - Bit 1: Down
    /// - Bit 2: Left
    /// - Bit 3: Right
    /// - Bit 4: Fire
    ///
    /// Most C64 games use port 2 because port 1 interferes with keyboard scanning.
    pub fn set_joystick(&mut self, port: u8, state: u8) {
        self.joystick_ports.set_port(port, state);
        // Update CIA1 with the physical port states
        self.sync_joystick_to_cia();
    }

    /// Sync joystick state to CIA1 ports.
    ///
    /// This must be called after modifying joystick state to update the CIA.
    fn sync_joystick_to_cia(&mut self) {
        // Physical port 1 → CIA1 port B
        // Physical port 2 → CIA1 port A
        let port1_state = self.joystick_ports.physical_port1().get();
        let port2_state = self.joystick_ports.physical_port2().get();

        self.cpu.memory_mut().cia1.set_joystick_port_b(port1_state);
        self.cpu.memory_mut().cia1.set_joystick_port_a(port2_state);
    }

    /// Check if joystick ports are swapped.
    pub fn joystick_ports_swapped(&self) -> bool {
        self.joystick_ports.is_swapped()
    }

    /// Set joystick port swap state.
    ///
    /// When swapped, port 2 input maps to physical port 1 and vice versa.
    /// This is useful for games that use port 1 instead of port 2.
    pub fn set_joystick_swap(&mut self, swapped: bool) {
        self.joystick_ports.set_swapped(swapped);
        // Re-sync to CIA after swap state change
        self.sync_joystick_to_cia();
    }

    /// Toggle joystick port swap.
    pub fn toggle_joystick_swap(&mut self) {
        self.joystick_ports.toggle_swap();
        self.sync_joystick_to_cia();
    }

    /// Release all joystick buttons on both ports.
    pub fn release_all_joysticks(&mut self) {
        self.joystick_ports.release_all();
        self.sync_joystick_to_cia();
    }

    /// Trigger RESTORE key (NMI).
    ///
    /// The RESTORE key on a real C64 is connected directly to the NMI line
    /// (it doesn't go through CIA2 like other keys). Pressing RESTORE pulls
    /// the NMI line low, which triggers a non-maskable interrupt.
    pub fn restore_key(&mut self) {
        self.cpu.trigger_nmi();
    }

    /// Get reference to CPU (for debugging).
    pub fn cpu(&self) -> &CPU<C64Memory> {
        &self.cpu
    }

    /// Get mutable reference to CPU (for debugging).
    pub fn cpu_mut(&mut self) -> &mut CPU<C64Memory> {
        &mut self.cpu
    }

    /// Get mutable reference to memory (for debugging).
    ///
    /// Note: lib6502 doesn't expose an immutable memory() getter,
    /// so we only provide the mutable version.
    pub fn memory_mut(&mut self) -> &mut C64Memory {
        self.cpu.memory_mut()
    }

    /// Load KERNAL ROM (8192 bytes).
    pub fn load_kernal(&mut self, data: &[u8]) -> bool {
        if data.len() != 8192 {
            return false;
        }
        self.cpu.memory_mut().load_kernal(data);
        true
    }

    /// Load BASIC ROM (8192 bytes).
    pub fn load_basic(&mut self, data: &[u8]) -> bool {
        if data.len() != 8192 {
            return false;
        }
        self.cpu.memory_mut().load_basic(data);
        true
    }

    /// Load Character ROM (4096 bytes).
    pub fn load_charrom(&mut self, data: &[u8]) -> bool {
        if data.len() != 4096 {
            return false;
        }
        self.cpu.memory_mut().load_charrom(data);
        true
    }

    /// Start the emulator.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the emulator.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get framebuffer as a flat array (mutable version).
    ///
    /// Returns a copy of the framebuffer as a flat Vec<u8> with 320×200 pixels.
    /// Each pixel is an indexed color value (0-15).
    pub fn get_framebuffer_flat(&mut self) -> Vec<u8> {
        let fb = self.cpu.memory_mut().vic.framebuffer();
        let mut result = Vec::with_capacity(320 * 200);
        for row in fb.iter() {
            result.extend_from_slice(row);
        }
        result
    }

    /// Get audio samples from SID.
    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.cpu.memory_mut().sid.take_samples()
    }

    /// Get a raw pointer to the VIC-II framebuffer.
    ///
    /// This is useful for WASM bindings where JavaScript can directly access
    /// the framebuffer memory without copying. The framebuffer is a contiguous
    /// 320×200 array of indexed colors (0-15).
    ///
    /// # Safety
    /// The returned pointer is valid as long as the C64System instance exists.
    pub fn get_framebuffer_ptr(&mut self) -> *const u8 {
        self.cpu.memory_mut().vic.framebuffer_ptr()
    }

    /// Get the current border color (0-15).
    pub fn get_border_color(&mut self) -> u8 {
        self.cpu.memory_mut().vic.border_color()
    }

    /// Get the current VIC-II raster line.
    pub fn get_current_raster(&mut self) -> u16 {
        self.cpu.memory_mut().vic.raster()
    }

    /// Set joystick 1 state (physical port 1, CIA1 port B).
    ///
    /// This bypasses port swap logic and sets the physical port directly.
    /// Use `set_joystick(1, state)` if you want port swap to be respected.
    pub fn set_joystick1(&mut self, state: u8) {
        self.joystick_ports.physical_port1_mut().set(state);
        self.cpu.memory_mut().cia1.set_joystick_port_b(state);
    }

    /// Set joystick 2 state (physical port 2, CIA1 port A).
    ///
    /// This bypasses port swap logic and sets the physical port directly.
    /// Use `set_joystick(2, state)` if you want port swap to be respected.
    pub fn set_joystick2(&mut self, state: u8) {
        self.joystick_ports.physical_port2_mut().set(state);
        self.cpu.memory_mut().cia1.set_joystick_port_a(state);
    }

    /// Load a PRG file into memory.
    pub fn load_prg(&mut self, data: &[u8]) -> Option<u16> {
        if data.len() < 3 {
            return None;
        }
        let load_addr = (data[0] as u16) | ((data[1] as u16) << 8);
        let mem = self.cpu.memory_mut();
        for (i, &byte) in data[2..].iter().enumerate() {
            let addr = load_addr.wrapping_add(i as u16);
            mem.write(addr, byte);
        }
        Some(load_addr)
    }

    /// Read a byte from memory (for debugging).
    pub fn peek(&mut self, addr: u16) -> u8 {
        self.cpu.memory_mut().read(addr)
    }

    /// Write a byte to memory (for debugging).
    pub fn poke(&mut self, addr: u16, value: u8) {
        self.cpu.memory_mut().write(addr, value);
    }

    /// Get the current program counter.
    pub fn pc(&self) -> u16 {
        self.cpu.pc()
    }

    // =========================================================================
    // Disk Drive API (T057-T059)
    // =========================================================================

    /// Get a reference to the IEC bus.
    pub fn iec_bus(&self) -> &IecBus {
        &self.iec_bus
    }

    /// Get a mutable reference to the IEC bus.
    pub fn iec_bus_mut(&mut self) -> &mut IecBus {
        &mut self.iec_bus
    }

    /// Mount a D64 disk image in drive 8.
    ///
    /// # Arguments
    /// * `data` - Raw D64 file data (174,848 or 175,531 bytes)
    ///
    /// # Returns
    /// `Ok(())` if mounted successfully, `Err` with error message otherwise.
    pub fn mount_d64(&mut self, data: Vec<u8>) -> Result<(), String> {
        self.iec_bus
            .drive_mut()
            .mount(data)
            .map_err(|e| e.to_string())
    }

    /// Unmount the current disk image.
    pub fn unmount_d64(&mut self) {
        self.iec_bus.drive_mut().unmount();
    }

    /// Check if a disk is mounted.
    pub fn has_mounted_disk(&self) -> bool {
        self.iec_bus.has_disk()
    }

    /// Get the disk name (if a disk is mounted).
    pub fn disk_name(&self) -> Option<String> {
        self.iec_bus
            .drive()
            .image()
            .and_then(|img| img.disk_name().ok())
    }

    /// Inject "RUN" command into the keyboard buffer.
    ///
    /// This simulates typing "RUN" followed by RETURN, which is useful
    /// after loading a BASIC program to auto-execute it.
    pub fn inject_basic_run(&mut self) {
        // KERNAL keyboard buffer is at $0277-$0280 (10 bytes)
        // Buffer length is at $C6
        let mem = self.cpu.memory_mut();

        // "RUN" + RETURN
        let run_cmd = [0x52, 0x55, 0x4E, 0x0D]; // R, U, N, CR

        for (i, &byte) in run_cmd.iter().enumerate() {
            mem.write(0x0277 + i as u16, byte);
        }

        // Set buffer length
        mem.write(0x00C6, run_cmd.len() as u8);
    }

    /// Inject a string into the keyboard buffer.
    ///
    /// This simulates typing the given string. Maximum 10 characters.
    /// Characters are converted from ASCII to PETSCII.
    pub fn inject_keys(&mut self, text: &str) {
        let mem = self.cpu.memory_mut();
        let bytes: Vec<u8> = text
            .bytes()
            .take(10) // Max 10 characters in buffer
            .map(ascii_to_petscii)
            .collect();

        for (i, &byte) in bytes.iter().enumerate() {
            mem.write(0x0277 + i as u16, byte);
        }

        mem.write(0x00C6, bytes.len() as u8);
    }

    // =========================================================================
    // Audio API (T086-T088)
    // =========================================================================

    /// Set the audio output sample rate.
    ///
    /// This affects the SID's internal resampling to convert from the
    /// C64 clock rate to the desired output rate. Common values are
    /// 44100 (CD quality) or 48000 (professional audio).
    ///
    /// The clock rate is determined by the current region (PAL/NTSC).
    ///
    /// # Arguments
    /// * `sample_rate` - Output sample rate in Hz (typically 44100 or 48000)
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        let clock_rate = self.region.clock_hz();
        self.cpu
            .memory_mut()
            .sid
            .set_sample_rate(sample_rate, clock_rate);
    }

    /// Get the current audio sample rate.
    pub fn sample_rate(&mut self) -> f32 {
        self.cpu.memory_mut().sid.sample_rate()
    }

    /// Enable or disable audio generation.
    ///
    /// When disabled, the SID will not generate audio samples, which
    /// saves CPU cycles when audio is muted. The SID still processes
    /// register writes so that games continue to function correctly.
    ///
    /// # Arguments
    /// * `enabled` - `true` to enable audio generation, `false` to disable
    pub fn set_audio_enabled(&mut self, enabled: bool) {
        self.cpu.memory_mut().sid.set_audio_enabled(enabled);
    }

    /// Check if audio generation is enabled.
    pub fn audio_enabled(&mut self) -> bool {
        self.cpu.memory_mut().sid.audio_enabled()
    }

    // =========================================================================
    // Save State Support (T099-T108)
    // =========================================================================

    /// Get the current scanline.
    pub fn current_scanline(&self) -> u16 {
        self.current_scanline
    }

    /// Get the current cycle within the scanline.
    pub fn cycle_in_scanline(&self) -> u16 {
        self.cycle_in_scanline
    }

    /// Set scanline state for save state restoration.
    pub fn set_scanline_state(&mut self, scanline: u16, cycle: u16) {
        self.current_scanline = scanline;
        self.cycle_in_scanline = cycle;
    }

    /// Set frame count for save state restoration.
    pub fn set_frame_count(&mut self, count: u64) {
        self.frame_count = count;
    }

    /// Capture joystick state for saving.
    ///
    /// Returns (port1_state, port2_state, swapped).
    pub fn capture_joystick_state(&self) -> (u8, u8, bool) {
        (
            self.joystick_ports.physical_port1().get(),
            self.joystick_ports.physical_port2().get(),
            self.joystick_ports.is_swapped(),
        )
    }

    /// Restore joystick state from a save state.
    pub fn restore_joystick_state(&mut self, port1: u8, port2: u8, swapped: bool) {
        self.joystick_ports.set_swapped(swapped);
        self.joystick_ports.physical_port1_mut().set(port1);
        self.joystick_ports.physical_port2_mut().set(port2);
        self.sync_joystick_to_cia();
    }

    // =========================================================================
    // Debug API (T123-T127)
    // =========================================================================

    /// Read a byte from memory (same as peek, for API consistency).
    pub fn read_memory(&mut self, address: u16) -> u8 {
        self.cpu.memory_mut().read(address)
    }

    /// Write a byte to memory (same as poke, for API consistency).
    pub fn write_memory(&mut self, address: u16, value: u8) {
        self.cpu.memory_mut().write(address, value);
    }

    /// Read a byte directly from RAM (ignoring ROMs and I/O).
    pub fn read_ram(&mut self, address: u16) -> u8 {
        self.cpu.memory_mut().read_ram(address)
    }

    /// Get a 256-byte memory page for inspection.
    pub fn get_memory_page(&mut self, page: u8) -> Vec<u8> {
        let start = (page as u16) << 8;
        let mut result = Vec::with_capacity(256);
        for i in 0..256 {
            result.push(self.cpu.memory_mut().read(start + i as u16));
        }
        result
    }

    /// Get CPU state for debugging.
    ///
    /// Returns: (a, x, y, sp, pc, status_flags, cycles)
    pub fn get_cpu_state(&self) -> (u8, u8, u8, u8, u16, u8, u64) {
        let cpu = &self.cpu;
        // Reconstruct status register from individual flags
        let flags: u8 = (if cpu.flag_n() { 0x80 } else { 0 })
            | (if cpu.flag_v() { 0x40 } else { 0 })
            | 0x20 // Unused bit is always 1
            | (if cpu.flag_b() { 0x10 } else { 0 })
            | (if cpu.flag_d() { 0x08 } else { 0 })
            | (if cpu.flag_i() { 0x04 } else { 0 })
            | (if cpu.flag_z() { 0x02 } else { 0 })
            | (if cpu.flag_c() { 0x01 } else { 0 });

        (
            cpu.a(),
            cpu.x(),
            cpu.y(),
            cpu.sp(),
            cpu.pc(),
            flags,
            cpu.cycles(),
        )
    }

    /// Get all VIC-II registers (47 bytes).
    pub fn get_vic_registers(&mut self) -> Vec<u8> {
        self.cpu.memory_mut().vic.get_all_registers().to_vec()
    }

    /// Get all SID registers (29 bytes).
    pub fn get_sid_registers(&mut self) -> Vec<u8> {
        self.cpu.memory_mut().sid.get_all_registers().to_vec()
    }

    /// Get all CIA1 registers (16 bytes).
    pub fn get_cia1_registers(&mut self) -> Vec<u8> {
        self.cpu.memory_mut().cia1.get_all_registers().to_vec()
    }

    /// Get all CIA2 registers (16 bytes).
    pub fn get_cia2_registers(&mut self) -> Vec<u8> {
        self.cpu.memory_mut().cia2.get_all_registers().to_vec()
    }

    /// Get memory bank configuration.
    ///
    /// Returns: (loram, hiram, charen, vic_bank)
    /// - loram: BASIC ROM visible
    /// - hiram: KERNAL ROM visible
    /// - charen: true = I/O visible, false = CHAR ROM visible
    /// - vic_bank: VIC-II bank 0-3
    pub fn get_bank_config(&mut self) -> (bool, bool, bool, u8) {
        let mem = self.cpu.memory_mut();
        let port_value = mem.port.read(0x01);
        let loram = port_value & 0x01 != 0;
        let hiram = port_value & 0x02 != 0;
        let charen = port_value & 0x04 != 0;
        let vic_bank = mem.vic_bank();

        (loram, hiram, charen, vic_bank)
    }
}

/// Convert ASCII character to PETSCII.
fn ascii_to_petscii(c: u8) -> u8 {
    match c {
        // Lowercase letters → uppercase PETSCII
        b'a'..=b'z' => c - 32,
        // Uppercase letters stay the same
        b'A'..=b'Z' => c,
        // CR/LF → CR
        b'\n' | b'\r' => 0x0D,
        // Most other ASCII characters are the same
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_values() {
        assert_eq!(Region::PAL.clock_hz(), 985_248);
        assert_eq!(Region::NTSC.clock_hz(), 1_022_727);
        assert_eq!(Region::PAL.scanlines(), 312);
        assert_eq!(Region::NTSC.scanlines(), 263);
        assert_eq!(Region::PAL.cycles_per_line(), 63);
        assert_eq!(Region::NTSC.cycles_per_line(), 65);
    }

    #[test]
    fn test_cycles_per_frame() {
        assert_eq!(Region::PAL.cycles_per_frame(), 312 * 63);
        assert_eq!(Region::NTSC.cycles_per_frame(), 263 * 65);
    }

    #[test]
    fn test_new_system() {
        let mut c64 = C64System::new(Region::PAL);
        assert_eq!(c64.region(), Region::PAL);
        assert!(!c64.roms_loaded_mut());
        assert!(!c64.is_running());
    }

    #[test]
    fn test_reset() {
        let mut c64 = C64System::new(Region::PAL);
        c64.reset();
        assert!(c64.is_running());
        assert_eq!(c64.frame_count(), 0);
    }

    #[test]
    fn test_pause_resume() {
        let mut c64 = C64System::new(Region::PAL);
        c64.reset();
        assert!(c64.is_running());

        c64.pause();
        assert!(!c64.is_running());

        c64.resume();
        assert!(c64.is_running());
    }

    #[test]
    fn test_iec_bus_integration() {
        let c64 = C64System::new(Region::PAL);
        assert!(!c64.has_mounted_disk());

        // IEC bus should be initialized with device 8
        assert_eq!(c64.iec_bus().drive().device_number(), 8);
    }

    #[test]
    fn test_inject_keys() {
        let mut c64 = C64System::new(Region::PAL);

        c64.inject_keys("TEST");

        // Check keyboard buffer
        assert_eq!(c64.peek(0x0277), b'T');
        assert_eq!(c64.peek(0x0278), b'E');
        assert_eq!(c64.peek(0x0279), b'S');
        assert_eq!(c64.peek(0x027A), b'T');
        assert_eq!(c64.peek(0x00C6), 4); // Buffer length
    }

    #[test]
    fn test_inject_basic_run() {
        let mut c64 = C64System::new(Region::PAL);

        c64.inject_basic_run();

        // Check for "RUN" + CR in buffer
        assert_eq!(c64.peek(0x0277), 0x52); // R
        assert_eq!(c64.peek(0x0278), 0x55); // U
        assert_eq!(c64.peek(0x0279), 0x4E); // N
        assert_eq!(c64.peek(0x027A), 0x0D); // CR
        assert_eq!(c64.peek(0x00C6), 4); // Buffer length
    }

    #[test]
    fn test_ascii_to_petscii() {
        assert_eq!(ascii_to_petscii(b'A'), b'A');
        assert_eq!(ascii_to_petscii(b'Z'), b'Z');
        assert_eq!(ascii_to_petscii(b'a'), b'A');
        assert_eq!(ascii_to_petscii(b'z'), b'Z');
        assert_eq!(ascii_to_petscii(b'\n'), 0x0D);
        assert_eq!(ascii_to_petscii(b'1'), b'1');
    }
}
