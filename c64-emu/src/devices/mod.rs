//! C64-specific hardware devices implementing the lib6502 Device trait.
//!
//! Each device emulates a specific chip from the Commodore 64:
//!
//! - [`VicII`]: MOS 6569/6567 Video Interface Chip (graphics, sprites, raster)
//! - [`Sid6581`]: MOS 6581 Sound Interface Device (audio synthesis)
//! - [`Cia6526`]: MOS 6526 Complex Interface Adapter (timers, I/O, keyboard)
//! - [`Port6510`]: 6510 CPU I/O port (memory bank switching)
//! - [`ColorRam`]: 1KB color RAM for VIC-II
//!
//! All devices implement `lib6502::Device` for memory-mapped access.

mod cia;
mod color_ram;
mod port_6510;
mod sid;
mod vic_ii;

pub use cia::Cia6526;
pub use color_ram::ColorRam;
pub use port_6510::Port6510;
pub use sid::Sid6581;
pub use vic_ii::VicII;
