//! Save state serialization for complete emulator state capture.
//!
//! This module provides the `SaveState` struct and serialization routines
//! to capture and restore the complete emulator state, enabling save/load
//! functionality for games and programs.
//!
//! ## State Components
//!
//! A complete save state includes:
//! - CPU registers (A, X, Y, SP, PC, flags, cycles)
//! - Full 64KB RAM contents
//! - 6510 I/O port state
//! - VIC-II registers and internal state
//! - SID registers and voice states
//! - CIA1 and CIA2 complete state
//! - Color RAM (1KB)
//! - Keyboard matrix state
//! - Joystick port state
//!
//! ## Binary Format
//!
//! Save states use a simple binary format with version header:
//! - 4 bytes: Magic number "C64S"
//! - 4 bytes: Version (u32 little-endian)
//! - 8 bytes: Timestamp (u64 little-endian, Unix epoch)
//! - Remaining: Serialized state data
//!
//! ## Usage
//!
//! ```rust,ignore
//! use c64_emu::system::SaveState;
//!
//! // Save state
//! let state = SaveState::capture(&mut c64_system);
//! let bytes = state.serialize();
//!
//! // ... store bytes to file or localStorage ...
//!
//! // Load state
//! let state = SaveState::deserialize(&bytes)?;
//! state.restore(&mut c64_system);
//! ```

use super::c64_memory::C64Memory;
use super::c64_system::C64System;
use super::keyboard::Keyboard;
use super::Region;
use crate::devices::{Cia6526, EnvelopeState, FilterMode, Sid6581};
use lib6502::Device;

/// Current save state format version.
///
/// Increment this when making breaking changes to the format.
pub const SAVESTATE_VERSION: u32 = 1;

/// Magic number for save state files ("C64S" in ASCII).
pub const SAVESTATE_MAGIC: [u8; 4] = [b'C', b'6', b'4', b'S'];

/// Size of the save state header (magic + version + timestamp).
const HEADER_SIZE: usize = 4 + 4 + 8;

/// Complete emulator save state.
///
/// Contains all state necessary to restore the emulator to an exact
/// previous configuration. Does not include ROM data (assumed to be
/// loaded separately) or mounted disk image contents (only a reference).
#[derive(Debug)]
pub struct SaveState {
    /// Format version for compatibility checking.
    pub version: u32,
    /// Unix timestamp when state was saved.
    pub timestamp: u64,

    // CPU State
    /// CPU accumulator register.
    pub cpu_a: u8,
    /// CPU X index register.
    pub cpu_x: u8,
    /// CPU Y index register.
    pub cpu_y: u8,
    /// CPU stack pointer.
    pub cpu_sp: u8,
    /// CPU program counter.
    pub cpu_pc: u16,
    /// CPU status flags (packed byte: NV-BDIZC).
    pub cpu_flags: u8,
    /// CPU total cycle count.
    pub cpu_cycles: u64,
    /// CPU IRQ pending flag.
    pub cpu_irq_pending: bool,
    /// CPU NMI pending flag.
    pub cpu_nmi_pending: bool,
    /// CPU NMI previous state (for edge detection).
    pub cpu_nmi_prev_state: bool,

    // Memory
    /// Complete 64KB RAM contents.
    pub ram: Box<[u8; 65536]>,
    /// 6510 I/O port DDR register.
    pub port_ddr: u8,
    /// 6510 I/O port data register.
    pub port_data: u8,
    /// 6510 I/O port external input.
    pub port_external: u8,

    // VIC-II State
    /// VIC-II registers (47 bytes).
    pub vic_registers: [u8; 47],
    /// VIC-II current raster line.
    pub vic_raster: u16,
    /// VIC-II cycle within scanline.
    pub vic_cycle_in_line: u8,
    /// VIC-II sprite-sprite collision flags.
    pub vic_collision_ss: u8,
    /// VIC-II sprite-background collision flags.
    pub vic_collision_sb: u8,
    /// VIC-II IRQ pending flag.
    pub vic_irq_pending: bool,

    // SID State
    /// SID voice states (3 voices).
    pub sid_voices: [SidVoiceState; 3],
    /// SID filter state.
    pub sid_filter: SidFilterState,
    /// SID master volume.
    pub sid_volume: u8,
    /// SID sample accumulator (for audio timing).
    pub sid_sample_accumulator: f32,
    /// SID audio enabled flag.
    pub sid_audio_enabled: bool,

    // CIA1 State
    /// CIA1 complete state.
    pub cia1: CiaState,

    // CIA2 State
    /// CIA2 complete state.
    pub cia2: CiaState,

    // Color RAM
    /// Color RAM (1KB, 4-bit values).
    pub color_ram: [u8; 1024],

    // Keyboard
    /// Keyboard matrix state (8x8 = 64 bits).
    pub keyboard_matrix: u64,

    // Joystick
    /// Joystick port 1 state.
    pub joystick1: u8,
    /// Joystick port 2 state.
    pub joystick2: u8,
    /// Joystick ports swapped flag.
    pub joystick_swapped: bool,

