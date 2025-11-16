//! Source map for bidirectional mapping between binary and source locations

/// Bidirectional mapping between binary and source locations
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Forward map: instruction address → source location
    /// Sorted by address for binary search
    address_to_source: Vec<(u16, SourceLocation)>,

    /// Reverse map: source line → instruction address ranges
    /// Sorted by line number for binary search
    source_to_address: Vec<(usize, AddressRange)>,
}

/// A location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column where instruction starts (0-indexed)
    pub column: usize,

    /// Length of instruction in source characters
    pub length: usize,
}

/// A range of instruction addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressRange {
    /// Starting address (inclusive)
    pub start: u16,

    /// Ending address (exclusive)
    pub end: u16,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self {
            address_to_source: Vec::new(),
            source_to_address: Vec::new(),
        }
    }

    /// Add a mapping from instruction address to source location
    pub fn add_mapping(&mut self, address: u16, location: SourceLocation) {
        self.address_to_source.push((address, location));
    }

    /// Add a mapping from source line to address range
    pub fn add_line_mapping(&mut self, line: usize, range: AddressRange) {
        self.source_to_address.push((line, range));
    }

    /// Get source location for a given instruction address
    pub fn get_source_location(&self, address: u16) -> Option<SourceLocation> {
        self.address_to_source
            .binary_search_by_key(&address, |(addr, _)| *addr)
            .ok()
            .map(|idx| self.address_to_source[idx].1)
    }

    /// Get address range for a given source line
    pub fn get_address_range(&self, line: usize) -> Option<AddressRange> {
        self.source_to_address
            .binary_search_by_key(&line, |(l, _)| *l)
            .ok()
            .map(|idx| self.source_to_address[idx].1)
    }

    /// Finalize the source map (sort for binary search)
    pub fn finalize(&mut self) {
        self.address_to_source.sort_by_key(|(addr, _)| *addr);
        self.source_to_address.sort_by_key(|(line, _)| *line);
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map_add_lookup() {
        let mut map = SourceMap::new();

        map.add_mapping(
            0x8000,
            SourceLocation {
                line: 1,
                column: 0,
                length: 10,
            },
        );
        map.add_mapping(
            0x8002,
            SourceLocation {
                line: 2,
                column: 4,
                length: 12,
            },
        );

        map.finalize();

        let loc = map.get_source_location(0x8000).unwrap();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 0);

        let loc2 = map.get_source_location(0x8002).unwrap();
        assert_eq!(loc2.line, 2);
        assert_eq!(loc2.column, 4);

        assert!(map.get_source_location(0x9000).is_none());
    }

    #[test]
    fn test_source_map_reverse_lookup() {
        let mut map = SourceMap::new();

        map.add_line_mapping(
            1,
            AddressRange {
                start: 0x8000,
                end: 0x8002,
            },
        );
        map.add_line_mapping(
            2,
            AddressRange {
                start: 0x8002,
                end: 0x8005,
            },
        );
        map.add_line_mapping(
            3,
            AddressRange {
                start: 0x8005,
                end: 0x8006,
            },
        );

        map.finalize();

        let range1 = map.get_address_range(1).unwrap();
        assert_eq!(range1.start, 0x8000);
        assert_eq!(range1.end, 0x8002);

        let range2 = map.get_address_range(2).unwrap();
        assert_eq!(range2.start, 0x8002);
        assert_eq!(range2.end, 0x8005);

        assert!(map.get_address_range(99).is_none());
    }

    #[test]
    fn test_source_map_bidirectional() {
        let mut map = SourceMap::new();

        // Add both forward and reverse mappings
        map.add_mapping(
            0x8000,
            SourceLocation {
                line: 1,
                column: 0,
                length: 10,
            },
        );
        map.add_line_mapping(
            1,
            AddressRange {
                start: 0x8000,
                end: 0x8002,
            },
        );

        map.add_mapping(
            0x8002,
            SourceLocation {
                line: 2,
                column: 4,
                length: 12,
            },
        );
        map.add_line_mapping(
            2,
            AddressRange {
                start: 0x8002,
                end: 0x8005,
            },
        );

        map.finalize();

        // Test forward lookup (address → source)
        let loc = map.get_source_location(0x8000).unwrap();
        assert_eq!(loc.line, 1);

        // Test reverse lookup (line → address)
        let range = map.get_address_range(1).unwrap();
        assert_eq!(range.start, 0x8000);
        assert_eq!(range.end, 0x8002);
    }

    #[test]
    fn test_source_map_empty() {
        let map = SourceMap::new();

        assert!(map.get_source_location(0x8000).is_none());
        assert!(map.get_address_range(1).is_none());
    }

    #[test]
    fn test_source_map_unsorted_input() {
        let mut map = SourceMap::new();

        // Add mappings in non-sorted order
        map.add_mapping(
            0x8005,
            SourceLocation {
                line: 3,
                column: 0,
                length: 5,
            },
        );
        map.add_mapping(
            0x8000,
            SourceLocation {
                line: 1,
                column: 0,
                length: 10,
            },
        );
        map.add_mapping(
            0x8002,
            SourceLocation {
                line: 2,
                column: 0,
                length: 8,
            },
        );

        map.finalize();

        // Should still find all entries after finalize sorts them
        assert!(map.get_source_location(0x8000).is_some());
        assert!(map.get_source_location(0x8002).is_some());
        assert!(map.get_source_location(0x8005).is_some());
    }

    #[test]
    fn test_source_location_equality() {
        let loc1 = SourceLocation {
            line: 1,
            column: 0,
            length: 10,
        };
        let loc2 = SourceLocation {
            line: 1,
            column: 0,
            length: 10,
        };
        let loc3 = SourceLocation {
            line: 2,
            column: 0,
            length: 10,
        };

        assert_eq!(loc1, loc2);
        assert_ne!(loc1, loc3);
    }

    #[test]
    fn test_address_range_equality() {
        let range1 = AddressRange {
            start: 0x8000,
            end: 0x8002,
        };
        let range2 = AddressRange {
            start: 0x8000,
            end: 0x8002,
        };
        let range3 = AddressRange {
            start: 0x8000,
            end: 0x8005,
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
    }
}
