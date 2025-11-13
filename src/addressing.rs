//! # Addressing Modes
//!
//! This module defines the 13 addressing modes supported by the 6502 processor.
//! Each mode determines how the CPU interprets operand bytes and calculates
//! effective addresses.

/// 6502 addressing mode enumeration.
///
/// The addressing mode determines how the CPU interprets the operand bytes
/// that follow an opcode and how it calculates the effective memory address
/// for the operation.
///
/// # Operand Sizes
///
/// - **0 bytes**: Implicit, Accumulator
/// - **1 byte**: Immediate, ZeroPage, ZeroPageX, ZeroPageY, Relative, IndirectX, IndirectY
/// - **2 bytes**: Absolute, AbsoluteX, AbsoluteY, Indirect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    /// No operand, operation implied by instruction.
    ///
    /// Examples: CLC, RTS, NOP
    Implicit,

    /// Operates directly on the accumulator register.
    ///
    /// Examples: LSR A, ROL A, ASL A
    Accumulator,

    /// 8-bit constant operand in instruction.
    ///
    /// Example: LDA #$10 (load immediate value 0x10 into accumulator)
    Immediate,

    /// 8-bit address in zero page (0x00-0xFF).
    ///
    /// Example: LDA $80 (load from address 0x0080)
    ZeroPage,

    /// Zero page address indexed by X register.
    ///
    /// Example: LDA $80,X (load from address 0x0080 + X, wraps within zero page)
    ZeroPageX,

    /// Zero page address indexed by Y register.
    ///
    /// Example: LDX $80,Y (load from address 0x0080 + Y, wraps within zero page)
    ZeroPageY,

    /// Signed 8-bit offset for branch instructions.
    ///
    /// Example: BEQ label (branch if zero flag set, offset is relative to PC)
    Relative,

    /// Full 16-bit address.
    ///
    /// Example: JMP $1234 (jump to address 0x1234)
    Absolute,

    /// 16-bit address indexed by X register.
    ///
    /// Example: LDA $1234,X (load from address 0x1234 + X)
    /// May incur +1 cycle penalty if page boundary is crossed.
    AbsoluteX,

    /// 16-bit address indexed by Y register.
    ///
    /// Example: LDA $1234,Y (load from address 0x1234 + Y)
    /// May incur +1 cycle penalty if page boundary is crossed.
    AbsoluteY,

    /// Indirect jump through 16-bit pointer.
    ///
    /// Example: JMP ($FFFC) (jump to address stored at 0xFFFC/0xFFFD)
    /// Only used by JMP instruction.
    Indirect,

    /// Indexed indirect: (ZP + X) then dereference.
    ///
    /// Example: LDA ($40,X) (add X to 0x40, read 16-bit address from that ZP location, load from result)
    /// Operand is added to X within zero page, then dereferenced.
    IndirectX,

    /// Indirect indexed: ZP dereference then + Y.
    ///
    /// Example: LDA ($40),Y (read 16-bit address from ZP 0x40, add Y, load from result)
    /// Operand is dereferenced to get base address, then Y is added.
    /// May incur +1 cycle penalty if page boundary is crossed.
    IndirectY,
}
