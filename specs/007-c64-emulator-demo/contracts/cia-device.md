# Contract: CIA Device Implementation

**Feature**: C64 Emulator Demo
**Component**: `CiaDevice` (src/devices/cia.rs)
**Trait**: `Device`, `InterruptDevice`
**Date**: 2025-11-20

This contract defines the behavior of the MOS 6526 CIA #1 chip emulation for keyboard scanning and timer interrupts.

---

## Overview

The `CiaDevice` implements both `Device` and `InterruptDevice` traits to emulate the 6526 Complex Interface Adapter. Phase 1 focuses on:
- Full keyboard matrix scanning (Port A/B)
- Functional 60Hz timer interrupt (Timer A)
- Stubbed Timer B, TOD clock, serial port

---

## Device Registration

**CIA #1 Address Range**: $DC00-$DCFF (256 bytes)
**CIA #2** (deferred): $DD00-$DDFF

**Registration**:
```rust
let cia1 = Box::new(CiaDevice::new());
mapped_memory.add_device(0xDC00, cia1)?;
```

---

## Register Map (Phase 1)

### I/O Ports (Fully Implemented)

| Offset | Address | Name | R/W | Default | Description |
|--------|---------|------|-----|---------|-------------|
| $00 | $DC00 | PRA (Port A) | R/W | $FF | Keyboard matrix columns (output) |
| $01 | $DC01 | PRB (Port B) | R | $FF | Keyboard matrix rows (input) |
| $02 | $DC02 | DDRA | R/W | $FF | Data direction Port A (1=output) |
| $03 | $DC03 | DDRB | R/W | $00 | Data direction Port B (0=input) |

### Timer Registers (Functional Implementation)

| Offset | Address | Name | R/W | Default | Description |
|--------|---------|------|-----|---------|-------------|
| $04 | $DC04 | Timer A Low | R/W | $FF | Timer A latch/counter low byte |
| $05 | $DC05 | Timer A High | R/W | $FF | Timer A latch/counter high byte |
| $06 | $DC06 | Timer B Low | R/W | $FF | Timer B latch/counter low byte [stub] |
| $07 | $DC07 | Timer B High | R/W | $FF | Timer B latch/counter high byte [stub] |

### Control Registers

| Offset | Address | Name | R/W | Default | Description |
|--------|---------|------|-----|---------|-------------|
| $0D | $DC0D | ICR (Interrupt Control) | R/W | $00 | Interrupt status/mask |
| $0E | $DC0E | CRA (Control Register A) | R/W | $00 | Timer A control |
| $0F | $DC0F | CRB (Control Register B) | R/W | $00 | Timer B control [stub] |

### Stubbed Registers (Phase 1)

| Offset Range | Address Range | Name | Behavior |
|--------------|---------------|------|----------|
| $08-$0B | $DC08-$DC0B | TOD Clock | Return $00, ignore writes |
| $0C | $DC0C | Serial Data Register | Return $00, ignore writes |

---

## Device Trait Implementation

### `read(&self, offset: u16) -> u8`

**Behavior**:
```rust
impl Device for CiaDevice {
    fn read(&self, offset: u16) -> u8 {
        match offset as u8 {
            // Port A: Return current output latch value
            0x00 => self.port_a,

            // Port B: Return keyboard matrix state for selected row
            0x01 => {
                // Determine which row is selected (active low in port_a)
                let row_select = !self.port_a;  // Invert to find which bit is 0

                // Scan all 8 rows, OR together results
                let mut result = 0xFF;  // Start with all keys released
                for row in 0..8 {
                    if row_select & (1 << row) != 0 {
                        result &= self.keyboard_matrix[row];
                    }
                }
                result
            }

            // Data direction registers
            0x02 => self.data_direction_a,
            0x03 => self.data_direction_b,

            // Timer A counter (read current value)
            0x04 => (self.timer_a_counter & 0xFF) as u8,
            0x05 => (self.timer_a_counter >> 8) as u8,

            // Timer B counter (stub)
            0x06 => (self.timer_b_counter & 0xFF) as u8,
            0x07 => (self.timer_b_counter >> 8) as u8,

            // TOD clock (stub)
            0x08..=0x0B => 0x00,

            // Serial register (stub)
            0x0C => 0x00,

            // Interrupt control register (read status, clear on read Phase 2)
            0x0D => {
                let status = self.interrupt_status;
                // Phase 1: Don't clear on read (Phase 2: self.interrupt_status = 0)
                status
            }

            // Control registers
            0x0E => self.control_register_a,
            0x0F => self.control_register_b,

            // Unimplemented registers
            _ => 0xFF,
        }
    }
}
```

