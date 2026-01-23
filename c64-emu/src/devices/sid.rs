//! SID (MOS 6581) Sound Interface Device emulation.
//!
//! The SID is the C64's legendary sound chip, featuring:
//! - 3 independent oscillator voices
//! - 4 waveforms per voice (triangle, sawtooth, pulse, noise)
//! - ADSR envelope generator per voice
//! - Programmable multimode filter (LP/BP/HP/Notch)
//!
//! This implementation provides audio suitable for games and most music,
//! using a simplified biquad filter rather than full analog modeling.

use lib6502::Device;
use std::any::Any;

/// SID register count (29 registers at $D400-$D41C).
#[allow(dead_code)]
pub const SID_REGISTER_COUNT: usize = 29;

/// Number of voices in the SID.
pub const VOICE_COUNT: usize = 3;

/// Envelope state machine phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Single SID voice state.
#[derive(Debug, Clone)]
pub struct SidVoice {
    /// 16-bit frequency register.
    pub freq: u16,
    /// 12-bit pulse width.
    pub pulse_width: u16,
    /// Control register (waveform, gate, sync, ring).
    pub control: u8,
    /// Attack/Decay nibbles.
    pub attack_decay: u8,
    /// Sustain/Release nibbles.
    pub sustain_release: u8,

    /// 24-bit phase accumulator.
    pub accumulator: u32,
    /// Previous MSB state for hard sync detection.
    /// Used to detect 0->1 transition on bit 23.
    pub prev_msb: bool,
    /// 23-bit noise LFSR.
    pub lfsr: u32,
    /// Current envelope state.
    pub envelope_state: EnvelopeState,
    /// Current envelope value (0-255).
    pub envelope_counter: u8,
    /// Rate counter for envelope timing.
    pub rate_counter: u16,
    /// Exponential decay counter.
    pub exp_counter: u8,
}

impl SidVoice {
    /// Create a new voice with default state.
    pub fn new() -> Self {
        Self {
            freq: 0,
            pulse_width: 0,
            control: 0,
            attack_decay: 0,
            sustain_release: 0,
            accumulator: 0,
            prev_msb: false,
            lfsr: 0x7FFFF8, // Initial LFSR state
            envelope_state: EnvelopeState::Release,
            envelope_counter: 0,
            rate_counter: 0,
            exp_counter: 0,
        }
    }

    /// Check if gate is on.
    #[inline]
    pub fn gate(&self) -> bool {
        self.control & 0x01 != 0
    }

    /// Get the selected waveform bits.
    #[inline]
    pub fn waveform(&self) -> u8 {
        (self.control >> 4) & 0x0F
    }

    /// Check if test bit is set (resets and holds accumulator).
    #[inline]
    pub fn test_bit(&self) -> bool {
        self.control & 0x08 != 0
    }

    /// Check if hard sync is enabled.
    #[inline]
    pub fn sync_enabled(&self) -> bool {
        self.control & 0x02 != 0
    }

    /// Get the current MSB of the accumulator (bit 23).
    #[inline]
    pub fn accumulator_msb(&self) -> bool {
        self.accumulator & 0x0080_0000 != 0
    }
}

impl Default for SidVoice {
    fn default() -> Self {
        Self::new()
    }
}

/// SID filter state.
#[derive(Debug, Clone)]
pub struct SidFilter {
    /// 11-bit cutoff frequency.
    pub cutoff: u16,
    /// 4-bit resonance.
    pub resonance: u8,
    /// Voice routing through filter (bits 0-2).
    pub routing: u8,
    /// Filter mode (bits 4-6 of $D418).
    pub mode: u8,
    /// Filter state: low-pass accumulator.
    #[allow(dead_code)]
    pub low: f32,
    /// Filter state: band-pass accumulator.
    #[allow(dead_code)]
    pub band: f32,
}

impl SidFilter {
    /// Create a new filter with default state.
    pub fn new() -> Self {
        Self {
            cutoff: 0,
            resonance: 0,
            routing: 0,
            mode: 0,
            low: 0.0,
            band: 0.0,
        }
    }
}

impl Default for SidFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// MOS 6581 Sound Interface Device.
#[derive(Debug)]
pub struct Sid6581 {
    /// The three oscillator voices.
    voices: [SidVoice; VOICE_COUNT],
    /// The multimode filter.
    filter: SidFilter,
    /// Master volume (0-15).
    volume: u8,

