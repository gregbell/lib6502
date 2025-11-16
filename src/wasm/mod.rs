//! WebAssembly bindings for the lib6502 emulator.
//!
//! This module provides JavaScript-callable interfaces to the 6502 CPU emulator,
//! enabling browser-based execution of 6502 assembly code.

#[cfg(feature = "wasm")]
pub mod api;

#[cfg(feature = "wasm")]
pub use api::Emulator6502;