---

### `write(&mut self, offset: u16, value: u8)`

**Behavior**:
```rust
impl Device for CiaDevice {
    fn write(&mut self, offset: u16, value: u8) {
        match offset as u8 {
            // Port A: Set output latch (selects keyboard row to scan)
            0x00 => self.port_a = value,

            // Port B: Input only, writes ignored
            0x01 => {}

            // Data direction registers
            0x02 => self.data_direction_a = value,
            0x03 => self.data_direction_b = value,

            // Timer A latch (write updates latch, not counter)
            0x04 => {
                self.timer_a_latch = (self.timer_a_latch & 0xFF00) | (value as u16);
            }
            0x05 => {
                self.timer_a_latch = (self.timer_a_latch & 0x00FF) | ((value as u16) << 8);
            }

            // Timer B latch (stub)
            0x06 => {
                self.timer_b_latch = (self.timer_b_latch & 0xFF00) | (value as u16);
            }
            0x07 => {
                self.timer_b_latch = (self.timer_b_latch & 0x00FF) | ((value as u16) << 8);
            }

            // TOD clock (stub)
            0x08..=0x0B => {}

            // Serial register (stub)
            0x0C => {}

            // Interrupt control register
            0x0D => {
                if value & 0x80 != 0 {
                    // Bit 7 set: Enable interrupts (OR mask with current)
                    self.interrupt_mask |= value & 0x7F;
                } else {
                    // Bit 7 clear: Disable interrupts (AND with inverted)
                    self.interrupt_mask &= !(value & 0x7F);
                }
            }

            // Control Register A
            0x0E => {
                self.control_register_a = value;

                // Bit 0: START (1=start timer, 0=stop timer)
                self.timer_a_running = value & 0x01 != 0;

                // Bit 4: LOAD (force reload from latch)
                if value & 0x10 != 0 {
                    self.timer_a_counter = self.timer_a_latch;
                }
            }

            // Control Register B (stub)
            0x0F => self.control_register_b = value,

            _ => {}
        }
    }
}
```

---

## Keyboard Matrix Implementation

### Matrix State

**Storage**:
```rust
pub struct CiaDevice {
    keyboard_matrix: [u8; 8],  // 8 rows, each byte = 8 columns (active low)
    // ...
}
```

**Encoding**:
- Each row is 8 bits (one byte)
- Bit value: 0 = key pressed, 1 = key released (active low)
- Example: `keyboard_matrix[1] = 0xFE` means key at row 1, col 0 pressed (RETURN)

### Keyboard Scanning Algorithm

**CPU writes to Port A** ($DC00):
```rust
// KERNAL writes $FE to scan row 0
cia.write(0x00, 0xFE);  // Binary 11111110 (bit 0 low = select row 0)
```

**CPU reads from Port B** ($DC01):
```rust
// Returns keyboard state for selected row(s)
let keys = cia.read(0x01);
// If SPACE pressed (row 4, col 0), reading with row 4 selected returns 0xFE
```

**Multi-Row Scanning**:
- If multiple bits in Port A are low, multiple rows scanned simultaneously
- Result is logical AND of all selected rows
- Example: Write $00 to Port A scans all rows (detects if ANY key pressed)

---

## Public Keyboard API

```rust
impl CiaDevice {
    /// Press key at matrix position
    pub fn press_key(&mut self, row: u8, col: u8) {
        if row < 8 && col < 8 {
            self.keyboard_matrix[row as usize] &= !(1 << col);  // Clear bit (active low)
        }
    }

    /// Release key at matrix position
    pub fn release_key(&mut self, row: u8, col: u8) {
        if row < 8 && col < 8 {
            self.keyboard_matrix[row as usize] |= (1 << col);   // Set bit (released)
        }
    }

    /// Check if key is pressed
    pub fn is_key_pressed(&self, row: u8, col: u8) -> bool {
        if row < 8 && col < 8 {
            self.keyboard_matrix[row as usize] & (1 << col) == 0  // Bit 0 = pressed
        } else {
            false
        }
    }

    /// Clear all keys (all released)
    pub fn release_all_keys(&mut self) {
        self.keyboard_matrix = [0xFF; 8];
    }
}
```

