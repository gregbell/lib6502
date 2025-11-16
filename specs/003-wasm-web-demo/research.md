# Technical Research: WASM Web Demo

**Date**: 2025-11-16
**Feature**: Interactive 6502 Assembly Web Demo
**Purpose**: Resolve technical unknowns and establish implementation patterns

## 1. WASM Build Tooling

### Decision: Use wasm-pack

**Rationale**: wasm-pack provides a complete build pipeline that wraps wasm-bindgen with optimization and packaging, aligning with the "clarity & hackability" principle by providing a batteries-included workflow.

**Workflow**:
```bash
# Install
cargo install wasm-pack

# Build for web deployment
wasm-pack build --target web --out-dir demo/lib6502_wasm

# Local testing
python3 -m http.server 8000 -d demo/
```

**Dependencies**:
- `wasm-bindgen = "0.2"` (only dependency needed)
- `wasm-pack` (build tool, not runtime dependency)

**Alternatives Considered**:
- **cargo-web**: Older tool, less maintained, more complex
- **wasm-bindgen alone**: Requires manual orchestration of build steps
- **Rejected because**: wasm-pack is the current standard and simplifies the workflow

## 2. WASM API Design Patterns

### Decision: Single CPU Instance with Result-Based Error Handling

**Pattern**:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Emulator6502 {
    cpu: lib6502::CPU<lib6502::FlatMemory>,
}

#[wasm_bindgen]
impl Emulator6502 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Emulator6502 {
            cpu: lib6502::CPU::new(lib6502::FlatMemory::new()),
        }
    }

    pub fn step(&mut self) -> Result<(), JsError> {
        self.cpu.step().map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    pub fn run_for_cycles(&mut self, cycles: u32) -> Result<u32, JsError> {
        self.cpu.run_for_cycles(cycles)
            .map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    // Batch register access
    pub fn get_a(&self) -> u8 { self.cpu.a() }
    pub fn get_x(&self) -> u8 { self.cpu.x() }
    pub fn get_y(&self) -> u8 { self.cpu.y() }
    pub fn get_pc(&self) -> u16 { self.cpu.pc() }
    pub fn get_sp(&self) -> u8 { self.cpu.sp() }
    pub fn get_cycles(&self) -> u64 { self.cpu.cycles() }

    // Flag accessors
    pub fn get_flag_n(&self) -> bool { self.cpu.flag_n() }
    pub fn get_flag_z(&self) -> bool { self.cpu.flag_z() }
    // ... other flags

    // Memory access patterns
    pub fn read_memory(&self, addr: u16) -> u8 {
        self.cpu.memory().read(addr)
    }

    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.cpu.memory_mut().write(addr, value)
    }

    // Bulk memory access for display (returns Vec<u8>)
    pub fn get_memory_page(&self, page: u8) -> Vec<u8> {
        let start = (page as u16) << 8;
        (0..256).map(|i| self.cpu.memory().read(start + i)).collect()
    }

    pub fn reset(&mut self) {
        // Reset CPU state
        self.cpu = lib6502::CPU::new(lib6502::FlatMemory::new());
    }

    // Load program into memory
    pub fn load_program(&mut self, program: &[u8], start_addr: u16) {
        for (i, &byte) in program.iter().enumerate() {
            let addr = start_addr.wrapping_add(i as u16);
            self.cpu.memory_mut().write(addr, byte);
        }
        // Set PC to start address
        self.cpu.set_pc(start_addr);
    }
}
```

**JavaScript Usage**:
```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502_wasm.js';

await init();
const emu = new Emulator6502();

// Load and run program
const machineCode = new Uint8Array([0xA9, 0x42]); // LDA #$42
emu.load_program(machineCode, 0x0600);
emu.step();

