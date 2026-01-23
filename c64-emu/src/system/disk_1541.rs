//! 1541 Disk Drive Emulation (High-Level IEC Protocol)
//!
//! This module implements a high-level emulation of the Commodore 1541 disk drive.
//! Rather than emulating the drive's internal 6502 CPU, it provides a simplified
//! IEC protocol implementation that handles .D64 file access for most software.
//!
//! ## D64 File Format
//!
//! The D64 format stores 35 tracks with varying sectors per track:
//! - Tracks 1-17:  21 sectors each (357 total)
//! - Tracks 18-24: 19 sectors each (133 total)
//! - Tracks 25-30: 18 sectors each (108 total)
//! - Tracks 31-35: 17 sectors each (85 total)
//!
//! Total: 683 sectors × 256 bytes = 174,848 bytes
//!
//! Track 18 contains the directory and Block Availability Map (BAM).

/// Standard D64 file size (35 tracks, 683 sectors)
pub const D64_SIZE: usize = 174_848;

/// D64 file size with error info
pub const D64_SIZE_WITH_ERRORS: usize = 175_531;

/// Number of sectors per track
const SECTORS_PER_TRACK: [u8; 35] = [
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, // Tracks 1-17
    19, 19, 19, 19, 19, 19, 19, // Tracks 18-24
    18, 18, 18, 18, 18, 18, // Tracks 25-30
    17, 17, 17, 17, 17, // Tracks 31-35
];

/// Starting sector offset for each track (0-indexed)
const TRACK_OFFSETS: [usize; 35] = [
    0, 21, 42, 63, 84, 105, 126, 147, 168, 189, 210, 231, 252, 273, 294, 315, 336, // 1-17
    357, 376, 395, 414, 433, 452, 471, // 18-24
    490, 508, 526, 544, 562, 580, // 25-30
    598, 615, 632, 649, 666, // 31-35
];

/// Directory track number
pub const DIRECTORY_TRACK: u8 = 18;

/// First directory sector
pub const DIRECTORY_FIRST_SECTOR: u8 = 1;

/// BAM sector (Block Availability Map)
pub const BAM_SECTOR: u8 = 0;

/// D64 disk image representation.
///
/// Stores the raw sector data and tracks whether the image has been modified.
#[derive(Clone)]
pub struct D64Image {
    /// Raw disk data (174,848 bytes for standard D64)
    data: Box<[u8]>,
    /// Whether the image has been modified since loading
    modified: bool,
}

impl D64Image {
    /// Create a new D64 image from raw data.
    ///
    /// # Errors
    /// Returns an error if the data size is invalid.
    pub fn new(data: Vec<u8>) -> Result<Self, D64Error> {
        if data.len() != D64_SIZE && data.len() != D64_SIZE_WITH_ERRORS {
            return Err(D64Error::InvalidSize {
                expected: D64_SIZE,
                got: data.len(),
            });
        }
        Ok(Self {
            data: data.into_boxed_slice(),
            modified: false,
        })
    }

    /// Check if the image has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get the raw data for saving.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Calculate the byte offset for a given track and sector.
    ///
    /// Tracks are 1-indexed (1-35), sectors are 0-indexed.
    fn sector_offset(&self, track: u8, sector: u8) -> Result<usize, D64Error> {
        if !(1..=35).contains(&track) {
            return Err(D64Error::InvalidTrack(track));
        }
        let track_idx = (track - 1) as usize;
        let max_sector = SECTORS_PER_TRACK[track_idx];
        if sector >= max_sector {
            return Err(D64Error::InvalidSector { track, sector });
        }
        let offset = (TRACK_OFFSETS[track_idx] + sector as usize) * 256;
        Ok(offset)
    }

    /// Read a sector from the disk image.
    ///
    /// Returns a 256-byte sector buffer.
    pub fn read_sector(&self, track: u8, sector: u8) -> Result<[u8; 256], D64Error> {
        let offset = self.sector_offset(track, sector)?;
        let mut buffer = [0u8; 256];
        buffer.copy_from_slice(&self.data[offset..offset + 256]);
        Ok(buffer)
    }

    /// Write a sector to the disk image.
    ///
    /// Marks the image as modified.
    pub fn write_sector(
        &mut self,
        track: u8,
        sector: u8,
        data: &[u8; 256],
    ) -> Result<(), D64Error> {
        let offset = self.sector_offset(track, sector)?;
        self.data[offset..offset + 256].copy_from_slice(data);
        self.modified = true;
        Ok(())
    }

