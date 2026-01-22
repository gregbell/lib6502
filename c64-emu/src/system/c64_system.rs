//! C64 system orchestration and timing.
//!
//! This module provides the top-level `C64System` struct that coordinates
//! CPU execution, VIC-II rendering, SID audio, and CIA timing.

use super::C64Memory;
use lib6502::{CPU, MemoryBus, OPCODE_TABLE};

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
            Region::PAL => 50.125, // Exact: 985248 / (312 * 63)
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
                self.current_scanline += 1;

                // Check for raster interrupt
                self.cpu.memory_mut().vic.check_raster_irq();

                if self.current_scanline >= self.region.scanlines() {
                    self.current_scanline = 0;
                }

                // Update VIC-II raster position
                self.cpu.memory_mut().vic.advance_scanline(self.region.scanlines());
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
    /// row and col are 0-7 corresponding to the C64 8x8 keyboard matrix.
    pub fn key_down(&mut self, _row: u8, _col: u8) {
        // TODO: Implement keyboard matrix handling (T034-T037)
    }

    /// Handle a key release on the C64 keyboard matrix.
    ///
    /// row and col are 0-7 corresponding to the C64 8x8 keyboard matrix.
    pub fn key_up(&mut self, _row: u8, _col: u8) {
        // TODO: Implement keyboard matrix handling (T034-T037)
    }

    /// Set joystick state for a port.
    ///
    /// Port 1 or 2, state is a bitmask:
    /// - Bit 0: Up
    /// - Bit 1: Down
    /// - Bit 2: Left
    /// - Bit 3: Right
    /// - Bit 4: Fire
    pub fn set_joystick(&mut self, _port: u8, _state: u8) {
        // TODO: Implement joystick handling (T092-T094)
    }

    /// Trigger RESTORE key (NMI).
    pub fn restore_key(&mut self) {
        // TODO: Implement NMI triggering (T037)
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

    /// Get framebuffer as a flat array.
    pub fn get_framebuffer(&self) -> Vec<u8> {
        // Note: We need to access the VIC without mutable borrow
        // For now, return empty - will be properly implemented when rendering is added
        vec![0u8; 320 * 200]
    }

    /// Get audio samples from SID.
    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.cpu.memory_mut().sid.take_samples()
    }

    /// Set joystick 1 state.
    pub fn set_joystick1(&mut self, state: u8) {
        self.cpu.memory_mut().cia1.set_joystick_port_b(state);
    }

    /// Set joystick 2 state.
    pub fn set_joystick2(&mut self, state: u8) {
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
}
