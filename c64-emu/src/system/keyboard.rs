//! C64 Keyboard matrix emulation.
//!
//! The C64 uses an 8×8 keyboard matrix scanned through CIA1's I/O ports:
//! - CIA1 Port A ($DC00): Column select (active low outputs)
//! - CIA1 Port B ($DC01): Row read (active low inputs)
//!
//! When a key is pressed, it connects a row to a column. The KERNAL scans
//! the keyboard by pulling each column low one at a time and reading which
//! rows go low.
//!
//! ## Keyboard Matrix Layout
//!
//! The 8×8 matrix maps physical keys to (row, col) coordinates:
//!
//! ```text
//! Row\Col |  0    1    2    3    4    5    6    7
//! --------|------------------------------------------
//!    0    | DEL  RET   →   F7   F1   F3   F5   ↓
//!    1    |  3    W    A    4    Z    S    E  LSHFT
//!    2    |  5    R    D    6    C    F    T    X
//!    3    |  7    Y    G    8    B    H    U    V
//!    4    |  9    I    J    0    M    K    O    N
//!    5    |  +    P    L    -    .    :    @    ,
//!    6    |  £    *    ;  HOME RSHFT =    ↑    /
//!    7    |  1    ←  CTRL   2  SPACE  C=   Q  STOP
//! ```
//!
//! Note: The RESTORE key is not part of the matrix - it's connected directly
//! to the NMI line and is handled separately.

/// C64 keyboard matrix state.
///
/// The keyboard is an 8×8 matrix where each key connects a specific row
/// to a specific column when pressed. The matrix state tracks which keys
/// are currently pressed.
#[derive(Debug, Clone)]
pub struct Keyboard {
    /// Key matrix state: [row][col] where true = key pressed.
    /// This mirrors the physical keyboard matrix layout.
    matrix: [[bool; 8]; 8],
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    /// Create a new keyboard with all keys released.
    pub fn new() -> Self {
        Self {
            matrix: [[false; 8]; 8],
        }
    }

    /// Press a key at the specified matrix position.
    ///
    /// # Arguments
    /// * `row` - Row 0-7 in the keyboard matrix
    /// * `col` - Column 0-7 in the keyboard matrix
    ///
    /// # Panics
    /// Panics if row or col is >= 8.
    pub fn key_down(&mut self, row: u8, col: u8) {
        debug_assert!(row < 8 && col < 8, "Matrix position out of range");
        self.matrix[row as usize][col as usize] = true;
    }

    /// Release a key at the specified matrix position.
    ///
    /// # Arguments
    /// * `row` - Row 0-7 in the keyboard matrix
    /// * `col` - Column 0-7 in the keyboard matrix
    ///
    /// # Panics
    /// Panics if row or col is >= 8.
    pub fn key_up(&mut self, row: u8, col: u8) {
        debug_assert!(row < 8 && col < 8, "Matrix position out of range");
        self.matrix[row as usize][col as usize] = false;
    }

    /// Check if a specific key is pressed.
    ///
    /// # Arguments
    /// * `row` - Row 0-7 in the keyboard matrix
    /// * `col` - Column 0-7 in the keyboard matrix
    ///
    /// # Returns
    /// `true` if the key is pressed, `false` otherwise.
    pub fn is_key_pressed(&self, row: u8, col: u8) -> bool {
        if row >= 8 || col >= 8 {
            return false;
        }
        self.matrix[row as usize][col as usize]
    }

    /// Scan the keyboard matrix given the column select value.
    ///
    /// The C64 keyboard is scanned by CIA1 by:
    /// 1. Writing to Port A ($DC00) to select columns (active low)
    /// 2. Reading Port B ($DC01) to see which rows are connected (active low)
    ///
    /// This method simulates the matrix behavior: for each column that is
    /// pulled low (bit = 0 in col_select), if a key in that column is pressed,
    /// the corresponding row bit in the result will be 0.
    ///
    /// # Arguments
    /// * `col_select` - Active-low column select (bit N = 0 means column N is selected)
    ///
    /// # Returns
    /// Active-low row values (bit N = 0 means a key in row N is pressed for selected columns)
    pub fn scan(&self, col_select: u8) -> u8 {
        let mut result = 0xFF; // Start with all rows high (no keys pressed)

        for col in 0..8 {
            // Check if this column is selected (pulled low)
            if col_select & (1 << col) == 0 {
                // Column is active (pulled low), check all rows
                for row in 0..8 {
                    if self.matrix[row][col] {
                        // Key pressed: pull this row low
                        result &= !(1 << row);
                    }
                }
            }
        }

        result
    }