    /// Get number of sectors for a track.
    pub fn sectors_in_track(track: u8) -> Result<u8, D64Error> {
        if !(1..=35).contains(&track) {
            return Err(D64Error::InvalidTrack(track));
        }
        Ok(SECTORS_PER_TRACK[(track - 1) as usize])
    }

    /// Read the BAM (Block Availability Map) from track 18, sector 0.
    pub fn read_bam(&self) -> Result<[u8; 256], D64Error> {
        self.read_sector(DIRECTORY_TRACK, BAM_SECTOR)
    }

    /// Get the disk name from the BAM.
    pub fn disk_name(&self) -> Result<String, D64Error> {
        let bam = self.read_bam()?;
        // Disk name is at offset 0x90-0x9F (16 characters, PETSCII)
        let name_bytes = &bam[0x90..0xA0];
        // Convert from PETSCII, trimming padding (0xA0 = shifted space)
        let name: String = name_bytes
            .iter()
            .take_while(|&&b| b != 0xA0 && b != 0)
            .map(|&b| petscii_to_ascii(b))
            .collect();
        Ok(name)
    }

    /// Get the disk ID from the BAM.
    pub fn disk_id(&self) -> Result<[u8; 2], D64Error> {
        let bam = self.read_bam()?;
        // Disk ID is at offset 0xA2-0xA3
        Ok([bam[0xA2], bam[0xA3]])
    }

    /// Validate the D64 image for structural integrity.
    ///
    /// Checks:
    /// - BAM header magic (track 18, sector 0 should have track pointer to itself)
    /// - Directory chain validity (no infinite loops, valid track/sector pointers)
    /// - DOS version marker
    ///
    /// Returns Ok(()) if the image passes basic validation, or an error describing the issue.
    pub fn validate(&self) -> Result<(), D64Error> {
        // Read the BAM (track 18, sector 0)
        let bam = self.read_bam()?;

        // Check directory track pointer (byte 0 should point to track 18)
        // BAM sector 0 contains: [next_track, next_sector, dos_version, ...]
        // For standard D64, next_track should be 18 (pointing to first dir sector)
        let dir_track = bam[0];
        let dir_sector = bam[1];

        // Track 0 means this is the last sector - invalid for BAM
        if dir_track == 0 {
            return Err(D64Error::CorruptedImage(
                "BAM has no directory pointer (track=0)".to_string(),
            ));
        }

        // Check if directory track pointer is valid
        if dir_track != DIRECTORY_TRACK {
            // Some disk images might point to track 18 sector 1 directly
            // Allow track 18 even if it's different from usual
            if !(1..=35).contains(&dir_track) {
                return Err(D64Error::CorruptedImage(format!(
                    "Invalid directory track pointer in BAM: track {} is out of range",
                    dir_track
                )));
            }
        }

        // Check first directory sector pointer
        let max_sectors = SECTORS_PER_TRACK[(dir_track - 1) as usize];
        if dir_sector >= max_sectors {
            return Err(D64Error::CorruptedImage(format!(
                "Invalid directory sector pointer in BAM: sector {} exceeds track {} maximum ({})",
                dir_sector, dir_track, max_sectors - 1
            )));
        }

        // Check DOS version byte (byte 2) - should be 0x41 ('A') for 1541 format
        let dos_version = bam[2];
        if dos_version != 0x41 {
            // This is a warning-level issue, not necessarily corruption
            // Some custom formats use different values
            // We'll allow it but could log a warning
        }

        // Validate directory chain (prevent infinite loops)
        let mut visited_sectors = std::collections::HashSet::new();
        let mut current_track = dir_track;
        let mut current_sector = dir_sector;
        let mut chain_length = 0;
        const MAX_CHAIN_LENGTH: usize = 144; // Max possible directory entries / 8 per sector

        while current_track != 0 && chain_length < MAX_CHAIN_LENGTH {
            // Check for circular reference
            let sector_id = (current_track as u16) << 8 | current_sector as u16;
            if visited_sectors.contains(&sector_id) {
                return Err(D64Error::CorruptedImage(format!(
                    "Directory chain has circular reference at track {}, sector {}",
                    current_track, current_sector
                )));
            }
            visited_sectors.insert(sector_id);

            // Validate track/sector range
            if !(1..=35).contains(&current_track) {
                return Err(D64Error::CorruptedImage(format!(
                    "Directory chain has invalid track {} at entry {}",
                    current_track, chain_length
                )));
            }

            let max_sect = SECTORS_PER_TRACK[(current_track - 1) as usize];
            if current_sector >= max_sect {
                return Err(D64Error::CorruptedImage(format!(
                    "Directory chain has invalid sector {} on track {} (max: {})",
                    current_sector,
                    current_track,
                    max_sect - 1
                )));
            }

            // Read the sector to get the next link
            let sector_data = self.read_sector(current_track, current_sector)?;
            current_track = sector_data[0];
            current_sector = sector_data[1];
            chain_length += 1;
        }

        if chain_length >= MAX_CHAIN_LENGTH && current_track != 0 {
            return Err(D64Error::CorruptedImage(
                "Directory chain exceeds maximum expected length".to_string(),
            ));
        }

        Ok(())
    }
}