    /// Sample buffer for Web Audio consumption.
    sample_buffer: Vec<f32>,
    /// Cycles per audio sample at target sample rate.
    cycles_per_sample: f32,
    /// Accumulated cycles since last sample.
    sample_accumulator: f32,

    /// Audio enabled flag.
    audio_enabled: bool,
}

impl Sid6581 {
    /// Create a new SID with default state.
    ///
    /// Default sample rate is 44100 Hz.
    pub fn new() -> Self {
        Self {
            voices: [SidVoice::new(), SidVoice::new(), SidVoice::new()],
            filter: SidFilter::new(),
            volume: 0,
            sample_buffer: Vec::with_capacity(1024),
            cycles_per_sample: 985248.0 / 44100.0, // PAL default
            sample_accumulator: 0.0,
            audio_enabled: true,
        }
    }

    /// Set the target sample rate for audio output.
    pub fn set_sample_rate(&mut self, sample_rate: u32, clock_rate: u32) {
        self.cycles_per_sample = clock_rate as f32 / sample_rate as f32;
    }

    /// Get and clear the sample buffer.
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.sample_buffer)
    }

    /// Get reference to sample buffer.
    pub fn samples(&self) -> &[f32] {
        &self.sample_buffer
    }

    /// Enable or disable audio output.
    pub fn set_audio_enabled(&mut self, enabled: bool) {
        self.audio_enabled = enabled;
    }

    /// Step the SID by one clock cycle.
    ///
    /// This clocks all three voices, updating their phase accumulators and
    /// envelope generators, then generates audio samples at the configured rate.
    pub fn clock(&mut self) {
        // Clock all voice phase accumulators and handle hard sync
        self.clock_phase_accumulators();

        // TODO: Implement envelope clocking (T081)
        // TODO: Implement waveform generation and mixing (T078-T080)
        // TODO: Implement filter processing (T083-T084)

        // Audio sample generation
        self.sample_accumulator += 1.0;
        if self.sample_accumulator >= self.cycles_per_sample {
            self.sample_accumulator -= self.cycles_per_sample;
            if self.audio_enabled {
                // Generate a sample (currently silent until T078+)
                self.sample_buffer.push(0.0);
            }
        }
    }

    /// Clock all voice phase accumulators with hard sync support.
    ///
    /// The 24-bit phase accumulator is incremented by the 16-bit frequency
    /// register value on each clock cycle. This produces the fundamental
    /// waveform frequency: F = (Fn × Fclk/16777216) Hz
    ///
    /// Where:
    /// - Fn = 16-bit frequency register value
    /// - Fclk = system clock (985248 Hz PAL, 1022727 Hz NTSC)
    ///
    /// Hard sync: When voice N has SYNC bit set, it resets when the MSB of
    /// the sync source voice (N-1, with wraparound) transitions from 0 to 1.
    fn clock_phase_accumulators(&mut self) {
        // Store previous MSB states before updating accumulators
        let prev_msb = [
            self.voices[0].accumulator_msb(),
            self.voices[1].accumulator_msb(),
            self.voices[2].accumulator_msb(),
        ];

        // Update each voice's accumulator
        for voice_idx in 0..VOICE_COUNT {
            let voice = &mut self.voices[voice_idx];

            // Store previous MSB for sync detection
            voice.prev_msb = prev_msb[voice_idx];

            // Handle test bit - when set, resets and holds accumulator at zero
            if voice.test_bit() {
                voice.accumulator = 0;
                continue;
            }

            // Increment phase accumulator by frequency value (wraps at 24 bits)
            voice.accumulator = voice.accumulator.wrapping_add(voice.freq as u32);
            voice.accumulator &= 0x00FF_FFFF; // Mask to 24 bits
        }

        // Apply hard sync after all accumulators have been updated
        // Sync source mapping: Voice 1 <- Voice 3, Voice 2 <- Voice 1, Voice 3 <- Voice 2
        for voice_idx in 0..VOICE_COUNT {
            if self.voices[voice_idx].sync_enabled() {
                // Get sync source voice index (with wraparound)
                let sync_source_idx = if voice_idx == 0 { 2 } else { voice_idx - 1 };

                // Check if sync source had a 0->1 MSB transition
                let source_had_transition =
                    !prev_msb[sync_source_idx] && self.voices[sync_source_idx].accumulator_msb();

                if source_had_transition {
                    // Reset this voice's accumulator
                    self.voices[voice_idx].accumulator = 0;
                }
            }
        }
    }

    /// Reset the SID to power-on state.
    pub fn reset(&mut self) {
        self.voices = [SidVoice::new(), SidVoice::new(), SidVoice::new()];
        self.filter = SidFilter::new();
        self.volume = 0;
        self.sample_buffer.clear();
        self.sample_accumulator = 0.0;
    }

    /// Get the master volume (0-15).
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Get reference to a voice.
    pub fn voice(&self, index: usize) -> Option<&SidVoice> {
        self.voices.get(index)
    }
}

