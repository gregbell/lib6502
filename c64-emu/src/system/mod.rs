//! C64 system integration, timing, and orchestration.
//!
//! This module provides the top-level `C64System` that coordinates all
//! hardware components (CPU, VIC-II, SID, CIAs) and manages timing.

mod c64_memory;
mod c64_system;
pub mod disk_1541;
pub mod iec_bus;
mod joystick;
mod keyboard;
pub mod savestate;

pub use c64_memory::C64Memory;
pub use c64_system::{C64System, Region};
pub use disk_1541::{ChannelMode, D64Error, D64Image, Drive1541, DriveChannel, DriveStatus};
pub use iec_bus::{IecBus, IecState};
pub use joystick::{bits as joystick_bits, JoystickPorts, JoystickState};
pub use keyboard::{keys, map_pc_keycode, KeyMapping, Keyboard};
pub use savestate::{CiaState, SaveState, SidFilterState, SidVoiceState, SAVESTATE_MAGIC, SAVESTATE_VERSION};
