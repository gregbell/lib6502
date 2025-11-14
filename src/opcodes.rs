//! # Opcode Metadata Table
//!
//! This module contains the complete 256-entry opcode metadata table that serves as the
//! single source of truth for all 6502 instruction information.
//!
//! The table covers:
//! - **151 documented instructions** - Official NMOS 6502 opcodes
//! - **105 illegal/undocumented opcodes** - Marked with "???" mnemonic
//!
//! Each opcode entry includes:
//! - Mnemonic (instruction name)
//! - Addressing mode
//! - Base cycle cost (excluding page-crossing penalties)
//! - Instruction size in bytes
//! - Implementation status flag

use crate::addressing::AddressingMode;

/// Metadata for a single 6502 opcode.
///
/// This struct contains all static information about an instruction needed for
/// decoding and execution planning.
///
/// # Fields
///
/// - `mnemonic`: Three-letter instruction name (e.g., "LDA", "STA", "???")
/// - `addressing_mode`: How the instruction interprets operand bytes
/// - `base_cycles`: Minimum cycle cost (page-crossing penalties added dynamically)
/// - `size_bytes`: Total instruction size including opcode and operands (1-3 bytes)
/// - `implemented`: Whether this instruction is currently implemented
///
/// # Examples
///
/// ```
/// use cpu6502::{OPCODE_TABLE, AddressingMode};
///
/// // Look up LDA immediate (opcode 0xA9)
/// let lda_imm = &OPCODE_TABLE[0xA9];
/// assert_eq!(lda_imm.mnemonic, "LDA");
/// assert_eq!(lda_imm.addressing_mode, AddressingMode::Immediate);
/// assert_eq!(lda_imm.base_cycles, 2);
/// assert_eq!(lda_imm.size_bytes, 2);
/// assert_eq!(lda_imm.implemented, false); // Not implemented in this feature
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeMetadata {
    /// Instruction mnemonic (e.g., "LDA", "STA", "???" for illegal opcodes).
    pub mnemonic: &'static str,

    /// Addressing mode for this instruction.
    pub addressing_mode: AddressingMode,

    /// Base cycle cost (before page crossing penalties).
    ///
    /// Documented instructions have cycles in the range 1-7.
    /// Illegal opcodes are marked with 0 cycles.
    pub base_cycles: u8,

    /// Total instruction size in bytes (opcode + operands).
    ///
    /// - 1 byte: Implicit, Accumulator modes
    /// - 2 bytes: Immediate, Zero Page, Relative, Indexed Indirect modes
    /// - 3 bytes: Absolute, Indirect modes
    pub size_bytes: u8,

    /// Whether this instruction is currently implemented.
    ///
    /// All entries are `false` in this foundational feature. Future instruction
    /// implementation features will set this to `true` for implemented opcodes.
    pub implemented: bool,
}

