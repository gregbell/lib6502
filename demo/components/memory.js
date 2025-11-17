/**
 * Memory Viewer Component
 * Virtual scrolling hex dump of 64KB memory
 */

export class MemoryViewer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.currentPage = 0; // Current 256-byte page
    this.cache = new Uint8Array(65536); // Cache of memory for dirty tracking
    this.render();
    this.setupEventListeners();
  }

  render() {
    this.container.innerHTML = `
            <div class="memory-controls">
                <label for="addr-jump">Jump to:</label>
                <input type="text" id="addr-jump" placeholder="$0600" maxlength="5">
                <button id="addr-jump-btn" class="btn btn-primary">Go</button>
            </div>
            <div class="memory-scroll-container" id="memory-scroll">
                <div class="memory-content" id="memory-content"></div>
            </div>
        `;

    this.scrollContainer = document.getElementById('memory-scroll');
    this.contentContainer = document.getElementById('memory-content');

    // Render initial view
    this.renderMemory();
  }

  setupEventListeners() {
    // Address jump
    document.getElementById('addr-jump-btn').addEventListener('click', () => {
      this.handleJump();
    });

    document.getElementById('addr-jump').addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        this.handleJump();
      }
    });

    // Scroll handling (virtual scrolling)
    this.scrollContainer.addEventListener('scroll', () => {
      this.handleScroll();
    });
  }

  handleJump() {
    const input = document.getElementById('addr-jump').value.trim();
    if (!input) return;

    let addr;
    if (input.startsWith('$')) {
      addr = parseInt(input.substring(1), 16);
    } else if (input.startsWith('0x')) {
      addr = parseInt(input.substring(2), 16);
    } else {
      addr = parseInt(input, 16);
    }

    if (isNaN(addr) || addr < 0 || addr > 0xFFFF) {
      console.error('Invalid address:', input);
      return;
    }

    this.jumpToAddress(addr);
  }

  jumpToAddress(addr) {
    // Calculate row for address (16 bytes per row)
    const row = Math.floor(addr / 16);
    const rowHeight = 20; // pixels
    const scrollPos = row * rowHeight;

    this.scrollContainer.scrollTop = scrollPos;

    // Highlight the address briefly
    setTimeout(() => {
      const byteIndex = addr % 16;
      const highlightClass = 'memory-highlight';
      // Implementation note: Would need to track DOM elements for highlighting
    }, 100);
  }

  handleScroll() {
    const scrollTop = this.scrollContainer.scrollTop;
    const rowHeight = 20; // pixels
    const startRow = Math.floor(scrollTop / rowHeight);

    // Render visible rows plus buffer
    this.renderMemory(startRow);
  }

  renderMemory(startRow = 0) {
    const rowsVisible = 25; // ~500px / 20px per row
    const bufferRows = 10;
    const totalRows = 4096; // 65536 bytes / 16 bytes per row

    const start = Math.max(0, startRow - bufferRows);
    const end = Math.min(totalRows, startRow + rowsVisible + bufferRows);

    let html = '';
    for (let row = start; row < end; row++) {
      const addr = row * 16;
      html += this.renderRow(addr);
    }

    this.contentContainer.innerHTML = html;

    // Set content height for proper scrolling
    this.contentContainer.style.height = (totalRows * 20) + 'px';
    this.contentContainer.style.paddingTop = (start * 20) + 'px';
  }

  renderRow(addr) {
    let html = `<div class="memory-row">`;
    html += `<span class="memory-addr">${this.formatHex(addr, 4)}:</span>`;

    // Hex bytes
    html += `<span class="memory-hex">`;
    for (let i = 0; i < 16; i++) {
      const byte = this.cache[addr + i] || 0;
      html += `<span class="memory-byte">${this.formatHex(byte, 2)}</span> `;
    }
    html += `</span>`;

    // ASCII
    html += `<span class="memory-ascii">`;
    for (let i = 0; i < 16; i++) {
      const byte = this.cache[addr + i] || 0;
      const char = (byte >= 32 && byte < 127) ? String.fromCharCode(byte) : '.';
      html += char;
    }
    html += `</span>`;

    html += `</div>`;
    return html;
  }

  update(emulator) {
    // Get visible pages
    const scrollTop = this.scrollContainer.scrollTop;
    const rowHeight = 20;
    const startRow = Math.floor(scrollTop / rowHeight);
    const startAddr = startRow * 16;
    const endAddr = Math.min(0xFFFF, startAddr + (25 * 16)); // 25 visible rows

    // Update cache for visible region
    for (let addr = startAddr; addr <= endAddr; addr++) {
      try {
        this.cache[addr] = emulator.read_memory(addr);
      } catch (e) {
        // Ignore read errors
      }
    }

    // Re-render visible rows
    this.renderMemory(startRow);
  }

  formatHex(value, digits) {
    return value.toString(16).toUpperCase().padStart(digits, '0');
  }
}