---

## Timer Implementation

### Phase 1: Functional 60Hz Interrupt

**Goal**: Generate interrupt every 1/60th second without cycle-accurate countdown.

**Approach**:
```rust
impl CiaDevice {
    /// Called by emulation loop with elapsed cycles
    pub fn tick(&mut self, cycles: usize) {
        if !self.timer_a_running {
            return;
        }

        // Phase 1: Simple cycle accumulator
        self.cycle_accumulator += cycles;

        // NTSC: 16667 cycles per frame ≈ 60 Hz
        if self.cycle_accumulator >= 16667 {
            self.cycle_accumulator -= 16667;

            // Generate Timer A interrupt
            self.interrupt_status |= 0x01;  // Set Timer A bit
            self.interrupt_pending = true;
        }
    }
}
```

**Alternative (even simpler)**:
```rust
// Called once per frame (already 60Hz from JavaScript)
pub fn trigger_frame_interrupt(&mut self) {
    if self.timer_a_running {
        self.interrupt_status |= 0x01;
        self.interrupt_pending = true;
    }
}
```

### Phase 2: Cycle-Accurate Countdown (Deferred)

**Goal**: Decrement timer every CPU cycle, reload on underflow.

**Approach**:
```rust
pub fn tick(&mut self, cycles: usize) {
    if !self.timer_a_running {
        return;
    }

    for _ in 0..cycles {
        if self.timer_a_counter == 0 {
            // Underflow: reload and generate interrupt
            self.timer_a_counter = self.timer_a_latch;
            self.interrupt_status |= 0x01;

            // Check if one-shot mode (bit 3 of CRA)
            if self.control_register_a & 0x08 != 0 {
                self.timer_a_running = false;  // Stop in one-shot mode
            }
        } else {
            self.timer_a_counter -= 1;
        }
    }

    self.interrupt_pending = (self.interrupt_status & self.interrupt_mask) != 0;
}
```

---

## Interrupt Control Register ($DC0D)

### Read Behavior

**Bit Layout** (status when reading):
```
Bit 7: IRQ - Any enabled interrupt occurred (logical OR of bits 0-4 & mask)
Bit 4: FLAG - FLAG line interrupt [stub]
Bit 3: SP - Serial port interrupt [stub]
Bit 2: ALRM - TOD alarm interrupt [stub]
Bit 1: TB - Timer B underflow [stub]
Bit 0: TA - Timer A underflow
```

**Read Clears Flags** (Phase 2):
- Reading $DC0D returns current status
- Automatically clears all interrupt flags after read
- Phase 1: Doesn't clear (requires manual acknowledgment)

### Write Behavior

**Bit Layout** (mask when writing):
- Same bit positions as read
- Bit 7: 1 = enable masked interrupts, 0 = disable masked interrupts
- Bits 0-4: Select which interrupt(s) to enable/disable

**Examples**:
```rust
// Enable Timer A interrupt
cia.write(0x0D, 0x81);  // Bit 7=1 (enable), bit 0=1 (Timer A)

// Disable Timer A interrupt
cia.write(0x0D, 0x01);  // Bit 7=0 (disable), bit 0=1 (Timer A)

// Enable all interrupts
cia.write(0x0D, 0x9F);  // Bit 7=1, bits 0-4=1
```

---

## InterruptDevice Trait Implementation

```rust
impl InterruptDevice for CiaDevice {
    fn has_interrupt(&self) -> bool {
        // Return true if any enabled interrupt is pending
        self.interrupt_pending && (self.interrupt_status & self.interrupt_mask) != 0
    }
}

impl Device for CiaDevice {
    fn as_interrupt_device(&self) -> Option<&dyn InterruptDevice> {
        Some(self)  // CIA supports interrupts
    }
}
```

**Integration with MappedMemory**:
```rust
// MappedMemory checks all devices for interrupts
impl MemoryBus for MappedMemory {
    fn irq_active(&self) -> bool {
        self.devices
            .iter()
            .filter_map(|m| m.device.as_interrupt_device())
            .any(|d| d.has_interrupt())
    }
}
```