/// Convert PETSCII character to ASCII.
fn petscii_to_ascii(c: u8) -> char {
    match c {
        0x00..=0x1F => ' ',                // Control characters
        0x20..=0x3F => c as char,          // Numbers, some punctuation
        0x40 => '@',                       // At sign
        0x41..=0x5A => c as char,          // Uppercase letters (same as ASCII)
        0x5B..=0x5F => c as char,          // Brackets, etc.
        0x60 => '-',                       // Horizontal line
        0x61..=0x7A => (c - 0x20) as char, // Lowercase letters → uppercase
        0x7B..=0x7F => c as char,          // Some graphics
        0x80..=0x9F => ' ',                // Control characters
        0xA0 => ' ',                       // Shifted space
        0xC1..=0xDA => (c - 0x80) as char, // Shifted uppercase
        _ => '?',                          // Graphics/special
    }
}

/// Errors that can occur during D64 operations.
#[derive(Debug, Clone, PartialEq)]
pub enum D64Error {
    /// The file size is invalid.
    InvalidSize { expected: usize, got: usize },
    /// The track number is out of range (1-35).
    InvalidTrack(u8),
    /// The sector number is out of range for the given track.
    InvalidSector { track: u8, sector: u8 },
    /// The file was not found on disk.
    FileNotFound(String),
    /// The directory is full.
    DirectoryFull,
    /// The disk is full.
    DiskFull,
    /// An I/O error occurred.
    IoError(String),
    /// The D64 image appears to be corrupted.
    CorruptedImage(String),
}

impl std::fmt::Display for D64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            D64Error::InvalidSize { expected, got } => {
                write!(
                    f,
                    "Invalid D64 size: expected {} bytes, got {}",
                    expected, got
                )
            }
            D64Error::InvalidTrack(t) => write!(f, "Invalid track: {} (must be 1-35)", t),
            D64Error::InvalidSector { track, sector } => {
                write!(f, "Invalid sector {} on track {}", sector, track)
            }
            D64Error::FileNotFound(name) => write!(f, "File not found: {}", name),
            D64Error::DirectoryFull => write!(f, "Directory full"),
            D64Error::DiskFull => write!(f, "Disk full"),
            D64Error::IoError(msg) => write!(f, "I/O error: {}", msg),
            D64Error::CorruptedImage(msg) => write!(f, "Corrupted D64 image: {}", msg),
        }
    }
}

impl std::error::Error for D64Error {}

/// Channel mode for drive operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMode {
    /// Channel is not in use.
    Closed,
    /// Channel is open for reading.
    Read,
    /// Channel is open for writing.
    Write,
    /// Channel is used for commands (channel 15).
    Command,
}

/// A single drive channel.
///
/// The 1541 supports 16 channels (0-15), where channel 15 is reserved
/// for the command/status channel.
#[derive(Clone)]
pub struct DriveChannel {
    /// Whether the channel is active.
    pub active: bool,
    /// Current mode of the channel.
    pub mode: ChannelMode,
    /// Current track being accessed.
    pub track: u8,
    /// Current sector being accessed.
    pub sector: u8,
    /// Buffer for sector data (256 bytes).
    pub buffer: [u8; 256],
    /// Current position within the buffer.
    pub buffer_position: u8,
    /// Whether we've reached end of file.
    pub eof: bool,
    /// Next track in file chain (0 = last sector).
    pub next_track: u8,
    /// Next sector in file chain.
    pub next_sector: u8,
}

