# UI Component Contracts

**Purpose**: Define the interface and behavior of vanilla JavaScript UI components

## Component Architecture

All components follow a consistent pattern:
- Constructor accepts a DOM selector
- `update(data)` method for state updates
- No framework dependencies (vanilla JS)
- Event-driven communication via custom events

---

## CodeEditor

**Purpose**: Assembly code editor with syntax highlighting

### Constructor

```typescript
class CodeEditor {
    constructor(selector: string);
}
```

**Parameters**:
- `selector`: CSS selector for container element (e.g., '#editor')

**DOM Requirements**:
```html
<div id="editor">
    <textarea id="asm-code" spellcheck="false"></textarea>
    <pre id="asm-highlighted"><code></code></pre>
</div>
```

### Methods

#### getValue()

```typescript
getValue(): string
```

**Returns**: Current assembly source code

#### setValue(code)

```typescript
setValue(code: string): void
```

**Sets**: Editor content (replaces existing code)

#### highlightLine(lineNum)

```typescript
highlightLine(lineNum: number): void
```

**Highlights**: Specific line (for error display)

### Events

#### 'code-changed'

**Dispatched**: When user modifies code
**Detail**: `{ code: string }`

---

## RegisterDisplay

**Purpose**: Display CPU register values

### Constructor

```typescript
class RegisterDisplay {
    constructor(selector: string);
}
```

### Methods

#### update(state)

```typescript
update(state: {
    a: number,
    x: number,
    y: number,
    pc: number,
    sp: number,
    cycles: bigint
}): void
```

**Updates**: All register displays with current values

**DOM Structure**:
```html
<div id="registers">
    <div class="register">
        <label>A</label>
        <code class="value">00</code>
    </div>
    <!-- X, Y, PC, SP, Cycles -->
</div>
```

**Display Format**:
- 8-bit registers (A, X, Y, SP): 2-digit hex (e.g., "42")
- 16-bit register (PC): 4-digit hex (e.g., "0600")
- Cycles: Decimal with thousands separators (e.g., "1,234")

---

## FlagsDisplay

**Purpose**: Display processor status flags

### Constructor

```typescript
class FlagsDisplay {
    constructor(selector: string);
}
```

### Methods

#### update(flags)

```typescript
update(flags: {
    flag_n: boolean,
    flag_v: boolean,
    flag_d: boolean,
    flag_i: boolean,
    flag_z: boolean,
    flag_c: boolean
}): void
```

**Updates**: Flag indicators (visual on/off state)

**DOM Structure**:
```html
<div id="flags">
    <div class="flag" data-flag="n">
        <label>N</label>
        <div class="indicator"></div>
    </div>
    <!-- V, D, I, Z, C -->
</div>
```

**Visual States**:
- `true`: Indicator lit (CSS class `active`)
- `false`: Indicator dim (no `active` class)

---

## MemoryViewer

**Purpose**: Virtual-scrolled hex dump of 64KB memory

### Constructor

```typescript
class MemoryViewer {
    constructor(selector: string, bytesPerRow?: number);
}
```

**Parameters**:
- `selector`: CSS selector for scrollable container
- `bytesPerRow`: Bytes per row (default: 16)

### Methods

#### updateMemory(memory)

```typescript
updateMemory(memory: Uint8Array): void
```

**Updates**: Memory display, tracking changed bytes

**Parameters**:
- `memory`: Full 64KB array or partial update

#### jumpToAddress(addr)

```typescript
jumpToAddress(addr: number): void
```

**Scrolls**: Viewer to show specified address

**Visual Feedback**: Highlights target address briefly

#### getVisibleRange()

```typescript
getVisibleRange(): { start: number, end: number, pages: number[] }
```

**Returns**: Currently visible address range and page numbers

**Use Case**: Optimize memory fetching (only fetch visible pages)

### Events

#### 'address-clicked'

**Dispatched**: When user clicks memory address
**Detail**: `{ address: number }`

### Display Format

```
Address   Hex Dump                                     ASCII
0600      A9 42 8D 00 10 00 00 00 00 00 00 00 00 00  .B..............
0610      00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
```

**Layout**:
- Address column: 4-digit hex, gray
- Hex dump: 16 bytes, space-separated, syntax-highlighted
- ASCII column: Printable chars only, gray

**Changed Byte Highlighting**:
- Background: Red flash
- Duration: 1 second fade-out
- Tracked via CSS class `dirty`

---

## ControlPanel

**Purpose**: Execution control buttons

### Constructor

```typescript
class ControlPanel {
    constructor(selector: string);
}
```

### Methods

#### setMode(mode)

```typescript
setMode(mode: 'idle' | 'running' | 'error'): void
```

**Updates**: Button enabled/disabled states based on execution mode

