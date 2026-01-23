//! High-Level IEC Bus Emulation
//!
//! This module provides a high-level emulation of the IEC (CBM Serial Bus) protocol
//! used by the C64 to communicate with disk drives and other peripherals.
//!
//! Rather than emulating the physical bus timing and handshaking, this implementation
//! intercepts KERNAL serial I/O calls to provide fast, compatible disk access.
//! This approach works for most software while being much simpler than full
//! low-level emulation.
//!
//! ## IEC Protocol Overview
//!
//! The IEC bus uses three signal lines:
//! - ATN (Attention): Controller asserts commands
//! - CLK (Clock): Data timing
//! - DATA: Bidirectional data transfer
//!
//! ## High-Level Commands
//!
//! This emulation handles:
//! - LISTEN (device address + $20): Put device in receive mode
//! - TALK (device address + $40): Put device in send mode
//! - OPEN (secondary address + $60): Open a channel
//! - CLOSE (secondary address + $E0): Close a channel
//! - UNLISTEN ($3F): Release listen mode
//! - UNTALK ($5F): Release talk mode

use super::disk_1541::{ChannelMode, Drive1541};

/// IEC bus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IecState {
    /// Bus is idle, no active transfer.
    #[default]
    Idle,
    /// Controller is sending commands (ATN active).
    Command,
    /// Device is listening for data.
    Listen,
    /// Device is sending data.
    Talk,
}

/// IEC bus controller for high-level serial communication.
///
/// This struct manages the communication protocol between the C64 and
/// connected devices (primarily the 1541 disk drive).
pub struct IecBus {
    /// Current bus state.
    state: IecState,
    /// Currently addressed device (8-30, or 0 if none).
    active_device: u8,
    /// Currently addressed secondary address / channel (0-15).
    active_channel: u8,
    /// Pending command byte (for two-byte commands).
    pending_command: Option<u8>,
    /// Buffer for command/filename being sent.
    command_buffer: Vec<u8>,
    /// The 1541 disk drive (device 8).
    drive: Drive1541,
    /// Last status byte from device.
    last_status: u8,
    /// End of file indicator.
    eof: bool,
}

impl Default for IecBus {
    fn default() -> Self {
        Self::new()
    }
}

impl IecBus {
    /// Create a new IEC bus with a 1541 drive at device 8.
    pub fn new() -> Self {
        Self {
            state: IecState::Idle,
            active_device: 0,
            active_channel: 0,
            pending_command: None,
            command_buffer: Vec::new(),
            drive: Drive1541::new(8),
            last_status: 0,
            eof: false,
        }
    }

    /// Get the current bus state.
    pub fn state(&self) -> IecState {
        self.state
    }

    /// Get the active device number.
    pub fn active_device(&self) -> u8 {
        self.active_device
    }

    /// Get a reference to the 1541 drive.
    pub fn drive(&self) -> &Drive1541 {
        &self.drive
    }

    /// Get a mutable reference to the 1541 drive.
    pub fn drive_mut(&mut self) -> &mut Drive1541 {
        &mut self.drive
    }

    /// Check if a disk is mounted in the drive.
    pub fn has_disk(&self) -> bool {
        self.drive.has_disk()
    }

    /// Check if end of file has been reached.
    pub fn is_eof(&self) -> bool {
        self.eof
    }

    /// Reset the bus to idle state.
    pub fn reset(&mut self) {
        self.state = IecState::Idle;
        self.active_device = 0;
        self.active_channel = 0;
        self.pending_command = None;
        self.command_buffer.clear();
        self.last_status = 0;
        self.eof = false;
        self.drive.close_all_channels();
    }