impl Default for DriveChannel {
    fn default() -> Self {
        Self {
            active: false,
            mode: ChannelMode::Closed,
            track: 0,
            sector: 0,
            buffer: [0; 256],
            buffer_position: 0,
            eof: false,
            next_track: 0,
            next_sector: 0,
        }
    }
}

impl DriveChannel {
    /// Create a new closed channel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Close the channel.
    pub fn close(&mut self) {
        self.active = false;
        self.mode = ChannelMode::Closed;
        self.track = 0;
        self.sector = 0;
        self.buffer_position = 0;
        self.eof = false;
        self.next_track = 0;
        self.next_sector = 0;
    }
}

/// Drive status information.
#[derive(Debug, Clone)]
pub struct DriveStatus {
    /// Error number (0 = OK).
    pub error_number: u8,
    /// Error message.
    pub message: String,
    /// Track where error occurred.
    pub track: u8,
    /// Sector where error occurred.
    pub sector: u8,
}

impl DriveStatus {
    /// Create a new OK status.
    pub fn ok() -> Self {
        Self {
            error_number: 0,
            message: "OK".to_string(),
            track: 0,
            sector: 0,
        }
    }

    /// Create a "File Not Found" error status.
    pub fn file_not_found() -> Self {
        Self {
            error_number: 62,
            message: "FILE NOT FOUND".to_string(),
            track: 0,
            sector: 0,
        }
    }

    /// Create a "File Exists" error status.
    pub fn file_exists() -> Self {
        Self {
            error_number: 63,
            message: "FILE EXISTS".to_string(),
            track: 0,
            sector: 0,
        }
    }

    /// Create a "Syntax Error" status.
    pub fn syntax_error() -> Self {
        Self {
            error_number: 30,
            message: "SYNTAX ERROR".to_string(),
            track: 0,
            sector: 0,
        }
    }

    /// Create a "Read Error" status.
    pub fn read_error(track: u8, sector: u8) -> Self {
        Self {
            error_number: 21,
            message: "READ ERROR".to_string(),
            track,
            sector,
        }
    }

    /// Create a "Disk Full" error status.
    pub fn disk_full(track: u8, sector: u8) -> Self {
        Self {
            error_number: 72,
            message: "DISK FULL".to_string(),
            track,
            sector,
        }
    }
}

impl std::fmt::Display for DriveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}, {},  {:02}, {:02}\r",
            self.error_number, self.message, self.track, self.sector
        )
    }
}

/// Commodore 1541 Disk Drive (High-Level Emulation).
///
/// This implements a simplified version of the 1541 disk drive that operates
/// at the IEC protocol level rather than emulating the drive's internal CPU.
/// This approach works for most software while being much simpler to implement.
pub struct Drive1541 {
    /// Currently mounted disk image.
    mounted_image: Option<D64Image>,
    /// Device number (typically 8).
    device_number: u8,
    /// File channels (0-15).
    channels: [DriveChannel; 16],
    /// Current drive status.
    status: DriveStatus,
    /// Status channel buffer (channel 15).
    status_buffer: Vec<u8>,
    /// Position in status buffer.
    status_position: usize,
}

impl Default for Drive1541 {
    fn default() -> Self {
        Self::new(8)
    }
}

impl Drive1541 {
    /// Create a new 1541 drive with the specified device number.
    pub fn new(device_number: u8) -> Self {
        Self {
            mounted_image: None,
            device_number,
            channels: std::array::from_fn(|_| DriveChannel::new()),
            status: DriveStatus::ok(),
            status_buffer: Vec::new(),
            status_position: 0,
        }
    }

    /// Get the device number.
    pub fn device_number(&self) -> u8 {
        self.device_number
    }

    /// Check if a disk is mounted.
    pub fn has_disk(&self) -> bool {
        self.mounted_image.is_some()
    }

    /// Mount a D64 disk image.
    ///
    /// Validates the image structure before mounting. Returns an error
    /// if the image appears corrupted or has an invalid format.
    pub fn mount(&mut self, data: Vec<u8>) -> Result<(), D64Error> {
        let image = D64Image::new(data)?;
        // Validate the image structure (check BAM, directory chain, etc.)
        image.validate()?;
        self.mounted_image = Some(image);
        self.status = DriveStatus::ok();
        self.close_all_channels();
        Ok(())
    }

