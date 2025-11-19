//! W65C51 ACIA UART device implementation.
//!
//! Provides serial communication via memory-mapped registers with callback interface
//! for external terminal integration.

use super::{Device, InterruptDevice};
use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;

/// W65C51 ACIA UART serial communication device.
///
/// The UART device emulates a 6551 ACIA chip with four memory-mapped registers:
///
/// ## Register Map (offsets from device base address)
///
/// | Offset | Register | Access | Description |
/// |--------|----------|--------|-------------|
/// | 0      | Data     | R/W    | Transmit/Receive data |
/// | 1      | Status   | R      | Status flags (read-only) |
/// | 2      | Command  | R/W    | Command register |
/// | 3      | Control  | R/W    | Control register |
///
/// ## Status Register (Offset 1, Read-Only)
///
/// | Bit | Name | Description |
/// |-----|------|-------------|
/// | 4   | TDRE | Transmit Data Register Empty (always 1 - ready) |
/// | 3   | RDRF | Receive Data Register Full (1 = data available) |
/// | 2   | OVRN | Overrun Error (1 = buffer overflow occurred) |
/// | 1-0 | -    | Reserved (always 0) |
///
/// ## Command Register (Offset 2, Read/Write)
///
/// | Bit | Name | Description |
/// |-----|------|-------------|
/// | 3   | ECHO | Echo mode (1 = auto-retransmit received bytes) |
/// | 1   | IRQ_EN | Interrupt enable (1 = trigger IRQ on data received) |
/// | 7-4,2,0 | - | User-defined (stored but not interpreted) |
///
/// ## Control Register (Offset 3, Read/Write)
///
/// User-defined - stored but not interpreted by emulator.
///
/// ## Receive Buffer
///
/// - **Capacity**: 256 bytes (FIFO via VecDeque)
/// - **Overflow**: Sets OVRN flag, drops new bytes when full
/// - **Clear**: Reading data register pops one byte from buffer
///
/// ## Transmit Behavior
///
/// - **Immediate**: Writes to data register invoke callback immediately
/// - **No buffering**: TDRE always set (always ready)
/// - **Echo mode**: When command register bit 3 set, received bytes auto-transmit
///
/// ## Interrupt Support
///
/// The UART supports receive interrupts via the CPU interrupt system:
///
/// - **Enable**: Set command register bit 1 (0x02) to enable interrupts
/// - **Trigger**: Interrupt fires when byte is received (via `receive_byte()`)
/// - **Clear**: Reading data register (offset 0) acknowledges interrupt
/// - **IRQ Line**: Device signals CPU via `InterruptDevice` trait
///
/// Interrupt Service Routine pattern:
/// ```text
/// isr:
///     LDA $A002      ; Read command register
///     AND #$02       ; Check if UART interrupts enabled
///     BEQ check_other_devices
///     LDA $A000      ; Read data (clears interrupt)
///     ; Process received byte...
///     RTS
/// ```
///
/// # Example
///
/// ```rust
/// use lib6502::{Uart6551, Device};
///
/// let mut uart = Uart6551::new();
///
/// // Set transmit callback
/// uart.set_transmit_callback(|byte| {
///     print!("{}", byte as char);
/// });
///
/// // Inject received byte
/// uart.receive_byte(b'A');
///
/// // Check status
/// assert_eq!(uart.status() & 0x08, 0x08); // RDRF set
///
/// // Read via Device trait (offset 0 = data register)
/// assert_eq!(uart.read(0), b'A');
/// ```
///
/// # WASM Integration
///
/// See `examples/wasm_terminal.rs` for browser terminal integration patterns.
pub struct Uart6551 {
    // Registers (4 bytes)
    data_register: u8,
    status_register: RefCell<u8>, // Mutable during read
    command_register: u8,
    control_register: u8,

    // Receive buffer (uses interior mutability for read-time mutation)
    rx_buffer: RefCell<VecDeque<u8>>,
    rx_buffer_capacity: usize,

    // Transmit callback
    on_transmit: Option<Box<dyn Fn(u8)>>,

    // Last received byte (uses interior mutability)
    last_rx_byte: RefCell<u8>,

    // Overrun flag (uses interior mutability)
    overrun_occurred: RefCell<bool>,

    // Interrupt support (uses interior mutability for state changes during read)
    interrupt_pending: RefCell<bool>,
    interrupt_enable: RefCell<bool>,
}

