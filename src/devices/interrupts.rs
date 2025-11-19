//! Interrupt support for memory-mapped devices.
//!
//! This module provides the `InterruptDevice` trait that devices can implement
//! to signal interrupt requests to the CPU. The interrupt model matches real
//! 6502 hardware behavior with a level-sensitive IRQ line.
//!
//! # Hardware-Accurate Interrupt Model
//!
//! The 6502 CPU has a single active-low IRQ (Interrupt Request) line that is
//! shared among all devices. Multiple devices can signal interrupts simultaneously,
//! and the IRQ line remains active until ALL devices have cleared their interrupt
//! requests.
//!
//! ## IRQ Line Behavior
//!
//! - **Level-sensitive**: The IRQ line reflects the current state of all devices
//! - **Logical OR**: IRQ line is active if ANY device has a pending interrupt
//! - **No queuing**: Interrupts are not queued - they are signaled immediately
//! - **ISR acknowledgment**: The interrupt service routine must explicitly poll
//!   and acknowledge each device by reading/writing memory-mapped registers
//!
//! ## Interrupt Service Sequence
//!
//! When the CPU detects an active IRQ line and the I flag is clear, it:
//!
//! 1. Completes the current instruction
//! 2. Pushes PC (high byte, then low byte) to stack (2 cycles)
//! 3. Pushes status register to stack (1 cycle)
//! 4. Sets I flag to prevent nested interrupts (0 cycles)
//! 5. Reads IRQ vector from 0xFFFE-0xFFFF (2 cycles)
//! 6. Jumps to interrupt handler (2 cycles)
//!
//! **Total: 7 cycles** (matches real 6502 hardware)
//!
//! ## ISR Pattern
//!
//! The interrupt service routine (ISR) should:
//!
//! 1. Poll all potential interrupt sources by reading device status registers
//! 2. Identify which devices have pending interrupts
//! 3. Handle each interrupt appropriately
//! 4. Acknowledge interrupts by writing to device control registers
//! 5. Return via RTI instruction
//!
//! If the IRQ line is still active after RTI (because a device didn't clear its
//! interrupt), the CPU will immediately re-enter the ISR.
//!
//! # Example
//!
//! ```rust
//! use lib6502::InterruptDevice;
//!
//! struct TimerDevice {
//!     interrupt_pending: bool,
//!     // ... other fields
//! }
//!
//! impl InterruptDevice for TimerDevice {
//!     fn has_interrupt(&self) -> bool {
//!         self.interrupt_pending
//!     }
//! }
//! ```
//!
//! See the `examples/interrupt_device.rs` example for a complete working device
//! implementation with memory-mapped registers.

/// Trait for devices that can signal interrupt requests to the CPU.
///
/// Devices implement this trait to participate in the shared IRQ line. The CPU
/// polls all registered interrupt-capable devices after each instruction to
/// determine if an interrupt should be serviced.
///
/// # Implementation Requirements
///
/// 1. **Implement `Device` trait**: All interrupt-capable devices must also
///    implement the `Device` trait to provide memory-mapped register access
/// 2. **Set interrupt flag**: Device sets internal `interrupt_pending` flag
///    when interrupt condition occurs (timer expires, data received, etc.)
/// 3. **Clear on acknowledgment**: Device clears `interrupt_pending` when ISR
///    reads/writes the appropriate register (device-specific)
/// 4. **Accurate state**: `has_interrupt()` must accurately reflect current state
///
/// # Hardware Fidelity
///
/// This trait models the real 6502 IRQ line behavior:
///
/// - **Level-sensitive**: Returns current state, not edge-triggered
/// - **Shared line**: Multiple devices can have interrupts simultaneously
/// - **No automatic clear**: Device remains pending until ISR acknowledges
///
/// # Example
///
/// ```rust
/// use lib6502::{Device, InterruptDevice};
/// use std::any::Any;
///
/// struct UartDevice {
///     base_address: u16,
///     interrupt_pending: bool,
///     data_register: u8,
///     status_register: u8,
/// }
///
/// impl InterruptDevice for UartDevice {
///     fn has_interrupt(&self) -> bool {
///         self.interrupt_pending
///     }
/// }
///
/// impl Device for UartDevice {
///     fn read(&self, offset: u16) -> u8 {
///         match offset {
///             0 => self.status_register,  // Reading status shows interrupt flag
///             1 => {
///                 // Reading data clears interrupt (device-specific choice)
///                 // In real implementation, would use interior mutability
///                 self.data_register
///             }
///             _ => 0,
///         }
///     }
///
///     fn write(&mut self, offset: u16, value: u8) {
///         match offset {
///             0 => {
///                 // Writing to control register acknowledges interrupt
///                 if value & 0x80 != 0 {
///                     self.interrupt_pending = false;
///                 }
///             }
///             1 => self.data_register = value,
///             _ => {}
///         }
///     }
///
///     fn size(&self) -> u16 {
///         4  // Four memory-mapped registers
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn Any {
///         self
///     }
/// }
/// ```
pub trait InterruptDevice {
    /// Check if device has a pending interrupt request.
    ///
    /// Returns `true` if the device has an unserviced interrupt that should
    /// contribute to the CPU's IRQ line state.
    ///
    /// # Contract
    ///
    /// - **MUST** return `true` when device has pending interrupt
    /// - **MUST** return `false` when interrupt has been acknowledged/cleared
    /// - **MUST NOT** modify device state (read-only query)
    /// - **MUST** be deterministic (same result if called multiple times)
    /// - **SHOULD** be O(1) performance (called after every instruction)
    ///
    /// # Hardware Semantics
    ///
    /// This method represents the device's contribution to the shared IRQ line.
    /// The CPU will OR together all devices' `has_interrupt()` results to
    /// determine the overall IRQ line state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use lib6502::InterruptDevice;
    /// # struct TimerDevice { interrupt_pending: bool }
    /// impl InterruptDevice for TimerDevice {
    ///     fn has_interrupt(&self) -> bool {
    ///         self.interrupt_pending  // Simple flag check
    ///     }
    /// }
    /// ```
    fn has_interrupt(&self) -> bool;
}