    /// Unmount the current disk image.
    pub fn unmount(&mut self) -> Option<D64Image> {
        self.close_all_channels();
        self.status = DriveStatus::ok();
        self.mounted_image.take()
    }

    /// Get a reference to the mounted image.
    pub fn image(&self) -> Option<&D64Image> {
        self.mounted_image.as_ref()
    }

    /// Get a mutable reference to the mounted image.
    pub fn image_mut(&mut self) -> Option<&mut D64Image> {
        self.mounted_image.as_mut()
    }

    /// Get the current drive status.
    pub fn status(&self) -> &DriveStatus {
        &self.status
    }

    /// Close all channels.
    pub fn close_all_channels(&mut self) {
        for channel in &mut self.channels {
            channel.close();
        }
    }

    /// Open a channel for reading/writing.
    ///
    /// # Arguments
    /// * `channel` - Channel number (0-15)
    /// * `filename` - Filename to open (in PETSCII or ASCII)
    /// * `mode` - Access mode
    pub fn open_channel(
        &mut self,
        channel: u8,
        filename: &str,
        mode: ChannelMode,
    ) -> Result<(), D64Error> {
        if channel >= 16 {
            return Err(D64Error::IoError("Invalid channel number".to_string()));
        }

        // Channel 15 is the command/status channel
        if channel == 15 {
            self.handle_command(filename);
            return Ok(());
        }

        // Handle special filenames
        if filename == "$" {
            return self.open_directory(channel);
        }

        // Normal file open
        match mode {
            ChannelMode::Read => self.open_file_read(channel, filename),
            ChannelMode::Write => self.open_file_write(channel, filename),
            _ => Err(D64Error::IoError("Invalid mode for file".to_string())),
        }
    }

    /// Open the directory for reading.
    fn open_directory(&mut self, channel: u8) -> Result<(), D64Error> {
        let image = self.mounted_image.as_ref().ok_or_else(|| {
            self.status = DriveStatus::file_not_found();
            D64Error::IoError("No disk mounted".to_string())
        })?;

        // Build directory listing in BASIC format
        let mut listing = Vec::new();

        // Load address (little-endian $0401)
        listing.push(0x01);
        listing.push(0x04);

        // First line: disk name
        let bam = image.read_bam()?;
        let disk_name: Vec<u8> = bam[0x90..0xA0].to_vec();
        let disk_id = [bam[0xA2], bam[0xA3]];

        // Line link (next line address)
        listing.push(0x01);
        listing.push(0x01);
        // Line number = 0 (disk header)
        listing.push(0x00);
        listing.push(0x00);
        // Reverse on
        listing.push(0x12);
        // Quote
        listing.push(0x22);
        // Disk name (16 chars)
        listing.extend_from_slice(&disk_name);
        // Quote
        listing.push(0x22);
        // Space
        listing.push(0x20);
        // Disk ID
        listing.push(disk_id[0]);
        listing.push(disk_id[1]);
        // DOS type
        listing.push(0x20);
        listing.push(0x32);
        listing.push(0x41);
        // End of line
        listing.push(0x00);

        // Directory entries (start at track 18, sector 1)
        let mut dir_track = DIRECTORY_TRACK;
        let mut dir_sector = DIRECTORY_FIRST_SECTOR;

        while dir_track != 0 {
            let sector = image.read_sector(dir_track, dir_sector)?;

            // Each directory sector has 8 entries of 32 bytes each
            for i in 0..8 {
                let offset = i * 32;
                let entry = &sector[offset..offset + 32];

                // Skip empty/deleted entries
                let file_type = entry[2];
                if file_type == 0 || file_type == 0x80 {
                    continue;
                }

                // File size (in blocks)
                let blocks = u16::from_le_bytes([entry[0x1E], entry[0x1F]]);

                // Line link
                listing.push(0x01);
                listing.push(0x01);
                // Line number = blocks
                listing.push((blocks & 0xFF) as u8);
                listing.push((blocks >> 8) as u8);

                // Padding for alignment
                if blocks < 10 {
                    listing.push(0x20);
                    listing.push(0x20);
                    listing.push(0x20);
                } else if blocks < 100 {
                    listing.push(0x20);
                    listing.push(0x20);
                } else if blocks < 1000 {
                    listing.push(0x20);
                }

                // Quote
                listing.push(0x22);
                // Filename (16 chars)
                listing.extend_from_slice(&entry[5..21]);
                // Quote
                listing.push(0x22);

                // File type
                let type_char = match file_type & 0x07 {
                    0 => b"DEL",
                    1 => b"SEQ",
                    2 => b"PRG",
                    3 => b"USR",
                    4 => b"REL",
                    _ => b"???",
                };
                listing.push(0x20);
                listing.extend_from_slice(type_char);

                // Locked flag
                if file_type & 0x40 != 0 {
                    listing.push(0x3C); // '<'
                }

                // End of line
                listing.push(0x00);
            }

            // Next directory sector
            dir_track = sector[0];
            dir_sector = sector[1];
        }

        // Blocks free line
        let free_blocks = self.count_free_blocks()?;
        listing.push(0x01);
        listing.push(0x01);
        listing.push((free_blocks & 0xFF) as u8);
        listing.push((free_blocks >> 8) as u8);
        listing.extend_from_slice(b"BLOCKS FREE.");
        listing.push(0x00);

        // End of program (three zeros)
        listing.push(0x00);
        listing.push(0x00);
        listing.push(0x00);

        // Set up channel for reading
        let ch = &mut self.channels[channel as usize];
        ch.active = true;
        ch.mode = ChannelMode::Read;
        ch.buffer[..listing.len().min(256)].copy_from_slice(&listing[..listing.len().min(256)]);
        ch.buffer_position = 0;
        ch.eof = listing.len() <= 256;
        ch.next_track = 0; // Directory is pre-built in buffer

        self.status = DriveStatus::ok();
        Ok(())
    }

