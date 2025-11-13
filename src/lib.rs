//! # 6502 CPU Emulator Core
//!
//! A cycle-accurate NMOS 6502 CPU emulator designed for modularity, clarity, and
//! WebAssembly portability.
//!
//! This crate provides the foundational architecture for emulating the MOS Technology 6502
//! processor, including CPU state structures, a trait-based memory bus abstraction,
//! and a table-driven opcode metadata system.
//!
//! ## Quick Start
//!
//! ```rust
//! use cpu6502::{CPU, FlatMemory, MemoryBus};
//!
//! // Create 64KB flat memory
//! let mut memory = FlatMemory::new();
//!
//! // Set reset vector to point to program start at 0x8000
//! memory.write(0xFFFC, 0x00); // Low byte
//! memory.write(0xFFFD, 0x80); // High byte
//!
//! // Initialize CPU - it will load PC from the reset vector
//! let mut cpu = CPU::new(memory);
//!
//! // Verify initial state
//! assert_eq!(cpu.pc(), 0x8000);
//! assert_eq!(cpu.sp(), 0xFD);
//! assert_eq!(cpu.flag_i(), true);
//! ```
//!
//! ## Architecture
//!
//! The emulator follows a modular architecture adhering to these principles:
//!
//! - **Modularity**: CPU state is separated from memory implementation via the `MemoryBus` trait
//! - **WebAssembly Portability**: No OS dependencies, deterministic execution
//! - **Cycle Accuracy**: Tracks cycle counts for timing-accurate emulation
//! - **Clarity & Hackability**: Simple, readable code with comprehensive documentation
//! - **Table-Driven Design**: All opcode metadata in a single source of truth
//!
//! ## Modules
//!
//! - `cpu` - CPU state and execution logic
//! - `memory` - MemoryBus trait and implementations
//! - `opcodes` - Opcode metadata table
//! - `addressing` - Addressing mode enumerations
//!
//! For detailed usage examples, see the `examples/` directory and the
//! [quickstart guide](../specs/001-cpu-core-foundation/quickstart.md).

pub mod addressing;
pub mod cpu;
pub mod memory;
pub mod opcodes;

// Internal instruction implementations (not part of public API)
mod instructions;

// Re-export public API
pub use addressing::AddressingMode;
pub use cpu::CPU;
pub use memory::{FlatMemory, MemoryBus};
pub use opcodes::{OpcodeMetadata, OPCODE_TABLE};

/// Errors that can occur during CPU execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    /// Instruction opcode has not been implemented yet.
    ///
    /// Contains the opcode byte value for debugging purposes.
    UnimplementedOpcode(u8),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::UnimplementedOpcode(opcode) => {
                write!(f, "Opcode 0x{:02X} is not implemented", opcode)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}