**State Transitions**:
```
idle:
  - Step: enabled
  - Run: enabled
  - Stop: disabled
  - Reset: enabled

running:
  - Step: disabled
  - Run: disabled
  - Stop: enabled
  - Reset: disabled

error:
  - Step: disabled
  - Run: disabled
  - Stop: disabled
  - Reset: enabled (to recover)
```

### Events

#### 'step-clicked'

**Dispatched**: When Step button clicked
**Detail**: none

#### 'run-clicked'

**Dispatched**: When Run button clicked
**Detail**: none

#### 'stop-clicked'

**Dispatched**: When Stop button clicked
**Detail**: none

#### 'reset-clicked'

**Dispatched**: When Reset button clicked
**Detail**: none

### DOM Structure

```html
<div id="controls">
    <button id="btn-step">Step</button>
    <button id="btn-run">Run</button>
    <button id="btn-stop">Stop</button>
    <button id="btn-reset">Reset</button>
</div>
```

---

## ExampleSelector

**Purpose**: Load pre-written example programs

### Constructor

```typescript
class ExampleSelector {
    constructor(selector: string, examples: ExampleProgram[]);
}
```

**Parameters**:
- `selector`: CSS selector for container
- `examples`: Array of example program objects

### Methods

#### loadExample(id)

```typescript
loadExample(id: string): void
```

**Loads**: Example program into editor

### Events

#### 'example-loaded'

**Dispatched**: When example is selected
**Detail**: `{ id: string, source: string }`

### DOM Structure

```html
<div id="examples">
    <label>Examples:</label>
    <select id="example-select">
        <option value="">-- Choose Example --</option>
        <option value="counter">Simple Counter</option>
        <option value="fibonacci">Fibonacci</option>
    </select>
</div>
```

---

## ErrorDisplay

**Purpose**: Show assembly or execution errors

### Constructor

```typescript
class ErrorDisplay {
    constructor(selector: string);
}
```

### Methods

#### showError(error)

```typescript
showError(error: {
    type: 'assembly' | 'runtime',
    message: string,
    line?: number,
    pc?: number
}): void
```

**Displays**: Error message with context

#### clear()

```typescript
clear(): void
```

**Clears**: Error display

### DOM Structure

```html
<div id="error-display" class="hidden">
    <div class="error-header">
        <span class="error-type"></span>
        <button class="close">×</button>
    </div>
    <div class="error-message"></div>
    <div class="error-context"></div>
</div>
```

**Visual States**:
- Hidden by default (CSS class `hidden`)
- Assembly error: Orange background, shows line number
- Runtime error: Red background, shows PC value

---

## Component Communication

Components communicate via **Custom Events** dispatched on the document:

```javascript
// Component dispatches event
document.dispatchEvent(new CustomEvent('step-clicked'));

// Application listens for event
document.addEventListener('step-clicked', () => {
    emulator.step();
    updateUI();
});
```

**Rationale**: Decouples components, no shared state, easy to test

---

## Styling Contract

All components use BEM-style CSS classes:

```
.component-name { /* Container */ }
.component-name__element { /* Sub-element */ }
.component-name--modifier { /* State variant */ }
```

**Color Scheme** (CSS Custom Properties):
```css
:root {
    --bg-primary: #0a0a0a;
    --bg-secondary: #1a1a1a;
    --text-primary: #f0f0f0;
    --text-secondary: #888888;
    --accent-primary: #4A9EFF;
    --accent-success: #50FA7B;
    --accent-error: #FF5555;
}
```

**Typography**:
```css
body {
    font-family: 'JetBrains Mono', monospace;
}

h1, h2 {
    font-family: 'Sixtyfour', monospace;
}

code, pre, .mono {
    font-family: 'JetBrains Mono', monospace;
}
```

---

## Accessibility

All components must meet WCAG 2.1 Level AA:

- **Keyboard Navigation**: All interactive elements focusable via Tab
- **Screen Readers**: Use semantic HTML and ARIA labels where needed
- **Color Contrast**: Minimum 4.5:1 for text, 3:1 for large text
- **Focus Indicators**: Visible focus rings on all interactive elements

**Example**:
```html
<button id="btn-step" aria-label="Execute one instruction">
    Step
</button>
```

---

## Testing Contract

Each component should be testable in isolation:

```javascript
// Example unit test
const editor = new CodeEditor('#test-editor');
editor.setValue('LDA #$42');
assert.equal(editor.getValue(), 'LDA #$42');
```

**Mock DOM**: Use jsdom or similar for headless testing

**Integration Test**: Load full page, simulate user interactions

---

## Performance Requirements

- **Update Frequency**: Components must handle 60fps updates
- **Memory**: No memory leaks (proper event listener cleanup)
- **Scroll Performance**: MemoryViewer must maintain 60fps while scrolling

**Anti-patterns to avoid**:
- Re-creating DOM on every update
- Inline styles (use CSS classes)
- Synchronous layout thrashing (batch DOM reads/writes)