    /// Release all keys.
    pub fn release_all(&mut self) {
        for row in 0..8 {
            for col in 0..8 {
                self.matrix[row][col] = false;
            }
        }
    }

    /// Get the raw matrix state (for debugging/serialization).
    pub fn matrix(&self) -> &[[bool; 8]; 8] {
        &self.matrix
    }

    /// Set the raw matrix state (for deserialization).
    pub fn set_matrix(&mut self, matrix: [[bool; 8]; 8]) {
        self.matrix = matrix;
    }
}

/// C64 keyboard matrix positions for common keys.
///
/// Each constant is a tuple (row, col) representing the position
/// in the 8×8 keyboard matrix.
#[allow(dead_code)]
pub mod keys {
    // Row 0
    pub const DEL: (u8, u8) = (0, 0);
    pub const RETURN: (u8, u8) = (0, 1);
    pub const CRSR_RIGHT: (u8, u8) = (0, 2);
    pub const F7: (u8, u8) = (0, 3);
    pub const F1: (u8, u8) = (0, 4);
    pub const F3: (u8, u8) = (0, 5);
    pub const F5: (u8, u8) = (0, 6);
    pub const CRSR_DOWN: (u8, u8) = (0, 7);

    // Row 1
    pub const DIGIT_3: (u8, u8) = (1, 0);
    pub const W: (u8, u8) = (1, 1);
    pub const A: (u8, u8) = (1, 2);
    pub const DIGIT_4: (u8, u8) = (1, 3);
    pub const Z: (u8, u8) = (1, 4);
    pub const S: (u8, u8) = (1, 5);
    pub const E: (u8, u8) = (1, 6);
    pub const LEFT_SHIFT: (u8, u8) = (1, 7);

    // Row 2
    pub const DIGIT_5: (u8, u8) = (2, 0);
    pub const R: (u8, u8) = (2, 1);
    pub const D: (u8, u8) = (2, 2);
    pub const DIGIT_6: (u8, u8) = (2, 3);
    pub const C: (u8, u8) = (2, 4);
    pub const F: (u8, u8) = (2, 5);
    pub const T: (u8, u8) = (2, 6);
    pub const X: (u8, u8) = (2, 7);

    // Row 3
    pub const DIGIT_7: (u8, u8) = (3, 0);
    pub const Y: (u8, u8) = (3, 1);
    pub const G: (u8, u8) = (3, 2);
    pub const DIGIT_8: (u8, u8) = (3, 3);
    pub const B: (u8, u8) = (3, 4);
    pub const H: (u8, u8) = (3, 5);
    pub const U: (u8, u8) = (3, 6);
    pub const V: (u8, u8) = (3, 7);

    // Row 4
    pub const DIGIT_9: (u8, u8) = (4, 0);
    pub const I: (u8, u8) = (4, 1);
    pub const J: (u8, u8) = (4, 2);
    pub const DIGIT_0: (u8, u8) = (4, 3);
    pub const M: (u8, u8) = (4, 4);
    pub const K: (u8, u8) = (4, 5);
    pub const O: (u8, u8) = (4, 6);
    pub const N: (u8, u8) = (4, 7);

    // Row 5
    pub const PLUS: (u8, u8) = (5, 0);
    pub const P: (u8, u8) = (5, 1);
    pub const L: (u8, u8) = (5, 2);
    pub const MINUS: (u8, u8) = (5, 3);
    pub const PERIOD: (u8, u8) = (5, 4);
    pub const COLON: (u8, u8) = (5, 5);
    pub const AT: (u8, u8) = (5, 6);
    pub const COMMA: (u8, u8) = (5, 7);

    // Row 6
    pub const POUND: (u8, u8) = (6, 0);
    pub const ASTERISK: (u8, u8) = (6, 1);
    pub const SEMICOLON: (u8, u8) = (6, 2);
    pub const HOME: (u8, u8) = (6, 3);
    pub const RIGHT_SHIFT: (u8, u8) = (6, 4);
    pub const EQUALS: (u8, u8) = (6, 5);
    pub const UP_ARROW: (u8, u8) = (6, 6);
    pub const SLASH: (u8, u8) = (6, 7);

