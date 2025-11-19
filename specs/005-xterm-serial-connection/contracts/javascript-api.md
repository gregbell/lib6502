# JavaScript API Contract: Terminal Component

**Feature**: 005-xterm-serial-connection
**Date**: 2025-11-18
**Purpose**: Define the public API for the Terminal component and event interfaces

## Overview

This contract defines the JavaScript APIs for:
1. Terminal component (wrapper for xterm.js)
2. CustomEvent interfaces for component communication
3. Integration with existing App class

## Terminal Component API

### Class: `Terminal`

**File**: `demo/components/terminal.js`

**Purpose**: Encapsulate xterm.js functionality and provide clean interface for app integration

---

### Constructor

```typescript
/**
 * Create and initialize terminal component
 * @param containerId - DOM element ID to mount terminal
 * @throws Error if container element not found
 */
constructor(containerId: string)
```

**Behavior**:
1. Create xterm.js Terminal instance with default config
2. Create and load FitAddon
3. Open terminal in specified container
4. Fit terminal to container size
5. Set up event listeners (onData, resize)
6. Display welcome message

**Example**:
```javascript
const terminal = new Terminal('terminal-container');
```

**Default Configuration**:
```javascript
{
  cols: 80,
  rows: 24,
  fontSize: 14,
  fontFamily: 'JetBrains Mono, Courier New, monospace',
  cursorBlink: true,
  cursorStyle: 'block',
  theme: {
    background: '#1a1a1a',
    foreground: '#ffffff',
    cursor: '#ffffff'
  }
}
```

---

### Method: `write`

```typescript
/**
 * Write text to terminal display
 * @param text - Text to display (supports ANSI escape codes)
 */
write(text: string): void
```

**Behavior**:
- Appends text to terminal display buffer
- Handles ANSI escape codes (colors, cursor movement)
- Auto-scrolls if cursor at bottom
- Non-blocking operation

**Example**:
```javascript
terminal.write('Hello, World!\r\n');
terminal.write('\x1b[32mGreen text\x1b[0m\r\n');
```

**ANSI Codes Supported**:
- `\r` - Carriage return
- `\n` - Newline
- `\x1b[32m` - Green text
- `\x1b[31m` - Red text
- `\x1b[0m` - Reset formatting
- Full xterm.js ANSI support

---

### Method: `clear`

```typescript
/**
 * Clear terminal display and reset cursor to top-left
 */
clear(): void
```

**Behavior**:
- Clears all lines from display buffer
- Resets cursor to (0, 0)
- Preserves configuration and event handlers

**Example**:
```javascript
terminal.clear();
```

---

### Method: `fit`

```typescript
/**
 * Resize terminal to fit container dimensions
 */
fit(): void
```

**Behavior**:
- Queries container element size
- Calculates optimal rows/cols for current font
- Resizes terminal display
- Triggered automatically on window resize

**Example**:
```javascript
// Manual fit (usually not needed)
terminal.fit();
```

---

### Internal Event: `onData`

**Not exposed in public API - handled internally**

**Behavior**:
```javascript
this.term.onData((data) => {
  // Dispatch CustomEvent for app consumption
  document.dispatchEvent(new CustomEvent('terminal-data', {
    detail: { data }
  }));
});
```

---

## CustomEvent Interfaces

### Event: `terminal-data`

**Dispatched By**: Terminal component
**Consumed By**: App class
**Direction**: Terminal → App → Emulator

**Interface**:
```typescript
interface TerminalDataEvent extends CustomEvent {
  detail: {
    data: string;  // Characters typed or pasted
  }
}
```

**Dispatch**:
```javascript
document.dispatchEvent(new CustomEvent('terminal-data', {
  detail: { data: userInput }
}));
```

**Consumption**:
```javascript
document.addEventListener('terminal-data', (e) => {
  const data = e.detail.data;
  for (let i = 0; i < data.length; i++) {
    const byte = data.charCodeAt(i);
    emulator.receive_char(byte);
  }
});
```

**Character Handling**:

| Input | data Value | Action |
|-------|------------|--------|
| Single char 'A' | `"A"` | Send 0x41 to UART |
| Enter key | `"\r"` | Send 0x0D to UART |
| Backspace | `"\x7f"` | Send 0x7F to UART |
| Paste "Hello" | `"Hello"` | Send 5 bytes to UART |
| Ctrl+C | `"\x03"` | Send 0x03 to UART (or handle specially) |

---

### Event: `terminal-clear` (Optional)

**Dispatched By**: Control panel or UI button
**Consumed By**: Terminal component
**Direction**: UI → Terminal

**Interface**:
```typescript
interface TerminalClearEvent extends CustomEvent {
  // No detail needed
}
```

**Dispatch**:
```javascript
// From clear button
document.getElementById('clear-terminal-btn').addEventListener('click', () => {
  document.dispatchEvent(new CustomEvent('terminal-clear'));
});
```

**Consumption**:
```javascript
document.addEventListener('terminal-clear', () => {
  terminal.clear();
});
```

---

## App Class Integration

### Modified Constructor

```typescript
class App {
  constructor() {
    this.emulator = null;
    this.editor = null;
    this.registerDisplay = null;
    this.flagsDisplay = null;
    this.memoryViewer = null;
    this.controlPanel = null;
    this.errorDisplay = null;
    this.exampleSelector = null;
    this.terminal = null;  // NEW

    this.mode = 'idle';
    this.assembled = false;
    this.programStart = 0x0600;
    this.programEnd = 0x0600;
    this.speed = 1000000;
    this.animationFrameId = null;
  }
}
```

---

### Modified `init()` Method

```typescript
async init() {
  try {
    await init();  // Initialize WASM

    // Create emulator with transmit callback
    this.emulator = new Emulator6502((char) => {
      this.terminal.write(char);
    });

    // Initialize UI components
    this.editor = new CodeEditor('editor-container');
    this.registerDisplay = new RegisterDisplay('registers-container');
    this.flagsDisplay = new FlagsDisplay('flags-container');
    this.memoryViewer = new MemoryViewer('memory-container');
    this.controlPanel = new ControlPanel('assemble-button-container', 'execution-controls-container');
    this.errorDisplay = new ErrorDisplay('error-container');
    this.exampleSelector = new ExampleSelector(this.editor);
    this.terminal = new Terminal('terminal-container');  // NEW

    this.setupEventListeners();
    this.updateDisplay();
    this.startAnimationLoop();

    console.log('✓ lib6502 demo initialized successfully');
  } catch (error) {
    console.error('Failed to initialize demo:', error);
    this.showError('Failed to load WebAssembly module. Please refresh the page.');
  }
}
```

---

### New Event Listener: `terminal-data`

```typescript
setupEventListeners() {
  // Existing listeners...
  document.addEventListener('assemble-clicked', () => this.handleAssemble());
  document.addEventListener('run-clicked', () => this.handleRun());
  // ... etc

  // NEW: Terminal input handling
  document.addEventListener('terminal-data', (e) => {
    this.handleTerminalInput(e.detail.data);
  });

  // NEW: Terminal clear handling
  document.addEventListener('reset-clicked', () => {
    // Existing reset logic...
    this.handleReset();

    // Also clear terminal
    this.terminal.clear();
  });
}
```

---

### New Method: `handleTerminalInput`

```typescript
/**
 * Handle characters typed in terminal
 * @param data - String of characters from terminal
 */
handleTerminalInput(data) {
  if (!data) return;

  for (let i = 0; i < data.length; i++) {
    const byte = data.charCodeAt(i);
    this.emulator.receive_char(byte);
  }
}
```

---

## Component Lifecycle

### Initialization Sequence

```
1. HTML loads → DOM ready
2. app.js executes → App.init()
3. WASM module initializes → init()
4. Emulator created → new Emulator6502(callback)
5. Terminal created → new Terminal('terminal-container')
6. Event listeners attached
7. Ready for user interaction
```

### Runtime Flow