    /// Process an IEC command byte sent under ATN.
    ///
    /// # Command Byte Format
    /// - $20-$3E: LISTEN (device = byte - $20)
    /// - $3F: UNLISTEN
    /// - $40-$5E: TALK (device = byte - $40)
    /// - $5F: UNTALK
    /// - $60-$6F: Secondary address for OPEN (channel = byte - $60)
    /// - $E0-$EF: Secondary address for CLOSE (channel = byte - $E0)
    /// - $F0-$FF: Secondary address for data (channel = byte - $F0)
    pub fn send_command(&mut self, byte: u8) {
        match byte {
            // LISTEN: device addresses $20-$3E
            0x20..=0x3E => {
                let device = byte - 0x20;
                if device == self.drive.device_number() {
                    self.active_device = device;
                    self.state = IecState::Listen;
                    self.pending_command = Some(byte);
                } else {
                    // Device not present
                    self.active_device = 0;
                    self.last_status = 0x80; // Device not present
                }
            }

            // UNLISTEN
            0x3F => {
                if self.state == IecState::Listen && !self.command_buffer.is_empty() {
                    // Flush any pending command to the device
                    self.flush_command_buffer();
                }
                self.state = IecState::Idle;
                self.active_device = 0;
                self.pending_command = None;
            }

            // TALK: device addresses $40-$5E
            0x40..=0x5E => {
                let device = byte - 0x40;
                if device == self.drive.device_number() {
                    self.active_device = device;
                    self.state = IecState::Talk;
                    self.eof = false;
                } else {
                    self.active_device = 0;
                    self.last_status = 0x80; // Device not present
                }
            }

            // UNTALK
            0x5F => {
                self.state = IecState::Idle;
                self.active_device = 0;
            }

            // OPEN secondary address $60-$6F
            0x60..=0x6F => {
                let channel = byte - 0x60;
                self.active_channel = channel;
                // Command buffer will be used as filename
                self.command_buffer.clear();
            }

            // CLOSE secondary address $E0-$EF
            0xE0..=0xEF => {
                let channel = byte - 0xE0;
                self.active_channel = channel;
                if self.active_device == self.drive.device_number() {
                    self.drive.close_channel(channel);
                }
            }

            // DATA secondary address $F0-$FF
            0xF0..=0xFF => {
                let channel = byte - 0xF0;
                self.active_channel = channel;
            }

            _ => {
                // Unknown command
                self.last_status = 0x80;
            }
        }
    }

    /// Send a data byte to the active device.
    ///
    /// In Listen mode, this sends data to the device.
    /// The byte is either added to the command buffer (after OPEN)
    /// or written directly to the channel.
    pub fn send_byte(&mut self, byte: u8) -> bool {
        if self.active_device != self.drive.device_number() {
            self.last_status = 0x80;
            return false;
        }

        match self.state {
            IecState::Listen => {
                // After OPEN ($60-$6F), bytes are the filename
                self.command_buffer.push(byte);
                self.last_status = 0;
                true
            }
            _ => {
                self.last_status = 0x80;
                false
            }
        }
    }

    /// Receive a byte from the active device.
    ///
    /// In Talk mode, this reads data from the device.
    /// Returns the byte and updates status (including EOI/EOF).
    pub fn receive_byte(&mut self) -> Option<u8> {
        if self.active_device != self.drive.device_number() {
            self.last_status = 0x80;
            return None;
        }

        if self.state != IecState::Talk {
            self.last_status = 0x80;
            return None;
        }

        match self.drive.read_byte(self.active_channel) {
            Ok(Some(byte)) => {
                self.last_status = 0;
                Some(byte)
            }
            Ok(None) => {
                // End of file
                self.eof = true;
                self.last_status = 0x40; // EOI
                None
            }
            Err(_) => {
                self.last_status = 0x80; // Error
                None
            }
        }
    }

    /// Flush the command buffer to open a file.
    fn flush_command_buffer(&mut self) {
        if self.command_buffer.is_empty() {
            return;
        }

        // Convert buffer to string (PETSCII to ASCII)
        let filename: String = self
            .command_buffer
            .iter()
            .map(|&b| petscii_to_ascii(b))
            .collect();

        // Determine mode from channel number
        let mode = match self.active_channel {
            0 => ChannelMode::Read,    // Default read channel
            1 => ChannelMode::Write,   // Default write channel
            15 => ChannelMode::Command, // Command channel
            _ => ChannelMode::Read,    // Other channels default to read
        };

        // Open the file
        if let Err(_e) = self.drive.open_channel(self.active_channel, &filename, mode) {
            self.last_status = 0x80; // Error
        } else {
            self.last_status = 0;
        }

        self.command_buffer.clear();
    }

    /// Get the last status byte.
    ///
    /// # Status Bits
    /// - Bit 6: EOI (end of file)
    /// - Bit 7: Device not present / error
    pub fn status(&self) -> u8 {
        self.last_status
    }