/// Complete 256-entry opcode metadata table indexed by opcode byte value.
///
/// This table serves as the single source of truth for all 6502 instruction metadata.
/// Index into the array using the opcode byte to retrieve its metadata.
///
/// # Organization
///
/// - **Documented opcodes** (151 entries): Official NMOS 6502 instructions with accurate
///   mnemonic, addressing mode, cycle cost, and size information.
/// - **Illegal opcodes** (105 entries): Undocumented opcodes marked with "???" mnemonic,
///   0 cycles, size 1, and `implemented: false`.
///
/// # Examples
///
/// ```
/// use cpu6502::OPCODE_TABLE;
///
/// // Look up instruction metadata
/// let brk = &OPCODE_TABLE[0x00];
/// println!("{} - {} cycles, {} bytes", brk.mnemonic, brk.base_cycles, brk.size_bytes);
/// // Output: BRK - 7 cycles, 1 bytes
///
/// // Check if opcode is illegal
/// let illegal = &OPCODE_TABLE[0x02];
/// assert_eq!(illegal.mnemonic, "???");
/// assert_eq!(illegal.base_cycles, 0);
/// ```
///
/// # Data Source
///
/// Extracted from the official 6502 reference documentation at
/// `docs/6502-reference/Reference.md` which documents all 56 instruction types
/// across their various addressing modes (151 total documented opcodes).
pub const OPCODE_TABLE: [OpcodeMetadata; 256] = [
    // 0x00
    OpcodeMetadata {
        mnemonic: "BRK",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 7,
        size_bytes: 1,
        implemented: true,
    },
    // 0x01
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x02 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x03 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x04 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x05
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0x06
    OpcodeMetadata {
        mnemonic: "ASL",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0x07 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x08
    OpcodeMetadata {
        mnemonic: "PHP",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 3,
        size_bytes: 1,
        implemented: false,
    },
    // 0x09
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: false,
    },
    // 0x0A
    OpcodeMetadata {
        mnemonic: "ASL",
        addressing_mode: AddressingMode::Accumulator,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0x0B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x0C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x0D
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x0E
    OpcodeMetadata {
        mnemonic: "ASL",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: true,
    },
    // 0x0F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x10
    OpcodeMetadata {
        mnemonic: "BPL",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x11
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0x12 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x13 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x14 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x15
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0x16
    OpcodeMetadata {
        mnemonic: "ASL",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0x17 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x18
    OpcodeMetadata {
        mnemonic: "CLC",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0x19
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x1A - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x1B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x1C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x1D
    OpcodeMetadata {
        mnemonic: "ORA",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x1E
    OpcodeMetadata {
        mnemonic: "ASL",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: true,
    },
    // 0x1F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x20
    OpcodeMetadata {
        mnemonic: "JSR",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: false,
    },
    // 0x21
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0x22 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x23 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x24
    OpcodeMetadata {
        mnemonic: "BIT",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0x25
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0x26
    OpcodeMetadata {
        mnemonic: "ROL",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0x27 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x28
    OpcodeMetadata {
        mnemonic: "PLP",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 4,
        size_bytes: 1,
        implemented: false,
    },
    // 0x29
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x2A
    OpcodeMetadata {
        mnemonic: "ROL",
        addressing_mode: AddressingMode::Accumulator,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x2B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x2C
    OpcodeMetadata {
        mnemonic: "BIT",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x2D
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x2E
    OpcodeMetadata {
        mnemonic: "ROL",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: false,
    },
    // 0x2F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x30
    OpcodeMetadata {
        mnemonic: "BMI",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x31
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0x32 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x33 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x34 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x35
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: true,
    },
    // 0x36
    OpcodeMetadata {
        mnemonic: "ROL",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x37 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x38
    OpcodeMetadata {
        mnemonic: "SEC",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x39
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x3A - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x3B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x3C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x3D
    OpcodeMetadata {
        mnemonic: "AND",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x3E
    OpcodeMetadata {
        mnemonic: "ROL",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: false,
    },
    // 0x3F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x40
    OpcodeMetadata {
        mnemonic: "RTI",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 6,
        size_bytes: 1,
        implemented: false,
    },
    // 0x41
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0x42 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x43 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x44 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x45
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0x46
    OpcodeMetadata {
        mnemonic: "LSR",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0x47 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x48
    OpcodeMetadata {
        mnemonic: "PHA",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 3,
        size_bytes: 1,
        implemented: false,
    },
    // 0x49
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x4A
    OpcodeMetadata {
        mnemonic: "LSR",
        addressing_mode: AddressingMode::Accumulator,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x4B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x4C
    OpcodeMetadata {
        mnemonic: "JMP",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 3,
        size_bytes: 3,
        implemented: false,
    },
    // 0x4D
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x4E
    OpcodeMetadata {
        mnemonic: "LSR",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: false,
    },
    // 0x4F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x50
    OpcodeMetadata {
        mnemonic: "BVC",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x51
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0x52 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x53 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x54 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x55
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: true,
    },
    // 0x56
    OpcodeMetadata {
        mnemonic: "LSR",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x57 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x58
    OpcodeMetadata {
        mnemonic: "CLI",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0x59
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x5A - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x5B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x5C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x5D
    OpcodeMetadata {
        mnemonic: "EOR",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x5E
    OpcodeMetadata {
        mnemonic: "LSR",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: false,
    },
    // 0x5F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x60
    OpcodeMetadata {
        mnemonic: "RTS",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 6,
        size_bytes: 1,
        implemented: false,
    },
    // 0x61
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0x62 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x63 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x64 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x65
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0x66
    OpcodeMetadata {
        mnemonic: "ROR",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0x67 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x68
    OpcodeMetadata {
        mnemonic: "PLA",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 4,
        size_bytes: 1,
        implemented: false,
    },
    // 0x69
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x6A
    OpcodeMetadata {
        mnemonic: "ROR",
        addressing_mode: AddressingMode::Accumulator,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x6B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x6C
    OpcodeMetadata {
        mnemonic: "JMP",
        addressing_mode: AddressingMode::Indirect,
        base_cycles: 5,
        size_bytes: 3,
        implemented: false,
    },
    // 0x6D
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x6E
    OpcodeMetadata {
        mnemonic: "ROR",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: false,
    },
    // 0x6F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x70
    OpcodeMetadata {
        mnemonic: "BVS",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x71
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0x72 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x73 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x74 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x75
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: true,
    },
    // 0x76
    OpcodeMetadata {
        mnemonic: "ROR",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x77 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x78
    OpcodeMetadata {
        mnemonic: "SEI",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x79
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x7A - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x7B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x7C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x7D
    OpcodeMetadata {
        mnemonic: "ADC",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0x7E
    OpcodeMetadata {
        mnemonic: "ROR",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: false,
    },
    // 0x7F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x80 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x81
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x82 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x83 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x84
    OpcodeMetadata {
        mnemonic: "STY",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0x85
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0x86
    OpcodeMetadata {
        mnemonic: "STX",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0x87 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x88
    OpcodeMetadata {
        mnemonic: "DEY",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0x89 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x8A
    OpcodeMetadata {
        mnemonic: "TXA",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x8B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x8C
    OpcodeMetadata {
        mnemonic: "STY",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x8D
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x8E
    OpcodeMetadata {
        mnemonic: "STX",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0x8F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x90
    OpcodeMetadata {
        mnemonic: "BCC",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0x91
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0x92 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x93 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x94
    OpcodeMetadata {
        mnemonic: "STY",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0x95
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0x96
    OpcodeMetadata {
        mnemonic: "STX",
        addressing_mode: AddressingMode::ZeroPageY,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0x97 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x98
    OpcodeMetadata {
        mnemonic: "TYA",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x99
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 5,
        size_bytes: 3,
        implemented: false,
    },
    // 0x9A
    OpcodeMetadata {
        mnemonic: "TXS",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0x9B - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x9C - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x9D
    OpcodeMetadata {
        mnemonic: "STA",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 5,
        size_bytes: 3,
        implemented: false,
    },
    // 0x9E - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0x9F - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xA0
    OpcodeMetadata {
        mnemonic: "LDY",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA1
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA2
    OpcodeMetadata {
        mnemonic: "LDX",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xA4
    OpcodeMetadata {
        mnemonic: "LDY",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA5
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA6
    OpcodeMetadata {
        mnemonic: "LDX",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0xA7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xA8
    OpcodeMetadata {
        mnemonic: "TAY",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xA9
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: false,
    },
    // 0xAA
    OpcodeMetadata {
        mnemonic: "TAX",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xAB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xAC
    OpcodeMetadata {
        mnemonic: "LDY",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xAD
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xAE
    OpcodeMetadata {
        mnemonic: "LDX",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xAF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xB0
    OpcodeMetadata {
        mnemonic: "BCS",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xB1
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0xB2 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xB3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xB4
    OpcodeMetadata {
        mnemonic: "LDY",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0xB5
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0xB6
    OpcodeMetadata {
        mnemonic: "LDX",
        addressing_mode: AddressingMode::ZeroPageY,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0xB7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xB8
    OpcodeMetadata {
        mnemonic: "CLV",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0xB9
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xBA
    OpcodeMetadata {
        mnemonic: "TSX",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xBB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xBC
    OpcodeMetadata {
        mnemonic: "LDY",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xBD
    OpcodeMetadata {
        mnemonic: "LDA",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xBE
    OpcodeMetadata {
        mnemonic: "LDX",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xBF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xC0
    OpcodeMetadata {
        mnemonic: "CPY",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xC1
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0xC2 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xC3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xC4
    OpcodeMetadata {
        mnemonic: "CPY",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0xC5
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0xC6
    OpcodeMetadata {
        mnemonic: "DEC",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0xC7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xC8
    OpcodeMetadata {
        mnemonic: "INY",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xC9
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xCA
    OpcodeMetadata {
        mnemonic: "DEX",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0xCB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xCC
    OpcodeMetadata {
        mnemonic: "CPY",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0xCD
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0xCE
    OpcodeMetadata {
        mnemonic: "DEC",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: true,
    },
    // 0xCF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xD0
    OpcodeMetadata {
        mnemonic: "BNE",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xD1
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: true,
    },
    // 0xD2 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xD3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xD4 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xD5
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: true,
    },
    // 0xD6
    OpcodeMetadata {
        mnemonic: "DEC",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: true,
    },
    // 0xD7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xD8
    OpcodeMetadata {
        mnemonic: "CLD",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0xD9
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0xDA - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xDB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xDC - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xDD
    OpcodeMetadata {
        mnemonic: "CMP",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0xDE
    OpcodeMetadata {
        mnemonic: "DEC",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: true,
    },
    // 0xDF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xE0
    OpcodeMetadata {
        mnemonic: "CPX",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xE1
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::IndirectX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0xE2 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xE3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xE4
    OpcodeMetadata {
        mnemonic: "CPX",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: true,
    },
    // 0xE5
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 3,
        size_bytes: 2,
        implemented: false,
    },
    // 0xE6
    OpcodeMetadata {
        mnemonic: "INC",
        addressing_mode: AddressingMode::ZeroPage,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0xE7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xE8
    OpcodeMetadata {
        mnemonic: "INX",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: true,
    },
    // 0xE9
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::Immediate,
        base_cycles: 2,
        size_bytes: 2,
        implemented: false,
    },
    // 0xEA
    OpcodeMetadata {
        mnemonic: "NOP",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xEB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xEC
    OpcodeMetadata {
        mnemonic: "CPX",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: true,
    },
    // 0xED
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xEE
    OpcodeMetadata {
        mnemonic: "INC",
        addressing_mode: AddressingMode::Absolute,
        base_cycles: 6,
        size_bytes: 3,
        implemented: false,
    },
    // 0xEF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF0
    OpcodeMetadata {
        mnemonic: "BEQ",
        addressing_mode: AddressingMode::Relative,
        base_cycles: 2,
        size_bytes: 2,
        implemented: true,
    },
    // 0xF1
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::IndirectY,
        base_cycles: 5,
        size_bytes: 2,
        implemented: false,
    },
    // 0xF2 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF3 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF4 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF5
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 4,
        size_bytes: 2,
        implemented: false,
    },
    // 0xF6
    OpcodeMetadata {
        mnemonic: "INC",
        addressing_mode: AddressingMode::ZeroPageX,
        base_cycles: 6,
        size_bytes: 2,
        implemented: false,
    },
    // 0xF7 - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF8
    OpcodeMetadata {
        mnemonic: "SED",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 2,
        size_bytes: 1,
        implemented: false,
    },
    // 0xF9
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::AbsoluteY,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xFA - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xFB - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xFC - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
    // 0xFD
    OpcodeMetadata {
        mnemonic: "SBC",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 4,
        size_bytes: 3,
        implemented: false,
    },
    // 0xFE
    OpcodeMetadata {
        mnemonic: "INC",
        addressing_mode: AddressingMode::AbsoluteX,
        base_cycles: 7,
        size_bytes: 3,
        implemented: false,
    },
    // 0xFF - Illegal/Undocumented opcode
    OpcodeMetadata {
        mnemonic: "???",
        addressing_mode: AddressingMode::Implicit,
        base_cycles: 0,
        size_bytes: 1,
        implemented: false,
    },
];