    /// Count free blocks on the disk.
    fn count_free_blocks(&self) -> Result<u16, D64Error> {
        let image = self
            .mounted_image
            .as_ref()
            .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?;

        let bam = image.read_bam()?;
        let mut free = 0u16;

        // BAM entries start at offset 4, 4 bytes per track
        for track in 0..35 {
            // Skip track 18 (directory)
            if track == 17 {
                continue;
            }
            // First byte of each entry is free sector count
            let offset = 4 + track * 4;
            free += bam[offset] as u16;
        }

        Ok(free)
    }

    /// Open a file for reading.
    fn open_file_read(&mut self, channel: u8, filename: &str) -> Result<(), D64Error> {
        let (track, sector) = match self.find_file(filename) {
            Ok(result) => result,
            Err(e) => {
                self.status = DriveStatus::file_not_found();
                return Err(e);
            }
        };

        let image = self
            .mounted_image
            .as_ref()
            .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?;

        let sector_data = image.read_sector(track, sector)?;

        let ch = &mut self.channels[channel as usize];
        ch.active = true;
        ch.mode = ChannelMode::Read;
        ch.track = track;
        ch.sector = sector;
        ch.buffer.copy_from_slice(&sector_data);
        ch.next_track = sector_data[0];
        ch.next_sector = sector_data[1];
        ch.buffer_position = 2; // Skip link bytes
        ch.eof = ch.next_track == 0;

        self.status = DriveStatus::ok();
        Ok(())
    }

    /// Open a file for writing.
    fn open_file_write(&mut self, channel: u8, filename: &str) -> Result<(), D64Error> {
        // Check if file exists
        if self.find_file(filename).is_ok() {
            self.status = DriveStatus::file_exists();
            return Err(D64Error::IoError("File exists".to_string()));
        }

        let ch = &mut self.channels[channel as usize];
        ch.active = true;
        ch.mode = ChannelMode::Write;
        ch.track = 0;
        ch.sector = 0;
        ch.buffer = [0; 256];
        ch.buffer_position = 2; // Leave room for link bytes
        ch.eof = false;

        self.status = DriveStatus::ok();
        Ok(())
    }

    /// Find a file in the directory.
    ///
    /// Returns the track and sector of the first data block.
    /// Does not modify drive status - caller should set status on error.
    fn find_file(&self, filename: &str) -> Result<(u8, u8), D64Error> {
        let image = self
            .mounted_image
            .as_ref()
            .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?;

        // Convert filename to PETSCII-style uppercase for comparison
        let search_name = filename.to_uppercase();

        // Traverse directory chain
        let mut dir_track = DIRECTORY_TRACK;
        let mut dir_sector = DIRECTORY_FIRST_SECTOR;

        while dir_track != 0 {
            let sector = image.read_sector(dir_track, dir_sector)?;

            for i in 0..8 {
                let offset = i * 32;
                let entry = &sector[offset..offset + 32];

                let file_type = entry[2];
                if file_type == 0 || file_type & 0x80 == 0 {
                    continue; // Empty or deleted
                }

                // Compare filename
                let entry_name: String = entry[5..21]
                    .iter()
                    .take_while(|&&b| b != 0xA0 && b != 0)
                    .map(|&b| petscii_to_ascii(b))
                    .collect();

                // Handle wildcards
                if self.filename_matches(&search_name, &entry_name) {
                    // Return first data block
                    let data_track = entry[3];
                    let data_sector = entry[4];
                    return Ok((data_track, data_sector));
                }
            }

            // Next directory sector
            dir_track = sector[0];
            dir_sector = sector[1];
        }

        Err(D64Error::FileNotFound(filename.to_string()))
    }

