//! # Commodore 64 Emulator
//!
//! A fully functional Commodore 64 emulator built on the lib6502 CPU core,
//! designed to run in the browser via WebAssembly.
//!
//! ## Architecture
//!
//! This crate implements the C64's custom hardware as memory-mapped devices
//! following the lib6502 `Device` trait pattern:
//!
//! - **VIC-II** (MOS 6569): Video chip with sprites, graphics modes, raster interrupts
//! - **SID** (MOS 6581): Sound chip with 3 voices, filters, ADSR envelopes
//! - **CIA** (MOS 6526): Timer/I/O chips for keyboard, joystick, and disk drive
//! - **Port 6510**: CPU I/O port for memory bank switching
//! - **Color RAM**: 1KB of 4-bit color memory
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use c64_emu::{C64System, Region};
//!
//! // Create a new C64 system (PAL region)
//! let mut c64 = C64System::new(Region::PAL);
//!
//! // Load ROMs (user must provide these)
//! c64.load_roms(&basic_rom, &kernal_rom, &char_rom).unwrap();
//!
//! // Run one frame of emulation
//! c64.step_frame();
//!
//! // Get the framebuffer for display
//! let fb = c64.framebuffer();
//! ```
//!
//! ## Module Organization
//!
//! - `devices`: Hardware device implementations (VIC-II, SID, CIA, etc.)
//! - `system`: C64 system integration, timing, and orchestration

pub mod devices;
pub mod system;

// WASM bindings (optional, enabled with "wasm" feature)
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used types
pub use devices::{Cia6526, ColorRam, Port6510, Sid6581, VicII};
pub use system::{
    keys, map_pc_keycode, C64Memory, C64System, ChannelMode, D64Error, D64Image, Drive1541,
    DriveChannel, DriveStatus, KeyMapping, Keyboard, Region,
};