impl Uart6551 {
    /// Create a new UART device with default settings.
    ///
    /// # Returns
    ///
    /// A new `Uart6551` instance with:
    /// - All registers initialized to 0x00
    /// - 256-byte receive buffer
    /// - No transmit callback (must be set separately)
    /// - TDRE (transmitter ready) always set to 1
    ///
    /// # Example
    ///
    /// ```rust
    /// use lib6502::Uart6551;
    ///
    /// let uart = Uart6551::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data_register: 0x00,
            status_register: RefCell::new(0x10), // TDRE (bit 4) = 1, transmitter always ready
            command_register: 0x00,
            control_register: 0x00,
            rx_buffer: RefCell::new(VecDeque::new()),
            rx_buffer_capacity: 256,
            on_transmit: None,
            last_rx_byte: RefCell::new(0x00),
            overrun_occurred: RefCell::new(false),
            interrupt_pending: RefCell::new(false),
            interrupt_enable: RefCell::new(false),
        }
    }

    /// Set the transmit callback function.
    ///
    /// The callback is invoked whenever the CPU writes to the data register (offset 0).
    /// This enables integration with external terminals (e.g., xterm.js in browser).
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call when byte is transmitted
    ///
    /// # Example
    ///
    /// ```rust
    /// use lib6502::Uart6551;
    ///
    /// let mut uart = Uart6551::new();
    ///
    /// uart.set_transmit_callback(|byte| {
    ///     println!("Transmitted: 0x{:02X}", byte);
    /// });
    /// ```
    pub fn set_transmit_callback<F>(&mut self, callback: F)
    where
        F: Fn(u8) + 'static,
    {
        self.on_transmit = Some(Box::new(callback));
    }

    /// Inject a received byte into the UART receive buffer.
    ///
    /// This method is called by external code (e.g., browser terminal) when
    /// data is received. The byte is added to the receive buffer and the
    /// RDRF (Receiver Data Register Full) status bit is set.
    ///
    /// If the buffer is full, the byte is dropped and the overrun flag is set.
    ///
    /// # Arguments
    ///
    /// * `byte` - Byte received from external source
    ///
    /// # Example
    ///
    /// ```rust
    /// use lib6502::Uart6551;
    ///
    /// let mut uart = Uart6551::new();
    ///
    /// // Simulate terminal sending 'A'
    /// uart.receive_byte(b'A');
    ///
    /// // Status bit 3 (RDRF) should now be set
    /// assert_eq!(uart.status() & 0x08, 0x08);
    /// ```
    pub fn receive_byte(&mut self, byte: u8) {
        if self.rx_buffer.borrow().len() < self.rx_buffer_capacity {
            self.rx_buffer.borrow_mut().push_back(byte);
            *self.last_rx_byte.borrow_mut() = byte;
            self.update_status_register();

            // Trigger interrupt if enabled (command bit 1)
            if *self.interrupt_enable.borrow() {
                *self.interrupt_pending.borrow_mut() = true;
            }

            // Echo mode: automatically retransmit if enabled (command bit 3)
            if self.command_register & 0x08 != 0 {
                if let Some(ref callback) = self.on_transmit {
                    callback(byte);
                }
            }
        } else {
            // Buffer overflow
            *self.overrun_occurred.borrow_mut() = true;
            self.update_status_register();
        }
    }

    /// Get current status register value (for testing).
    ///
    /// # Returns
    ///
    /// Status register byte with flags:
    /// - Bit 4 (TDRE): Transmitter Data Register Empty (always 1)
    /// - Bit 3 (RDRF): Receiver Data Register Full
    /// - Bit 2: Overrun error
    ///
    /// # Example
    ///
    /// ```rust
    /// use lib6502::Uart6551;
    ///
    /// let uart = Uart6551::new();
    /// assert_eq!(uart.status() & 0x10, 0x10); // TDRE always set
    /// ```
    pub fn status(&self) -> u8 {
        *self.status_register.borrow()
    }

    /// Get current receive buffer length (for testing).
    ///
    /// # Returns
    ///
    /// Number of bytes in receive buffer
    ///
    /// # Example
    ///
    /// ```rust
    /// use lib6502::Uart6551;
    ///
    /// let mut uart = Uart6551::new();
    /// uart.receive_byte(b'A');
    /// uart.receive_byte(b'B');
    /// assert_eq!(uart.rx_buffer_len(), 2);
    /// ```
    pub fn rx_buffer_len(&self) -> usize {
        self.rx_buffer.borrow().len()
    }

    /// Write to data register (offset 0).
    ///
    /// Invokes transmit callback immediately (no buffering). TDRE always
    /// remains set (transmitter always ready).
    fn write_data_register(&mut self, value: u8) {
        self.data_register = value;

        // Invoke transmit callback if set
        if let Some(ref callback) = self.on_transmit {
            callback(value);
        }

        // TDRE remains 1 (no buffering, always ready for next byte)
    }

    /// Update status register based on current state.
    fn update_status_register(&mut self) {
        let mut status = 0x10; // TDRE (bit 4) always 1

        // RDRF (bit 3): Set if receive buffer has data
        if !self.rx_buffer.borrow().is_empty() {
            status |= 0x08;
        }

        // Overrun (bit 2): Set if buffer overflow occurred
        if *self.overrun_occurred.borrow() {
            status |= 0x04;
        }

        *self.status_register.borrow_mut() = status;
    }
}