---

## Control Register A ($DC0E)

**Bit Layout**:
```
Bit 7: TOD - 0=60Hz, 1=50Hz [ignored Phase 1]
Bit 6: SPMODE - Serial port mode [ignored]
Bit 5: INMODE - Timer A input mode (0=CPU clock, 1=CNT pin) [ignored]
Bit 4: LOAD - Force load from latch (strobe, auto-clears)
Bit 3: RUNMODE - 0=continuous, 1=one-shot [Phase 2]
Bit 2: OUTMODE - 0=pulse, 1=toggle PB6 [ignored]
Bit 1: PBON - 1=enable PB6 output [ignored]
Bit 0: START - 1=start timer, 0=stop timer
```

**Phase 1 Implementation**:
```rust
fn write_control_register_a(&mut self, value: u8) {
    self.control_register_a = value;

    // Bit 0: Start/stop timer
    self.timer_a_running = value & 0x01 != 0;

    // Bit 4: Force load
    if value & 0x10 != 0 {
        self.timer_a_counter = self.timer_a_latch;
    }

    // Other bits ignored in Phase 1
}
```

---

## KERNAL Initialization Sequence

**IOINIT Routine** (KERNAL $FF84):
1. Write `$00` to $DC0E (stop Timer A)
2. Write `$25` to $DC04 (latch low = $25)
3. Write `$40` to $DC05 (latch high = $40, total = $4025 = 16421)
4. Write `$81` to $DC0D (enable Timer A interrupt)
5. Write `$01` to $DC0E (start Timer A)

**Expected State After Boot**:
- Timer A latch = 16421 ($4025)
- Timer A running
- Interrupt enabled for Timer A
- Interrupt fires every ~16421 CPU cycles (≈60 Hz at 985 kHz)

---

## Public API (Beyond Device Trait)

```rust
impl CiaDevice {
    /// Create new CIA device with defaults
    pub fn new() -> Self;

    /// Press/release key API
    pub fn press_key(&mut self, row: u8, col: u8);
    pub fn release_key(&mut self, row: u8, col: u8);
    pub fn is_key_pressed(&self, row: u8, col: u8) -> bool;
    pub fn release_all_keys(&mut self);

    /// Timer tick (called by emulation loop)
    pub fn tick(&mut self, cycles: usize);

    /// Acknowledge interrupt (Phase 1: manual clear)
    pub fn acknowledge_interrupt(&mut self) {
        self.interrupt_status = 0;
        self.interrupt_pending = false;
    }

    /// Get timer latch value
    pub fn timer_a_latch(&self) -> u16;

    /// Check if timer is running
    pub fn timer_a_running(&self) -> bool;
}
```

---

## Phase 1 vs Phase 2 Differences

| Feature | Phase 1 | Phase 2 |
|---------|---------|---------|
| Keyboard matrix | ✅ Full | ✅ Full |
| Port A/B I/O | ✅ Full | ✅ Full |
| Timer A interrupt | ✅ 60Hz functional | ✅ Cycle-accurate countdown |
| Timer A latch/reload | ✅ Basic | ✅ Full with LOAD strobe |
| Interrupt mask | ✅ Storage only | ✅ Actual masking |
| ICR read clears flags | ❌ Manual clear | ✅ Auto-clear on read |
| Timer B | ❌ Stub | ✅ Full countdown + cascade |
| TOD clock | ❌ Stub | Future |
| Serial port | ❌ Stub | Future |
| One-shot mode | ❌ Ignored | ✅ Timer stops on underflow |

---

## Contract Validation

**CIA Device Must**:
- Initialize keyboard matrix to all released (0xFF per row)
- Return active-low values from Port B (0=pressed)
- Support multiple simultaneous key presses
- Generate Timer A interrupt at ~60 Hz
- Implement `InterruptDevice::has_interrupt()` correctly
- Accept writes to all registers without panicking

**CPU/KERNAL May Assume**:
- Port B reads reflect current keyboard state immediately
- Timer A fires periodically when enabled
- Interrupt flag persists until acknowledged (Phase 1) or ICR read (Phase 2)
- Port A output latch remembers written value

---

**Contract Status**: ✅ Complete
**All contracts generated**: rust-to-javascript, vic2-device, cia-device
