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
//! The 8×8 matrix maps physical keys. The KERNAL's keyboard decode table at
//! $EB81 is indexed as `col * 8 + row`, so key mappings use (col, row) order.
//!
//! ```text
//! Col\Row |  0    1    2    3    4    5    6    7
//! --------|------------------------------------------
//!    0    | DEL   3    5    7    9    +    £    1
//!    1    | RET   W    R    Y    I    P    *    ←
//!    2    |  →    A    D    G    J    L    ;  CTRL
//!    3    | F7    4    6    8    0    -  HOME   2
//!    4    | F1    Z    C    B    M    .  RSHFT SPC
//!    5    | F3    S    F    H    K    :    =   C=
//!    6    | F5    E    T    U    O    @    ↑    Q
//!    7    |  ↓  LSHFT  X    V    N    ,    /  STOP
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

/// PC-to-C64 key mapping result.
///
/// Represents how a PC keycode maps to the C64 keyboard matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapping {
    /// Row in the C64 keyboard matrix (0-7).
    pub row: u8,
    /// Column in the C64 keyboard matrix (0-7).
    pub col: u8,
    /// Whether this mapping requires holding SHIFT on the C64.
    ///
    /// Some PC keys (like '!' which is Shift+1) need to be converted to
    /// their C64 equivalents with the appropriate shift state.
    pub requires_shift: bool,
}

impl KeyMapping {
    /// Create a new key mapping.
    pub const fn new(row: u8, col: u8) -> Self {
        Self {
            row,
            col,
            requires_shift: false,
        }
    }

    /// Create a new key mapping that requires SHIFT.
    pub const fn with_shift(row: u8, col: u8) -> Self {
        Self {
            row,
            col,
            requires_shift: true,
        }
    }
}

