//! WebAssembly bindings for the C64 emulator.
//!
//! This module provides JavaScript-callable APIs for running the C64 emulator
//! in a web browser via WebAssembly.

use wasm_bindgen::prelude::*;

use crate::system::{map_pc_keycode, C64System, Region};

/// Screen width in pixels.
pub const SCREEN_WIDTH: u32 = 320;

/// Screen height in pixels.
pub const SCREEN_HEIGHT: u32 = 200;

/// Framebuffer size in bytes (320 × 200 indexed color pixels).
pub const FRAMEBUFFER_SIZE: u32 = SCREEN_WIDTH * SCREEN_HEIGHT;

/// WASM wrapper for the C64 emulator system.
#[wasm_bindgen]
pub struct C64Emulator {
    system: C64System,
}

#[wasm_bindgen]
impl C64Emulator {
    /// Create a new C64 emulator instance (PAL region by default).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            system: C64System::new(Region::PAL),
        }
    }

    /// Create a new C64 emulator with specified region.
    #[wasm_bindgen]
    pub fn new_with_region(ntsc: bool) -> Self {
        let region = if ntsc { Region::NTSC } else { Region::PAL };
        Self {
            system: C64System::new(region),
        }
    }

    /// Load KERNAL ROM (8192 bytes).
    #[wasm_bindgen]
    pub fn load_kernal(&mut self, data: &[u8]) -> bool {
        self.system.load_kernal(data)
    }

    /// Load BASIC ROM (8192 bytes).
    #[wasm_bindgen]
    pub fn load_basic(&mut self, data: &[u8]) -> bool {
        self.system.load_basic(data)
    }

    /// Load Character ROM (4096 bytes).
    #[wasm_bindgen]
    pub fn load_charrom(&mut self, data: &[u8]) -> bool {
        self.system.load_charrom(data)
    }

    /// Check if all required ROMs are loaded.
    #[wasm_bindgen]
    pub fn roms_loaded(&mut self) -> bool {
        self.system.roms_loaded_mut()
    }

    /// Reset the emulator to initial state.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.system.reset();
    }

    /// Run emulation for one frame.
    /// Returns the number of CPU cycles executed.
    #[wasm_bindgen]
    pub fn step_frame(&mut self) -> u32 {
        self.system.step_frame()
    }

    /// Get the current frame count.
    #[wasm_bindgen]
    pub fn frame_count(&self) -> u64 {
        self.system.frame_count()
    }

    /// Check if the emulator is running.
    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.system.is_running()
    }

    /// Start the emulator.
    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.system.start();
    }

    /// Stop/pause the emulator.
    #[wasm_bindgen]
    pub fn stop(&mut self) {
        self.system.stop();
    }

    /// Get the framebuffer as a flat array of indexed colors (0-15).
    /// Returns 64000 bytes (320x200).
    #[wasm_bindgen]
    pub fn get_framebuffer(&mut self) -> Vec<u8> {
        self.system.get_framebuffer_flat()
    }

    /// Get audio samples generated since last call.
    /// Returns f32 samples suitable for Web Audio API.
    #[wasm_bindgen]
    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.system.get_audio_samples()
    }

    // =========================================================================
    // Audio API (T086-T088)
    // =========================================================================

    /// Set audio output sample rate.
    ///
    /// Affects internal SID resampling ratio. Common values are 44100 (CD quality)
    /// or 48000 (professional audio). This should be called before starting
    /// emulation, typically matching the AudioContext sample rate.
    ///
    /// # Arguments
    /// * `rate` - Sample rate in Hz (typically 44100 or 48000)
    #[wasm_bindgen]
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.system.set_sample_rate(rate);
    }

    /// Get the current audio sample rate.
    #[wasm_bindgen]
    pub fn get_sample_rate(&mut self) -> f32 {
        self.system.sample_rate()
    }

    /// Enable or disable audio generation.
    ///
    /// Disabling audio saves CPU when audio is muted, as the SID won't
    /// generate samples. The SID still processes register writes so that
    /// games continue to function correctly.
    ///
    /// # Arguments
    /// * `enabled` - `true` to enable audio, `false` to disable (mute)
    #[wasm_bindgen]
    pub fn set_audio_enabled(&mut self, enabled: bool) {
        self.system.set_audio_enabled(enabled);
    }

    /// Check if audio generation is enabled.
    #[wasm_bindgen]
    pub fn is_audio_enabled(&mut self) -> bool {
        self.system.audio_enabled()
    }

    /// Press a key on the C64 keyboard matrix.
    /// row and col are 0-7 corresponding to the C64 keyboard matrix.
    #[wasm_bindgen]
    pub fn key_down(&mut self, row: u8, col: u8) {
        self.system.key_down(row, col);
    }

    /// Release a key on the C64 keyboard matrix.
    #[wasm_bindgen]
    pub fn key_up(&mut self, row: u8, col: u8) {
        self.system.key_up(row, col);
    }

    // =========================================================================
    // Joystick API (T092-T098)
    // =========================================================================

    /// Set joystick state for a logical port.
    ///
    /// This is the main joystick API that respects port swapping settings.
    /// Most C64 games use port 2, so you'll typically call `set_joystick(2, state)`.
    ///
    /// # Arguments
    /// * `port` - Logical port number (1 or 2)
    /// * `state` - Bitmask of directions/fire (active-high):
    ///   - Bit 0 (0x01): Up
    ///   - Bit 1 (0x02): Down
    ///   - Bit 2 (0x04): Left
    ///   - Bit 3 (0x08): Right
    ///   - Bit 4 (0x10): Fire
    ///
    /// # Example
    /// ```javascript
    /// // Press up + fire on joystick 2
    /// emulator.set_joystick(2, JOY_UP | JOY_FIRE);
    /// // Release all
    /// emulator.set_joystick(2, 0);
    /// ```
    #[wasm_bindgen]
    pub fn set_joystick(&mut self, port: u8, state: u8) {
        self.system.set_joystick(port, state);
    }

    /// Set joystick 1 state (directly on CIA1 port B).
    ///
    /// This bypasses port swap logic and sets physical port 1 directly.
    /// Use `set_joystick(1, state)` if you want port swap to be respected.
    ///
    /// Bits: 0=up, 1=down, 2=left, 3=right, 4=fire (active-high).
    #[wasm_bindgen]
    pub fn set_joystick1(&mut self, state: u8) {
        self.system.set_joystick1(state);
    }

    /// Set joystick 2 state (directly on CIA1 port A).
    ///
    /// This bypasses port swap logic and sets physical port 2 directly.
    /// Use `set_joystick(2, state)` if you want port swap to be respected.
    ///
    /// Bits: 0=up, 1=down, 2=left, 3=right, 4=fire (active-high).
    #[wasm_bindgen]
    pub fn set_joystick2(&mut self, state: u8) {
        self.system.set_joystick2(state);
    }

    /// Check if joystick ports are swapped.
    ///
    /// When swapped, port 2 input maps to physical port 1 and vice versa.
    #[wasm_bindgen]
    pub fn joystick_ports_swapped(&self) -> bool {
        self.system.joystick_ports_swapped()
    }

    /// Set joystick port swap state.
    ///
    /// When swapped, port 2 input maps to physical port 1 and vice versa.
    /// This is useful for games that use port 1 instead of the typical port 2.
    #[wasm_bindgen]
    pub fn set_joystick_swap(&mut self, swapped: bool) {
        self.system.set_joystick_swap(swapped);
    }

    /// Toggle joystick port swap.
    #[wasm_bindgen]
    pub fn toggle_joystick_swap(&mut self) {
        self.system.toggle_joystick_swap();
    }

    /// Release all joystick buttons on both ports.
    #[wasm_bindgen]
    pub fn release_all_joysticks(&mut self) {
        self.system.release_all_joysticks();
    }

    /// Load a PRG file into memory.
    /// Returns the load address if successful.
    #[wasm_bindgen]
    pub fn load_prg(&mut self, data: &[u8]) -> Option<u16> {
        self.system.load_prg(data)
    }

    /// Read a byte from memory (for debugging).
    #[wasm_bindgen]
    pub fn peek(&mut self, addr: u16) -> u8 {
        self.system.peek(addr)
    }

    /// Write a byte to memory (for debugging).
    #[wasm_bindgen]
    pub fn poke(&mut self, addr: u16, value: u8) {
        self.system.poke(addr, value);
    }

    /// Get the current program counter.
    #[wasm_bindgen]
    pub fn pc(&self) -> u16 {
        self.system.pc()
    }

    // =========================================================================
    // Display API (T038-T039)
    // =========================================================================

    /// Get pointer to VIC-II framebuffer in WASM memory.
    ///
    /// Buffer is 320×200 bytes, indexed color (0-15).
    /// Use this with `new Uint8Array(wasm.memory.buffer, ptr, 64000)` in JavaScript.
    ///
    /// # Safety
    /// The returned pointer is valid as long as the C64Emulator instance exists.
    /// Do not store the pointer across calls that may reallocate the framebuffer.
    #[wasm_bindgen]
    pub fn get_framebuffer_ptr(&mut self) -> *const u8 {
        self.system.get_framebuffer_ptr()
    }

    /// Get framebuffer width in pixels.
    #[wasm_bindgen]
    pub fn get_framebuffer_width(&self) -> u32 {
        SCREEN_WIDTH
    }

    /// Get framebuffer height in pixels.
    #[wasm_bindgen]
    pub fn get_framebuffer_height(&self) -> u32 {
        SCREEN_HEIGHT
    }

    /// Get framebuffer size in bytes.
    #[wasm_bindgen]
    pub fn get_framebuffer_size(&self) -> u32 {
        FRAMEBUFFER_SIZE
    }

    /// Get current VIC-II border color (0-15).
    #[wasm_bindgen]
    pub fn get_border_color(&mut self) -> u8 {
        self.system.get_border_color()
    }

    /// Get current VIC-II raster line.
    ///
    /// Returns value 0-311 (PAL) or 0-262 (NTSC).
    #[wasm_bindgen]
    pub fn get_current_raster(&mut self) -> u16 {
        self.system.get_current_raster()
    }

    // =========================================================================
    // PC Keyboard Mapping API (T041)
    // =========================================================================

    /// Signal key press using PC keycode (DOM KeyboardEvent.code).
    ///
    /// Automatically maps PC keycodes to C64 matrix positions.
    /// Supported keycodes: KeyA-KeyZ, Digit0-Digit9, F1-F12, Space, Enter, etc.
    ///
    /// # Example keycodes
    /// - "KeyA" → A key
    /// - "Digit1" → 1 key
    /// - "Enter" → RETURN key
    /// - "Space" → SPACE key
    /// - "ShiftLeft" → Left SHIFT key
    #[wasm_bindgen]
    pub fn key_down_pc(&mut self, keycode: &str) {
        if let Some(mapping) = map_pc_keycode(keycode) {
            self.system.key_down(mapping.row, mapping.col);
        }
    }

    /// Signal key release using PC keycode (DOM KeyboardEvent.code).
    ///
    /// See `key_down_pc` for supported keycodes.
    #[wasm_bindgen]
    pub fn key_up_pc(&mut self, keycode: &str) {
        if let Some(mapping) = map_pc_keycode(keycode) {
            self.system.key_up(mapping.row, mapping.col);
        }
    }

    // =========================================================================
    // Special Keys API (T042)
    // =========================================================================

    /// Trigger RESTORE key (NMI).
    ///
    /// Unlike normal keys, RESTORE triggers a non-maskable interrupt.
    /// On a real C64, this is used to break out of infinite loops or reset.
    #[wasm_bindgen]
    pub fn restore_key(&mut self) {
        self.system.restore_key();
    }

    /// Release all keys on the keyboard.
    ///
    /// Useful when the browser tab loses focus.
    #[wasm_bindgen]
    pub fn release_all_keys(&mut self) {
        self.system.release_all_keys();
    }

    // =========================================================================
    // Disk Drive API (T060-T063)
    // =========================================================================

    /// Mount a D64 disk image in virtual drive 8.
    ///
    /// D64 is the standard disk image format for C64, containing a complete
    /// 1541 disk (170KB, 35 tracks, 683 sectors).
    ///
    /// # Arguments
    /// * `data` - Complete D64 file contents (174,848 bytes standard, 175,531 with errors)
    ///
    /// # Returns
    /// `true` if mounted successfully, `false` on error.
    #[wasm_bindgen]
    pub fn mount_d64(&mut self, data: &[u8]) -> bool {
        self.system.mount_d64(data.to_vec()).is_ok()
    }

    /// Mount a D64 disk image with detailed error reporting.
    ///
    /// Similar to `mount_d64`, but returns an error message string on failure
    /// instead of just `false`. Use this when you need to display the specific
    /// reason for a mount failure to the user.
    ///
    /// # Arguments
    /// * `data` - Complete D64 file contents (174,848 bytes standard, 175,531 with errors)
    ///
    /// # Returns
    /// `None` if mounted successfully, or `Some(error_message)` on failure.
    #[wasm_bindgen]
    pub fn mount_d64_with_error(&mut self, data: &[u8]) -> Option<String> {
        match self.system.mount_d64(data.to_vec()) {
            Ok(()) => None,
            Err(e) => Some(e),
        }
    }

    /// Unmount the current disk image from drive 8.
    #[wasm_bindgen]
    pub fn unmount_d64(&mut self) {
        self.system.unmount_d64();
    }

    /// Check if a disk image is mounted in drive 8.
    #[wasm_bindgen]
    pub fn has_mounted_disk(&self) -> bool {
        self.system.has_mounted_disk()
    }

    /// Get the name of the mounted disk (if any).
    ///
    /// Returns the disk name as stored in the D64 directory header.
    #[wasm_bindgen]
    pub fn disk_name(&self) -> Option<String> {
        self.system.disk_name()
    }

    /// Inject "RUN" command into the keyboard buffer.
    ///
    /// This simulates typing "RUN" followed by RETURN, which is useful
    /// after loading a BASIC program to automatically execute it.
    ///
    /// Typical usage pattern:
    /// 1. Load a PRG file with `load_prg()`
    /// 2. Call `inject_basic_run()` to auto-execute
    #[wasm_bindgen]
    pub fn inject_basic_run(&mut self) {
        self.system.inject_basic_run();
    }

    /// Inject a string into the keyboard buffer.
    ///
    /// This simulates typing the given string. Maximum 10 characters
    /// (the size of the C64 keyboard buffer).
    ///
    /// Characters are automatically converted from ASCII to PETSCII.
    #[wasm_bindgen]
    pub fn inject_keys(&mut self, text: &str) {
        self.system.inject_keys(text);
    }

    // =========================================================================
    // Save State API (T106-T108)
    // =========================================================================

    /// Save the complete emulator state to a byte array.
    ///
    /// Returns a binary blob that can be stored in localStorage or downloaded
    /// as a file. The format is self-contained with version information for
    /// future compatibility.
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const state = emulator.save_state();
    /// localStorage.setItem('saveSlot1', btoa(String.fromCharCode(...state)));
    /// ```
    #[wasm_bindgen]
    pub fn save_state(&mut self) -> Vec<u8> {
        use crate::system::SaveState;
        let state = SaveState::capture(&mut self.system);
        state.serialize()
    }

    /// Load emulator state from a previously saved byte array.
    ///
    /// Returns `true` if the state was loaded successfully, `false` on error.
    /// Errors can occur if:
    /// - The data is too short or corrupted
    /// - The version is incompatible
    /// - The format is invalid
    ///
    /// # Arguments
    /// * `data` - Previously saved state from `save_state()`
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const base64 = localStorage.getItem('saveSlot1');
    /// const state = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
    /// if (!emulator.load_state(state)) {
    ///     console.error('Failed to load state');
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn load_state(&mut self, data: &[u8]) -> bool {
        use crate::system::SaveState;
        match SaveState::deserialize(data) {
            Ok(state) => state.restore(&mut self.system).is_ok(),
            Err(_) => false,
        }
    }

    /// Get the approximate size of a save state in bytes.
    ///
    /// Useful for UI display showing how much space states consume.
    /// The actual size may vary slightly.
    #[wasm_bindgen]
    pub fn get_state_size(&self) -> usize {
        use crate::system::SaveState;
        SaveState::estimated_size()
    }

    /// Get the current save state format version.
    ///
    /// Useful for displaying compatibility information in the UI.
    #[wasm_bindgen]
    pub fn get_state_version(&self) -> u32 {
        crate::system::SAVESTATE_VERSION
    }
}

impl Default for C64Emulator {
    fn default() -> Self {
        Self::new()
    }
}
