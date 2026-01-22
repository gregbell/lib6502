//! WebAssembly bindings for the C64 emulator.
//!
//! This module provides JavaScript-callable APIs for running the C64 emulator
//! in a web browser via WebAssembly.

use wasm_bindgen::prelude::*;

use crate::system::{C64System, Region};

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
    pub fn get_framebuffer(&self) -> Vec<u8> {
        self.system.get_framebuffer()
    }

    /// Get audio samples generated since last call.
    /// Returns f32 samples suitable for Web Audio API.
    #[wasm_bindgen]
    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.system.get_audio_samples()
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

    /// Set joystick 1 state (directly on CIA1 port B).
    /// Bits: 0=up, 1=down, 2=left, 3=right, 4=fire (accent-low).
    #[wasm_bindgen]
    pub fn set_joystick1(&mut self, state: u8) {
        self.system.set_joystick1(state);
    }

    /// Set joystick 2 state (directly on CIA1 port A).
    #[wasm_bindgen]
    pub fn set_joystick2(&mut self, state: u8) {
        self.system.set_joystick2(state);
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
}

impl Default for C64Emulator {
    fn default() -> Self {
        Self::new()
    }
}