    // Row 7
    pub const DIGIT_1: (u8, u8) = (7, 0);
    pub const LEFT_ARROW: (u8, u8) = (7, 1);
    pub const CTRL: (u8, u8) = (7, 2);
    pub const DIGIT_2: (u8, u8) = (7, 3);
    pub const SPACE: (u8, u8) = (7, 4);
    pub const COMMODORE: (u8, u8) = (7, 5);
    pub const Q: (u8, u8) = (7, 6);
    pub const RUN_STOP: (u8, u8) = (7, 7);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_keyboard_all_released() {
        let kb = Keyboard::new();
        for row in 0..8 {
            for col in 0..8 {
                assert!(!kb.is_key_pressed(row, col));
            }
        }
    }

    #[test]
    fn test_key_down_up() {
        let mut kb = Keyboard::new();

        // Press 'A' key (row 1, col 2)
        kb.key_down(1, 2);
        assert!(kb.is_key_pressed(1, 2));
        assert!(!kb.is_key_pressed(0, 0));

        // Release 'A' key
        kb.key_up(1, 2);
        assert!(!kb.is_key_pressed(1, 2));
    }

    #[test]
    fn test_scan_no_keys_pressed() {
        let kb = Keyboard::new();

        // Select all columns (all low)
        assert_eq!(kb.scan(0x00), 0xFF);

        // Select column 0 only
        assert_eq!(kb.scan(0xFE), 0xFF);
    }

    #[test]
    fn test_scan_single_key() {
        let mut kb = Keyboard::new();

        // Press 'A' key (row 1, col 2)
        kb.key_down(1, 2);

        // Scan without column 2 selected - should not see key
        // 0xFF = all bits high = no columns selected
        assert_eq!(kb.scan(0xFF), 0xFF);

        // Select column 0 only (bit 0 = 0) - 'A' is in col 2, so not visible
        // 0xFE = 0b11111110, only column 0 selected
        assert_eq!(kb.scan(0xFE), 0xFF);

        // Scan with column 2 selected (bit 2 = 0)
        // 0xFB = 0b11111011, bit 2 IS 0, so column 2 IS selected
        // Row 1 has a key pressed in col 2, so row 1 should be low (bit 1 = 0)
        // Expected: 0b11111101 = 0xFD
        assert_eq!(kb.scan(0xFB), 0xFD);

        // Scan with all columns selected (0x00)
        // Row 1 should be low: 0xFD
        assert_eq!(kb.scan(0x00), 0xFD);
    }

    #[test]
    fn test_scan_multiple_keys_same_row() {
        let mut kb = Keyboard::new();

        // Press 'A' (row 1, col 2) and 'W' (row 1, col 1)
        kb.key_down(1, 2); // A
        kb.key_down(1, 1); // W

        // Scan all columns - row 1 should be low
        assert_eq!(kb.scan(0x00), 0xFD);

        // Scan only column 1 - row 1 should be low
        assert_eq!(kb.scan(0xFD), 0xFD);

        // Scan only column 2 - row 1 should be low
        assert_eq!(kb.scan(0xFB), 0xFD);

        // Scan only column 0 - row 1 should be high (no key there)
        assert_eq!(kb.scan(0xFE), 0xFF);
    }

    #[test]
    fn test_scan_multiple_keys_different_rows() {
        let mut kb = Keyboard::new();

        // Press 'A' (row 1, col 2) and 'D' (row 2, col 2)
        kb.key_down(1, 2); // A
        kb.key_down(2, 2); // D

        // Scan column 2 - both rows 1 and 2 should be low
        // Expected: 0xFF & ~0x02 & ~0x04 = 0xF9
        assert_eq!(kb.scan(0xFB), 0xF9);

        // Scan all columns
        assert_eq!(kb.scan(0x00), 0xF9);
    }

    #[test]
    fn test_release_all() {
        let mut kb = Keyboard::new();

        kb.key_down(0, 0);
        kb.key_down(1, 1);
        kb.key_down(7, 7);

        kb.release_all();

        for row in 0..8 {
            for col in 0..8 {
                assert!(!kb.is_key_pressed(row, col));
            }
        }
    }

    #[test]
    fn test_key_constants() {
        // Verify a few key positions match the C64 matrix
        assert_eq!(keys::A, (1, 2));
        assert_eq!(keys::RETURN, (0, 1));
        assert_eq!(keys::SPACE, (7, 4));
        assert_eq!(keys::LEFT_SHIFT, (1, 7));
        assert_eq!(keys::RIGHT_SHIFT, (6, 4));
    }

    #[test]
    fn test_matrix_access() {
        let mut kb = Keyboard::new();
        kb.key_down(2, 3);

        let matrix = kb.matrix();
        assert!(matrix[2][3]);
        assert!(!matrix[0][0]);
    }
}
