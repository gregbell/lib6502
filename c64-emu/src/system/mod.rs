//! C64 system integration, timing, and orchestration.
//!
//! This module provides the top-level `C64System` that coordinates all
//! hardware components (CPU, VIC-II, SID, CIAs) and manages timing.

mod c64_memory;
mod c64_system;
mod keyboard;

pub use c64_memory::C64Memory;
pub use c64_system::{C64System, Region};
pub use keyboard::{keys, Keyboard};