/// Map a PC keyboard event `code` to a C64 key matrix position.
///
/// This uses the Web API `KeyboardEvent.code` values (e.g., "KeyA", "Digit1",
/// "Space", "Enter"). Returns `None` if the key has no C64 equivalent.
///
/// # Examples
///
/// ```
/// use c64_emu::map_pc_keycode;
///
/// // Map the 'A' key
/// if let Some(mapping) = map_pc_keycode("KeyA") {
///     assert_eq!(mapping.row, 2);
///     assert_eq!(mapping.col, 1);
///     assert!(!mapping.requires_shift);
/// }
///
/// // Map Enter to RETURN
/// let enter = map_pc_keycode("Enter").unwrap();
/// assert_eq!((enter.row, enter.col), (1, 0));
/// ```
///
/// # Supported Keys
///
/// - Letters: KeyA through KeyZ
/// - Digits: Digit0 through Digit9
/// - Function keys: F1, F3, F5, F7 (native), F2, F4, F6, F8 (shifted F1/F3/F5/F7)
/// - Modifiers: ShiftLeft, ShiftRight, ControlLeft, ControlRight
/// - Special: Space, Enter, Backspace, Escape, Tab
/// - Punctuation: Period, Comma, Slash, Semicolon, Quote, etc.
/// - Navigation: ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Home
pub fn map_pc_keycode(code: &str) -> Option<KeyMapping> {
    // NOTE: The C64 KERNAL uses col*8+row indexing for the keyboard decode table.
    // Our matrix is stored as matrix[row][col], but the KERNAL expects the scan
    // to produce (col, row) pairs. So we swap the arguments here: new(col, row).
    match code {
        // Letters (direct mapping) - arguments are (col, row) for KERNAL compatibility
        "KeyA" => Some(KeyMapping::new(2, 1)),
        "KeyB" => Some(KeyMapping::new(4, 3)),
        "KeyC" => Some(KeyMapping::new(4, 2)),
        "KeyD" => Some(KeyMapping::new(2, 2)),
        "KeyE" => Some(KeyMapping::new(6, 1)),
        "KeyF" => Some(KeyMapping::new(5, 2)),
        "KeyG" => Some(KeyMapping::new(2, 3)),
        "KeyH" => Some(KeyMapping::new(5, 3)),
        "KeyI" => Some(KeyMapping::new(1, 4)),
        "KeyJ" => Some(KeyMapping::new(2, 4)),
        "KeyK" => Some(KeyMapping::new(5, 4)),
        "KeyL" => Some(KeyMapping::new(2, 5)),
        "KeyM" => Some(KeyMapping::new(4, 4)),
        "KeyN" => Some(KeyMapping::new(7, 4)),
        "KeyO" => Some(KeyMapping::new(6, 4)),
        "KeyP" => Some(KeyMapping::new(1, 5)),
        "KeyQ" => Some(KeyMapping::new(6, 7)),
        "KeyR" => Some(KeyMapping::new(1, 2)),
        "KeyS" => Some(KeyMapping::new(5, 1)),
        "KeyT" => Some(KeyMapping::new(6, 2)),
        "KeyU" => Some(KeyMapping::new(6, 3)),
        "KeyV" => Some(KeyMapping::new(7, 3)),
        "KeyW" => Some(KeyMapping::new(1, 1)),
        "KeyX" => Some(KeyMapping::new(7, 2)),
        "KeyY" => Some(KeyMapping::new(1, 3)),
        "KeyZ" => Some(KeyMapping::new(4, 1)),

        // Digits (top row) - arguments are (col, row)
        "Digit1" => Some(KeyMapping::new(0, 7)),
        "Digit2" => Some(KeyMapping::new(3, 7)),
        "Digit3" => Some(KeyMapping::new(0, 1)),
        "Digit4" => Some(KeyMapping::new(3, 1)),
        "Digit5" => Some(KeyMapping::new(0, 2)),
        "Digit6" => Some(KeyMapping::new(3, 2)),
        "Digit7" => Some(KeyMapping::new(0, 3)),
        "Digit8" => Some(KeyMapping::new(3, 3)),
        "Digit9" => Some(KeyMapping::new(0, 4)),
        "Digit0" => Some(KeyMapping::new(3, 4)),

        // Function keys - arguments are (col, row)
        // C64 has F1, F3, F5, F7 - F2/F4/F6/F8 are Shift versions
        "F1" => Some(KeyMapping::new(4, 0)),
        "F2" => Some(KeyMapping::with_shift(4, 0)), // Shift+F1
        "F3" => Some(KeyMapping::new(5, 0)),
        "F4" => Some(KeyMapping::with_shift(5, 0)), // Shift+F3
        "F5" => Some(KeyMapping::new(6, 0)),
        "F6" => Some(KeyMapping::with_shift(6, 0)), // Shift+F5
        "F7" => Some(KeyMapping::new(3, 0)),
        "F8" => Some(KeyMapping::with_shift(3, 0)), // Shift+F7

        // Modifiers - arguments are (col, row)
        "ShiftLeft" => Some(KeyMapping::new(7, 1)),
        "ShiftRight" => Some(KeyMapping::new(4, 6)),
        "ControlLeft" | "ControlRight" => Some(KeyMapping::new(2, 7)), // CTRL
        "AltLeft" | "AltRight" | "MetaLeft" | "MetaRight" => {
            Some(KeyMapping::new(5, 7)) // Commodore key
        }

        // Common keys - arguments are (col, row)
        "Space" => Some(KeyMapping::new(4, 7)),
        "Enter" | "NumpadEnter" => Some(KeyMapping::new(1, 0)), // RETURN
        "Backspace" => Some(KeyMapping::new(0, 0)),             // DEL/INST
        "Escape" => Some(KeyMapping::new(7, 7)),                // RUN/STOP
        "Tab" => Some(KeyMapping::new(2, 7)),                   // Map to CTRL

        // Navigation - arguments are (col, row)
        "ArrowUp" => Some(KeyMapping::with_shift(7, 0)), // Shift + CRSR DOWN
        "ArrowDown" => Some(KeyMapping::new(7, 0)),      // CRSR DOWN
        "ArrowLeft" => Some(KeyMapping::with_shift(2, 0)), // Shift + CRSR RIGHT
        "ArrowRight" => Some(KeyMapping::new(2, 0)),     // CRSR RIGHT
        "Home" => Some(KeyMapping::new(3, 6)),           // CLR/HOME

        // Punctuation - arguments are (col, row)
        "Period" => Some(KeyMapping::new(4, 5)),       // .
        "Comma" => Some(KeyMapping::new(7, 5)),        // ,
        "Slash" => Some(KeyMapping::new(7, 6)),        // /
        "Semicolon" => Some(KeyMapping::new(2, 6)),    // ; (shifted: ])
        "Quote" => Some(KeyMapping::with_shift(0, 3)), // ' is Shift+7 on C64
        "BracketLeft" => Some(KeyMapping::new(5, 5)),  // : (C64 has : not [)
        "BracketRight" => Some(KeyMapping::new(1, 6)), // * on C64
        "Backslash" => Some(KeyMapping::new(0, 6)),    // £ (Pound sign)
        "Backquote" => Some(KeyMapping::new(1, 7)),    // ← (left arrow)
        "Minus" => Some(KeyMapping::new(3, 5)),        // - on C64
        "Equal" => Some(KeyMapping::new(5, 6)),        // = on C64

        // Numpad (map to their regular digit/operator equivalents) - arguments are (col, row)
        "Numpad0" => Some(KeyMapping::new(3, 4)),
        "Numpad1" => Some(KeyMapping::new(0, 7)),
        "Numpad2" => Some(KeyMapping::new(3, 7)),
        "Numpad3" => Some(KeyMapping::new(0, 1)),
        "Numpad4" => Some(KeyMapping::new(3, 1)),
        "Numpad5" => Some(KeyMapping::new(0, 2)),
        "Numpad6" => Some(KeyMapping::new(3, 2)),
        "Numpad7" => Some(KeyMapping::new(0, 3)),
        "Numpad8" => Some(KeyMapping::new(3, 3)),
        "Numpad9" => Some(KeyMapping::new(0, 4)),
        "NumpadAdd" => Some(KeyMapping::new(0, 5)), // +
        "NumpadSubtract" => Some(KeyMapping::new(3, 5)), // -
        "NumpadMultiply" => Some(KeyMapping::new(1, 6)), // *
        "NumpadDivide" => Some(KeyMapping::new(7, 6)), // /
        "NumpadDecimal" => Some(KeyMapping::new(4, 5)), // .

        // Insert/Delete - arguments are (col, row)
        "Insert" => Some(KeyMapping::with_shift(0, 0)), // Shift+DEL = INST
        "Delete" => Some(KeyMapping::new(0, 0)),        // DEL

        // Special characters that require shift
        // These are handled by the requires_shift flag for proper emulation

        // No mapping for this key
        _ => None,
    }
}

