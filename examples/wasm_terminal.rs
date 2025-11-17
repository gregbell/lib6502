//! WASM Terminal Integration Example
//!
//! This example demonstrates how to integrate the 6502 emulator with UART device
//! for browser-based serial terminal communication using WASM.
//!
//! ## Overview
//!
//! This example provides patterns for:
//! - Setting up WASM-bindgen callbacks for transmit
//! - Handling browser terminal input via receive_byte()
//! - Integrating with terminal libraries like xterm.js
//! - Managing bidirectional character flow in the browser
//!
//! ## WASM Integration Pattern
//!
//! ```rust,ignore
//! use wasm_bindgen::prelude::*;
//! use lib6502::{CPU, MappedMemory, RamDevice, RomDevice, Uart6551};
//! use std::cell::RefCell;
//! use std::rc::Rc;
//!
//! #[wasm_bindgen]
//! pub struct Emulator {
//!     cpu: CPU<MappedMemory>,
//!     // Store callback reference to prevent it from being dropped
//!     _transmit_callback: Rc<RefCell<Box<dyn Fn(u8)>>>,
//! }
//!
//! #[wasm_bindgen]
//! impl Emulator {
//!     #[wasm_bindgen(constructor)]
//!     pub fn new(on_transmit: js_sys::Function) -> Result<Emulator, JsValue> {
//!         let mut memory = MappedMemory::new();
//!
//!         // Add 32KB RAM
//!         memory.add_device(0x0000, Box::new(RamDevice::new(32768)))
//!             .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
//!
//!         // Add UART at 0x8000
//!         let mut uart = Uart6551::new();
//!
//!         // Create transmit callback that calls JavaScript function
//!         let callback: Rc<RefCell<Box<dyn Fn(u8)>>> = Rc::new(RefCell::new(Box::new(move |byte: u8| {
//!             // Convert byte to JS string and call JavaScript callback
//!             let char_str = String::from_utf8(vec![byte]).unwrap_or_else(|_| "?".to_string());
//!             let _ = on_transmit.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
//!         })));
//!
//!         // Clone for UART
//!         let callback_clone = Rc::clone(&callback);
//!         uart.set_transmit_callback(move |byte| {
//!             (callback_clone.borrow())(byte);
//!         });
//!
//!         memory.add_device(0x8000, Box::new(uart))
//!             .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
//!
//!         // Add 16KB ROM at 0xC000 with reset vector
//!         let mut rom = vec![0xEA; 16384]; // Fill with NOP
//!         rom[0x3FFC] = 0x00; // Reset vector low byte -> 0x0200
//!         rom[0x3FFD] = 0x02; // Reset vector high byte
//!         memory.add_device(0xC000, Box::new(RomDevice::new(rom)))
//!             .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
//!
//!         let cpu = CPU::new(memory);
//!
//!         Ok(Emulator {
//!             cpu,
//!             _transmit_callback: callback,
//!         })
//!     }
//!
//!     /// Step the CPU one instruction
//!     #[wasm_bindgen]
//!     pub fn step(&mut self) -> Result<(), JsValue> {
//!         self.cpu.step()
//!             .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
//!     }
//!
//!     /// Run for a certain number of cycles
//!     #[wasm_bindgen]
//!     pub fn run_cycles(&mut self, cycles: u32) -> Result<(), JsValue> {
//!         self.cpu.run_for_cycles(cycles as usize)
//!             .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
//!     }
//!
//!     /// Handle character input from terminal (e.g., xterm.js onData event)
//!     #[wasm_bindgen]
//!     pub fn receive_char(&mut self, byte: u8) -> Result<(), JsValue> {
//!         // Get mutable access to UART device at 0x8000
//!         // Note: This requires unsafe or a different architecture
//!         // In practice, you'd need to expose receive_byte through the memory interface
//!         // or store the UART separately
//!
//!         // For now, this is a pattern demonstration
//!         // Real implementation would need architecture adjustments
//!         Ok(())
//!     }
//!
//!     /// Load program into RAM
//!     #[wasm_bindgen]
//!     pub fn load_program(&mut self, address: u16, bytes: &[u8]) {
//!         for (i, &byte) in bytes.iter().enumerate() {
//!             let addr = address.wrapping_add(i as u16);
//!             self.cpu.memory_mut().write(addr, byte);
//!         }
//!     }
//!
//!     /// Get current PC
//!     #[wasm_bindgen]
//!     pub fn pc(&self) -> u16 {
//!         self.cpu.pc()
//!     }
//! }
//! ```
//!
//! ## JavaScript Integration (xterm.js)
//!
//! ```javascript
//! import { Terminal } from 'xterm';
//! import { FitAddon } from 'xterm-addon-fit';
//! import init, { Emulator } from './pkg/lib6502.js';
//!
//! // Initialize WASM module
//! await init();
//!
//! // Create terminal
//! const term = new Terminal({
//!     cursorBlink: true,
//!     fontSize: 14,
//!     fontFamily: 'Courier New, monospace',
//! });
//!
//! const fitAddon = new FitAddon();
//! term.loadAddon(fitAddon);
//! term.open(document.getElementById('terminal'));
//! fitAddon.fit();
//!
//! // Create emulator with transmit callback
//! const emulator = new Emulator((char) => {
//!     // Write to terminal when UART transmits
//!     term.write(char);
//! });
//!
//! // Handle terminal input
//! term.onData((data) => {
//!     // Send each character to UART
//!     for (let i = 0; i < data.length; i++) {
//!         const byte = data.charCodeAt(i);
//!         emulator.receive_char(byte);
//!     }
//! });
//!
//! // Load a simple echo program
//! // LDA $8000   ; Read from UART
//! // STA $8000   ; Write back to UART
//! // JMP $0200   ; Loop
//! const program = new Uint8Array([
//!     0xAD, 0x00, 0x80,  // LDA $8000
//!     0x8D, 0x00, 0x80,  // STA $8000
//!     0x4C, 0x00, 0x02,  // JMP $0200
//! ]);
//! emulator.load_program(0x0200, program);
//!
//! // Run emulator loop
//! function runEmulator() {
//!     try {
//!         // Run for ~1000 cycles per frame (adjust for performance)
//!         emulator.run_cycles(1000);
//!     } catch (e) {
//!         console.error('Emulator error:', e);
//!     }
//!     requestAnimationFrame(runEmulator);
//! }
//!
//! // Start emulation
//! runEmulator();
//! ```
//!
//! ## HTML Setup
//!
//! ```html
//! <!DOCTYPE html>
//! <html>
//! <head>
//!     <meta charset="utf-8">
//!     <title>6502 UART Terminal</title>
//!     <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5/css/xterm.css" />
//!     <style>
//!         #terminal {
//!             width: 800px;
//!             height: 600px;
//!             margin: 20px auto;
//!             border: 1px solid #333;
//!             background: #000;
//!         }
//!     </style>
//! </head>
//! <body>
//!     <div id="terminal"></div>
//!     <script type="module" src="./app.js"></script>
//! </body>
//! </html>
//! ```
//!
//! ## Browser Compatibility
//!
//! ### Supported Browsers
//!
//! - **Chrome/Edge**: Full support (Chromium 85+)
//!   - WebAssembly, ES6 modules, BigInt support
//!   - Best performance with V8 engine optimizations
//!
//! - **Firefox**: Full support (Firefox 78+)
//!   - WebAssembly, ES6 modules support
//!   - Good performance with SpiderMonkey
//!
//! - **Safari**: Full support (Safari 14+)
//!   - WebAssembly, ES6 modules support
//!   - May require polyfills for older versions
//!
//! ### Requirements
//!
//! - WebAssembly support
//! - ES6 module support
//! - HTTPS or localhost (for WASM security model)
//!
//! ### Known Issues
//!
//! - **Mobile browsers**: Virtual keyboard handling may require special care
//! - **Older browsers**: May need polyfills for BigInt and other ES6+ features
//! - **CORS**: Must serve WASM files with proper MIME type (application/wasm)
//!
//! ## UART Receive Pattern
//!
//! The UART device's `receive_byte()` method needs to be called from JavaScript
//! when terminal input occurs. There are two architectural approaches:
//!
//! ### Approach 1: Store UART Separately (Recommended)
//!
//! ```rust,ignore
//! #[wasm_bindgen]
//! pub struct Emulator {
//!     cpu: CPU<MappedMemory>,
//!     uart: Rc<RefCell<Uart6551>>,  // Separate reference
//! }
//!
//! #[wasm_bindgen]
//! impl Emulator {
//!     #[wasm_bindgen]
//!     pub fn receive_char(&mut self, byte: u8) {
//!         self.uart.borrow_mut().receive_byte(byte);
//!     }
//! }
//! ```
//!
//! ### Approach 2: Expose Through Memory Interface
//!
//! Add a method to MappedMemory to access devices by address:
//!
//! ```rust,ignore
//! impl MappedMemory {
//!     pub fn get_device_mut(&mut self, addr: u16) -> Option<&mut Box<dyn Device>> {
//!         // Return mutable reference to device at address
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Cycle Budget**: Run 1000-2000 cycles per frame for 60 FPS (~1 MHz)
//! - **Buffering**: UART has 256-byte receive buffer to handle burst input
//! - **Transmit Callbacks**: Keep JavaScript callbacks lightweight
//! - **Status Polling**: 6502 programs should check RDRF flag before reading
//!
//! ## Testing in Browser
//!
//! 1. Build WASM: `wasm-pack build --target web`
//! 2. Serve locally: `python3 -m http.server 8000`
//! 3. Open browser: `http://localhost:8000`
//! 4. Type characters to test echo
//! 5. Verify status flags update correctly
//!
//! ## Example Programs
//!
//! ### Echo Program
//! ```asm
//! loop:
//!     LDA $8001   ; Read status register
//!     AND #$08    ; Check RDRF (receive data ready)
//!     BEQ loop    ; If no data, keep polling
//!     LDA $8000   ; Read data register
//!     STA $8000   ; Echo back
//!     JMP loop
//! ```
//!
//! ### Echo with Command Mode
//! ```asm
//!     LDA #$08    ; Echo mode bit
//!     STA $8002   ; Set command register
//! loop:
//!     JMP loop    ; Echo happens automatically
//! ```

// This file is an example/documentation only - no runnable code
// To use these patterns, integrate with your WASM build setup

fn main() {
    println!("This example demonstrates WASM integration patterns.");
    println!("See the doc comments above for complete implementation details.");
    println!("\nKey integration points:");
    println!("  1. WASM-bindgen callback setup for transmit");
    println!("  2. Terminal receive_byte() invocation from JavaScript");
    println!("  3. xterm.js onData handler integration");
    println!("  4. Browser compatibility notes");
    println!("\nFor a working WASM build, use wasm-pack:");
    println!("  wasm-pack build --target web");
}