    // System
    /// Current region (0 = PAL, 1 = NTSC).
    pub region: u8,
    /// Current scanline.
    pub current_scanline: u16,
    /// Cycle within current scanline.
    pub cycle_in_scanline: u16,
    /// Total frame count.
    pub frame_count: u64,
    /// Emulator running flag.
    pub running: bool,
}

/// Serialized state for a single SID voice.
#[derive(Debug, Clone, Copy)]
pub struct SidVoiceState {
    pub freq: u16,
    pub pulse_width: u16,
    pub control: u8,
    pub attack_decay: u8,
    pub sustain_release: u8,
    pub accumulator: u32,
    pub prev_msb: bool,
    pub prev_bit19: bool,
    pub lfsr: u32,
    pub envelope_state: u8, // 0=Attack, 1=Decay, 2=Sustain, 3=Release
    pub envelope_counter: u8,
    pub rate_counter: u16,
    pub exp_counter: u8,
}

/// Serialized state for SID filter.
#[derive(Debug, Clone, Copy)]
pub struct SidFilterState {
    pub cutoff: u16,
    pub resonance: u8,
    pub routing: u8,
    pub mode_bits: u8,
    pub low: f32,
    pub band: f32,
}

/// Serialized state for a CIA chip.
#[derive(Debug, Clone)]
pub struct CiaState {
    // Ports
    pub port_a_data: u8,
    pub port_a_ddr: u8,
    pub port_b_data: u8,
    pub port_b_ddr: u8,

    // Timer A
    pub timer_a_counter: u16,
    pub timer_a_latch: u16,
    pub timer_a_running: bool,
    pub timer_a_one_shot: bool,
    pub timer_a_underflow: bool,

    // Timer B
    pub timer_b_counter: u16,
    pub timer_b_latch: u16,
    pub timer_b_running: bool,
    pub timer_b_one_shot: bool,
    pub timer_b_underflow: bool,

    // TOD
    pub tod_tenths: u8,
    pub tod_seconds: u8,
    pub tod_minutes: u8,
    pub tod_hours: u8,
    pub tod_alarm_tenths: u8,
    pub tod_alarm_seconds: u8,
    pub tod_alarm_minutes: u8,
    pub tod_alarm_hours: u8,
    pub tod_stopped: bool,
    pub tod_latched: bool,
    pub tod_latch_tenths: u8,
    pub tod_latch_seconds: u8,
    pub tod_latch_minutes: u8,

    // Interrupts
    pub sdr: u8,
    pub interrupt_flags: u8,
    pub interrupt_mask: u8,
    pub interrupt_pending: bool,

    // Control
    pub cra: u8,
    pub crb: u8,

    // External inputs
    pub external_a: u8,
    pub external_b: u8,
}

impl SaveState {
    /// Capture the current emulator state.
    pub fn capture(system: &mut C64System) -> Self {
        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Capture joystick state and system state first (before borrowing CPU)
        let (joystick1, joystick2, joystick_swapped) = system.capture_joystick_state();
        let region = if system.region() == Region::PAL { 0 } else { 1 };
        let current_scanline = system.current_scanline();
        let cycle_in_scanline = system.cycle_in_scanline();
        let frame_count = system.frame_count();
        let running = system.is_running();

        // Capture CPU state (immutable borrow)
        let cpu = system.cpu();
        let cpu_a = cpu.a();
        let cpu_x = cpu.x();
        let cpu_y = cpu.y();
        let cpu_sp = cpu.sp();
        let cpu_pc = cpu.pc();
        let cpu_flags = Self::pack_cpu_flags(cpu);
        let cpu_cycles = cpu.cycles();
        let cpu_irq_pending = cpu.irq_active();
        let cpu_nmi_pending = cpu.nmi_pending();
        let cpu_nmi_prev_state = cpu.nmi_prev_state();
        // Drop the CPU borrow

        // Now get mutable access to capture memory state
        let mem = system.cpu_mut().memory_mut();

        // Capture RAM
        let mut ram = Box::new([0u8; 65536]);
        for (i, byte) in ram.iter_mut().enumerate() {
            *byte = mem.read_ram(i as u16);
        }

        // Capture 6510 port
        let port_ddr = mem.port.ddr();
        let port_data = mem.port.data();

        // Capture VIC-II registers
        let mut vic_registers = [0u8; 47];
        for (i, reg) in vic_registers.iter_mut().enumerate() {
            *reg = mem.vic.read_register(i as u16);
        }
        let vic_raster = mem.vic.raster();
        let vic_collision_ss = mem.vic.collision_ss();
        let vic_collision_sb = mem.vic.collision_sb();
        let vic_irq_pending = mem.vic.irq_pending();

        // Capture SID voice states
        let sid_voices = [
            Self::capture_sid_voice(&mem.sid, 0),
            Self::capture_sid_voice(&mem.sid, 1),
            Self::capture_sid_voice(&mem.sid, 2),
        ];

        // Capture SID filter state
        let sid_filter = Self::capture_sid_filter(&mem.sid);
        let sid_volume = mem.sid.volume();
        let sid_sample_accumulator = mem.sid.sample_accumulator();
        let sid_audio_enabled = mem.sid.audio_enabled();

        // Capture CIA states
        let cia1 = Self::capture_cia(&mem.cia1);
        let cia2 = Self::capture_cia(&mem.cia2);

        // Capture color RAM
        let mut color_ram = [0u8; 1024];
        for (i, byte) in color_ram.iter_mut().enumerate() {
            *byte = mem.color_ram.read(i as u16);
        }

        // Capture keyboard matrix as packed u64
        let keyboard_matrix = Self::pack_keyboard(&mem.keyboard);

        Self {
            version: SAVESTATE_VERSION,
            timestamp,

            // CPU
            cpu_a,
            cpu_x,
            cpu_y,
            cpu_sp,
            cpu_pc,
            cpu_flags,
            cpu_cycles,
            cpu_irq_pending,
            cpu_nmi_pending,
            cpu_nmi_prev_state,

            // Memory
            ram,
            port_ddr,
            port_data,
            port_external: 0, // External is typically 0

            // VIC-II
            vic_registers,
            vic_raster,
            vic_cycle_in_line: 0, // Internal state, reset on load
            vic_collision_ss,
            vic_collision_sb,
            vic_irq_pending,

            // SID
            sid_voices,
            sid_filter,
            sid_volume,
            sid_sample_accumulator,
            sid_audio_enabled,

            // CIA
            cia1,
            cia2,

            // Color RAM
            color_ram,

            // Keyboard
            keyboard_matrix,

            // Joystick
            joystick1,
            joystick2,
            joystick_swapped,

            // System
            region,
            current_scanline,
            cycle_in_scanline,
            frame_count,
            running,
        }
    }

