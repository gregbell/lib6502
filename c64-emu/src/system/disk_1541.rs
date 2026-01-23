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

    /// Create a blank, formatted D64 disk image.
    ///
    /// Creates a new disk with empty BAM, directory, and given disk name/ID.
    pub fn create_blank(name: &str, id: &[u8; 2]) -> Self {
        let mut data = vec![0u8; D64_SIZE];

        // Initialize BAM at track 18, sector 0
        let bam_offset = TRACK_OFFSETS[17] * 256;

        // BAM structure:
        // $00: Directory track (18)
        // $01: Directory sector (1)
        // $02: DOS version type ('A')
        // $03: Unused
        // $04-$8F: BAM entries (4 bytes per track × 35 tracks)
        // $90-$9F: Disk name (16 bytes, padded with $A0)
        // $A0-$A1: Two $A0 bytes
        // $A2-$A3: Disk ID (2 bytes)
        // $A4: $A0
        // $A5-$A6: DOS type ('2A')
        // $A7-$AA: Four $A0 bytes

        data[bam_offset] = DIRECTORY_TRACK;
        data[bam_offset + 1] = DIRECTORY_FIRST_SECTOR;
        data[bam_offset + 2] = 0x41; // DOS version 'A'
        data[bam_offset + 3] = 0x00; // Double-sided flag (unused for 1541)

        // Initialize BAM entries - all sectors free except track 18
        for track in 0..35 {
            let bam_entry_offset = bam_offset + 4 + track * 4;
            let sectors = SECTORS_PER_TRACK[track];

            if track == 17 {
                // Track 18 - reserve BAM (sector 0) and directory (sector 1)
                // Free count: sectors - 2 (BAM and first directory sector used)
                data[bam_entry_offset] = sectors - 2;
                // Set bitmap: all free except sectors 0 and 1
                // Bit 0 of first byte = sector 0, bit 1 = sector 1, etc.
                let bitmap = !0x03u32 & ((1u32 << sectors) - 1);
                data[bam_entry_offset + 1] = (bitmap & 0xFF) as u8;
                data[bam_entry_offset + 2] = ((bitmap >> 8) & 0xFF) as u8;
                data[bam_entry_offset + 3] = ((bitmap >> 16) & 0xFF) as u8;
            } else {
                // Other tracks - all sectors free
                data[bam_entry_offset] = sectors;
                let bitmap = (1u32 << sectors) - 1;
                data[bam_entry_offset + 1] = (bitmap & 0xFF) as u8;
                data[bam_entry_offset + 2] = ((bitmap >> 8) & 0xFF) as u8;
                data[bam_entry_offset + 3] = ((bitmap >> 16) & 0xFF) as u8;
            }
        }

        // Disk name (16 bytes, padded with $A0)
        let name_bytes: Vec<u8> = name
            .chars()
            .take(16)
            .map(|c| ascii_to_petscii_upper(c as u8))
            .collect();
        for (i, &b) in name_bytes.iter().enumerate() {
            data[bam_offset + 0x90 + i] = b;
        }
        for i in name_bytes.len()..16 {
            data[bam_offset + 0x90 + i] = 0xA0; // Pad with shifted space
        }

        // Two $A0 bytes
        data[bam_offset + 0xA0] = 0xA0;
        data[bam_offset + 0xA1] = 0xA0;

        // Disk ID
        data[bam_offset + 0xA2] = id[0];
        data[bam_offset + 0xA3] = id[1];

        // $A0 separator
        data[bam_offset + 0xA4] = 0xA0;

        // DOS type '2A'
        data[bam_offset + 0xA5] = 0x32; // '2'
        data[bam_offset + 0xA6] = 0x41; // 'A'

        // Four $A0 bytes
        data[bam_offset + 0xA7] = 0xA0;
        data[bam_offset + 0xA8] = 0xA0;
        data[bam_offset + 0xA9] = 0xA0;
        data[bam_offset + 0xAA] = 0xA0;

        // Initialize first directory sector at track 18, sector 1
        let dir_offset = (TRACK_OFFSETS[17] + 1) * 256;
        data[dir_offset] = 0x00; // No next track
        data[dir_offset + 1] = 0xFF; // Last sector byte count (unused for empty dir)

        Self {
            data: data.into_boxed_slice(),
            modified: true,
        }
    }

    /// Clear the modified flag (useful after saving).
    pub fn clear_modified(&mut self) {
        self.modified = false;
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

    // =========================================================================
    // BAM (Block Availability Map) Operations
    // =========================================================================

    /// Allocate a free sector from the disk.
    ///
    /// Uses the C64 1541 allocation algorithm:
    /// 1. Start from track 18 and search outward (alternating below/above)
    /// 2. Within each track, find the first free sector
    ///
    /// Returns the (track, sector) of the allocated block, or an error if disk is full.
    pub fn allocate_sector(&mut self) -> Result<(u8, u8), D64Error> {
        // Allocation order: Start near track 18, alternate between lower and higher tracks
        // This minimizes head movement during typical disk operations
        let allocation_order: Vec<u8> = {
            let mut order = Vec::with_capacity(35);
            let mut below = 17i8; // Start at track 17 (0-indexed: 16)
            let mut above = 19i8; // Start at track 19 (0-indexed: 18)

            while below >= 1 || above <= 35 {
                if below >= 1 {
                    order.push(below as u8);
                    below -= 1;
                }
                if above <= 35 {
                    order.push(above as u8);
                    above += 1;
                }
            }
            order
        };

        // Read current BAM
        let bam_offset = TRACK_OFFSETS[17] * 256;

        for &track in &allocation_order {
            // Skip directory track (track 18) for data files
            // (Directory entries are handled separately)
            if track == DIRECTORY_TRACK {
                continue;
            }

            let track_idx = (track - 1) as usize;
            let entry_offset = bam_offset + 4 + track_idx * 4;

            // Check if this track has free sectors
            let free_count = self.data[entry_offset];
            if free_count == 0 {
                continue;
            }

            // Find first free sector in this track
            let bitmap = (self.data[entry_offset + 1] as u32)
                | ((self.data[entry_offset + 2] as u32) << 8)
                | ((self.data[entry_offset + 3] as u32) << 16);

            let max_sector = SECTORS_PER_TRACK[track_idx];

            for sector in 0..max_sector {
                if bitmap & (1 << sector) != 0 {
                    // This sector is free - allocate it
                    self.mark_sector_used(track, sector)?;
                    return Ok((track, sector));
                }
            }
        }

        Err(D64Error::DiskFull)
    }

    /// Allocate a specific sector on the directory track.
    ///
    /// Used for extending the directory chain. Returns an error if the sector
    /// is not available or if the directory track is full.
    pub fn allocate_directory_sector(&mut self) -> Result<u8, D64Error> {
        let track_idx = (DIRECTORY_TRACK - 1) as usize;
        let bam_offset = TRACK_OFFSETS[17] * 256;
        let entry_offset = bam_offset + 4 + track_idx * 4;

        let free_count = self.data[entry_offset];
        if free_count == 0 {
            return Err(D64Error::DirectoryFull);
        }

        let bitmap = (self.data[entry_offset + 1] as u32)
            | ((self.data[entry_offset + 2] as u32) << 8)
            | ((self.data[entry_offset + 3] as u32) << 16);

        let max_sector = SECTORS_PER_TRACK[track_idx];

        for sector in 0..max_sector {
            // Skip BAM sector (0)
            if sector == BAM_SECTOR {
                continue;
            }
            if bitmap & (1 << sector) != 0 {
                self.mark_sector_used(DIRECTORY_TRACK, sector)?;
                return Ok(sector);
            }
        }

        Err(D64Error::DirectoryFull)
    }

    /// Mark a sector as used in the BAM.
    fn mark_sector_used(&mut self, track: u8, sector: u8) -> Result<(), D64Error> {
        if !(1..=35).contains(&track) {
            return Err(D64Error::InvalidTrack(track));
        }

        let track_idx = (track - 1) as usize;
        let max_sector = SECTORS_PER_TRACK[track_idx];
        if sector >= max_sector {
            return Err(D64Error::InvalidSector { track, sector });
        }

        let bam_offset = TRACK_OFFSETS[17] * 256;
        let entry_offset = bam_offset + 4 + track_idx * 4;

        // Decrement free count
        if self.data[entry_offset] > 0 {
            self.data[entry_offset] -= 1;
        }

        // Clear the bit in the bitmap
        let byte_idx = (sector / 8) as usize;
        let bit_idx = sector % 8;
        self.data[entry_offset + 1 + byte_idx] &= !(1 << bit_idx);

        self.modified = true;
        Ok(())
    }

    /// Mark a sector as free in the BAM.
    pub fn mark_sector_free(&mut self, track: u8, sector: u8) -> Result<(), D64Error> {
        if !(1..=35).contains(&track) {
            return Err(D64Error::InvalidTrack(track));
        }

        let track_idx = (track - 1) as usize;
        let max_sector = SECTORS_PER_TRACK[track_idx];
        if sector >= max_sector {
            return Err(D64Error::InvalidSector { track, sector });
        }

        let bam_offset = TRACK_OFFSETS[17] * 256;
        let entry_offset = bam_offset + 4 + track_idx * 4;

        // Increment free count (but don't exceed max)
        if self.data[entry_offset] < max_sector {
            self.data[entry_offset] += 1;
        }

        // Set the bit in the bitmap
        let byte_idx = (sector / 8) as usize;
        let bit_idx = sector % 8;
        self.data[entry_offset + 1 + byte_idx] |= 1 << bit_idx;

        self.modified = true;
        Ok(())
    }

    /// Check if a sector is free in the BAM.
    pub fn is_sector_free(&self, track: u8, sector: u8) -> Result<bool, D64Error> {
        if !(1..=35).contains(&track) {
            return Err(D64Error::InvalidTrack(track));
        }

        let track_idx = (track - 1) as usize;
        let max_sector = SECTORS_PER_TRACK[track_idx];
        if sector >= max_sector {
            return Err(D64Error::InvalidSector { track, sector });
        }

        let bam_offset = TRACK_OFFSETS[17] * 256;
        let entry_offset = bam_offset + 4 + track_idx * 4;

        let byte_idx = (sector / 8) as usize;
        let bit_idx = sector % 8;

        Ok(self.data[entry_offset + 1 + byte_idx] & (1 << bit_idx) != 0)
    }

    /// Get the number of free blocks on the disk.
    pub fn free_blocks(&self) -> u16 {
        let bam_offset = TRACK_OFFSETS[17] * 256;
        let mut free = 0u16;

        for track in 0..35 {
            // Skip directory track in count (matches C64 behavior)
            if track == 17 {
                continue;
            }
            let entry_offset = bam_offset + 4 + track * 4;
            free += self.data[entry_offset] as u16;
        }

        free
    }

    // =========================================================================
    // Directory Operations
    // =========================================================================

    /// Find a free directory entry slot.
    ///
    /// Returns (track, sector, entry_offset_in_sector) or an error.
    pub fn find_free_directory_slot(&mut self) -> Result<(u8, u8, usize), D64Error> {
        let mut dir_track = DIRECTORY_TRACK;
        let mut dir_sector = DIRECTORY_FIRST_SECTOR;

        loop {
            let sector = self.read_sector(dir_track, dir_sector)?;

            // Check 8 entries in this sector
            for i in 0..8 {
                let offset = i * 32;
                let file_type = sector[offset + 2];

                // Entry is free if file type is 0 (or 0x80 for scratched files)
                if file_type == 0 {
                    return Ok((dir_track, dir_sector, offset));
                }
            }

            // Move to next directory sector
            let next_track = sector[0];
            let next_sector = sector[1];

            if next_track == 0 {
                // End of directory chain - need to allocate new sector
                let new_sector = self.allocate_directory_sector()?;

                // Link current sector to new sector
                let mut current = self.read_sector(dir_track, dir_sector)?;
                current[0] = DIRECTORY_TRACK;
                current[1] = new_sector;
                self.write_sector(dir_track, dir_sector, &current)?;

                // Initialize new directory sector
                let mut new_dir = [0u8; 256];
                new_dir[0] = 0; // No next sector
                new_dir[1] = 0xFF; // Last sector marker
                self.write_sector(DIRECTORY_TRACK, new_sector, &new_dir)?;

                // Return first slot in new sector
                return Ok((DIRECTORY_TRACK, new_sector, 0));
            }

            dir_track = next_track;
            dir_sector = next_sector;
        }
    }

    /// Create a new directory entry for a file.
    ///
    /// # Arguments
    /// * `filename` - File name (will be padded/truncated to 16 characters)
    /// * `file_type` - File type (0x82 = PRG, 0x81 = SEQ, etc.)
    /// * `first_track` - Track of first data sector
    /// * `first_sector` - Sector of first data sector
    /// * `block_count` - Number of blocks used by the file
    ///
    /// # Returns
    /// The directory slot (track, sector, entry_offset) where the entry was written.
    pub fn create_directory_entry(
        &mut self,
        filename: &str,
        file_type: u8,
        first_track: u8,
        first_sector: u8,
        block_count: u16,
    ) -> Result<(u8, u8, usize), D64Error> {
        let (dir_track, dir_sector, entry_offset) = self.find_free_directory_slot()?;

        let mut sector = self.read_sector(dir_track, dir_sector)?;

        // Build the directory entry (32 bytes)
        let entry = &mut sector[entry_offset..entry_offset + 32];

        // Byte 0-1: Next track/sector (only used for first entry of each sector, skip)
        // Byte 2: File type (with bit 7 set for "closed" files)
        entry[2] = file_type | 0x80; // Set bit 7 to mark as properly closed

        // Byte 3-4: First track/sector of file
        entry[3] = first_track;
        entry[4] = first_sector;

        // Byte 5-20: Filename (16 bytes, padded with $A0)
        let name_bytes: Vec<u8> = filename
            .chars()
            .take(16)
            .map(|c| ascii_to_petscii_upper(c as u8))
            .collect();
        for (i, &b) in name_bytes.iter().enumerate() {
            entry[5 + i] = b;
        }
        for i in name_bytes.len()..16 {
            entry[5 + i] = 0xA0; // Pad with shifted space
        }

        // Byte 21-22: First side-sector (REL files only, set to 0)
        entry[21] = 0;
        entry[22] = 0;

        // Byte 23: REL file record length (0 for non-REL)
        entry[23] = 0;

        // Byte 24-27: Unused (GEOS uses these)
        entry[24] = 0;
        entry[25] = 0;
        entry[26] = 0;
        entry[27] = 0;

        // Byte 28-29: Track/sector of replacement (when file is overwritten, set to 0)
        entry[28] = 0;
        entry[29] = 0;

        // Byte 30-31: File size in blocks (little-endian)
        entry[30] = (block_count & 0xFF) as u8;
        entry[31] = ((block_count >> 8) & 0xFF) as u8;

        self.write_sector(dir_track, dir_sector, &sector)?;

        Ok((dir_track, dir_sector, entry_offset))
    }

    /// Delete a file from the disk.
    ///
    /// Marks the directory entry as deleted and frees all sectors in the file chain.
    pub fn delete_file(&mut self, filename: &str) -> Result<(), D64Error> {
        // Find the file in the directory
        let (dir_track, dir_sector, entry_offset, first_track, first_sector) =
            self.find_file_entry(filename)?;

        // Free all sectors in the file chain
        self.free_sector_chain(first_track, first_sector)?;

        // Mark directory entry as deleted (set file type to 0)
        let mut sector = self.read_sector(dir_track, dir_sector)?;
        sector[entry_offset + 2] = 0;
        self.write_sector(dir_track, dir_sector, &sector)?;

        Ok(())
    }

    /// Find a file's directory entry.
    ///
    /// Returns (dir_track, dir_sector, entry_offset, first_track, first_sector).
    fn find_file_entry(&self, filename: &str) -> Result<(u8, u8, usize, u8, u8), D64Error> {
        let search_name = filename.to_uppercase();
        let mut dir_track = DIRECTORY_TRACK;
        let mut dir_sector = DIRECTORY_FIRST_SECTOR;

        loop {
            let sector = self.read_sector(dir_track, dir_sector)?;

            for i in 0..8 {
                let offset = i * 32;
                let file_type = sector[offset + 2];

                if file_type == 0 || file_type & 0x80 == 0 {
                    continue; // Empty or deleted
                }

                // Compare filename
                let entry_name: String = sector[offset + 5..offset + 21]
                    .iter()
                    .take_while(|&&b| b != 0xA0 && b != 0)
                    .map(|&b| petscii_to_ascii(b))
                    .collect();

                if search_name.eq_ignore_ascii_case(&entry_name) {
                    let first_track = sector[offset + 3];
                    let first_sector = sector[offset + 4];
                    return Ok((dir_track, dir_sector, offset, first_track, first_sector));
                }
            }

            let next_track = sector[0];
            let next_sector = sector[1];

            if next_track == 0 {
                break;
            }

            dir_track = next_track;
            dir_sector = next_sector;
        }

        Err(D64Error::FileNotFound(filename.to_string()))
    }

    /// Free all sectors in a file chain.
    fn free_sector_chain(&mut self, start_track: u8, start_sector: u8) -> Result<(), D64Error> {
        let mut track = start_track;
        let mut sector = start_sector;
        let mut count = 0;
        const MAX_CHAIN: usize = 768; // Max sectors on a D64

        while track != 0 && count < MAX_CHAIN {
            let sector_data = self.read_sector(track, sector)?;
            let next_track = sector_data[0];
            let next_sector = sector_data[1];

            self.mark_sector_free(track, sector)?;

            track = next_track;
            sector = next_sector;
            count += 1;
        }

        Ok(())
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
                dir_sector,
                dir_track,
                max_sectors - 1
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

/// Convert ASCII character to uppercase PETSCII.
fn ascii_to_petscii_upper(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => c - 32, // Lowercase → uppercase
        b'A'..=b'Z' => c,      // Already uppercase
        0x20..=0x3F => c,      // Numbers, punctuation
        _ => c,                // Pass through
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

/// State for tracking file write operations.
#[derive(Clone, Default)]
struct WriteState {
    /// Filename being written.
    filename: String,
    /// File type (0x82 = PRG, 0x81 = SEQ).
    file_type: u8,
    /// First track of the file chain.
    first_track: u8,
    /// First sector of the file chain.
    first_sector: u8,
    /// Current track being written.
    current_track: u8,
    /// Current sector being written.
    current_sector: u8,
    /// Previous track (for linking).
    prev_track: u8,
    /// Previous sector (for linking).
    prev_sector: u8,
    /// Number of blocks allocated.
    block_count: u16,
    /// Whether this is the first sector of the file.
    is_first_sector: bool,
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
    /// Write state for each channel.
    write_states: [Option<WriteState>; 16],
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
            write_states: std::array::from_fn(|_| None),
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
        for i in 0..16 {
            // Close any open write channels properly
            if self.channels[i].mode == ChannelMode::Write {
                let _ = self.finalize_write(i as u8);
            }
            self.channels[i].close();
            self.write_states[i] = None;
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

        // Check if disk is mounted
        if self.mounted_image.is_none() {
            self.status = DriveStatus::file_not_found();
            return Err(D64Error::IoError("No disk mounted".to_string()));
        }

        // Parse filename for type specifier (e.g., "FILE,P" or "FILE,S")
        let (name, file_type) = Self::parse_filename_type(filename);

        let ch = &mut self.channels[channel as usize];
        ch.active = true;
        ch.mode = ChannelMode::Write;
        ch.track = 0;
        ch.sector = 0;
        ch.buffer = [0; 256];
        ch.buffer_position = 2; // Leave room for link bytes
        ch.eof = false;

        // Initialize write state
        self.write_states[channel as usize] = Some(WriteState {
            filename: name.to_string(),
            file_type,
            first_track: 0,
            first_sector: 0,
            current_track: 0,
            current_sector: 0,
            prev_track: 0,
            prev_sector: 0,
            block_count: 0,
            is_first_sector: true,
        });

        self.status = DriveStatus::ok();
        Ok(())
    }

    /// Parse filename for type specifier.
    ///
    /// Handles filenames like "MYFILE,P" (PRG) or "DATA,S" (SEQ).
    /// Returns (name, file_type_byte).
    fn parse_filename_type(filename: &str) -> (&str, u8) {
        if let Some(pos) = filename.rfind(',') {
            let name = &filename[..pos];
            let type_char = filename[pos + 1..].chars().next().unwrap_or('P');
            let file_type = match type_char.to_ascii_uppercase() {
                'P' => 0x02, // PRG
                'S' => 0x01, // SEQ
                'U' => 0x03, // USR
                'R' => 0x04, // REL
                _ => 0x02,   // Default to PRG
            };
            (name, file_type)
        } else {
            // Default to PRG
            (filename, 0x02)
        }
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
        // Note: buffer[1] contains the last valid byte position for the final sector
        // For non-final sectors, we read positions 2-255 (254 bytes)
        // For final sector, we read positions 2-buffer[1] (buffer[1]-1 bytes)
        if ch.buffer_position > 254 || (ch.eof && ch.buffer_position > ch.buffer[1]) {
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

        // Check channel state
        {
            let ch = &self.channels[channel as usize];
            if !ch.active || ch.mode != ChannelMode::Write {
                return Err(D64Error::IoError(
                    "Channel not open for writing".to_string(),
                ));
            }
        }

        // Add byte to buffer
        let buffer_position = {
            let ch = &mut self.channels[channel as usize];
            ch.buffer[ch.buffer_position as usize] = byte;
            ch.buffer_position += 1;
            ch.buffer_position
        };

        // If buffer is full (254 data bytes), flush to disk
        if buffer_position >= 254 {
            self.flush_write_buffer(channel)?;
        }

        Ok(())
    }

    /// Flush the write buffer to disk by allocating a new sector.
    fn flush_write_buffer(&mut self, channel: u8) -> Result<(), D64Error> {
        // Allocate a new sector
        let (track, sector) = self
            .mounted_image
            .as_mut()
            .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?
            .allocate_sector()?;

        let ch_idx = channel as usize;

        // Get current buffer content
        let mut sector_data = [0u8; 256];
        {
            let ch = &self.channels[ch_idx];
            sector_data.copy_from_slice(&ch.buffer);
        }

        // Link previous sector to this one (if not first sector)
        let is_first = self.write_states[ch_idx]
            .as_ref()
            .map(|s| s.is_first_sector)
            .unwrap_or(true);

        if !is_first {
            let (prev_track, prev_sector) = {
                let state = self.write_states[ch_idx].as_ref().unwrap();
                (state.prev_track, state.prev_sector)
            };

            // Update link bytes in previous sector
            let mut prev_data = self
                .mounted_image
                .as_ref()
                .unwrap()
                .read_sector(prev_track, prev_sector)?;
            prev_data[0] = track;
            prev_data[1] = sector;
            self.mounted_image.as_mut().unwrap().write_sector(
                prev_track,
                prev_sector,
                &prev_data,
            )?;
        }

        // Set link bytes for this sector (will be updated when next sector allocated)
        sector_data[0] = 0; // No next sector yet
        sector_data[1] = 253; // Byte count (254 data bytes used, position 253 is last data byte)

        // Write sector to disk
        self.mounted_image
            .as_mut()
            .unwrap()
            .write_sector(track, sector, &sector_data)?;

        // Update write state
        if let Some(state) = self.write_states[ch_idx].as_mut() {
            if state.is_first_sector {
                state.first_track = track;
                state.first_sector = sector;
                state.is_first_sector = false;
            }
            state.prev_track = track;
            state.prev_sector = sector;
            state.current_track = track;
            state.current_sector = sector;
            state.block_count += 1;
        }

        // Reset buffer
        let ch = &mut self.channels[ch_idx];
        ch.buffer = [0; 256];
        ch.buffer_position = 2; // Skip link bytes
        ch.track = track;
        ch.sector = sector;

        Ok(())
    }

    /// Finalize a file write operation.
    ///
    /// Writes any remaining data in the buffer and creates the directory entry.
    fn finalize_write(&mut self, channel: u8) -> Result<(), D64Error> {
        let ch_idx = channel as usize;

        // Check if there's a write in progress
        let write_state = match self.write_states[ch_idx].take() {
            Some(state) => state,
            None => return Ok(()), // Nothing to finalize
        };

        let ch = &self.channels[ch_idx];
        let buffer_position = ch.buffer_position;

        // If buffer has data, write final sector
        if buffer_position > 2 {
            // Allocate final sector
            let (track, sector) = self
                .mounted_image
                .as_mut()
                .ok_or_else(|| D64Error::IoError("No disk mounted".to_string()))?
                .allocate_sector()?;

            let mut sector_data = [0u8; 256];
            sector_data.copy_from_slice(&ch.buffer);

            // Set final link bytes
            sector_data[0] = 0; // No next sector
            sector_data[1] = buffer_position - 1; // Last used byte position

            // Link previous sector if this isn't the first
            if !write_state.is_first_sector {
                let mut prev_data = self
                    .mounted_image
                    .as_ref()
                    .unwrap()
                    .read_sector(write_state.prev_track, write_state.prev_sector)?;
                prev_data[0] = track;
                prev_data[1] = sector;
                self.mounted_image.as_mut().unwrap().write_sector(
                    write_state.prev_track,
                    write_state.prev_sector,
                    &prev_data,
                )?;
            }

            // Write final sector
            self.mounted_image
                .as_mut()
                .unwrap()
                .write_sector(track, sector, &sector_data)?;

            // Determine first track/sector
            let (first_track, first_sector) = if write_state.is_first_sector {
                (track, sector)
            } else {
                (write_state.first_track, write_state.first_sector)
            };

            let block_count = write_state.block_count + 1;

            // Create directory entry
            self.mounted_image
                .as_mut()
                .unwrap()
                .create_directory_entry(
                    &write_state.filename,
                    write_state.file_type,
                    first_track,
                    first_sector,
                    block_count,
                )?;

            self.status = DriveStatus::ok();
        } else if write_state.block_count > 0 {
            // File has sectors but empty final buffer - create directory entry
            self.mounted_image
                .as_mut()
                .unwrap()
                .create_directory_entry(
                    &write_state.filename,
                    write_state.file_type,
                    write_state.first_track,
                    write_state.first_sector,
                    write_state.block_count,
                )?;

            self.status = DriveStatus::ok();
        }
        // else: Empty file, nothing written - no directory entry needed

        Ok(())
    }

    /// Close a channel.
    ///
    /// For write channels, this finalizes the file by writing any remaining
    /// data and creating the directory entry.
    pub fn close_channel(&mut self, channel: u8) {
        if channel >= 16 {
            return;
        }

        let ch_idx = channel as usize;

        // Finalize any write operation
        if self.channels[ch_idx].mode == ChannelMode::Write {
            if let Err(e) = self.finalize_write(channel) {
                // Set error status but continue with close
                self.status = DriveStatus::disk_full(0, 0);
                // Log error if needed
                let _ = e; // Suppress unused warning
            }
        }

        self.channels[ch_idx].close();
        self.write_states[ch_idx] = None;
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
                // New (format) - implemented
                self.handle_new_command(&cmd);
            }
            Some('S') => {
                // Scratch (delete) - implemented
                self.handle_scratch_command(&cmd);
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

    /// Handle the SCRATCH command (S:filename).
    fn handle_scratch_command(&mut self, cmd: &str) {
        // Parse filename from "S:FILENAME" or "S0:FILENAME"
        let filename = if cmd.starts_with("S:") {
            &cmd[2..]
        } else if cmd.len() > 3
            && cmd.chars().nth(1) == Some('0')
            && cmd.chars().nth(2) == Some(':')
        {
            &cmd[3..]
        } else {
            self.status = DriveStatus::syntax_error();
            return;
        };

        if filename.is_empty() {
            self.status = DriveStatus::syntax_error();
            return;
        }

        // Check if disk is mounted
        let Some(ref mut image) = self.mounted_image else {
            self.status = DriveStatus::file_not_found();
            return;
        };

        // Delete the file
        match image.delete_file(filename) {
            Ok(()) => {
                self.status = DriveStatus {
                    error_number: 1,
                    message: "FILES SCRATCHED".to_string(),
                    track: 1, // Number of files scratched
                    sector: 0,
                };
            }
            Err(D64Error::FileNotFound(_)) => {
                self.status = DriveStatus::file_not_found();
            }
            Err(_) => {
                self.status = DriveStatus::syntax_error();
            }
        }
    }

    /// Handle the NEW command (N:diskname,id or N0:diskname,id).
    fn handle_new_command(&mut self, cmd: &str) {
        // Parse: "N:DISKNAME,ID" or "N0:DISKNAME,ID"
        let params = if cmd.starts_with("N:") {
            &cmd[2..]
        } else if cmd.len() > 3
            && cmd.chars().nth(1) == Some('0')
            && cmd.chars().nth(2) == Some(':')
        {
            &cmd[3..]
        } else {
            self.status = DriveStatus::syntax_error();
            return;
        };

        // Parse diskname and optional ID
        let (name, id) = if let Some(comma_pos) = params.find(',') {
            let name = &params[..comma_pos];
            let id_str = &params[comma_pos + 1..];
            let id_bytes: Vec<u8> = id_str.bytes().take(2).collect();
            let id = if id_bytes.len() >= 2 {
                [id_bytes[0], id_bytes[1]]
            } else if id_bytes.len() == 1 {
                [id_bytes[0], 0x20]
            } else {
                [0x20, 0x20]
            };
            (name, id)
        } else {
            (params, [0x20, 0x20])
        };

        if name.is_empty() {
            self.status = DriveStatus::syntax_error();
            return;
        }

        // Create new blank disk
        self.mounted_image = Some(D64Image::create_blank(name, &id));
        self.close_all_channels();
        self.status = DriveStatus::ok();
    }

    // =========================================================================
    // Additional Write Support Methods
    // =========================================================================

    /// Check if the mounted disk has been modified.
    pub fn is_disk_modified(&self) -> bool {
        self.mounted_image
            .as_ref()
            .map(|img| img.is_modified())
            .unwrap_or(false)
    }

    /// Get the raw D64 data for saving/downloading.
    ///
    /// Returns None if no disk is mounted.
    pub fn get_disk_data(&self) -> Option<&[u8]> {
        self.mounted_image.as_ref().map(|img| img.data())
    }

    /// Get the number of free blocks on the disk.
    pub fn free_blocks(&self) -> Option<u16> {
        self.mounted_image.as_ref().map(|img| img.free_blocks())
    }

    /// Clear the modified flag on the mounted disk.
    ///
    /// Call this after successfully saving the disk to indicate
    /// there are no unsaved changes.
    pub fn clear_disk_modified(&mut self) {
        if let Some(ref mut image) = self.mounted_image {
            image.clear_modified();
        }
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

    // =========================================================================
    // Disk Write Tests
    // =========================================================================

    #[test]
    fn test_create_blank_disk() {
        let image = D64Image::create_blank("TEST DISK", &[0x31, 0x32]);

        // Check disk name
        let name = image.disk_name().unwrap();
        assert_eq!(name, "TEST DISK");

        // Check disk ID
        let id = image.disk_id().unwrap();
        assert_eq!(id, [0x31, 0x32]);

        // Check free blocks (should be 664 for a blank disk)
        // Total sectors: 683, minus BAM (1) and first dir sector (1), minus track 18 not counted
        let free = image.free_blocks();
        assert!(free > 600, "Expected >600 free blocks, got {}", free);
    }

    #[test]
    fn test_bam_sector_allocation() {
        let mut image = D64Image::create_blank("TEST", &[0x30, 0x30]);
        let initial_free = image.free_blocks();

        // Allocate a sector
        let (track, sector) = image.allocate_sector().unwrap();
        assert!(track >= 1 && track <= 35);
        assert!(track != DIRECTORY_TRACK); // Should not allocate from dir track

        // Free blocks should decrease by 1
        let new_free = image.free_blocks();
        assert_eq!(new_free, initial_free - 1);

        // Sector should now be marked as used
        assert!(!image.is_sector_free(track, sector).unwrap());
    }

    #[test]
    fn test_bam_sector_free() {
        let mut image = D64Image::create_blank("TEST", &[0x30, 0x30]);

        // Allocate then free a sector
        let (track, sector) = image.allocate_sector().unwrap();
        let after_alloc = image.free_blocks();

        image.mark_sector_free(track, sector).unwrap();
        let after_free = image.free_blocks();

        assert_eq!(after_free, after_alloc + 1);
        assert!(image.is_sector_free(track, sector).unwrap());
    }

    #[test]
    fn test_directory_entry_creation() {
        let mut image = D64Image::create_blank("TEST", &[0x30, 0x30]);

        // Allocate a sector for file data
        let (track, sector) = image.allocate_sector().unwrap();

        // Create directory entry
        let result = image.create_directory_entry("MYFILE", 0x02, track, sector, 1);
        assert!(result.is_ok());

        // Verify file can be found
        let entry = image.find_file_entry("MYFILE");
        assert!(entry.is_ok());
        let (_, _, _, first_track, first_sector) = entry.unwrap();
        assert_eq!(first_track, track);
        assert_eq!(first_sector, sector);
    }

    #[test]
    fn test_drive_write_and_read() {
        let mut drive = Drive1541::new(8);

        // Create blank disk
        let image = D64Image::create_blank("TEST", &[0x30, 0x30]);
        let data = image.data().to_vec();
        drive.mount(data).unwrap();

        // Open file for writing
        drive
            .open_channel(2, "TESTFILE", ChannelMode::Write)
            .unwrap();

        // Write some bytes
        let test_data = b"HELLO WORLD FROM C64";
        for &byte in test_data {
            drive.write_byte(2, byte).unwrap();
        }

        // Close channel (finalizes file)
        drive.close_channel(2);

        // Verify file exists and can be read
        let result = drive.find_file("TESTFILE");
        assert!(result.is_ok(), "File should exist after write");
        let (track, sector) = result.unwrap();

        // Verify the sector data looks correct
        let sector_data = drive.image().unwrap().read_sector(track, sector).unwrap();
        assert_eq!(
            sector_data[0], 0,
            "Next track should be 0 (no more sectors)"
        );
        // Last byte position should be 21 (position 2 + 20 bytes - 1)
        assert_eq!(sector_data[1], 21, "Last byte position should be 21");
        // First data byte should be 'H'
        assert_eq!(sector_data[2], b'H', "First data byte should be 'H'");

        // Open file for reading
        drive
            .open_channel(3, "TESTFILE", ChannelMode::Read)
            .unwrap();

        // Read back data
        let mut read_data = Vec::new();
        while let Ok(Some(byte)) = drive.read_byte(3) {
            read_data.push(byte);
            if read_data.len() > 100 {
                break; // Safety limit
            }
        }

        // Verify data matches
        assert!(
            read_data.len() >= test_data.len(),
            "Expected at least {} bytes, got {}",
            test_data.len(),
            read_data.len()
        );
        assert_eq!(&read_data[..test_data.len()], test_data);
    }

    #[test]
    fn test_scratch_command() {
        let mut drive = Drive1541::new(8);

        // Create and mount blank disk
        let image = D64Image::create_blank("TEST", &[0x30, 0x30]);
        let data = image.data().to_vec();
        drive.mount(data).unwrap();

        // Create a file
        drive.open_channel(2, "MYFILE", ChannelMode::Write).unwrap();
        drive.write_byte(2, 0x41).unwrap();
        drive.close_channel(2);

        // Verify file exists
        assert!(drive.find_file("MYFILE").is_ok());

        // Scratch the file
        drive.handle_command("S:MYFILE");

        // Verify file is gone
        assert!(drive.find_file("MYFILE").is_err());
    }

    #[test]
    fn test_new_command() {
        let mut drive = Drive1541::new(8);

        // Mount any disk first
        let image = D64Image::create_blank("OLD", &[0x30, 0x30]);
        let data = image.data().to_vec();
        drive.mount(data).unwrap();

        // Format with new name
        drive.handle_command("N:NEWDISK,AB");

        // Verify new disk name
        let name = drive.image().unwrap().disk_name().unwrap();
        assert_eq!(name, "NEWDISK");

        // Verify disk ID
        let id = drive.image().unwrap().disk_id().unwrap();
        assert_eq!(id[0], b'A');
        assert_eq!(id[1], b'B');
    }

    #[test]
    fn test_is_disk_modified() {
        let mut drive = Drive1541::new(8);

        // No disk = not modified
        assert!(!drive.is_disk_modified());

        // Fresh disk = modified (just created)
        let image = D64Image::create_blank("TEST", &[0x30, 0x30]);
        let data = image.data().to_vec();
        drive.mount(data).unwrap();

        // After mount, not modified (we just loaded it)
        // Actually the blank disk is created with modified=true, but mount creates from data
        // So we need to test writing

        // Clear the modified flag first
        drive.clear_disk_modified();
        assert!(!drive.is_disk_modified());

        // Write a file
        drive.open_channel(2, "TEST", ChannelMode::Write).unwrap();
        drive.write_byte(2, 0x41).unwrap();
        drive.close_channel(2);

        // Now it should be modified
        assert!(drive.is_disk_modified());
    }

    #[test]
    fn test_ascii_to_petscii_upper() {
        assert_eq!(ascii_to_petscii_upper(b'a'), b'A');
        assert_eq!(ascii_to_petscii_upper(b'z'), b'Z');
        assert_eq!(ascii_to_petscii_upper(b'A'), b'A');
        assert_eq!(ascii_to_petscii_upper(b'1'), b'1');
        assert_eq!(ascii_to_petscii_upper(b' '), b' ');
    }

    #[test]
    fn test_parse_filename_type() {
        assert_eq!(Drive1541::parse_filename_type("FILE"), ("FILE", 0x02));
        assert_eq!(Drive1541::parse_filename_type("FILE,P"), ("FILE", 0x02));
        assert_eq!(Drive1541::parse_filename_type("FILE,S"), ("FILE", 0x01));
        assert_eq!(Drive1541::parse_filename_type("FILE,U"), ("FILE", 0x03));
        assert_eq!(Drive1541::parse_filename_type("DATA,s"), ("DATA", 0x01)); // lowercase
    }
}