```
┌─────────────────┐
│  User types in  │
│    Terminal     │
└────────┬────────┘
         │ onData
         ▼
┌─────────────────┐
│    Terminal     │
│   component     │
└────────┬────────┘
         │ dispatch('terminal-data')
         ▼
┌─────────────────┐
│   App class     │
│ event listener  │
└────────┬────────┘
         │ handleTerminalInput()
         ▼
┌─────────────────┐
│   Emulator      │
│ receive_char()  │
└────────┬────────┘
         │ 6502 executes
         ▼
┌─────────────────┐
│   UART device   │
│  STA $A000      │
└────────┬────────┘
         │ transmit callback
         ▼
┌─────────────────┐
│   Emulator      │
│    callback     │
└────────┬────────┘
         │ terminal.write()
         ▼
┌─────────────────┐
│    Terminal     │
│    display      │
└─────────────────┘
```

---

## Type Definitions (TypeScript)

```typescript
/**
 * Terminal component class
 */
declare class Terminal {
  constructor(containerId: string);
  write(text: string): void;
  clear(): void;
  fit(): void;
}

/**
 * CustomEvent interfaces
 */
interface TerminalDataEvent extends CustomEvent {
  detail: {
    data: string;
  };
}

interface TerminalClearEvent extends CustomEvent {
  // No detail
}

/**
 * Modified App class (partial)
 */
declare class App {
  terminal: Terminal;
  emulator: Emulator6502;

  handleTerminalInput(data: string): void;
}
```

---

## Error Handling

### Terminal Creation Errors

```javascript
constructor(containerId) {
  const container = document.getElementById(containerId);
  if (!container) {
    throw new Error(`Terminal container not found: ${containerId}`);
  }
  // ... continue initialization
}
```

### xterm.js Load Errors

```javascript
try {
  const term = new Terminal();
} catch (error) {
  console.error('Failed to initialize xterm.js:', error);
  // Fallback: Display error in container
  container.innerHTML = '<p style="color:red;">Terminal failed to load</p>';
}
```

### Event Handler Errors

```javascript
document.addEventListener('terminal-data', (e) => {
  try {
    this.handleTerminalInput(e.detail.data);
  } catch (error) {
    console.error('Terminal input error:', error);
    this.showError('Terminal input failed');
  }
});
```

---

## Performance Considerations

### Buffered Writes

```javascript
// Batch writes for efficiency
write(text) {
  // xterm.js handles internal buffering
  this.term.write(text);
}
```

### Debounced Resize

```javascript
constructor(containerId) {
  // ... initialization

  let resizeTimeout;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      this.fitAddon.fit();
    }, 100); // Debounce 100ms
  });
}
```

---

## Testing Contracts

### Unit Tests (JavaScript)

Required test cases for Terminal component:

1. **Constructor**: Verify terminal mounts to container
2. **write()**: Test text display and ANSI codes
3. **clear()**: Verify display cleared
4. **fit()**: Test resize behavior
5. **onData**: Verify CustomEvent dispatch

### Integration Tests

Required test cases for full flow:

1. **Terminal → UART**: Type char → verify UART receive
2. **UART → Terminal**: Write $A000 → verify terminal display
3. **Round-trip**: Type → CPU echo → display
4. **Rapid input**: Stress test event handling

---

## Example Usage

### Complete Terminal Setup

```javascript
import { Terminal } from './components/terminal.js';

// Create terminal
const terminal = new Terminal('terminal-container');

// Write welcome message
terminal.write('6502 Terminal Ready\r\n\r\n');

// Handle input
document.addEventListener('terminal-data', (e) => {
  const data = e.detail.data;

  // Echo locally (remove if CPU should echo)
  // terminal.write(data);

  // Send to emulator
  for (let i = 0; i < data.length; i++) {
    const byte = data.charCodeAt(i);
    emulator.receive_char(byte);
  }
});

// Handle clear
document.addEventListener('terminal-clear', () => {
  terminal.clear();
  terminal.write('Terminal cleared\r\n');
});
```

---

## Summary

The JavaScript API contract defines:
- **Terminal component**: Clean wrapper for xterm.js with `write()`, `clear()`, `fit()`
- **CustomEvent interface**: `terminal-data` for user input → emulator
- **App integration**: Modified constructor, init(), and event handlers
- **Error handling**: Constructor validation, event handler try/catch
- **Performance**: Debounced resize, efficient write buffering

This contract enables seamless integration of the terminal component into the existing demo architecture while maintaining clean separation of concerns.