/// C64 keyboard matrix positions for common keys.
///
/// Each constant is a tuple (col, row) representing the position
/// in the 8×8 keyboard matrix. This matches the KERNAL's decode table
/// indexing: index = col * 8 + row.
#[allow(dead_code)]
pub mod keys {
    // Column 0 keys
    pub const DEL: (u8, u8) = (0, 0);
    pub const DIGIT_3: (u8, u8) = (0, 1);
    pub const DIGIT_5: (u8, u8) = (0, 2);
    pub const DIGIT_7: (u8, u8) = (0, 3);
    pub const DIGIT_9: (u8, u8) = (0, 4);
    pub const PLUS: (u8, u8) = (0, 5);
    pub const POUND: (u8, u8) = (0, 6);
    pub const DIGIT_1: (u8, u8) = (0, 7);

    // Column 1 keys
    pub const RETURN: (u8, u8) = (1, 0);
    pub const W: (u8, u8) = (1, 1);
    pub const R: (u8, u8) = (1, 2);
    pub const Y: (u8, u8) = (1, 3);
    pub const I: (u8, u8) = (1, 4);
    pub const P: (u8, u8) = (1, 5);
    pub const ASTERISK: (u8, u8) = (1, 6);
    pub const LEFT_ARROW: (u8, u8) = (1, 7);