impl Default for Sid6581 {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Sid6581 {
    fn read(&self, offset: u16) -> u8 {
        match offset as usize {
            // Voice 3 oscillator output (read-only)
            0x1B => ((self.voices[2].accumulator >> 16) & 0xFF) as u8,
            // Voice 3 envelope output (read-only)
            0x1C => self.voices[2].envelope_counter,
            // Paddle X (not implemented)
            0x19 => 0xFF,
            // Paddle Y (not implemented)
            0x1A => 0xFF,
            // All other registers are write-only, return last written value or 0
            _ => 0,
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset as usize {
            // Voice 1 registers ($D400-$D406)
            0x00 => self.voices[0].freq = (self.voices[0].freq & 0xFF00) | value as u16,
            0x01 => self.voices[0].freq = (self.voices[0].freq & 0x00FF) | ((value as u16) << 8),
            0x02 => {
                self.voices[0].pulse_width = (self.voices[0].pulse_width & 0xF00) | value as u16
            }
            0x03 => {
                self.voices[0].pulse_width =
                    (self.voices[0].pulse_width & 0x0FF) | (((value & 0x0F) as u16) << 8)
            }
            0x04 => {
                let was_gate = self.voices[0].gate();
                self.voices[0].control = value;
                // Gate transition handling
                if !was_gate && self.voices[0].gate() {
                    self.voices[0].envelope_state = EnvelopeState::Attack;
                } else if was_gate && !self.voices[0].gate() {
                    self.voices[0].envelope_state = EnvelopeState::Release;
                }
            }
            0x05 => self.voices[0].attack_decay = value,
            0x06 => self.voices[0].sustain_release = value,

            // Voice 2 registers ($D407-$D40D)
            0x07 => self.voices[1].freq = (self.voices[1].freq & 0xFF00) | value as u16,
            0x08 => self.voices[1].freq = (self.voices[1].freq & 0x00FF) | ((value as u16) << 8),
            0x09 => {
                self.voices[1].pulse_width = (self.voices[1].pulse_width & 0xF00) | value as u16
            }
            0x0A => {
                self.voices[1].pulse_width =
                    (self.voices[1].pulse_width & 0x0FF) | (((value & 0x0F) as u16) << 8)
            }
            0x0B => {
                let was_gate = self.voices[1].gate();
                self.voices[1].control = value;
                if !was_gate && self.voices[1].gate() {
                    self.voices[1].envelope_state = EnvelopeState::Attack;
                } else if was_gate && !self.voices[1].gate() {
                    self.voices[1].envelope_state = EnvelopeState::Release;
                }
            }
            0x0C => self.voices[1].attack_decay = value,
            0x0D => self.voices[1].sustain_release = value,

            // Voice 3 registers ($D40E-$D414)
            0x0E => self.voices[2].freq = (self.voices[2].freq & 0xFF00) | value as u16,
            0x0F => self.voices[2].freq = (self.voices[2].freq & 0x00FF) | ((value as u16) << 8),
            0x10 => {
                self.voices[2].pulse_width = (self.voices[2].pulse_width & 0xF00) | value as u16
            }
            0x11 => {
                self.voices[2].pulse_width =
                    (self.voices[2].pulse_width & 0x0FF) | (((value & 0x0F) as u16) << 8)
            }
            0x12 => {
                let was_gate = self.voices[2].gate();
                self.voices[2].control = value;
                if !was_gate && self.voices[2].gate() {
                    self.voices[2].envelope_state = EnvelopeState::Attack;
                } else if was_gate && !self.voices[2].gate() {
                    self.voices[2].envelope_state = EnvelopeState::Release;
                }
            }
            0x13 => self.voices[2].attack_decay = value,
            0x14 => self.voices[2].sustain_release = value,

            // Filter registers ($D415-$D418)
            0x15 => self.filter.cutoff = (self.filter.cutoff & 0x7F8) | (value & 0x07) as u16,
            0x16 => self.filter.cutoff = (self.filter.cutoff & 0x007) | ((value as u16) << 3),
            0x17 => {
                self.filter.routing = value & 0x0F;
                self.filter.resonance = (value >> 4) & 0x0F;
            }
            0x18 => {
                self.volume = value & 0x0F;
                self.filter.mode = (value >> 4) & 0x07;
            }

            // Read-only registers or out of range
            _ => {}
        }
    }

    fn size(&self) -> u16 {
        32 // SID registers occupy $D400-$D41F (32 bytes)
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
    fn test_new_sid() {
        let sid = Sid6581::new();
        assert_eq!(sid.volume, 0);
        assert!(sid.audio_enabled);
    }

    #[test]
    fn test_voice_registers() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency to 0x1234
        sid.write(0x00, 0x34);
        sid.write(0x01, 0x12);
        assert_eq!(sid.voices[0].freq, 0x1234);

        // Set voice 1 pulse width to 0xABC
        sid.write(0x02, 0xBC);
        sid.write(0x03, 0x0A);
        assert_eq!(sid.voices[0].pulse_width, 0xABC);
    }

    #[test]
    fn test_gate_transition() {
        let mut sid = Sid6581::new();

        // Initially in Release
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);

        // Gate on -> Attack
        sid.write(0x04, 0x01);
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Attack);

