//! C64 joystick emulation.
//!
//! The C64 has two joystick ports, each supporting 4 directions and a fire button.
//! The ports are active-low, meaning a pressed button reads as 0.
//!
//! Port 1: Connected to CIA1 port B (shared with keyboard column 0)
//! Port 2: Connected to CIA1 port A (shared with keyboard row 0)
//!
//! Most C64 games use joystick port 2 because port 1 interferes with
//! keyboard scanning.

/// Joystick direction and button constants (active-high for input).
pub mod bits {
    /// Up direction bit.
    pub const JOY_UP: u8 = 0x01;
    /// Down direction bit.
    pub const JOY_DOWN: u8 = 0x02;
    /// Left direction bit.
    pub const JOY_LEFT: u8 = 0x04;
    /// Right direction bit.
    pub const JOY_RIGHT: u8 = 0x08;
    /// Fire button bit.
    pub const JOY_FIRE: u8 = 0x10;
}

pub use bits::*;

/// State of a single joystick port.
///
/// Bits are active-high (1 = pressed) for the public API,
/// but internally converted to active-low for CIA register compatibility.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JoystickState {
    /// Current button/direction state (active-high).
    state: u8,
}

impl JoystickState {
    /// Create a new joystick state with no buttons pressed.
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    /// Set the complete joystick state.
    ///
    /// # Arguments
    /// * `state` - Bitmask of directions/fire (active-high)
    ///   - Bit 0: Up
    ///   - Bit 1: Down
    ///   - Bit 2: Left
    ///   - Bit 3: Right
    ///   - Bit 4: Fire
    #[inline]
    pub fn set(&mut self, state: u8) {
        self.state = state & 0x1F;
    }

    /// Get the current state (active-high).
    #[inline]
    pub fn get(&self) -> u8 {
        self.state
    }

    /// Get the state for CIA registers (active-low).
    ///
    /// In the C64 hardware, pressed buttons read as 0 (active-low).
    #[inline]
    pub fn get_active_low(&self) -> u8 {
        !self.state & 0x1F
    }

    /// Check if up is pressed.
    #[inline]
    pub fn up(&self) -> bool {
        self.state & JOY_UP != 0
    }

    /// Check if down is pressed.
    #[inline]
    pub fn down(&self) -> bool {
        self.state & JOY_DOWN != 0
    }

    /// Check if left is pressed.
    #[inline]
    pub fn left(&self) -> bool {
        self.state & JOY_LEFT != 0
    }

    /// Check if right is pressed.
    #[inline]
    pub fn right(&self) -> bool {
        self.state & JOY_RIGHT != 0
    }

    /// Check if fire is pressed.
    #[inline]
    pub fn fire(&self) -> bool {
        self.state & JOY_FIRE != 0
    }

    /// Press up.
    #[inline]
    pub fn press_up(&mut self) {
        self.state |= JOY_UP;
    }

    /// Release up.
    #[inline]
    pub fn release_up(&mut self) {
        self.state &= !JOY_UP;
    }

    /// Press down.
    #[inline]
    pub fn press_down(&mut self) {
        self.state |= JOY_DOWN;
    }

    /// Release down.
    #[inline]
    pub fn release_down(&mut self) {
        self.state &= !JOY_DOWN;
    }

    /// Press left.
    #[inline]
    pub fn press_left(&mut self) {
        self.state |= JOY_LEFT;
    }

    /// Release left.
    #[inline]
    pub fn release_left(&mut self) {
        self.state &= !JOY_LEFT;
    }

    /// Press right.
    #[inline]
    pub fn press_right(&mut self) {
        self.state |= JOY_RIGHT;
    }

    /// Release right.
    #[inline]
    pub fn release_right(&mut self) {
        self.state &= !JOY_RIGHT;
    }

    /// Press fire.
    #[inline]
    pub fn press_fire(&mut self) {
        self.state |= JOY_FIRE;
    }

    /// Release fire.
    #[inline]
    pub fn release_fire(&mut self) {
        self.state &= !JOY_FIRE;
    }

    /// Release all buttons.
    #[inline]
    pub fn release_all(&mut self) {
        self.state = 0;
    }
}

/// Joystick ports manager.
///
/// Handles both joystick ports and provides port swapping functionality.
#[derive(Debug, Clone)]
pub struct JoystickPorts {
    /// Port 1 state (CIA1 port B).
    port1: JoystickState,
    /// Port 2 state (CIA1 port A).
    port2: JoystickState,
    /// Swap ports (maps logical port 2 input to physical port 1 and vice versa).
    swapped: bool,
}

impl JoystickPorts {
    /// Create new joystick ports manager.
    pub fn new() -> Self {
        Self {
            port1: JoystickState::new(),
            port2: JoystickState::new(),
            swapped: false,
        }
    }