console.log('A register:', emu.get_a().toString(16));
```

**Performance Optimization**:
- Individual register getters (acceptable overhead for ~8 registers)
- Page-based memory access (256 bytes at a time) for viewer
- Batch execution via `run_for_cycles()` to minimize JS↔WASM calls

**Alternatives Considered**:
- **Serde-based state serialization**: Adds dependency (`serde-wasm-bindgen`)
- **Rejected because**: Individual getters align with zero-dependency principle and provide clearer API

## 3. Syntax Highlighting Without Dependencies

### Decision: Regex-Based Highlighting

**Implementation**:
```javascript
class Asm6502Highlighter {
    constructor() {
        this.patterns = [
            { regex: /;.*$/gm, className: 'asm-comment' },
            { regex: /^[a-zA-Z_]\w*:/gm, className: 'asm-label' },
            { regex: /\b(LDA|STA|LDX|STX|LDY|STY|ADC|SBC|AND|ORA|EOR|CMP|CPX|CPY|INC|DEC|INX|DEX|INY|DEY|ASL|LSR|ROL|ROR|JMP|JSR|RTS|BEQ|BNE|BCS|BCC|BMI|BPL|BVS|BVC|TAX|TAY|TXA|TYA|TSX|TXS|PHA|PLA|PHP|PLP|CLC|SEC|CLD|SED|CLI|SEI|CLV|NOP|BRK|RTI)\b/gi, className: 'asm-instruction' },
            { regex: /\$[0-9A-Fa-f]+/g, className: 'asm-hex' },
            { regex: /#\$[0-9A-Fa-f]+/g, className: 'asm-immediate' },
        ];
    }

    highlight(code) {
        let html = this.escapeHtml(code);
        for (const { regex, className } of this.patterns) {
            html = html.replace(regex, match => `<span class="${className}">${match}</span>`);
        }
        return html;
    }

    escapeHtml(text) {
        return text.replace(/[&<>]/g, m => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[m]));
    }
}
```

**CSS** (JetBrains Mono font):
```css
.asm-editor {
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    line-height: 1.5;
}