    /// Restore the emulator to this saved state.
    pub fn restore(&self, system: &mut C64System) -> Result<(), String> {
        // Validate version
        if self.version != SAVESTATE_VERSION {
            return Err(format!(
                "Incompatible save state version: expected {}, got {}",
                SAVESTATE_VERSION, self.version
            ));
        }

        // Restore region first as it affects timing
        system.set_region(if self.region == 0 {
            Region::PAL
        } else {
            Region::NTSC
        });

        // Restore CPU state
        let cpu = system.cpu_mut();
        cpu.set_a(self.cpu_a);
        cpu.set_x(self.cpu_x);
        cpu.set_y(self.cpu_y);
        cpu.set_sp(self.cpu_sp);
        cpu.set_pc(self.cpu_pc);
        Self::unpack_cpu_flags(cpu, self.cpu_flags);
        cpu.set_cycles(self.cpu_cycles);
        cpu.set_irq_pending(self.cpu_irq_pending);
        cpu.set_nmi_pending(self.cpu_nmi_pending);
        cpu.set_nmi_prev_state(self.cpu_nmi_prev_state);

        // Restore RAM
        let mem = system.cpu_mut().memory_mut();
        for (i, &byte) in self.ram.iter().enumerate() {
            mem.write_ram(i as u16, byte);
        }

        // Restore 6510 port
        mem.port.set_ddr(self.port_ddr);
        mem.port.set_data(self.port_data);

        // Restore VIC-II registers
        for (i, &reg) in self.vic_registers.iter().enumerate() {
            mem.vic.write_register(i as u16, reg);
        }
        mem.vic.set_raster(self.vic_raster);
        mem.vic.set_collision_ss(self.vic_collision_ss);
        mem.vic.set_collision_sb(self.vic_collision_sb);
        mem.vic.set_irq_pending(self.vic_irq_pending);

        // Restore SID state
        for (i, voice_state) in self.sid_voices.iter().enumerate() {
            Self::restore_sid_voice(&mut mem.sid, i, voice_state);
        }
        Self::restore_sid_filter(&mut mem.sid, &self.sid_filter);
        mem.sid.set_volume(self.sid_volume);
        mem.sid.set_sample_accumulator(self.sid_sample_accumulator);
        mem.sid.set_audio_enabled(self.sid_audio_enabled);

        // Restore CIA states
        Self::restore_cia(&mut mem.cia1, &self.cia1);
        Self::restore_cia(&mut mem.cia2, &self.cia2);

        // Restore color RAM
        for (i, &byte) in self.color_ram.iter().enumerate() {
            mem.color_ram.write(i as u16, byte);
        }

        // Restore keyboard
        Self::unpack_keyboard(&mut mem.keyboard, self.keyboard_matrix);

        // Restore joystick state
        system.restore_joystick_state(self.joystick1, self.joystick2, self.joystick_swapped);

        // Restore system state
        system.set_scanline_state(self.current_scanline, self.cycle_in_scanline);
        system.set_frame_count(self.frame_count);
        if self.running {
            system.resume();
        } else {
            system.pause();
        }

        Ok(())
    }

    /// Serialize the save state to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::estimated_size());