    // Column 2 keys
    pub const CRSR_RIGHT: (u8, u8) = (2, 0);
    pub const A: (u8, u8) = (2, 1);
    pub const D: (u8, u8) = (2, 2);
    pub const G: (u8, u8) = (2, 3);
    pub const J: (u8, u8) = (2, 4);
    pub const L: (u8, u8) = (2, 5);
    pub const SEMICOLON: (u8, u8) = (2, 6);
    pub const CTRL: (u8, u8) = (2, 7);

    // Column 3 keys
    pub const F7: (u8, u8) = (3, 0);
    pub const DIGIT_4: (u8, u8) = (3, 1);
    pub const DIGIT_6: (u8, u8) = (3, 2);
    pub const DIGIT_8: (u8, u8) = (3, 3);
    pub const DIGIT_0: (u8, u8) = (3, 4);
    pub const MINUS: (u8, u8) = (3, 5);
    pub const HOME: (u8, u8) = (3, 6);
    pub const DIGIT_2: (u8, u8) = (3, 7);

    // Column 4 keys
    pub const F1: (u8, u8) = (4, 0);
    pub const Z: (u8, u8) = (4, 1);
    pub const C: (u8, u8) = (4, 2);
    pub const B: (u8, u8) = (4, 3);
    pub const M: (u8, u8) = (4, 4);
    pub const PERIOD: (u8, u8) = (4, 5);
    pub const RIGHT_SHIFT: (u8, u8) = (4, 6);
    pub const SPACE: (u8, u8) = (4, 7);

    // Column 5 keys
    pub const F3: (u8, u8) = (5, 0);
    pub const S: (u8, u8) = (5, 1);
    pub const F: (u8, u8) = (5, 2);
    pub const H: (u8, u8) = (5, 3);
    pub const K: (u8, u8) = (5, 4);
    pub const COLON: (u8, u8) = (5, 5);
    pub const EQUALS: (u8, u8) = (5, 6);
    pub const COMMODORE: (u8, u8) = (5, 7);

    // Column 6 keys
    pub const F5: (u8, u8) = (6, 0);
    pub const E: (u8, u8) = (6, 1);
    pub const T: (u8, u8) = (6, 2);
    pub const U: (u8, u8) = (6, 3);
    pub const O: (u8, u8) = (6, 4);
    pub const AT: (u8, u8) = (6, 5);
    pub const UP_ARROW: (u8, u8) = (6, 6);
    pub const Q: (u8, u8) = (6, 7);