.asm-comment { color: #6A9955; font-style: italic; }
.asm-label { color: #4EC9B0; font-weight: 600; }
.asm-instruction { color: #DCDCAA; font-weight: 500; }
.asm-hex, .asm-immediate { color: #B5CEA8; }
```

**Performance**: <1ms for 200 lines, no dependencies

**Alternatives Considered**:
- **CodeMirror/Monaco**: Heavy dependencies (100KB+), overkill for simple highlighting
- **Rejected because**: Violates zero-dependency principle, adds complexity

## 4. Memory Viewer Implementation

### Decision: Virtual Scrolling with 16 bytes/row

**Implementation Pattern**:
```javascript
class MemoryViewer {
    constructor(container) {
        this.container = container;
        this.bytesPerRow = 16;
        this.rowHeight = 20; // pixels
        this.memory = new Uint8Array(65536);
        this.dirtyBytes = new Set();

        this.setupVirtualScroll();
    }

    setupVirtualScroll() {
        const totalRows = 4096; // 64KB / 16 bytes

        this.viewport = document.createElement('div');
        this.viewport.style.height = `${totalRows * this.rowHeight}px`;
        this.viewport.style.position = 'relative';

        this.visibleWindow = document.createElement('div');
        this.visibleWindow.style.position = 'absolute';

        this.viewport.appendChild(this.visibleWindow);
        this.container.appendChild(this.viewport);

        this.container.addEventListener('scroll', () => this.render());
        this.render();
    }

    render() {
        const scrollTop = this.container.scrollTop;
        const startRow = Math.floor(scrollTop / this.rowHeight);
        const visibleRows = Math.ceil(this.container.clientHeight / this.rowHeight) + 2;

        this.visibleWindow.style.top = `${startRow * this.rowHeight}px`;

        let html = '';
        for (let row = startRow; row < startRow + visibleRows; row++) {
            if (row >= 4096) break;
            html += this.renderRow(row);
        }
        this.visibleWindow.innerHTML = html;
    }

    renderRow(row) {
        const addr = row * 16;
        let hex = '';
        let ascii = '';

        for (let i = 0; i < 16; i++) {
            const byteAddr = addr + i;
            const byte = this.memory[byteAddr];
            const isDirty = this.dirtyBytes.has(byteAddr);

            const hexStr = byte.toString(16).padStart(2, '0').toUpperCase();
            hex += `<span class="${isDirty ? 'dirty' : ''}">${hexStr}</span> `;

            const char = (byte >= 32 && byte < 127) ? String.fromCharCode(byte) : '.';
            ascii += char;
        }

        return `<div class="mem-row">
            <span class="addr">${addr.toString(16).padStart(4, '0')}</span>
            <span class="hex">${hex}</span>
            <span class="ascii">${ascii}</span>
        </div>`;
    }

    updateMemory(newMemory) {
        this.dirtyBytes.clear();
        for (let i = 0; i < 65536; i++) {
            if (this.memory[i] !== newMemory[i]) {
                this.dirtyBytes.add(i);
            }
        }
        this.memory = new Uint8Array(newMemory);
        this.render();

        setTimeout(() => {
            this.dirtyBytes.clear();
            this.render();
        }, 1000);
    }
}
```

**Display Format**:
```
0600  A9 42 85 10 A9 FF 85 11  4C 00 06 00 00 00 00 00  .B......L.......
0610  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  ................
```

**Performance**:
- Renders only ~25 visible rows (not 4096)
- 60fps scrolling via position: absolute
- Dirty byte highlighting with 1s fade

**Alternatives Considered**:
- **Full table rendering**: 4096 DOM rows causes browser lag
- **Canvas-based rendering**: More complex, harder to select/copy text
- **Rejected because**: Virtual scrolling provides best balance of performance and usability

## 5. GitHub Pages Deployment

### Decision: GitHub Actions with wasm-pack

**Workflow** (`.github/workflows/deploy-demo.yml`):
```yaml
name: Deploy WASM Demo

on:
  push:
    branches: [ main, 003-wasm-web-demo ]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: wasm-pack build --target web --out-dir demo/lib6502_wasm

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: './demo'

      - name: Deploy to GitHub Pages
        uses: actions/deploy-pages@v4
```

**MIME Types**: GitHub Pages automatically serves `.wasm` with `application/wasm` - no configuration needed

**CORS**: All content served with `Access-Control-Allow-Origin: *` - perfect for WASM

**Local Testing**:
```bash
wasm-pack build --target web --out-dir demo/lib6502_wasm
python3 -m http.server 8000 -d demo/
# Open http://localhost:8000
```

**Alternatives Considered**:
- **Manual wasm-bindgen invocation**: More steps, error-prone
- **Netlify/Vercel**: Adds external dependency
- **Rejected because**: GitHub Actions + Pages is zero-cost, zero-config, aligns with project hosting

## 6. UI Design (Oxide.computer Inspired)

### Design System

**Typography**:
- **Headings/Display**: Sixtyfour (Google Fonts) - retro-futuristic pixel font
- **Body/Code**: JetBrains Mono (Google Fonts) - excellent monospace for code and data

**Color Palette**:
```css
:root {
    --bg-primary: #0a0a0a;
    --bg-secondary: #1a1a1a;
    --bg-panel: #141414;
    --text-primary: #f0f0f0;
    --text-secondary: #888888;
    --accent-primary: #4A9EFF;
    --accent-success: #50FA7B;
    --accent-warning: #FFB86C;
    --accent-error: #FF5555;
    --border: #2a2a2a;
}
```

**Layout Principles**:
- Dark theme (reduced eye strain)
- High contrast text (WCAG AA minimum)
- 8px spacing grid (8, 16, 24, 32, 48, 64)
- Minimal borders and shadows
- Monospace for all technical data (registers, memory, code)

**Component Styling**:
```css
body {
    font-family: 'JetBrains Mono', monospace;
    background: var(--bg-primary);
    color: var(--text-primary);
}

h1, h2, h3 {
    font-family: 'Sixtyfour', monospace;
    font-weight: 400;
}

.panel {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 16px;
}

button {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    cursor: pointer;
}
```

**Alternatives Considered**:
- **Material Design**: Too opinionated, heavy framework
- **Tailwind CSS**: Adds build dependency
- **Rejected because**: Custom CSS aligns with minimal, hackable approach

## Summary of Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Build Tool | wasm-pack | Complete workflow, industry standard |
| API Pattern | Individual getters + Result errors | Clear API, zero extra dependencies |
| Syntax Highlighting | Custom regex | 50 lines, zero deps, <1ms |
| Memory Viewer | Virtual scrolling | Performance (25 rows vs 4096) |
| Deployment | GitHub Actions → Pages | Zero-cost, zero-config |
| Typography | Sixtyfour + JetBrains Mono | Retro aesthetic + excellent code font |
| Design | Oxide-inspired dark theme | Minimal, technical, high contrast |

All decisions align with constitutional principles: WASM portability, zero external dependencies (except wasm-bindgen), clarity, and hackability.