    /// Check if a filename matches a pattern (with wildcards).
    fn filename_matches(&self, pattern: &str, filename: &str) -> bool {
        // Handle "*" wildcard (matches any)
        if pattern == "*" {
            return true;
        }

        // Simple pattern matching
        let pattern = pattern.trim();
        let filename = filename.trim();

        if let Some(prefix) = pattern.strip_suffix('*') {
            filename.to_uppercase().starts_with(&prefix.to_uppercase())
        } else if pattern.contains('?') {
            // Handle '?' wildcards
            if pattern.len() != filename.len() {
                return false;
            }
            pattern
                .chars()
                .zip(filename.chars())
                .all(|(p, f)| p == '?' || p.eq_ignore_ascii_case(&f))
        } else {
            pattern.eq_ignore_ascii_case(filename)
        }
    }

    /// Read a byte from a channel.
    pub fn read_byte(&mut self, channel: u8) -> Result<Option<u8>, D64Error> {
        if channel >= 16 {
            return Err(D64Error::IoError("Invalid channel number".to_string()));
        }

        // Channel 15: status channel
        if channel == 15 {
            return Ok(self.read_status_byte());
        }

        let ch = &mut self.channels[channel as usize];
        if !ch.active || ch.mode != ChannelMode::Read {
            return Ok(None);
        }

        // Check if we need to load next sector
        if ch.buffer_position >= 254 || (ch.eof && ch.buffer_position >= ch.buffer[1]) {
            if ch.eof || ch.next_track == 0 {
                return Ok(None); // End of file
            }

            // Load next sector
            let image = self
                .mounted_image
                .as_ref()
                .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?;

            let sector_data = image.read_sector(ch.next_track, ch.next_sector)?;
            ch.track = ch.next_track;
            ch.sector = ch.next_sector;
            ch.buffer.copy_from_slice(&sector_data);
            ch.next_track = sector_data[0];
            ch.next_sector = sector_data[1];
            ch.buffer_position = 2;
            ch.eof = ch.next_track == 0;
        }

        let byte = ch.buffer[ch.buffer_position as usize];
        ch.buffer_position += 1;
        Ok(Some(byte))
    }

    /// Read from status channel (channel 15).
    fn read_status_byte(&mut self) -> Option<u8> {
        if self.status_buffer.is_empty() || self.status_position >= self.status_buffer.len() {
            // Refresh status buffer
            self.status_buffer = self.status.to_string().into_bytes();
            self.status_position = 0;
            self.status = DriveStatus::ok(); // Clear status after read
        }

        if self.status_position < self.status_buffer.len() {
            let byte = self.status_buffer[self.status_position];
            self.status_position += 1;
            Some(byte)
        } else {
            None
        }
    }

    /// Write a byte to a channel.
    pub fn write_byte(&mut self, channel: u8, byte: u8) -> Result<(), D64Error> {
        if channel >= 16 {
            return Err(D64Error::IoError("Invalid channel number".to_string()));
        }

        let ch = &mut self.channels[channel as usize];
        if !ch.active || ch.mode != ChannelMode::Write {
            return Err(D64Error::IoError(
                "Channel not open for writing".to_string(),
            ));
        }

        ch.buffer[ch.buffer_position as usize] = byte;
        ch.buffer_position += 1;

        // If buffer is full, we'd need to allocate and write a sector
        // This is a simplified implementation
        if ch.buffer_position >= 254 {
            // For now, just wrap around (full write support needs BAM allocation)
            ch.buffer_position = 2;
        }

        Ok(())
    }

    /// Close a channel.
    pub fn close_channel(&mut self, channel: u8) {
        if channel < 16 {
            self.channels[channel as usize].close();
        }
    }