    // Column 7 keys
    pub const CRSR_DOWN: (u8, u8) = (7, 0);
    pub const LEFT_SHIFT: (u8, u8) = (7, 1);
    pub const X: (u8, u8) = (7, 2);
    pub const V: (u8, u8) = (7, 3);
    pub const N: (u8, u8) = (7, 4);
    pub const COMMA: (u8, u8) = (7, 5);
    pub const SLASH: (u8, u8) = (7, 6);
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
        // Verify a few key positions match the C64 matrix (col, row) convention
        assert_eq!(keys::A, (2, 1));
        assert_eq!(keys::RETURN, (1, 0));
        assert_eq!(keys::SPACE, (4, 7));
        assert_eq!(keys::LEFT_SHIFT, (7, 1));
        assert_eq!(keys::RIGHT_SHIFT, (4, 6));
    }

    #[test]
    fn test_matrix_access() {
        let mut kb = Keyboard::new();
        kb.key_down(2, 3);

        let matrix = kb.matrix();
        assert!(matrix[2][3]);
        assert!(!matrix[0][0]);
    }

    #[test]
    fn test_pc_key_mapping_letters() {
        // Verify letter mappings match the key constants
        let a = map_pc_keycode("KeyA").unwrap();
        assert_eq!((a.row, a.col), keys::A);
        assert!(!a.requires_shift);

        let q = map_pc_keycode("KeyQ").unwrap();
        assert_eq!((q.row, q.col), keys::Q);

        let z = map_pc_keycode("KeyZ").unwrap();
        assert_eq!((z.row, z.col), keys::Z);
    }

    #[test]
    fn test_pc_key_mapping_digits() {
        let d1 = map_pc_keycode("Digit1").unwrap();
        assert_eq!((d1.row, d1.col), keys::DIGIT_1);

        let d0 = map_pc_keycode("Digit0").unwrap();
        assert_eq!((d0.row, d0.col), keys::DIGIT_0);
    }

    #[test]
    fn test_pc_key_mapping_special_keys() {
        // Enter -> RETURN
        let enter = map_pc_keycode("Enter").unwrap();
        assert_eq!((enter.row, enter.col), keys::RETURN);

        // Space
        let space = map_pc_keycode("Space").unwrap();
        assert_eq!((space.row, space.col), keys::SPACE);

        // Backspace -> DEL
        let backspace = map_pc_keycode("Backspace").unwrap();
        assert_eq!((backspace.row, backspace.col), keys::DEL);

        // Escape -> RUN/STOP
        let esc = map_pc_keycode("Escape").unwrap();
        assert_eq!((esc.row, esc.col), keys::RUN_STOP);
    }

    #[test]
    fn test_pc_key_mapping_modifiers() {
        let shift_left = map_pc_keycode("ShiftLeft").unwrap();
        assert_eq!((shift_left.row, shift_left.col), keys::LEFT_SHIFT);

        let shift_right = map_pc_keycode("ShiftRight").unwrap();
        assert_eq!((shift_right.row, shift_right.col), keys::RIGHT_SHIFT);

        let ctrl = map_pc_keycode("ControlLeft").unwrap();
        assert_eq!((ctrl.row, ctrl.col), keys::CTRL);
    }

    #[test]
    fn test_pc_key_mapping_function_keys() {
        // F1 - direct mapping
        let f1 = map_pc_keycode("F1").unwrap();
        assert_eq!((f1.row, f1.col), keys::F1);
        assert!(!f1.requires_shift);

        // F2 - requires shift (it's Shift+F1 on C64)
        let f2 = map_pc_keycode("F2").unwrap();
        assert_eq!((f2.row, f2.col), keys::F1);
        assert!(f2.requires_shift);

        // F7 - direct mapping
        let f7 = map_pc_keycode("F7").unwrap();
        assert_eq!((f7.row, f7.col), keys::F7);
        assert!(!f7.requires_shift);

        // F8 - requires shift (it's Shift+F7 on C64)
        let f8 = map_pc_keycode("F8").unwrap();
        assert_eq!((f8.row, f8.col), keys::F7);
        assert!(f8.requires_shift);
    }

    #[test]
    fn test_pc_key_mapping_navigation() {
        // Arrow Right -> CRSR RIGHT (direct)
        let right = map_pc_keycode("ArrowRight").unwrap();
        assert_eq!((right.row, right.col), keys::CRSR_RIGHT);
        assert!(!right.requires_shift);

        // Arrow Left -> CRSR RIGHT + SHIFT
        let left = map_pc_keycode("ArrowLeft").unwrap();
        assert_eq!((left.row, left.col), keys::CRSR_RIGHT);
        assert!(left.requires_shift);

        // Arrow Down -> CRSR DOWN (direct)
        let down = map_pc_keycode("ArrowDown").unwrap();
        assert_eq!((down.row, down.col), keys::CRSR_DOWN);
        assert!(!down.requires_shift);

        // Arrow Up -> CRSR DOWN + SHIFT
        let up = map_pc_keycode("ArrowUp").unwrap();
        assert_eq!((up.row, up.col), keys::CRSR_DOWN);
        assert!(up.requires_shift);

        // Home -> CLR/HOME
        let home = map_pc_keycode("Home").unwrap();
        assert_eq!((home.row, home.col), keys::HOME);
    }

    #[test]
    fn test_pc_key_mapping_unknown_key() {
        assert!(map_pc_keycode("UnknownKey").is_none());
        assert!(map_pc_keycode("").is_none());
    }

    #[test]
    fn test_key_mapping_struct() {
        let mapping = KeyMapping::new(1, 2);
        assert_eq!(mapping.row, 1);
        assert_eq!(mapping.col, 2);
        assert!(!mapping.requires_shift);

        let shifted = KeyMapping::with_shift(3, 4);
        assert_eq!(shifted.row, 3);
        assert_eq!(shifted.col, 4);
        assert!(shifted.requires_shift);
    }
}