        // Header
        data.extend_from_slice(&SAVESTATE_MAGIC);
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());

        // CPU state
        data.push(self.cpu_a);
        data.push(self.cpu_x);
        data.push(self.cpu_y);
        data.push(self.cpu_sp);
        data.extend_from_slice(&self.cpu_pc.to_le_bytes());
        data.push(self.cpu_flags);
        data.extend_from_slice(&self.cpu_cycles.to_le_bytes());
        data.push(self.cpu_irq_pending as u8);
        data.push(self.cpu_nmi_pending as u8);
        data.push(self.cpu_nmi_prev_state as u8);

        // Memory
        data.extend_from_slice(&self.ram[..]);
        data.push(self.port_ddr);
        data.push(self.port_data);
        data.push(self.port_external);

        // VIC-II
        data.extend_from_slice(&self.vic_registers);
        data.extend_from_slice(&self.vic_raster.to_le_bytes());
        data.push(self.vic_cycle_in_line);
        data.push(self.vic_collision_ss);
        data.push(self.vic_collision_sb);
        data.push(self.vic_irq_pending as u8);

        // SID voices
        for voice in &self.sid_voices {
            Self::serialize_sid_voice(&mut data, voice);
        }

        // SID filter
        Self::serialize_sid_filter(&mut data, &self.sid_filter);

        // SID global
        data.push(self.sid_volume);
        data.extend_from_slice(&self.sid_sample_accumulator.to_le_bytes());
        data.push(self.sid_audio_enabled as u8);

        // CIAs
        Self::serialize_cia(&mut data, &self.cia1);
        Self::serialize_cia(&mut data, &self.cia2);

        // Color RAM
        data.extend_from_slice(&self.color_ram);

        // Keyboard
        data.extend_from_slice(&self.keyboard_matrix.to_le_bytes());

        // Joystick
        data.push(self.joystick1);
        data.push(self.joystick2);
        data.push(self.joystick_swapped as u8);

        // System
        data.push(self.region);
        data.extend_from_slice(&self.current_scanline.to_le_bytes());
        data.extend_from_slice(&self.cycle_in_scanline.to_le_bytes());
        data.extend_from_slice(&self.frame_count.to_le_bytes());
        data.push(self.running as u8);

        data
    }

    /// Deserialize a save state from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        let mut pos = 0;

        // Check minimum size
        if data.len() < HEADER_SIZE {
            return Err("Save state too small".to_string());
        }

        // Verify magic
        if &data[0..4] != &SAVESTATE_MAGIC {
            return Err("Invalid save state magic number".to_string());
        }
        pos += 4;

        // Read version
        let version = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;

        if version != SAVESTATE_VERSION {
            return Err(format!(
                "Incompatible save state version: expected {}, got {}",
                SAVESTATE_VERSION, version
            ));
        }

        // Read timestamp
        let timestamp = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;

        // CPU state
        let cpu_a = data[pos];
        pos += 1;
        let cpu_x = data[pos];
        pos += 1;
        let cpu_y = data[pos];
        pos += 1;
        let cpu_sp = data[pos];
        pos += 1;
        let cpu_pc = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        let cpu_flags = data[pos];
        pos += 1;
        let cpu_cycles = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;
        let cpu_irq_pending = data[pos] != 0;
        pos += 1;
        let cpu_nmi_pending = data[pos] != 0;
        pos += 1;
        let cpu_nmi_prev_state = data[pos] != 0;
        pos += 1;

        // Check we have enough data for RAM
        if data.len() < pos + 65536 {
            return Err("Save state truncated at RAM".to_string());
        }

        // Memory
        let mut ram = Box::new([0u8; 65536]);
        ram.copy_from_slice(&data[pos..pos + 65536]);
        pos += 65536;

        let port_ddr = data[pos];
        pos += 1;
        let port_data = data[pos];
        pos += 1;
        let port_external = data[pos];
        pos += 1;

        // VIC-II
        let mut vic_registers = [0u8; 47];
        vic_registers.copy_from_slice(&data[pos..pos + 47]);
        pos += 47;
        let vic_raster = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        let vic_cycle_in_line = data[pos];
        pos += 1;
        let vic_collision_ss = data[pos];
        pos += 1;
        let vic_collision_sb = data[pos];
        pos += 1;
        let vic_irq_pending = data[pos] != 0;
        pos += 1;

        // SID voices
        let sid_voices = [
            Self::deserialize_sid_voice(&data, &mut pos)?,
            Self::deserialize_sid_voice(&data, &mut pos)?,
            Self::deserialize_sid_voice(&data, &mut pos)?,
        ];

        // SID filter
        let sid_filter = Self::deserialize_sid_filter(&data, &mut pos)?;

        // SID global
        let sid_volume = data[pos];
        pos += 1;
        let sid_sample_accumulator = f32::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
        ]);
        pos += 4;
        let sid_audio_enabled = data[pos] != 0;
        pos += 1;

        // CIAs
        let cia1 = Self::deserialize_cia(&data, &mut pos)?;
        let cia2 = Self::deserialize_cia(&data, &mut pos)?;

        // Color RAM
        let mut color_ram = [0u8; 1024];
        color_ram.copy_from_slice(&data[pos..pos + 1024]);
        pos += 1024;

        // Keyboard
        let keyboard_matrix = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;

        // Joystick
        let joystick1 = data[pos];
        pos += 1;
        let joystick2 = data[pos];
        pos += 1;
        let joystick_swapped = data[pos] != 0;
        pos += 1;

        // System
        let region = data[pos];
        pos += 1;
        let current_scanline = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        let cycle_in_scanline = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        let frame_count = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;
        let running = data[pos] != 0;

        Ok(Self {
            version,
            timestamp,
            cpu_a,
            cpu_x,
            cpu_y,
            cpu_sp,
            cpu_pc,
            cpu_flags,
            cpu_cycles,
            cpu_irq_pending,
            cpu_nmi_pending,
            cpu_nmi_prev_state,
            ram,
            port_ddr,
            port_data,
            port_external,
            vic_registers,
            vic_raster,
            vic_cycle_in_line,
            vic_collision_ss,
            vic_collision_sb,
            vic_irq_pending,
            sid_voices,
            sid_filter,
            sid_volume,
            sid_sample_accumulator,
            sid_audio_enabled,
            cia1,
            cia2,
            color_ram,
            keyboard_matrix,
            joystick1,
            joystick2,
            joystick_swapped,
            region,
            current_scanline,
            cycle_in_scanline,
            frame_count,
            running,
        })
    }

    /// Get the approximate serialized size of a save state.
    pub fn estimated_size() -> usize {
        // Header + CPU + RAM + Port + VIC + SID + CIAs + ColorRAM + Keyboard + Joystick + System
        HEADER_SIZE + 20 + 65536 + 3 + 53 + (3 * 24) + 18 + (2 * 48) + 1024 + 8 + 3 + 15
    }

    /// Get the actual serialized size of this save state.
    pub fn serialized_size(&self) -> usize {
        self.serialize().len()
    }

    // Helper methods for CPU flags
    fn pack_cpu_flags(cpu: &lib6502::CPU<C64Memory>) -> u8 {
        let mut flags = 0u8;
        if cpu.flag_n() {
            flags |= 0x80;
        }
        if cpu.flag_v() {
            flags |= 0x40;
        }
        // Bit 5 is always 1
        flags |= 0x20;
        if cpu.flag_b() {
            flags |= 0x10;
        }
        if cpu.flag_d() {
            flags |= 0x08;
        }
        if cpu.flag_i() {
            flags |= 0x04;
        }
        if cpu.flag_z() {
            flags |= 0x02;
        }
        if cpu.flag_c() {
            flags |= 0x01;
        }
        flags
    }

    fn unpack_cpu_flags(cpu: &mut lib6502::CPU<C64Memory>, flags: u8) {
        cpu.set_flag_n(flags & 0x80 != 0);
        cpu.set_flag_v(flags & 0x40 != 0);
        cpu.set_flag_b(flags & 0x10 != 0);
        cpu.set_flag_d(flags & 0x08 != 0);
        cpu.set_flag_i(flags & 0x04 != 0);
        cpu.set_flag_z(flags & 0x02 != 0);
        cpu.set_flag_c(flags & 0x01 != 0);
    }

    // SID voice capture/restore
    fn capture_sid_voice(sid: &Sid6581, index: usize) -> SidVoiceState {
        let voice = sid.voice(index).expect("Valid voice index");
        SidVoiceState {
            freq: voice.freq,
            pulse_width: voice.pulse_width,
            control: voice.control,
            attack_decay: voice.attack_decay,
            sustain_release: voice.sustain_release,
            accumulator: voice.accumulator,
            prev_msb: voice.prev_msb,
            prev_bit19: voice.prev_bit19,
            lfsr: voice.lfsr,
            envelope_state: match voice.envelope_state {
                EnvelopeState::Attack => 0,
                EnvelopeState::Decay => 1,
                EnvelopeState::Sustain => 2,
                EnvelopeState::Release => 3,
            },
            envelope_counter: voice.envelope_counter,
            rate_counter: voice.rate_counter,
            exp_counter: voice.exp_counter,
        }
    }

    fn restore_sid_voice(sid: &mut Sid6581, index: usize, state: &SidVoiceState) {
        let voice = sid.voice_mut(index);
        voice.freq = state.freq;
        voice.pulse_width = state.pulse_width;
        voice.control = state.control;
        voice.attack_decay = state.attack_decay;
        voice.sustain_release = state.sustain_release;
        voice.accumulator = state.accumulator;
        voice.prev_msb = state.prev_msb;
        voice.prev_bit19 = state.prev_bit19;
        voice.lfsr = state.lfsr;
        voice.envelope_state = match state.envelope_state {
            0 => EnvelopeState::Attack,
            1 => EnvelopeState::Decay,
            2 => EnvelopeState::Sustain,
            _ => EnvelopeState::Release,
        };
        voice.envelope_counter = state.envelope_counter;
        voice.rate_counter = state.rate_counter;
        voice.exp_counter = state.exp_counter;
    }

    fn serialize_sid_voice(data: &mut Vec<u8>, voice: &SidVoiceState) {
        data.extend_from_slice(&voice.freq.to_le_bytes());
        data.extend_from_slice(&voice.pulse_width.to_le_bytes());
        data.push(voice.control);
        data.push(voice.attack_decay);
        data.push(voice.sustain_release);
        data.extend_from_slice(&voice.accumulator.to_le_bytes());
        data.push(voice.prev_msb as u8);
        data.push(voice.prev_bit19 as u8);
        data.extend_from_slice(&voice.lfsr.to_le_bytes());
        data.push(voice.envelope_state);
        data.push(voice.envelope_counter);
        data.extend_from_slice(&voice.rate_counter.to_le_bytes());
        data.push(voice.exp_counter);
    }

    fn deserialize_sid_voice(data: &[u8], pos: &mut usize) -> Result<SidVoiceState, String> {
        if data.len() < *pos + 24 {
            return Err("Save state truncated at SID voice".to_string());
        }

        let freq = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let pulse_width = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let control = data[*pos];
        *pos += 1;
        let attack_decay = data[*pos];
        *pos += 1;
        let sustain_release = data[*pos];
        *pos += 1;
        let accumulator = u32::from_le_bytes([
            data[*pos],
            data[*pos + 1],
            data[*pos + 2],
            data[*pos + 3],
        ]);
        *pos += 4;
        let prev_msb = data[*pos] != 0;
        *pos += 1;
        let prev_bit19 = data[*pos] != 0;
        *pos += 1;
        let lfsr = u32::from_le_bytes([
            data[*pos],
            data[*pos + 1],
            data[*pos + 2],
            data[*pos + 3],
        ]);
        *pos += 4;
        let envelope_state = data[*pos];
        *pos += 1;
        let envelope_counter = data[*pos];
        *pos += 1;
        let rate_counter = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let exp_counter = data[*pos];
        *pos += 1;

        Ok(SidVoiceState {
            freq,
            pulse_width,
            control,
            attack_decay,
            sustain_release,
            accumulator,
            prev_msb,
            prev_bit19,
            lfsr,
            envelope_state,
            envelope_counter,
            rate_counter,
            exp_counter,
        })
    }

    // SID filter capture/restore
    fn capture_sid_filter(sid: &Sid6581) -> SidFilterState {
        let filter = sid.filter();
        SidFilterState {
            cutoff: filter.cutoff,
            resonance: filter.resonance,
            routing: filter.routing,
            mode_bits: filter.mode_bits,
            low: filter.low,
            band: filter.band,
        }
    }

    fn restore_sid_filter(sid: &mut Sid6581, state: &SidFilterState) {
        let filter = sid.filter_mut();
        filter.cutoff = state.cutoff;
        filter.resonance = state.resonance;
        filter.routing = state.routing;
        filter.mode_bits = state.mode_bits;
        filter.mode = FilterMode::from_register(state.mode_bits << 4);
        filter.low = state.low;
        filter.band = state.band;
    }

    fn serialize_sid_filter(data: &mut Vec<u8>, filter: &SidFilterState) {
        data.extend_from_slice(&filter.cutoff.to_le_bytes());
        data.push(filter.resonance);
        data.push(filter.routing);
        data.push(filter.mode_bits);
        data.extend_from_slice(&filter.low.to_le_bytes());
        data.extend_from_slice(&filter.band.to_le_bytes());
    }

    fn deserialize_sid_filter(data: &[u8], pos: &mut usize) -> Result<SidFilterState, String> {
        if data.len() < *pos + 13 {
            return Err("Save state truncated at SID filter".to_string());
        }

        let cutoff = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let resonance = data[*pos];
        *pos += 1;
        let routing = data[*pos];
        *pos += 1;
        let mode_bits = data[*pos];
        *pos += 1;
        let low = f32::from_le_bytes([
            data[*pos],
            data[*pos + 1],
            data[*pos + 2],
            data[*pos + 3],
        ]);
        *pos += 4;
        let band = f32::from_le_bytes([
            data[*pos],
            data[*pos + 1],
            data[*pos + 2],
            data[*pos + 3],
        ]);
        *pos += 4;

        Ok(SidFilterState {
            cutoff,
            resonance,
            routing,
            mode_bits,
            low,
            band,
        })
    }

    // CIA capture/restore
    fn capture_cia(cia: &Cia6526) -> CiaState {
        CiaState {
            port_a_data: cia.port_a.data,
            port_a_ddr: cia.port_a.ddr,
            port_b_data: cia.port_b.data,
            port_b_ddr: cia.port_b.ddr,

            timer_a_counter: cia.timer_a.counter,
            timer_a_latch: cia.timer_a.latch,
            timer_a_running: cia.timer_a.running,
            timer_a_one_shot: cia.timer_a.one_shot,
            timer_a_underflow: cia.timer_a.underflow,

            timer_b_counter: cia.timer_b.counter,
            timer_b_latch: cia.timer_b.latch,
            timer_b_running: cia.timer_b.running,
            timer_b_one_shot: cia.timer_b.one_shot,
            timer_b_underflow: cia.timer_b.underflow,

            tod_tenths: cia.tod.tenths,
            tod_seconds: cia.tod.seconds,
            tod_minutes: cia.tod.minutes,
            tod_hours: cia.tod.hours,
            tod_alarm_tenths: cia.tod.alarm_tenths,
            tod_alarm_seconds: cia.tod.alarm_seconds,
            tod_alarm_minutes: cia.tod.alarm_minutes,
            tod_alarm_hours: cia.tod.alarm_hours,
            tod_stopped: cia.tod.stopped,
            tod_latched: cia.tod.latched,
            tod_latch_tenths: cia.tod_latch_tenths(),
            tod_latch_seconds: cia.tod_latch_seconds(),
            tod_latch_minutes: cia.tod_latch_minutes(),

            sdr: cia.sdr,
            interrupt_flags: cia.interrupt_flags(),
            interrupt_mask: cia.interrupt_mask(),
            interrupt_pending: cia.has_interrupt(),

            cra: cia.cra(),
            crb: cia.crb(),

            external_a: cia.external_a,
            external_b: cia.external_b,
        }
    }

    fn restore_cia(cia: &mut Cia6526, state: &CiaState) {
        cia.port_a.data = state.port_a_data;
        cia.port_a.ddr = state.port_a_ddr;
        cia.port_b.data = state.port_b_data;
        cia.port_b.ddr = state.port_b_ddr;

        cia.timer_a.counter = state.timer_a_counter;
        cia.timer_a.latch = state.timer_a_latch;
        cia.timer_a.running = state.timer_a_running;
        cia.timer_a.one_shot = state.timer_a_one_shot;
        cia.timer_a.underflow = state.timer_a_underflow;

        cia.timer_b.counter = state.timer_b_counter;
        cia.timer_b.latch = state.timer_b_latch;
        cia.timer_b.running = state.timer_b_running;
        cia.timer_b.one_shot = state.timer_b_one_shot;
        cia.timer_b.underflow = state.timer_b_underflow;

        cia.tod.tenths = state.tod_tenths;
        cia.tod.seconds = state.tod_seconds;
        cia.tod.minutes = state.tod_minutes;
        cia.tod.hours = state.tod_hours;
        cia.tod.alarm_tenths = state.tod_alarm_tenths;
        cia.tod.alarm_seconds = state.tod_alarm_seconds;
        cia.tod.alarm_minutes = state.tod_alarm_minutes;
        cia.tod.alarm_hours = state.tod_alarm_hours;
        cia.tod.stopped = state.tod_stopped;
        cia.tod.latched = state.tod_latched;
        cia.set_tod_latch(state.tod_latch_tenths, state.tod_latch_seconds, state.tod_latch_minutes);

        cia.sdr = state.sdr;
        cia.set_interrupt_flags(state.interrupt_flags);
        cia.set_interrupt_mask(state.interrupt_mask);
        cia.set_interrupt_pending(state.interrupt_pending);

        cia.set_cra(state.cra);
        cia.set_crb(state.crb);

        cia.external_a = state.external_a;
        cia.external_b = state.external_b;
    }

    fn serialize_cia(data: &mut Vec<u8>, cia: &CiaState) {
        data.push(cia.port_a_data);
        data.push(cia.port_a_ddr);
        data.push(cia.port_b_data);
        data.push(cia.port_b_ddr);

        data.extend_from_slice(&cia.timer_a_counter.to_le_bytes());
        data.extend_from_slice(&cia.timer_a_latch.to_le_bytes());
        data.push(cia.timer_a_running as u8);
        data.push(cia.timer_a_one_shot as u8);
        data.push(cia.timer_a_underflow as u8);

        data.extend_from_slice(&cia.timer_b_counter.to_le_bytes());
        data.extend_from_slice(&cia.timer_b_latch.to_le_bytes());
        data.push(cia.timer_b_running as u8);
        data.push(cia.timer_b_one_shot as u8);
        data.push(cia.timer_b_underflow as u8);

        data.push(cia.tod_tenths);
        data.push(cia.tod_seconds);
        data.push(cia.tod_minutes);
        data.push(cia.tod_hours);
        data.push(cia.tod_alarm_tenths);
        data.push(cia.tod_alarm_seconds);
        data.push(cia.tod_alarm_minutes);
        data.push(cia.tod_alarm_hours);
        data.push(cia.tod_stopped as u8);
        data.push(cia.tod_latched as u8);
        data.push(cia.tod_latch_tenths);
        data.push(cia.tod_latch_seconds);
        data.push(cia.tod_latch_minutes);

        data.push(cia.sdr);
        data.push(cia.interrupt_flags);
        data.push(cia.interrupt_mask);
        data.push(cia.interrupt_pending as u8);

        data.push(cia.cra);
        data.push(cia.crb);

        data.push(cia.external_a);
        data.push(cia.external_b);
    }

    fn deserialize_cia(data: &[u8], pos: &mut usize) -> Result<CiaState, String> {
        if data.len() < *pos + 39 {
            return Err("Save state truncated at CIA".to_string());
        }

        let port_a_data = data[*pos];
        *pos += 1;
        let port_a_ddr = data[*pos];
        *pos += 1;
        let port_b_data = data[*pos];
        *pos += 1;
        let port_b_ddr = data[*pos];
        *pos += 1;

        let timer_a_counter = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let timer_a_latch = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let timer_a_running = data[*pos] != 0;
        *pos += 1;
        let timer_a_one_shot = data[*pos] != 0;
        *pos += 1;
        let timer_a_underflow = data[*pos] != 0;
        *pos += 1;

        let timer_b_counter = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let timer_b_latch = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
        *pos += 2;
        let timer_b_running = data[*pos] != 0;
        *pos += 1;
        let timer_b_one_shot = data[*pos] != 0;
        *pos += 1;
        let timer_b_underflow = data[*pos] != 0;
        *pos += 1;

        let tod_tenths = data[*pos];
        *pos += 1;
        let tod_seconds = data[*pos];
        *pos += 1;
        let tod_minutes = data[*pos];
        *pos += 1;
        let tod_hours = data[*pos];
        *pos += 1;
        let tod_alarm_tenths = data[*pos];
        *pos += 1;
        let tod_alarm_seconds = data[*pos];
        *pos += 1;
        let tod_alarm_minutes = data[*pos];
        *pos += 1;
        let tod_alarm_hours = data[*pos];
        *pos += 1;
        let tod_stopped = data[*pos] != 0;
        *pos += 1;
        let tod_latched = data[*pos] != 0;
        *pos += 1;
        let tod_latch_tenths = data[*pos];
        *pos += 1;
        let tod_latch_seconds = data[*pos];
        *pos += 1;
        let tod_latch_minutes = data[*pos];
        *pos += 1;

        let sdr = data[*pos];
        *pos += 1;
        let interrupt_flags = data[*pos];
        *pos += 1;
        let interrupt_mask = data[*pos];
        *pos += 1;
        let interrupt_pending = data[*pos] != 0;
        *pos += 1;

        let cra = data[*pos];
        *pos += 1;
        let crb = data[*pos];
        *pos += 1;

        let external_a = data[*pos];
        *pos += 1;
        let external_b = data[*pos];
        *pos += 1;

        Ok(CiaState {
            port_a_data,
            port_a_ddr,
            port_b_data,
            port_b_ddr,
            timer_a_counter,
            timer_a_latch,
            timer_a_running,
            timer_a_one_shot,
            timer_a_underflow,
            timer_b_counter,
            timer_b_latch,
            timer_b_running,
            timer_b_one_shot,
            timer_b_underflow,
            tod_tenths,
            tod_seconds,
            tod_minutes,
            tod_hours,
            tod_alarm_tenths,
            tod_alarm_seconds,
            tod_alarm_minutes,
            tod_alarm_hours,
            tod_stopped,
            tod_latched,
            tod_latch_tenths,
            tod_latch_seconds,
            tod_latch_minutes,
            sdr,
            interrupt_flags,
            interrupt_mask,
            interrupt_pending,
            cra,
            crb,
            external_a,
            external_b,
        })
    }

    // Keyboard matrix packing/unpacking
    fn pack_keyboard(keyboard: &Keyboard) -> u64 {
        let mut packed = 0u64;
        for row in 0..8 {
            for col in 0..8 {
                if keyboard.is_key_pressed(row, col) {
                    packed |= 1u64 << (row * 8 + col);
                }
            }
        }
        packed
    }

    fn unpack_keyboard(keyboard: &mut Keyboard, packed: u64) {
        keyboard.release_all();
        for row in 0..8 {
            for col in 0..8 {
                if packed & (1u64 << (row * 8 + col)) != 0 {
                    keyboard.key_down(row, col);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_savestate_serialize_deserialize_roundtrip() {
        // Create a simple test state
        let mut system = C64System::new(Region::PAL);
        system.reset();

        // Set some recognizable values
        system.cpu_mut().set_a(0x42);
        system.cpu_mut().set_x(0xAB);
        system.cpu_mut().set_y(0xCD);
        system.cpu_mut().set_pc(0x1234);

        // Capture state
        let state = SaveState::capture(&mut system);

        // Verify captured values
        assert_eq!(state.cpu_a, 0x42);
        assert_eq!(state.cpu_x, 0xAB);
        assert_eq!(state.cpu_y, 0xCD);
        assert_eq!(state.cpu_pc, 0x1234);
        assert_eq!(state.version, SAVESTATE_VERSION);

        // Serialize
        let bytes = state.serialize();
        assert!(!bytes.is_empty());

        // Deserialize
        let restored = SaveState::deserialize(&bytes).expect("Failed to deserialize");

        // Verify roundtrip
        assert_eq!(restored.version, state.version);
        assert_eq!(restored.cpu_a, state.cpu_a);
        assert_eq!(restored.cpu_x, state.cpu_x);
        assert_eq!(restored.cpu_y, state.cpu_y);
        assert_eq!(restored.cpu_pc, state.cpu_pc);
        assert_eq!(restored.region, state.region);
    }

    #[test]
    fn test_savestate_magic_validation() {
        // Create a buffer with valid size but wrong magic
        let mut bad_magic = vec![0u8; 16]; // Header size: 4 magic + 4 version + 8 timestamp
        let result = SaveState::deserialize(&bad_magic);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("magic"));
    }

    #[test]
    fn test_savestate_version_validation() {
        let mut bad_version = SAVESTATE_MAGIC.to_vec();
        bad_version.extend_from_slice(&999u32.to_le_bytes());
        bad_version.extend_from_slice(&0u64.to_le_bytes());

        let result = SaveState::deserialize(&bad_version);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version"));
    }

    #[test]
    fn test_keyboard_pack_unpack() {
        let mut keyboard = Keyboard::new();
        keyboard.key_down(0, 0);
        keyboard.key_down(3, 5);
        keyboard.key_down(7, 7);

        let packed = SaveState::pack_keyboard(&keyboard);

        let mut restored = Keyboard::new();
        SaveState::unpack_keyboard(&mut restored, packed);

        assert!(restored.is_key_pressed(0, 0));
        assert!(restored.is_key_pressed(3, 5));
        assert!(restored.is_key_pressed(7, 7));
        assert!(!restored.is_key_pressed(1, 1));
    }
}