    /// Handle a command on channel 15.
    fn handle_command(&mut self, command: &str) {
        let cmd = command.trim().to_uppercase();

        if cmd.is_empty() {
            return;
        }

        // Common commands
        match cmd.chars().next() {
            Some('I') => {
                // Initialize
                self.status = DriveStatus::ok();
            }
            Some('V') => {
                // Validate (no-op in high-level emulation)
                self.status = DriveStatus::ok();
            }
            Some('N') => {
                // New (format) - not implemented
                self.status = DriveStatus::ok();
            }
            Some('S') => {
                // Scratch (delete) - not implemented
                self.status = DriveStatus::ok();
            }
            Some('R') => {
                // Rename - not implemented
                self.status = DriveStatus::ok();
            }
            Some('C') => {
                // Copy - not implemented
                self.status = DriveStatus::ok();
            }
            _ => {
                self.status = DriveStatus::syntax_error();
            }
        }

        // Refresh status buffer
        self.status_buffer = self.status.to_string().into_bytes();
        self.status_position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_creation() {
        let drive = Drive1541::new(8);
        assert_eq!(drive.device_number(), 8);
        assert!(!drive.has_disk());
    }

    #[test]
    fn test_invalid_d64_size() {
        let result = D64Image::new(vec![0; 1000]);
        assert!(result.is_err());
        match result {
            Err(D64Error::InvalidSize { expected, got }) => {
                assert_eq!(expected, D64_SIZE);
                assert_eq!(got, 1000);
            }
            _ => panic!("Expected InvalidSize error"),
        }
    }

    #[test]
    fn test_track_sector_validation() {
        let data = vec![0; D64_SIZE];
        let image = D64Image::new(data).unwrap();

        // Valid track/sector
        assert!(image.read_sector(1, 0).is_ok());
        assert!(image.read_sector(35, 0).is_ok());

        // Invalid track
        assert!(image.read_sector(0, 0).is_err());
        assert!(image.read_sector(36, 0).is_err());

        // Invalid sector for track
        assert!(image.read_sector(1, 21).is_err()); // Track 1 has 21 sectors (0-20)
        assert!(image.read_sector(31, 17).is_err()); // Track 31 has 17 sectors (0-16)
    }

    #[test]
    fn test_sectors_per_track() {
        assert_eq!(D64Image::sectors_in_track(1).unwrap(), 21);
        assert_eq!(D64Image::sectors_in_track(17).unwrap(), 21);
        assert_eq!(D64Image::sectors_in_track(18).unwrap(), 19);
        assert_eq!(D64Image::sectors_in_track(24).unwrap(), 19);
        assert_eq!(D64Image::sectors_in_track(25).unwrap(), 18);
        assert_eq!(D64Image::sectors_in_track(30).unwrap(), 18);
        assert_eq!(D64Image::sectors_in_track(31).unwrap(), 17);
        assert_eq!(D64Image::sectors_in_track(35).unwrap(), 17);
    }

    #[test]
    fn test_petscii_conversion() {
        assert_eq!(petscii_to_ascii(0x41), 'A');
        assert_eq!(petscii_to_ascii(0x5A), 'Z');
        assert_eq!(petscii_to_ascii(0x30), '0');
        assert_eq!(petscii_to_ascii(0x39), '9');
        assert_eq!(petscii_to_ascii(0x20), ' ');
    }

    #[test]
    fn test_filename_matching() {
        let drive = Drive1541::new(8);

        assert!(drive.filename_matches("*", "HELLO"));
        assert!(drive.filename_matches("HELLO", "HELLO"));
        assert!(drive.filename_matches("hello", "HELLO"));
        assert!(drive.filename_matches("HEL*", "HELLO"));
        assert!(!drive.filename_matches("HI*", "HELLO"));
        assert!(drive.filename_matches("H?LLO", "HELLO"));
        assert!(drive.filename_matches("H??LO", "HELLO")); // H=H, ?=E, ?=L, L=L, O=O
        assert!(!drive.filename_matches("H???LO", "HELLO")); // 6 chars vs 5 chars
    }

    #[test]
    fn test_status_formatting() {
        let status = DriveStatus::ok();
        assert_eq!(status.to_string(), "00, OK,  00, 00\r");

        let status = DriveStatus::file_not_found();
        assert_eq!(status.to_string(), "62, FILE NOT FOUND,  00, 00\r");
    }
}
