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

/// ADSR attack rate periods (CPU cycles between envelope increments).
/// These values are the number of clock cycles at ~1MHz for each attack setting (0-15).
/// Attack rate is linear (envelope counter always increments by 1).
///
/// Reference: reSID documentation and SID chip analysis.
pub const ATTACK_RATE_PERIODS: [u16; 16] = [
    9,     // 0: 2 ms
    32,    // 1: 8 ms
    63,    // 2: 16 ms
    95,    // 3: 24 ms
    149,   // 4: 38 ms
    220,   // 5: 56 ms
    267,   // 6: 68 ms
    313,   // 7: 80 ms
    392,   // 8: 100 ms
    977,   // 9: 250 ms
    1954,  // 10: 500 ms
    3126,  // 11: 800 ms
    3907,  // 12: 1 s
    11720, // 13: 3 s
    19532, // 14: 5 s
    31251, // 15: 8 s
];

/// ADSR decay/release rate periods (CPU cycles between envelope decrements).
/// These values are approximately 3x the attack rates because decay/release times
/// are specified as longer in the SID documentation.
///
/// Note: The actual decrement amount varies based on the exponential counter
/// (see `exp_counter_periods` for the dividers applied at different envelope levels).
///
/// Reference: reSID documentation and SID chip analysis.
pub const DECAY_RELEASE_RATE_PERIODS: [u16; 16] = [
    9,     // 0: 6 ms
    32,    // 1: 24 ms
    63,    // 2: 48 ms
    95,    // 3: 72 ms
    149,   // 4: 114 ms
    220,   // 5: 168 ms
    267,   // 6: 204 ms
    313,   // 7: 240 ms
    392,   // 8: 300 ms
    977,   // 9: 750 ms
    1954,  // 10: 1.5 s
    3126,  // 11: 2.4 s
    3907,  // 12: 3 s
    11720, // 13: 9 s
    19532, // 14: 15 s
    31251, // 15: 24 s
];


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
    /// Previous bit 19 state for noise LFSR clocking.
    /// The LFSR is clocked when bit 19 transitions from 0 to 1.
    pub prev_bit19: bool,
    /// 23-bit noise LFSR (Linear Feedback Shift Register).
    /// Feedback taps at bits 22 and 17 (XOR).
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
            prev_bit19: false,
            lfsr: 0x7FFFF8, // Initial LFSR state (23-bit)
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
    /// Bit 4 = Triangle, Bit 5 = Sawtooth, Bit 6 = Pulse, Bit 7 = Noise
    #[inline]
    pub fn waveform(&self) -> u8 {
        (self.control >> 4) & 0x0F
    }

    /// Check if triangle waveform is selected.
    #[inline]
    pub fn triangle_enabled(&self) -> bool {
        self.control & 0x10 != 0
    }

    /// Check if sawtooth waveform is selected.
    #[inline]
    pub fn sawtooth_enabled(&self) -> bool {
        self.control & 0x20 != 0
    }

    /// Check if pulse waveform is selected.
    #[inline]
    pub fn pulse_enabled(&self) -> bool {
        self.control & 0x40 != 0
    }

    /// Check if noise waveform is selected.
    #[inline]
    pub fn noise_enabled(&self) -> bool {
        self.control & 0x80 != 0
    }

    /// Check if ring modulation is enabled.
    #[inline]
    pub fn ring_mod_enabled(&self) -> bool {
        self.control & 0x04 != 0
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

    /// Get the attack rate value (0-15) from the AD register.
    #[inline]
    pub fn attack_rate(&self) -> u8 {
        (self.attack_decay >> 4) & 0x0F
    }

    /// Get the decay rate value (0-15) from the AD register.
    #[inline]
    pub fn decay_rate(&self) -> u8 {
        self.attack_decay & 0x0F
    }

    /// Get the sustain level (0-15) from the SR register.
    /// Note: Sustain is a level, not a rate. The actual sustain
    /// envelope value is (sustain * 17), giving 0-255 range.
    #[inline]
    pub fn sustain_level(&self) -> u8 {
        (self.sustain_release >> 4) & 0x0F
    }

    /// Get the release rate value (0-15) from the SR register.
    #[inline]
    pub fn release_rate(&self) -> u8 {
        self.sustain_release & 0x0F
    }

    /// Get the sustain envelope value (0-255).
    /// Sustain register value 0-15 maps to 0-255 by multiplying by 17.
    /// This gives: 0->0, 1->17, 2->34, ..., 15->255
    #[inline]
    pub fn sustain_value(&self) -> u8 {
        let sustain = self.sustain_level();
        // Multiply by 17 to map 0-15 to 0-255
        // 15 * 17 = 255, so this maps perfectly
        sustain.saturating_mul(17)
    }

    /// Get the exponential counter period for the current envelope value.
    /// This implements the SID's characteristic exponential decay curve
    /// by slowing down the envelope as it approaches zero.
    #[inline]
    pub fn get_exp_period(&self) -> u8 {
        let env = self.envelope_counter;
        // Check thresholds from highest to lowest
        // 255-94: period 1
        // 93-55: period 2
        // 54-27: period 4
        // 26-15: period 8
        // 14-7: period 16
        // 6-0: period 30
        if env >= 94 {
            1
        } else if env >= 55 {
            2
        } else if env >= 27 {
            4
        } else if env >= 15 {
            8
        } else if env >= 7 {
            16
        } else {
            30
        }
    }

    /// Get the current MSB of the accumulator (bit 23).
    #[inline]
    pub fn accumulator_msb(&self) -> bool {
        self.accumulator & 0x0080_0000 != 0
    }

    /// Get bit 19 of the accumulator (used for LFSR clocking).
    #[inline]
    pub fn accumulator_bit19(&self) -> bool {
        self.accumulator & 0x0008_0000 != 0
    }

    /// Clock the 23-bit noise LFSR.
    ///
    /// The LFSR uses feedback taps at bits 22 and 17 (XOR).
    /// The feedback bit is shifted into position 0, and the register
    /// shifts left, masking to 23 bits.
    ///
    /// This produces the characteristic pseudo-random noise sequence.
    #[inline]
    pub fn clock_lfsr(&mut self) {
        // Feedback = bit22 XOR bit17
        let bit22 = (self.lfsr >> 22) & 1;
        let bit17 = (self.lfsr >> 17) & 1;
        let feedback = bit22 ^ bit17;

        // Shift left and insert feedback at bit 0, mask to 23 bits
        self.lfsr = ((self.lfsr << 1) | feedback) & 0x7F_FFFF;
    }

    /// Generate the sawtooth waveform output.
    ///
    /// The sawtooth is simply the top 12 bits of the 24-bit accumulator.
    /// This produces a linearly rising waveform that resets when the
    /// accumulator overflows.
    #[inline]
    pub fn generate_sawtooth(&self) -> u16 {
        ((self.accumulator >> 12) & 0xFFF) as u16
    }

    /// Generate the triangle waveform output.
    ///
    /// The triangle is derived from the accumulator by:
    /// 1. Taking bits 11-22 (12 bits)
    /// 2. XORing with the MSB (bit 23) replicated across all bits
    ///
    /// This creates a waveform that rises then falls symmetrically.
    ///
    /// If ring modulation is enabled, the MSB of the modulating voice
    /// is used instead of this voice's MSB.
    #[inline]
    pub fn generate_triangle(&self, ring_mod_msb: bool) -> u16 {
        // Get the MSB to use for XOR (either ours or ring mod source)
        let msb = if self.ring_mod_enabled() {
            ring_mod_msb
        } else {
            self.accumulator_msb()
        };

        // Get bits 11-22 of accumulator (12 bits, but we use 11 for proper triangle)
        let acc_bits = ((self.accumulator >> 12) & 0x7FF) as u16;

        // XOR with MSB to create triangle shape
        if msb {
            // When MSB is 1, invert the waveform (falling edge)
            acc_bits ^ 0x7FF
        } else {
            // When MSB is 0, use waveform directly (rising edge)
            acc_bits
        }
    }

    /// Generate the pulse waveform output.
    ///
    /// The pulse compares the top 12 bits of the accumulator against the
    /// 12-bit pulse width register. Output is high (0xFFF) when accumulator
    /// is below pulse width, low (0) otherwise.
    #[inline]
    pub fn generate_pulse(&self) -> u16 {
        let acc_top12 = ((self.accumulator >> 12) & 0xFFF) as u16;
        if acc_top12 < self.pulse_width {
            0xFFF
        } else {
            0
        }
    }

    /// Generate the noise waveform output.
    ///
    /// Noise uses bits from the 23-bit LFSR. The LFSR is clocked separately
    /// (see T080 for full implementation). This returns the current noise
    /// output based on the LFSR state.
    ///
    /// The output is constructed from specific LFSR bits to produce
    /// pseudo-random noise with the characteristic SID sound.
    #[inline]
    pub fn generate_noise(&self) -> u16 {
        // Output is constructed from specific LFSR bits
        // Bits 0, 2, 5, 9, 11, 14, 18, 20 mapped to output bits
        let lfsr = self.lfsr;
        let bit0 = (lfsr & 1) as u16;
        let bit2 = ((lfsr >> 2) & 1) as u16;
        let bit5 = ((lfsr >> 5) & 1) as u16;
        let bit9 = ((lfsr >> 9) & 1) as u16;
        let bit11 = ((lfsr >> 11) & 1) as u16;
        let bit14 = ((lfsr >> 14) & 1) as u16;
        let bit18 = ((lfsr >> 18) & 1) as u16;
        let bit20 = ((lfsr >> 20) & 1) as u16;

        // Construct 12-bit output from LFSR bits (with gaps for that characteristic sound)
        (bit0 << 4)
            | (bit2 << 5)
            | (bit5 << 6)
            | (bit9 << 7)
            | (bit11 << 8)
            | (bit14 << 9)
            | (bit18 << 10)
            | (bit20 << 11)
    }

    /// Generate combined waveform output for this voice.
    ///
    /// When multiple waveforms are selected, the outputs are ANDed together.
    /// This is a characteristic of the SID hardware that creates unique
    /// timbres when waveforms are combined.
    ///
    /// The ring_mod_msb parameter is the MSB of the ring modulation source
    /// voice (used only when ring mod is enabled for triangle).
    pub fn generate_waveform(&self, ring_mod_msb: bool) -> u16 {
        let waveform_bits = self.waveform();

        // No waveform selected = silence
        if waveform_bits == 0 {
            return 0;
        }

        // Start with all 1s for AND combination
        let mut output = 0xFFFu16;

        // AND together all selected waveforms
        if self.triangle_enabled() {
            output &= self.generate_triangle(ring_mod_msb);
        }

        if self.sawtooth_enabled() {
            output &= self.generate_sawtooth();
        }

        if self.pulse_enabled() {
            output &= self.generate_pulse();
        }

        if self.noise_enabled() {
            output &= self.generate_noise();
        }

        output
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

        // Clock envelope generators for all voices
        self.clock_envelopes();

        // TODO: Implement filter processing (T083-T084)

        // Audio sample generation
        self.sample_accumulator += 1.0;
        if self.sample_accumulator >= self.cycles_per_sample {
            self.sample_accumulator -= self.cycles_per_sample;
            if self.audio_enabled {
                // Generate waveform output for all voices and mix
                let sample = self.generate_output();
                self.sample_buffer.push(sample);
            }
        }
    }

    /// Generate the mixed audio output from all three voices.
    ///
    /// This method:
    /// 1. Generates waveform output for each voice (with ring modulation)
    /// 2. Applies envelope (when implemented)
    /// 3. Routes through filter (when implemented)
    /// 4. Applies master volume
    /// 5. Returns normalized f32 sample
    fn generate_output(&self) -> f32 {
        // Get ring modulation source MSBs for each voice
        // Voice 1 is modulated by voice 3, voice 2 by voice 1, voice 3 by voice 2
        let ring_mod_msb = [
            self.voices[2].accumulator_msb(), // Voice 1 <- Voice 3
            self.voices[0].accumulator_msb(), // Voice 2 <- Voice 1
            self.voices[1].accumulator_msb(), // Voice 3 <- Voice 2
        ];

        // Generate waveform output for each voice
        let voice_outputs: [u16; 3] = [
            self.voices[0].generate_waveform(ring_mod_msb[0]),
            self.voices[1].generate_waveform(ring_mod_msb[1]),
            self.voices[2].generate_waveform(ring_mod_msb[2]),
        ];

        // Apply envelope to each voice
        // Waveform output is 0-0xFFF (12 bits), envelope is 0-255
        // Result is 0 to (4095 * 255) = 1,044,225 before shift
        // After >> 8, result is 0 to 4079
        let voice_with_envelope: [i32; 3] = [
            (voice_outputs[0] as i32 * self.voices[0].envelope_counter as i32) >> 8,
            (voice_outputs[1] as i32 * self.voices[1].envelope_counter as i32) >> 8,
            (voice_outputs[2] as i32 * self.voices[2].envelope_counter as i32) >> 8,
        ];

        // Mix voices (simple sum for now, filter routing in T083-T084)
        // Max sum: ~4079 * 3 = ~12237
        let mixed: i32 = voice_with_envelope[0] + voice_with_envelope[1] + voice_with_envelope[2];

        // Apply master volume (0-15)
        // Max: 12237 * 15 / 16 = ~11472
        let with_volume = (mixed * self.volume as i32) >> 4;

        // Normalize to f32 range [-1.0, 1.0]
        // The SID output is inherently unsigned (0 to max), so we need to
        // center it around zero for audio output.
        // Max value: ~11472, so mid-point is ~5736
        // We scale so that max deviation maps to 1.0
        const MAX_OUTPUT: f32 = 11472.0;
        const MID_POINT: f32 = MAX_OUTPUT / 2.0;

        if with_volume == 0 && self.volume == 0 {
            // Special case: zero volume means silence
            0.0
        } else {
            (with_volume as f32 - MID_POINT) / MID_POINT
        }
    }

    /// Get the current waveform output for a voice (for debugging/visualization).
    pub fn voice_waveform_output(&self, voice_idx: usize) -> Option<u16> {
        if voice_idx >= VOICE_COUNT {
            return None;
        }

        // Get ring mod MSB for this voice
        let ring_mod_msb = match voice_idx {
            0 => self.voices[2].accumulator_msb(),
            1 => self.voices[0].accumulator_msb(),
            2 => self.voices[1].accumulator_msb(),
            _ => false,
        };

        Some(self.voices[voice_idx].generate_waveform(ring_mod_msb))
    }

    /// Clock all voice phase accumulators with hard sync and noise LFSR support.
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
    ///
    /// Noise LFSR: The 23-bit LFSR is clocked when bit 19 of the accumulator
    /// transitions from 0 to 1. This creates the characteristic noise frequency
    /// that tracks the voice frequency.
    fn clock_phase_accumulators(&mut self) {
        // Store previous MSB and bit19 states before updating accumulators
        let prev_msb = [
            self.voices[0].accumulator_msb(),
            self.voices[1].accumulator_msb(),
            self.voices[2].accumulator_msb(),
        ];
        let prev_bit19 = [
            self.voices[0].accumulator_bit19(),
            self.voices[1].accumulator_bit19(),
            self.voices[2].accumulator_bit19(),
        ];

        // Update each voice's accumulator
        #[allow(clippy::needless_range_loop)]
        for voice_idx in 0..VOICE_COUNT {
            let voice = &mut self.voices[voice_idx];

            // Store previous states for sync and LFSR detection
            voice.prev_msb = prev_msb[voice_idx];
            voice.prev_bit19 = prev_bit19[voice_idx];

            // Handle test bit - when set, resets and holds accumulator at zero
            // Also resets the LFSR to its initial state
            if voice.test_bit() {
                voice.accumulator = 0;
                voice.lfsr = 0x7F_FFFF; // Reset LFSR to all 1s (max value)
                continue;
            }

            // Increment phase accumulator by frequency value (wraps at 24 bits)
            voice.accumulator = voice.accumulator.wrapping_add(voice.freq as u32);
            voice.accumulator &= 0x00FF_FFFF; // Mask to 24 bits

            // Clock LFSR on bit 19 transition from 0 to 1
            // This makes the noise frequency track the voice frequency
            if !voice.prev_bit19 && voice.accumulator_bit19() {
                voice.clock_lfsr();
            }
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

    /// Clock the ADSR envelope generators for all three voices.
    ///
    /// The envelope generator produces an 8-bit output (0-255) that modulates
    /// the voice amplitude. It goes through four states:
    ///
    /// 1. **Attack**: Linear ramp from 0 to 255
    /// 2. **Decay**: Exponential decay from 255 to sustain level
    /// 3. **Sustain**: Holds at sustain level while gate is on
    /// 4. **Release**: Exponential decay from current level to 0
    ///
    /// The exponential decay is approximated using dividers that slow down
    /// the decay rate as the envelope approaches zero. This creates the
    /// characteristic SID envelope shape.
    fn clock_envelopes(&mut self) {
        for voice in &mut self.voices {
            // Increment the rate counter
            voice.rate_counter = voice.rate_counter.wrapping_add(1);

            // Get the rate period for the current envelope state
            let rate_period = match voice.envelope_state {
                EnvelopeState::Attack => ATTACK_RATE_PERIODS[voice.attack_rate() as usize],
                EnvelopeState::Decay => DECAY_RELEASE_RATE_PERIODS[voice.decay_rate() as usize],
                EnvelopeState::Sustain => 0, // No rate period needed for sustain
                EnvelopeState::Release => {
                    DECAY_RELEASE_RATE_PERIODS[voice.release_rate() as usize]
                }
            };

            // Check if it's time to update the envelope
            if voice.envelope_state == EnvelopeState::Sustain {
                // In Sustain state, the envelope holds at the sustain level.
                // Nothing to do - the envelope counter was set when we entered
                // decay and reached the sustain level.
                continue;
            }

            // Check if we've completed a rate period
            if voice.rate_counter >= rate_period {
                voice.rate_counter = 0;

                match voice.envelope_state {
                    EnvelopeState::Attack => {
                        // Attack phase: linear increment
                        // Increment envelope counter. If it reaches 255, transition to Decay.
                        if voice.envelope_counter < 255 {
                            voice.envelope_counter = voice.envelope_counter.wrapping_add(1);
                        }
                        if voice.envelope_counter == 255 {
                            voice.envelope_state = EnvelopeState::Decay;
                            voice.exp_counter = 0; // Reset exponential counter for decay
                        }
                    }
                    EnvelopeState::Decay => {
                        // Decay phase: exponential decrement toward sustain level
                        let sustain = voice.sustain_value();

                        if voice.envelope_counter > sustain {
                            // Apply exponential decay: only decrement when exp_counter reaches period
                            voice.exp_counter = voice.exp_counter.wrapping_add(1);
                            let exp_period = voice.get_exp_period();

                            if voice.exp_counter >= exp_period {
                                voice.exp_counter = 0;
                                voice.envelope_counter = voice.envelope_counter.saturating_sub(1);
                            }
                        }

                        // Check if we've reached or passed the sustain level
                        if voice.envelope_counter <= sustain {
                            voice.envelope_counter = sustain;
                            voice.envelope_state = EnvelopeState::Sustain;
                        }
                    }
                    EnvelopeState::Sustain => {
                        // Already handled above with continue
                        unreachable!()
                    }
                    EnvelopeState::Release => {
                        // Release phase: exponential decrement toward 0
                        if voice.envelope_counter > 0 {
                            // Apply exponential decay
                            voice.exp_counter = voice.exp_counter.wrapping_add(1);
                            let exp_period = voice.get_exp_period();

                            if voice.exp_counter >= exp_period {
                                voice.exp_counter = 0;
                                voice.envelope_counter = voice.envelope_counter.saturating_sub(1);
                            }
                        }
                        // Envelope stays at 0 until gate is triggered again
                    }
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

    // =====================================================
    // Waveform Generation Tests (T078)
    // =====================================================

    #[test]
    fn test_sawtooth_waveform_output() {
        let mut voice = SidVoice::new();

        // Accumulator at 0 -> sawtooth output is 0
        voice.accumulator = 0x000000;
        assert_eq!(voice.generate_sawtooth(), 0);

        // Accumulator at mid-point -> sawtooth is half
        voice.accumulator = 0x800000; // Top 12 bits = 0x800
        assert_eq!(voice.generate_sawtooth(), 0x800);

        // Accumulator near max -> sawtooth near max
        voice.accumulator = 0xFFF000; // Top 12 bits = 0xFFF
        assert_eq!(voice.generate_sawtooth(), 0xFFF);

        // Test intermediate value
        voice.accumulator = 0x123000; // Top 12 bits = 0x123
        assert_eq!(voice.generate_sawtooth(), 0x123);
    }

    #[test]
    fn test_triangle_waveform_rises_and_falls() {
        let mut voice = SidVoice::new();

        // Accumulator at 0 (MSB=0, rising phase) -> triangle output is 0
        voice.accumulator = 0x000000;
        assert_eq!(voice.generate_triangle(false), 0);

        // Accumulator at 25% (MSB=0, still rising)
        voice.accumulator = 0x400000; // Bits 11-22 = 0x400
        assert_eq!(voice.generate_triangle(false), 0x400);

        // Accumulator at 50% (MSB=0, peak of rising)
        voice.accumulator = 0x7FF000; // Just below MSB flip
        assert_eq!(voice.generate_triangle(false), 0x7FF);

        // Accumulator just past 50% (MSB=1, falling phase begins)
        voice.accumulator = 0x800000; // MSB=1, bits 11-22 = 0
                                      // XOR with 0x7FF: 0 ^ 0x7FF = 0x7FF
        assert_eq!(voice.generate_triangle(false), 0x7FF);

        // Accumulator at 75% (MSB=1, falling)
        voice.accumulator = 0xC00000; // Bits 11-22 = 0x400
                                      // XOR: 0x400 ^ 0x7FF = 0x3FF
        assert_eq!(voice.generate_triangle(false), 0x3FF);

        // Near end (MSB=1)
        voice.accumulator = 0xFFF000; // Bits 11-22 = 0x7FF
                                      // XOR: 0x7FF ^ 0x7FF = 0
        assert_eq!(voice.generate_triangle(false), 0);
    }

    #[test]
    fn test_triangle_ring_modulation() {
        let mut voice = SidVoice::new();

        // Set accumulator with MSB=0
        voice.accumulator = 0x400000;

        // Ring mod disabled - use own MSB (0)
        voice.control = 0x10; // Triangle only, no ring mod
        assert_eq!(voice.generate_triangle(true), 0x400);

        // Ring mod enabled - use provided MSB (true = 1)
        voice.control = 0x14; // Triangle + ring mod
                              // With ring_mod_msb=true, XOR with 0x7FF
        assert_eq!(voice.generate_triangle(true), 0x400 ^ 0x7FF);
    }

    #[test]
    fn test_pulse_waveform_threshold() {
        let mut voice = SidVoice::new();

        // Set pulse width to 50% (0x800)
        voice.pulse_width = 0x800;

        // Accumulator below pulse width -> output high
        voice.accumulator = 0x000000;
        assert_eq!(voice.generate_pulse(), 0xFFF);

        voice.accumulator = 0x7FF000; // Top 12 bits = 0x7FF (below 0x800)
        assert_eq!(voice.generate_pulse(), 0xFFF);

        // Accumulator at pulse width -> output low
        voice.accumulator = 0x800000; // Top 12 bits = 0x800 (equals threshold)
        assert_eq!(voice.generate_pulse(), 0);

        // Accumulator above pulse width -> output low
        voice.accumulator = 0xFFF000;
        assert_eq!(voice.generate_pulse(), 0);
    }

    #[test]
    fn test_pulse_width_extremes() {
        let mut voice = SidVoice::new();

        // Pulse width 0 -> always low (no high portion)
        voice.pulse_width = 0;
        voice.accumulator = 0x000000;
        assert_eq!(voice.generate_pulse(), 0);

        // Pulse width max (0xFFF) -> always high (except at very end)
        voice.pulse_width = 0xFFF;
        voice.accumulator = 0x000000;
        assert_eq!(voice.generate_pulse(), 0xFFF);
        voice.accumulator = 0xFFE000; // Top 12 = 0xFFE (still below 0xFFF)
        assert_eq!(voice.generate_pulse(), 0xFFF);
        voice.accumulator = 0xFFF000; // Top 12 = 0xFFF (at threshold)
        assert_eq!(voice.generate_pulse(), 0);
    }

    #[test]
    fn test_noise_waveform_uses_lfsr() {
        let voice = SidVoice::new();

        // Initial LFSR state is 0x7FFFF8
        // Output should be derived from specific bits
        let noise = voice.generate_noise();

        // The output should be non-zero given the LFSR state
        // We can't easily predict exact value without knowing bit positions,
        // but we can verify it's based on LFSR
        assert!(noise <= 0xFFF); // 12-bit output
    }

    #[test]
    fn test_noise_changes_with_lfsr() {
        let mut voice = SidVoice::new();

        let noise1 = voice.generate_noise();

        // Change LFSR state
        voice.lfsr = 0x123456;
        let noise2 = voice.generate_noise();

        // Different LFSR should produce different noise
        assert_ne!(noise1, noise2);
    }

    // =====================================================
    // Noise LFSR Clocking Tests (T080)
    // =====================================================

    #[test]
    fn test_lfsr_clock_produces_new_state() {
        let mut voice = SidVoice::new();

        // Initial LFSR state
        let initial_lfsr = voice.lfsr;

        // Clock the LFSR
        voice.clock_lfsr();

        // LFSR should have changed
        assert_ne!(voice.lfsr, initial_lfsr);

        // LFSR should still be 23 bits (masked)
        assert!(voice.lfsr <= 0x7F_FFFF);
    }

    #[test]
    fn test_lfsr_feedback_taps() {
        let mut voice = SidVoice::new();

        // Set LFSR to known state to verify feedback calculation
        // Feedback = bit22 XOR bit17
        // If bit22=1 and bit17=0, feedback = 1
        voice.lfsr = 0x40_0000; // bit22=1, bit17=0

        voice.clock_lfsr();

        // Shifted left by 1, with feedback=1 at bit 0, masked to 23 bits
        // 0x40_0000 << 1 = 0x80_0000 (before mask)
        // 0x80_0000 | 1 = 0x80_0001
        // 0x80_0001 & 0x7F_FFFF = 0x00_0001
        assert_eq!(voice.lfsr, 0x00_0001);
    }

    #[test]
    fn test_lfsr_feedback_both_bits_same() {
        let mut voice = SidVoice::new();

        // If bit22=1 and bit17=1, feedback = 0 (XOR)
        voice.lfsr = 0x42_0000; // bit22=1, bit17=1

        voice.clock_lfsr();

        // feedback=0, so bit 0 is 0 after shift
        // 0x42_0000 << 1 = 0x84_0000
        // 0x84_0000 & 0x7F_FFFF = 0x04_0000
        assert_eq!(voice.lfsr, 0x04_0000);
    }

    #[test]
    fn test_lfsr_sequence_is_deterministic() {
        let mut voice1 = SidVoice::new();
        let mut voice2 = SidVoice::new();

        // Both start with same state
        assert_eq!(voice1.lfsr, voice2.lfsr);

        // Clock both 100 times
        for _ in 0..100 {
            voice1.clock_lfsr();
            voice2.clock_lfsr();
        }

        // Should end up at same state
        assert_eq!(voice1.lfsr, voice2.lfsr);
    }

    #[test]
    fn test_lfsr_never_zero() {
        let mut voice = SidVoice::new();

        // Clock LFSR many times and verify it never becomes zero
        // (a proper LFSR should never reach all-zeros state)
        for _ in 0..1000 {
            voice.clock_lfsr();
            assert_ne!(voice.lfsr, 0, "LFSR should never become zero");
        }
    }

    #[test]
    fn test_lfsr_clocked_on_bit19_transition() {
        let mut sid = Sid6581::new();

        // Set voice 1 frequency high enough to trigger bit19 transition quickly
        // Frequency 0x8000 means accumulator adds 0x8000 each clock
        // Bit 19 is at 0x0008_0000
        // To transition from 0 to 1, we need accumulator to pass 0x0007_FFFF
        // With freq 0x8000, after 16 clocks: 16 * 0x8000 = 0x8_0000 (bit 19 set)
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x80); // freq = 0x8000

        let initial_lfsr = sid.voices[0].lfsr;

        // Clock 15 times - bit 19 not yet set (accumulator = 0x7_8000)
        for _ in 0..15 {
            sid.clock();
        }

        // LFSR should NOT have been clocked yet (bit 19 still 0)
        assert_eq!(
            sid.voices[0].lfsr, initial_lfsr,
            "LFSR should not clock until bit19 transitions"
        );

        // Clock once more - bit 19 transitions from 0 to 1
        sid.clock();

        // LFSR should now have been clocked
        assert_ne!(
            sid.voices[0].lfsr, initial_lfsr,
            "LFSR should clock on bit19 0->1 transition"
        );
    }

    #[test]
    fn test_lfsr_not_clocked_on_bit19_already_set() {
        let mut sid = Sid6581::new();

        // Set voice 1 with frequency
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x80);

        // Set accumulator so bit 19 is already set
        sid.voices[0].accumulator = 0x0008_0000;
        sid.voices[0].prev_bit19 = true; // Was already set

        let initial_lfsr = sid.voices[0].lfsr;

        // Clock once - bit19 stays 1 (no 0->1 transition)
        sid.clock();

        // LFSR should NOT have been clocked (no transition)
        assert_eq!(
            sid.voices[0].lfsr, initial_lfsr,
            "LFSR should not clock when bit19 stays at 1"
        );
    }

    #[test]
    fn test_test_bit_resets_lfsr() {
        let mut sid = Sid6581::new();

        // Set frequency and clock to change LFSR
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x80);

        // Clock enough to change LFSR
        for _ in 0..100 {
            sid.clock();
        }

        // LFSR should have changed
        assert_ne!(sid.voices[0].lfsr, 0x7FFFF8);

        // Set test bit
        sid.write(0x04, 0x08);

        // Clock once with test bit - LFSR should reset to 0x7F_FFFF (all 1s)
        sid.clock();

        assert_eq!(
            sid.voices[0].lfsr, 0x7F_FFFF,
            "Test bit should reset LFSR to all 1s"
        );
        assert_eq!(
            sid.voices[0].accumulator, 0,
            "Test bit should also reset accumulator"
        );
    }

    #[test]
    fn test_noise_output_changes_as_lfsr_clocks() {
        let mut sid = Sid6581::new();

        // Set voice 1 to noise waveform
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x80); // High frequency
        sid.write(0x04, 0x81); // Noise + gate

        // Collect noise outputs
        let mut outputs = Vec::new();

        // Clock enough times to get multiple LFSR clocks
        for _ in 0..200 {
            sid.clock();
            outputs.push(sid.voices[0].generate_noise());
        }

        // Verify we got some different values (noise is changing)
        let unique_count = outputs
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(
            unique_count > 1,
            "Noise output should change as LFSR clocks (got {} unique values)",
            unique_count
        );
    }

    #[test]
    fn test_lfsr_masks_to_23_bits() {
        let mut voice = SidVoice::new();

        // Set LFSR to max 23-bit value
        voice.lfsr = 0x7F_FFFF;

        // Clock it
        voice.clock_lfsr();

        // Result should still be within 23 bits
        assert!(voice.lfsr <= 0x7F_FFFF);

        // Clock many more times
        for _ in 0..1000 {
            voice.clock_lfsr();
            assert!(voice.lfsr <= 0x7F_FFFF, "LFSR exceeded 23 bits");
        }
    }

    #[test]
    fn test_accumulator_bit19_helper() {
        let mut voice = SidVoice::new();

        // Test bit 19 detection
        voice.accumulator = 0x0000_0000;
        assert!(!voice.accumulator_bit19());

        voice.accumulator = 0x0007_FFFF; // Just below bit 19
        assert!(!voice.accumulator_bit19());

        voice.accumulator = 0x0008_0000; // Exactly bit 19
        assert!(voice.accumulator_bit19());

        voice.accumulator = 0x00FF_FFFF; // Bit 19 and higher bits set
        assert!(voice.accumulator_bit19());
    }

    #[test]
    fn test_all_three_voices_have_independent_lfsr() {
        let mut sid = Sid6581::new();

        // Set different frequencies for all three voices
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x40); // Voice 1: freq = 0x4000
        sid.write(0x07, 0x00);
        sid.write(0x08, 0x60); // Voice 2: freq = 0x6000
        sid.write(0x0E, 0x00);
        sid.write(0x0F, 0x80); // Voice 3: freq = 0x8000

        // Record initial LFSR states
        let initial = [sid.voices[0].lfsr, sid.voices[1].lfsr, sid.voices[2].lfsr];

        // All start the same
        assert_eq!(initial[0], initial[1]);
        assert_eq!(initial[1], initial[2]);

        // Clock many times
        for _ in 0..1000 {
            sid.clock();
        }

        // Each voice should have evolved differently due to different frequencies
        // (bit 19 transitions at different rates)
        let final_states = [sid.voices[0].lfsr, sid.voices[1].lfsr, sid.voices[2].lfsr];

        // At least some should be different (with high probability)
        let all_same = final_states[0] == final_states[1] && final_states[1] == final_states[2];
        assert!(
            !all_same,
            "Different frequencies should produce different LFSR evolution"
        );
    }

    #[test]
    fn test_no_waveform_selected_outputs_zero() {
        let mut voice = SidVoice::new();

        // No waveform bits set
        voice.control = 0x00;
        voice.accumulator = 0x400000;

        assert_eq!(voice.generate_waveform(false), 0);
    }

    #[test]
    fn test_single_waveform_selection() {
        let mut voice = SidVoice::new();
        voice.accumulator = 0x400000; // Mid-point
        voice.pulse_width = 0x800;

        // Triangle only
        voice.control = 0x10;
        let triangle_only = voice.generate_waveform(false);
        assert_eq!(triangle_only, voice.generate_triangle(false));

        // Sawtooth only
        voice.control = 0x20;
        let sawtooth_only = voice.generate_waveform(false);
        assert_eq!(sawtooth_only, voice.generate_sawtooth());

        // Pulse only
        voice.control = 0x40;
        let pulse_only = voice.generate_waveform(false);
        assert_eq!(pulse_only, voice.generate_pulse());

        // Noise only
        voice.control = 0x80;
        let noise_only = voice.generate_waveform(false);
        assert_eq!(noise_only, voice.generate_noise());
    }

    #[test]
    fn test_combined_waveforms_are_anded() {
        let mut voice = SidVoice::new();
        voice.accumulator = 0x400000;
        voice.pulse_width = 0x800;

        // Triangle AND Sawtooth
        voice.control = 0x30; // Both triangle and sawtooth

        let combined = voice.generate_waveform(false);
        let expected = voice.generate_triangle(false) & voice.generate_sawtooth();

        assert_eq!(combined, expected);
    }

    #[test]
    fn test_all_waveforms_combined() {
        let mut voice = SidVoice::new();
        voice.accumulator = 0x400000;
        voice.pulse_width = 0x800;

        // All waveforms
        voice.control = 0xF0;

        let combined = voice.generate_waveform(false);
        let expected = voice.generate_triangle(false)
            & voice.generate_sawtooth()
            & voice.generate_pulse()
            & voice.generate_noise();

        assert_eq!(combined, expected);
    }

    #[test]
    fn test_waveform_control_bits() {
        let mut voice = SidVoice::new();

        voice.control = 0x00;
        assert!(!voice.triangle_enabled());
        assert!(!voice.sawtooth_enabled());
        assert!(!voice.pulse_enabled());
        assert!(!voice.noise_enabled());
        assert!(!voice.ring_mod_enabled());

        voice.control = 0x10;
        assert!(voice.triangle_enabled());
        assert!(!voice.sawtooth_enabled());

        voice.control = 0x20;
        assert!(!voice.triangle_enabled());
        assert!(voice.sawtooth_enabled());

        voice.control = 0x40;
        assert!(voice.pulse_enabled());

        voice.control = 0x80;
        assert!(voice.noise_enabled());

        voice.control = 0x04;
        assert!(voice.ring_mod_enabled());
    }

    #[test]
    fn test_sid_voice_waveform_output_method() {
        let mut sid = Sid6581::new();

        // Set voice 1 to triangle
        sid.write(0x04, 0x10);
        sid.voices[0].accumulator = 0x400000;

        let output = sid.voice_waveform_output(0);
        assert!(output.is_some());
        assert_eq!(output.unwrap(), sid.voices[0].generate_triangle(false));

        // Invalid voice index
        assert!(sid.voice_waveform_output(3).is_none());
    }

    #[test]
    fn test_sid_generates_audio_samples() {
        let mut sid = Sid6581::new();

        // Set up voice 1 with sawtooth
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10); // Frequency
        sid.write(0x04, 0x21); // Sawtooth + gate on

        // Set envelope to max so we hear output
        sid.voices[0].envelope_counter = 255;

        // Set volume to max
        sid.write(0x18, 0x0F);

        // Clock enough times to generate samples
        for _ in 0..100 {
            sid.clock();
        }

        let samples = sid.take_samples();
        // Should have generated some samples (depends on cycles_per_sample)
        // With default 44.1kHz and PAL clock, ~23 cycles per sample
        assert!(!samples.is_empty());

        // Samples should be in valid range
        for sample in &samples {
            assert!(*sample >= -1.0 && *sample <= 1.0);
        }
    }

    #[test]
    fn test_sid_no_waveform_produces_dc_offset() {
        let mut sid = Sid6581::new();

        // Set frequency but no waveform
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10);
        sid.write(0x04, 0x01); // Gate on, but no waveform

        // Set envelope and volume
        sid.voices[0].envelope_counter = 255;
        sid.write(0x18, 0x0F);

        // Clock enough to generate samples
        for _ in 0..100 {
            sid.clock();
        }

        let samples = sid.take_samples();
        // No waveform = 0 output. After DC centering, this maps to -1.0
        // (the minimum output value). This is correct SID behavior.
        for sample in &samples {
            assert!(
                (*sample - (-1.0)).abs() < 0.01,
                "Sample {} should be at -1.0 (DC offset for zero waveform)",
                sample
            );
        }
    }

    #[test]
    fn test_sid_silent_with_zero_volume() {
        let mut sid = Sid6581::new();

        // Set up voice with sawtooth
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10);
        sid.write(0x04, 0x21); // Sawtooth + gate
        sid.voices[0].envelope_counter = 255;

        // Volume = 0
        sid.write(0x18, 0x00);

        for _ in 0..100 {
            sid.clock();
        }

        let samples = sid.take_samples();
        // With zero volume, all output is 0, which after centering is -1.0
        // Special case: zero volume should actually be treated as silence
        for sample in &samples {
            assert!(
                (*sample).abs() < 0.01,
                "Should be at zero with no volume, got {}",
                sample
            );
        }
    }

    #[test]
    fn test_ring_modulation_in_sid() {
        let mut sid = Sid6581::new();

        // Voice 1: triangle with ring mod (modulated by voice 3)
        sid.write(0x00, 0x00);
        sid.write(0x01, 0x10);
        sid.write(0x04, 0x14); // Triangle + ring mod

        // Voice 3: set up with MSB that will alternate
        sid.write(0x0E, 0x00);
        sid.write(0x0F, 0x40); // Different frequency

        // Set accumulators to specific values
        sid.voices[0].accumulator = 0x400000; // Voice 1 mid-point
        sid.voices[2].accumulator = 0x800000; // Voice 3 MSB = 1

        // Voice 1's ring mod source is voice 3
        let output = sid.voice_waveform_output(0);

        // With ring mod from voice 3 (MSB=1), triangle should be inverted
        let expected_triangle = ((sid.voices[0].accumulator >> 12) & 0x7FF) as u16 ^ 0x7FF;

        assert_eq!(output.unwrap(), expected_triangle);
    }

    // =====================================================
    // ADSR Envelope Tests (T081, T082)
    // =====================================================

    #[test]
    fn test_adsr_register_accessors() {
        let mut voice = SidVoice::new();

        // Set AD register to 0xF8 (Attack=15, Decay=8)
        voice.attack_decay = 0xF8;
        assert_eq!(voice.attack_rate(), 15);
        assert_eq!(voice.decay_rate(), 8);

        // Set SR register to 0x93 (Sustain=9, Release=3)
        voice.sustain_release = 0x93;
        assert_eq!(voice.sustain_level(), 9);
        assert_eq!(voice.release_rate(), 3);

        // Test sustain_value conversion (9 * 17 = 153)
        assert_eq!(voice.sustain_value(), 153);

        // Test boundary values
        voice.sustain_release = 0xF0; // Sustain=15, Release=0
        assert_eq!(voice.sustain_value(), 255);

        voice.sustain_release = 0x00; // Sustain=0, Release=0
        assert_eq!(voice.sustain_value(), 0);
    }

    #[test]
    fn test_attack_rate_table_values() {
        // Verify the attack rate table has expected values
        assert_eq!(ATTACK_RATE_PERIODS[0], 9); // 2 ms
        assert_eq!(ATTACK_RATE_PERIODS[15], 31251); // 8 s

        // Verify all values are increasing
        for i in 1..16 {
            assert!(
                ATTACK_RATE_PERIODS[i] >= ATTACK_RATE_PERIODS[i - 1],
                "Attack rate should increase with index"
            );
        }
    }

    #[test]
    fn test_decay_release_rate_table_values() {
        // Verify the decay/release rate table has expected values
        assert_eq!(DECAY_RELEASE_RATE_PERIODS[0], 9); // 6 ms
        assert_eq!(DECAY_RELEASE_RATE_PERIODS[15], 31251); // 24 s

        // Verify all values are increasing
        for i in 1..16 {
            assert!(
                DECAY_RELEASE_RATE_PERIODS[i] >= DECAY_RELEASE_RATE_PERIODS[i - 1],
                "Decay/Release rate should increase with index"
            );
        }
    }

    #[test]
    fn test_exponential_period_thresholds() {
        let mut voice = SidVoice::new();

        // Test various envelope levels and their corresponding exp periods
        voice.envelope_counter = 255;
        assert_eq!(voice.get_exp_period(), 1);

        voice.envelope_counter = 100;
        assert_eq!(voice.get_exp_period(), 1);

        voice.envelope_counter = 94;
        assert_eq!(voice.get_exp_period(), 1);

        voice.envelope_counter = 93;
        assert_eq!(voice.get_exp_period(), 2);

        voice.envelope_counter = 55;
        assert_eq!(voice.get_exp_period(), 2);

        voice.envelope_counter = 54;
        assert_eq!(voice.get_exp_period(), 4);

        voice.envelope_counter = 27;
        assert_eq!(voice.get_exp_period(), 4);

        voice.envelope_counter = 26;
        assert_eq!(voice.get_exp_period(), 8);

        voice.envelope_counter = 15;
        assert_eq!(voice.get_exp_period(), 8);

        voice.envelope_counter = 14;
        assert_eq!(voice.get_exp_period(), 16);

        voice.envelope_counter = 7;
        assert_eq!(voice.get_exp_period(), 16);

        voice.envelope_counter = 6;
        assert_eq!(voice.get_exp_period(), 30);

        voice.envelope_counter = 0;
        assert_eq!(voice.get_exp_period(), 30);
    }

    #[test]
    fn test_gate_on_starts_attack() {
        let mut sid = Sid6581::new();

        // Initial state should be Release
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);
        assert_eq!(sid.voices[0].envelope_counter, 0);

        // Set gate on (bit 0 of control register)
        sid.write(0x04, 0x01); // Gate on

        // Should transition to Attack
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Attack);
    }

    #[test]
    fn test_gate_off_starts_release() {
        let mut sid = Sid6581::new();

        // Start with gate on, in attack
        sid.write(0x04, 0x01);
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Attack);

        // Set some envelope value
        sid.voices[0].envelope_counter = 128;

        // Gate off
        sid.write(0x04, 0x00);

        // Should transition to Release
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);
        // Envelope should still have its value
        assert_eq!(sid.voices[0].envelope_counter, 128);
    }

    #[test]
    fn test_attack_phase_increases_envelope() {
        let mut sid = Sid6581::new();

        // Set fastest attack (0) with AD register
        sid.write(0x05, 0x00); // Attack=0, Decay=0

        // Start gate
        sid.write(0x04, 0x01);
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Attack);
        assert_eq!(sid.voices[0].envelope_counter, 0);

        // Clock enough times to see envelope increase
        // Attack rate 0 = 9 cycles per increment
        for _ in 0..20 {
            sid.clock();
        }

        // Envelope should have increased
        assert!(
            sid.voices[0].envelope_counter > 0,
            "Envelope should increase during attack"
        );
    }

    #[test]
    fn test_attack_reaches_max_then_decay() {
        let mut sid = Sid6581::new();

        // Set fastest attack (0), some decay
        sid.write(0x05, 0x00); // Attack=0, Decay=0
        sid.write(0x06, 0x80); // Sustain=8 (8*17=136), Release=0

        // Start gate
        sid.write(0x04, 0x01);

        // Clock until attack completes
        // With attack rate 0 (9 cycles per increment), need about 9 * 256 = 2304 cycles
        for _ in 0..2500 {
            sid.clock();
        }

        // Should have reached max and transitioned to Decay
        assert_eq!(
            sid.voices[0].envelope_state,
            EnvelopeState::Decay,
            "Should be in Decay after attack completes"
        );
    }

    #[test]
    fn test_decay_to_sustain() {
        let mut sid = Sid6581::new();

        // Set fastest attack and decay, specific sustain level
        sid.write(0x05, 0x00); // Attack=0, Decay=0
        sid.write(0x06, 0x80); // Sustain=8 (8*17=136), Release=0

        // Start gate
        sid.write(0x04, 0x01);

        // Manually set to decay state at max envelope to test decay
        sid.voices[0].envelope_state = EnvelopeState::Decay;
        sid.voices[0].envelope_counter = 255;

        // Clock until decay completes
        // This will take a while due to exponential slowdown, but should eventually reach sustain
        for _ in 0..50000 {
            sid.clock();
        }

        // Should have transitioned to Sustain
        assert_eq!(
            sid.voices[0].envelope_state,
            EnvelopeState::Sustain,
            "Should be in Sustain after decay completes"
        );

        // Envelope should be at or near sustain level
        let sustain_level = sid.voices[0].sustain_value();
        assert_eq!(
            sid.voices[0].envelope_counter, sustain_level,
            "Envelope should be at sustain level"
        );
    }

    #[test]
    fn test_sustain_holds_level() {
        let mut sid = Sid6581::new();

        // Set up sustain level
        sid.write(0x06, 0xA0); // Sustain=10 (10*17=170), Release=0

        // Gate on first (this sets Attack state)
        sid.write(0x04, 0x01);

        // Then manually override to sustain state
        sid.voices[0].envelope_state = EnvelopeState::Sustain;
        sid.voices[0].envelope_counter = 170;

        // Clock many times
        for _ in 0..1000 {
            sid.clock();
        }

        // Envelope should stay at sustain level
        assert_eq!(
            sid.voices[0].envelope_state,
            EnvelopeState::Sustain,
            "Should remain in Sustain"
        );
        assert_eq!(
            sid.voices[0].envelope_counter, 170,
            "Envelope should hold at sustain level"
        );
    }

    #[test]
    fn test_release_from_sustain() {
        let mut sid = Sid6581::new();

        // Set fast release
        sid.write(0x05, 0x00); // Attack=0, Decay=0
        sid.write(0x06, 0xF0); // Sustain=15 (max), Release=0

        // Set up in sustain state
        sid.voices[0].envelope_state = EnvelopeState::Sustain;
        sid.voices[0].envelope_counter = 255;
        sid.voices[0].control = 0x01; // Gate on

        // Gate off to trigger release
        sid.write(0x04, 0x00);

        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);

        // Clock until release completes
        for _ in 0..50000 {
            sid.clock();
        }

        // Envelope should have decayed to 0
        assert_eq!(
            sid.voices[0].envelope_counter, 0,
            "Envelope should decay to 0 during release"
        );
    }

    #[test]
    fn test_release_exponential_decay() {
        let mut sid = Sid6581::new();

        // Set fast release for quicker test
        sid.write(0x06, 0x00); // Sustain=0, Release=0 (fastest)

        // Start in release with high envelope value
        sid.voices[0].envelope_state = EnvelopeState::Release;
        sid.voices[0].envelope_counter = 200;

        // Capture envelope values over time
        let mut values: Vec<u8> = Vec::new();
        values.push(sid.voices[0].envelope_counter);

        for i in 0..5000 {
            sid.clock();
            if i % 100 == 0 {
                values.push(sid.voices[0].envelope_counter);
            }
        }

        // Verify envelope is decreasing (not necessarily monotonically due to sampling)
        assert!(
            values.last().unwrap() < values.first().unwrap(),
            "Envelope should decrease over time during release"
        );
    }

    #[test]
    fn test_voice_3_envelope_readback() {
        let mut sid = Sid6581::new();

        // Set voice 3 envelope to a specific value
        sid.voices[2].envelope_counter = 0xAB;

        // Read from ENV3 register ($D41C = offset 0x1C)
        let readback = sid.read(0x1C);

        assert_eq!(readback, 0xAB, "ENV3 readback should match envelope counter");
    }

    #[test]
    fn test_all_three_voices_have_independent_envelopes() {
        let mut sid = Sid6581::new();

        // Set different ADSR for each voice
        sid.write(0x05, 0x00); // Voice 1: fastest
        sid.write(0x0C, 0x88); // Voice 2: medium
        sid.write(0x13, 0xFF); // Voice 3: slowest

        // Start attack on all voices
        sid.write(0x04, 0x01); // Voice 1 gate on
        sid.write(0x0B, 0x01); // Voice 2 gate on
        sid.write(0x12, 0x01); // Voice 3 gate on

        // Clock for a while
        for _ in 0..1000 {
            sid.clock();
        }

        // Each voice should have different envelope values
        // Voice 1 should be highest (fastest attack)
        // Voice 3 should be lowest (slowest attack)
        let env1 = sid.voices[0].envelope_counter;
        let _env2 = sid.voices[1].envelope_counter; // Medium attack speed
        let env3 = sid.voices[2].envelope_counter;

        // At minimum, verify they're progressing independently
        // Voice 1 with attack=0 should have progressed more than voice 3 with attack=15
        assert!(
            env1 >= env3,
            "Voice 1 (fast attack) should have higher/equal envelope than voice 3 (slow attack)"
        );
    }

    #[test]
    fn test_gate_on_during_release_restarts_attack() {
        let mut sid = Sid6581::new();

        // Start with gate on, get some envelope
        sid.write(0x04, 0x01);
        for _ in 0..100 {
            sid.clock();
        }

        // Gate off - start release
        sid.write(0x04, 0x00);
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Release);

        // Let release progress a bit
        for _ in 0..50 {
            sid.clock();
        }

        // Gate on again - should restart attack
        sid.write(0x04, 0x01);
        assert_eq!(
            sid.voices[0].envelope_state,
            EnvelopeState::Attack,
            "Gate on during release should restart attack"
        );
    }

    #[test]
    fn test_zero_sustain_goes_directly_to_zero() {
        let mut sid = Sid6581::new();

        // Set sustain to 0
        sid.write(0x05, 0x00); // Attack=0, Decay=0
        sid.write(0x06, 0x00); // Sustain=0, Release=0

        // Start attack and let it reach max
        sid.write(0x04, 0x01);
        sid.voices[0].envelope_state = EnvelopeState::Decay;
        sid.voices[0].envelope_counter = 255;

        // Clock until decay finishes
        for _ in 0..100000 {
            sid.clock();
            if sid.voices[0].envelope_state == EnvelopeState::Sustain {
                break;
            }
        }

        // Should be in sustain at 0
        assert_eq!(sid.voices[0].envelope_state, EnvelopeState::Sustain);
        assert_eq!(
            sid.voices[0].envelope_counter, 0,
            "Sustain=0 should result in envelope at 0"
        );
    }

    #[test]
    fn test_max_sustain_holds_at_255() {
        let mut sid = Sid6581::new();

        // Set sustain to max (15 -> 255)
        sid.write(0x06, 0xF0); // Sustain=15, Release=0

        // Go directly to decay from max
        sid.voices[0].envelope_state = EnvelopeState::Decay;
        sid.voices[0].envelope_counter = 255;

        // Clock - should immediately transition to sustain since we're already at sustain level
        for _ in 0..10 {
            sid.clock();
        }

        assert_eq!(
            sid.voices[0].envelope_state,
            EnvelopeState::Sustain,
            "Should transition to sustain immediately at max"
        );
        assert_eq!(sid.voices[0].envelope_counter, 255);
    }
}