    /// Set joystick state for a logical port (1 or 2).
    ///
    /// # Arguments
    /// * `port` - Logical port number (1 or 2)
    /// * `state` - Bitmask of directions/fire (active-high)
    ///
    /// If ports are swapped, port 2 input goes to physical port 1 and vice versa.
    pub fn set_port(&mut self, port: u8, state: u8) {
        match port {
            1 => {
                if self.swapped {
                    self.port2.set(state);
                } else {
                    self.port1.set(state);
                }
            }
            2 => {
                if self.swapped {
                    self.port1.set(state);
                } else {
                    self.port2.set(state);
                }
            }
            _ => {} // Ignore invalid port numbers
        }
    }

    /// Get physical port 1 state (for CIA1 port B).
    #[inline]
    pub fn physical_port1(&self) -> &JoystickState {
        &self.port1
    }

    /// Get physical port 2 state (for CIA1 port A).
    #[inline]
    pub fn physical_port2(&self) -> &JoystickState {
        &self.port2
    }

    /// Get mutable physical port 1 state.
    #[inline]
    pub fn physical_port1_mut(&mut self) -> &mut JoystickState {
        &mut self.port1
    }

    /// Get mutable physical port 2 state.
    #[inline]
    pub fn physical_port2_mut(&mut self) -> &mut JoystickState {
        &mut self.port2
    }

    /// Check if ports are swapped.
    #[inline]
    pub fn is_swapped(&self) -> bool {
        self.swapped
    }

    /// Set port swap state.
    #[inline]
    pub fn set_swapped(&mut self, swapped: bool) {
        self.swapped = swapped;
    }

    /// Toggle port swap.
    #[inline]
    pub fn toggle_swap(&mut self) {
        self.swapped = !self.swapped;
    }

    /// Release all buttons on both ports.
    pub fn release_all(&mut self) {
        self.port1.release_all();
        self.port2.release_all();
    }
}

impl Default for JoystickPorts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joystick_state_new() {
        let joy = JoystickState::new();
        assert_eq!(joy.get(), 0);
        assert!(!joy.up());
        assert!(!joy.down());
        assert!(!joy.left());
        assert!(!joy.right());
        assert!(!joy.fire());
    }

    #[test]
    fn test_joystick_state_set() {
        let mut joy = JoystickState::new();
        joy.set(JOY_UP | JOY_FIRE);
        assert!(joy.up());
        assert!(joy.fire());
        assert!(!joy.down());
        assert!(!joy.left());
        assert!(!joy.right());
    }

    #[test]
    fn test_joystick_state_active_low() {
        let mut joy = JoystickState::new();
        // No buttons pressed = all 1s (active-low)
        assert_eq!(joy.get_active_low(), 0x1F);

        // All buttons pressed = all 0s (active-low)
        joy.set(0x1F);
        assert_eq!(joy.get_active_low(), 0x00);

        // Up pressed = bit 0 is 0
        joy.set(JOY_UP);
        assert_eq!(joy.get_active_low(), 0x1E);
    }

    #[test]
    fn test_joystick_state_individual_buttons() {
        let mut joy = JoystickState::new();

        joy.press_up();
        assert!(joy.up());
        joy.release_up();
        assert!(!joy.up());

        joy.press_down();
        assert!(joy.down());

        joy.press_left();
        assert!(joy.left());

        joy.press_right();
        assert!(joy.right());

        joy.press_fire();
        assert!(joy.fire());

        // Multiple buttons can be pressed
        assert_eq!(joy.get(), JOY_DOWN | JOY_LEFT | JOY_RIGHT | JOY_FIRE);

        joy.release_all();
        assert_eq!(joy.get(), 0);
    }

    #[test]
    fn test_joystick_ports_basic() {
        let mut ports = JoystickPorts::new();

        ports.set_port(1, JOY_UP);
        ports.set_port(2, JOY_FIRE);

        assert!(ports.physical_port1().up());
        assert!(ports.physical_port2().fire());
    }

    #[test]
    fn test_joystick_ports_swapped() {
        let mut ports = JoystickPorts::new();
        ports.set_swapped(true);

        // With swap, port 2 input goes to physical port 1
        ports.set_port(2, JOY_UP);
        ports.set_port(1, JOY_FIRE);

        assert!(ports.physical_port1().up()); // Port 2 input went here
        assert!(ports.physical_port2().fire()); // Port 1 input went here
    }

    #[test]
    fn test_joystick_ports_toggle_swap() {
        let mut ports = JoystickPorts::new();
        assert!(!ports.is_swapped());

        ports.toggle_swap();
        assert!(ports.is_swapped());

        ports.toggle_swap();
        assert!(!ports.is_swapped());
    }

    #[test]
    fn test_joystick_ports_release_all() {
        let mut ports = JoystickPorts::new();
        ports.set_port(1, JOY_UP | JOY_FIRE);
        ports.set_port(2, JOY_DOWN | JOY_LEFT);

        ports.release_all();

        assert_eq!(ports.physical_port1().get(), 0);
        assert_eq!(ports.physical_port2().get(), 0);
    }
}