        // Gate off -> Release
        sid.write(0x04, 0x00);
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);
    }

    #[test]
    fn test_filter_registers() {
        let mut sid = Sid6581::new();

        // Set cutoff to 0x7FF (max)
        sid.write(0x15, 0x07);
        sid.write(0x16, 0xFF);
        assert_eq!(sid.filter.cutoff, 0x7FF);

        // Set resonance and routing
        sid.write(0x17, 0xF7);
        assert_eq!(sid.filter.routing, 0x07);
        assert_eq!(sid.filter.resonance, 0x0F);

        // Set volume and filter mode
        sid.write(0x18, 0x7F);
        assert_eq!(sid.volume, 0x0F);
        assert_eq!(sid.filter.mode, 0x07);
    }

    #[test]
    fn test_read_only_registers() {
        let mut sid = Sid6581::new();

        // Set voice 3 state for testing readback
        sid.voices[2].accumulator = 0x550000;
        sid.voices[2].envelope_counter = 0xAA;

        // Read oscillator output
        assert_eq!(sid.read(0x1B), 0x55);

        // Read envelope output
        assert_eq!(sid.read(0x1C), 0xAA);
    }

    #[test]
    fn test_size() {
        let sid = Sid6581::new();
        assert_eq!(sid.size(), 32);
    }

    // =====================================================
    // Phase Accumulator Tests (T077)
    // =====================================================

    #[test]
    fn test_phase_accumulator_increments() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency to 0x0100 (256)
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x01);

        // Initial accumulator is 0
        assert_eq!(sid.voices[0].accumulator, 0);

        // After one clock, accumulator should be 256
        sid.clock();
        assert_eq!(sid.voices[0].accumulator, 256);

        // After two clocks, accumulator should be 512
        sid.clock();
        assert_eq!(sid.voices[0].accumulator, 512);
    }

    #[test]
    fn test_phase_accumulator_wraps_at_24_bits() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency to 0x1000 (4096)
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10);

        // Set accumulator near the wrap point
        sid.voices[0].accumulator = 0x00FF_F000; // Near 24-bit max

        // Clock should wrap the accumulator
        sid.clock();
        // Expected: 0x00FF_F000 + 0x1000 = 0x0100_0000, masked to 24 bits = 0x0000_0000
        assert_eq!(sid.voices[0].accumulator, 0);
    }

    #[test]
    fn test_phase_accumulator_frequency_change() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency to 0x0001
        sid.write(0x00, 0x01);
        sid.write(0x01, 0x00);

        // Clock a few times
        for _ in 0..10 {
            sid.clock();
        }
        assert_eq!(sid.voices[0].accumulator, 10);

        // Change frequency to 0x0100
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x01);

        // Clock a few more times
        for _ in 0..10 {
            sid.clock();
        }
        // Should now be 10 + (10 * 256) = 2570
        assert_eq!(sid.voices[0].accumulator, 2570);
    }

    #[test]
    fn test_test_bit_resets_accumulator() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10);

        // Clock to build up accumulator
        for _ in 0..100 {
            sid.clock();
        }
        assert!(sid.voices[0].accumulator > 0);

        // Set test bit (bit 3 of control register)
        sid.write(0x04, 0x08);

        // Clock - accumulator should be held at 0
        sid.clock();
        assert_eq!(sid.voices[0].accumulator, 0);

        // Clear test bit
        sid.write(0x04, 0x00);

        // Clock - accumulator should start incrementing again
        sid.clock();
        assert_eq!(sid.voices[0].accumulator, 0x1000);
    }

    #[test]
    fn test_hard_sync_resets_target_voice() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency high (will quickly wrap)
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x80); // 0x8000

        // Set voice 2 frequency lower
        sid.write(0x07, 0x00);
        sid.write(0x08, 0x10); // 0x1000

        // Enable sync on voice 2 (synced by voice 1)
        // Control register for voice 2 is at offset 0x0B
        sid.write(0x0B, 0x02); // SYNC bit set

        // Clock until voice 1's MSB transitions 0->1
        // MSB is bit 23, so we need accumulator >= 0x800000
        // With freq 0x8000, after 256 clocks: 256 * 0x8000 = 0x800000
        for _ in 0..256 {
            sid.clock();
        }

        // At this point voice 1's accumulator MSB should have just transitioned
        // Voice 2 should have been reset (accumulator = 0)
        // Voice 2's accumulator would have been: 256 * 0x1000 = 0x100000
        // But sync should have reset it to 0
        assert_eq!(
            sid.voices[1].accumulator, 0,
            "Voice 2 should be reset by sync from voice 1"
        );
    }

    #[test]
    fn test_all_three_voices_accumulate_independently() {
        let mut sid = Sid6581::new();

        // Set different frequencies for all three voices
        sid.write(0x00, 0x01);
        sid.write(0x01, 0x00); // Voice 1: freq = 1
        sid.write(0x07, 0x02);
        sid.write(0x08, 0x00); // Voice 2: freq = 2
        sid.write(0x0E, 0x03);
        sid.write(0x0F, 0x00); // Voice 3: freq = 3

        // Clock 100 times
        for _ in 0..100 {
            sid.clock();
        }

        // Each voice should have accumulated its frequency * 100
        assert_eq!(sid.voices[0].accumulator, 100);
        assert_eq!(sid.voices[1].accumulator, 200);
        assert_eq!(sid.voices[2].accumulator, 300);
    }

    #[test]
    fn test_voice_3_readback_reflects_accumulator() {
        let mut sid = Sid6581::new();

        // Set voice 3 frequency
        sid.write(0x0E, 0x00);
        sid.write(0x0F, 0x10); // 0x1000

        // Clock to build up accumulator
        for _ in 0..256 {
            sid.clock();
        }
        // Accumulator: 256 * 0x1000 = 0x100000

        // Read voice 3 oscillator output (register $D41B = offset 0x1B)
        // Returns bits 16-23 of accumulator
        let osc_output = sid.read(0x1B);
        assert_eq!(
            osc_output, 0x10,
            "OSC3 should reflect accumulator bits 16-23"
        );
    }

    #[test]
    fn test_sync_voice_mapping() {
        // Test the sync source mapping:
        // Voice 1 is synced by Voice 3
        // Voice 2 is synced by Voice 1
        // Voice 3 is synced by Voice 2

        let mut sid = Sid6581::new();

        // Set all voices to high frequency that will wrap quickly
        for voice in 0..3 {
            let base_offset = voice * 7;
            sid.write(base_offset as u16, 0x00);
            sid.write(base_offset as u16 + 1, 0x80); // 0x8000
        }

        // Enable sync on voice 1 (synced by voice 3)
        sid.write(0x04, 0x02);

        // Run until voice 3's MSB transitions
        // Initial accumulator = 0, freq = 0x8000
        // After 256 clocks: 0x800000 (MSB = 1)
        for _ in 0..256 {
            sid.clock();
        }

        // Voice 1 should have been reset by voice 3's sync
        assert_eq!(
            sid.voices[0].accumulator, 0,
            "Voice 1 should be reset by voice 3"
        );
    }
}