    /// Open a file on the drive.
    ///
    /// This is a high-level helper that bypasses the byte-by-byte protocol.
    pub fn open_file(&mut self, channel: u8, filename: &str, mode: ChannelMode) -> bool {
        self.active_channel = channel;
        self.active_device = self.drive.device_number();

        match self.drive.open_channel(channel, filename, mode) {
            Ok(()) => {
                self.last_status = 0;
                self.eof = false;
                true
            }
            Err(_) => {
                self.last_status = 0x80;
                false
            }
        }
    }

    /// Close a channel on the drive.
    pub fn close_file(&mut self, channel: u8) {
        self.drive.close_channel(channel);
    }

    /// Read a byte from the currently open channel.
    pub fn read_byte(&mut self) -> Option<u8> {
        match self.drive.read_byte(self.active_channel) {
            Ok(Some(byte)) => {
                self.last_status = 0;
                Some(byte)
            }
            Ok(None) => {
                self.eof = true;
                self.last_status = 0x40; // EOI
                None
            }
            Err(_) => {
                self.last_status = 0x80;
                None
            }
        }
    }

    /// Read multiple bytes from the currently open channel.
    pub fn read_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(max_bytes);
        for _ in 0..max_bytes {
            match self.read_byte() {
                Some(byte) => result.push(byte),
                None => break,
            }
        }
        result
    }
}

/// Convert PETSCII character to ASCII.
fn petscii_to_ascii(c: u8) -> char {
    match c {
        0x00..=0x1F => ' ', // Control characters
        0x20..=0x3F => c as char, // Numbers, some punctuation
        0x40 => '@',
        0x41..=0x5A => c as char, // Uppercase letters
        0x5B..=0x5F => c as char,
        0x60 => '-',
        0x61..=0x7A => (c - 0x20) as char, // Lowercase → uppercase
        0x7B..=0x7F => c as char,
        0x80..=0x9F => ' ',
        0xA0 => ' ', // Shifted space
        0xC1..=0xDA => (c - 0x80) as char,
        _ => '?',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bus() {
        let bus = IecBus::new();
        assert_eq!(bus.state(), IecState::Idle);
        assert_eq!(bus.active_device(), 0);
        assert!(!bus.has_disk());
    }

    #[test]
    fn test_listen_command() {
        let mut bus = IecBus::new();

        // Listen to device 8
        bus.send_command(0x28); // $20 + 8 = $28
        assert_eq!(bus.state(), IecState::Listen);
        assert_eq!(bus.active_device(), 8);
    }

    #[test]
    fn test_talk_command() {
        let mut bus = IecBus::new();

        // Talk to device 8
        bus.send_command(0x48); // $40 + 8 = $48
        assert_eq!(bus.state(), IecState::Talk);
        assert_eq!(bus.active_device(), 8);
    }

    #[test]
    fn test_unlisten() {
        let mut bus = IecBus::new();

        bus.send_command(0x28); // Listen device 8
        assert_eq!(bus.state(), IecState::Listen);

        bus.send_command(0x3F); // Unlisten
        assert_eq!(bus.state(), IecState::Idle);
    }

    #[test]
    fn test_untalk() {
        let mut bus = IecBus::new();

        bus.send_command(0x48); // Talk device 8
        assert_eq!(bus.state(), IecState::Talk);

        bus.send_command(0x5F); // Untalk
        assert_eq!(bus.state(), IecState::Idle);
    }

    #[test]
    fn test_device_not_present() {
        let mut bus = IecBus::new();

        // Try to talk to device 9 (not connected)
        bus.send_command(0x49); // $40 + 9 = $49
        assert_eq!(bus.state(), IecState::Idle);
        assert_eq!(bus.status(), 0x80);
    }

    #[test]
    fn test_secondary_address() {
        let mut bus = IecBus::new();

        bus.send_command(0x28); // Listen device 8
        bus.send_command(0x60); // Open channel 0

        assert_eq!(bus.active_channel, 0);
    }

    #[test]
    fn test_petscii_conversion() {
        assert_eq!(petscii_to_ascii(0x41), 'A');
        assert_eq!(petscii_to_ascii(0x5A), 'Z');
        assert_eq!(petscii_to_ascii(0x30), '0');
        assert_eq!(petscii_to_ascii(0x20), ' ');
    }
}