impl Default for Uart6551 {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Uart6551 {
    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => {
                // Data register - pop byte from receive buffer
                // Uses interior mutability to modify buffer during read
                let mut rx_buffer = self.rx_buffer.borrow_mut();
                if let Some(byte) = rx_buffer.pop_front() {
                    // Clear overrun flag on successful read
                    *self.overrun_occurred.borrow_mut() = false;
                    *self.last_rx_byte.borrow_mut() = byte;

                    // Clear interrupt on data read (acknowledge interrupt)
                    *self.interrupt_pending.borrow_mut() = false;

                    // Update status (RDRF may clear if buffer now empty)
                    let mut status = 0x10; // TDRE always 1
                    if !rx_buffer.is_empty() {
                        status |= 0x08; // RDRF
                    }
                    drop(rx_buffer); // Release borrow before updating status
                    *self.status_register.borrow_mut() = status;

                    byte
                } else {
                    drop(rx_buffer); // Release borrow
                                     // Buffer empty - return last byte or 0x00
                    *self.last_rx_byte.borrow()
                }
            }
            1 => *self.status_register.borrow(),
            2 => self.command_register,
            3 => self.control_register,
            _ => 0x00, // Invalid offset
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset {
            0 => self.write_data_register(value),
            1 => {
                // Status register is read-only, writes ignored
            }
            2 => {
                self.command_register = value;
                // Interrupt enable is bit 1
                *self.interrupt_enable.borrow_mut() = (value & 0x02) != 0;
                // If disabling interrupts, clear any pending interrupt
                if (value & 0x02) == 0 {
                    *self.interrupt_pending.borrow_mut() = false;
                }
                // Echo mode is bit 3, handled in receive_byte
            }
            3 => {
                self.control_register = value;
                // Baud rate and configuration stored but not enforced
            }
            _ => {
                // Invalid offset, write ignored
            }
        }
    }

    fn size(&self) -> u16 {
        4 // Four registers: data, status, command, control
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_interrupt_device(&self) -> Option<&dyn InterruptDevice> {
        Some(self)
    }
}

impl InterruptDevice for Uart6551 {
    fn has_interrupt(&self) -> bool {
        *self.interrupt_pending.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_uart_new() {
        let uart = Uart6551::new();
        assert_eq!(uart.size(), 4);
        assert_eq!(uart.status() & 0x10, 0x10); // TDRE always set
        assert_eq!(uart.rx_buffer_len(), 0);
    }

    #[test]
    fn test_uart_transmit() {
        let mut uart = Uart6551::new();
        let transmitted = Rc::new(RefCell::new(Vec::new()));
        let transmitted_clone = Rc::clone(&transmitted);

        uart.set_transmit_callback(move |byte| {
            transmitted_clone.borrow_mut().push(byte);
        });

        // Write to data register
        uart.write(0, 0x42);
        uart.write(0, 0x43);

        assert_eq!(*transmitted.borrow(), vec![0x42, 0x43]);
    }

    #[test]
    fn test_uart_receive() {
        let mut uart = Uart6551::new();

        // Buffer should be empty
        assert_eq!(uart.rx_buffer_len(), 0);
        assert_eq!(uart.status() & 0x08, 0x00); // RDRF not set

        // Receive a byte
        uart.receive_byte(0x41);

        assert_eq!(uart.rx_buffer_len(), 1);
        assert_eq!(uart.status() & 0x08, 0x08); // RDRF set

        // Read it back
        assert_eq!(uart.read(0), 0x41);
    }

    #[test]
    fn test_uart_status_register_read_only() {
        let mut uart = Uart6551::new();
        let initial_status = uart.status();

        // Try to write to status register (offset 1)
        uart.write(1, 0xFF);

        // Status should be unchanged
        assert_eq!(uart.status(), initial_status);
    }

    #[test]
    fn test_uart_command_control_registers() {
        let mut uart = Uart6551::new();

        // Write to command register
        uart.write(2, 0xAA);
        assert_eq!(uart.read(2), 0xAA);

        // Write to control register
        uart.write(3, 0x55);
        assert_eq!(uart.read(3), 0x55);
    }

    #[test]
    fn test_uart_buffer_overflow() {
        let mut uart = Uart6551::new();

        // Fill buffer to capacity (256 bytes)
        for i in 0..256 {
            uart.receive_byte(i as u8);
        }

        assert_eq!(uart.rx_buffer_len(), 256);
        assert_eq!(uart.status() & 0x04, 0x00); // No overrun yet

        // Try to add one more (should cause overrun)
        uart.receive_byte(0xFF);

        assert_eq!(uart.rx_buffer_len(), 256); // Buffer still at capacity
        assert_eq!(uart.status() & 0x04, 0x04); // Overrun flag set
    }

    #[test]
    fn test_uart_echo_mode() {
        let mut uart = Uart6551::new();
        let echoed = Rc::new(RefCell::new(Vec::new()));
        let echoed_clone = Rc::clone(&echoed);

        uart.set_transmit_callback(move |byte| {
            echoed_clone.borrow_mut().push(byte);
        });

        // Enable echo mode (bit 3 of command register)
        uart.write(2, 0x08);

        // Receive bytes - they should be automatically echoed
        uart.receive_byte(b'A');
        uart.receive_byte(b'B');

        assert_eq!(*echoed.borrow(), vec![b'A', b'B']);
    }
}
