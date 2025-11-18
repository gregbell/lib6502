/**
 * Terminal Component - xterm.js wrapper for serial I/O
 *
 * Provides a terminal interface connected to the 6502 UART device.
 * User input from the terminal is dispatched as CustomEvents for the emulator to handle.
 * Output from the UART is displayed in the terminal via the write() method.
 */

export class Terminal {
    /**
     * Create a new terminal instance
     * @param {string} containerId - ID of the DOM element to attach the terminal to
     */
    constructor(containerId) {
        // Verify container exists
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Terminal container '${containerId}' not found`);
        }

        // Create xterm.js terminal with configuration
        this.term = new window.Terminal({
            cursorBlink: true,
            cursorStyle: 'block',
            fontSize: 14,
            fontFamily: '"JetBrains Mono", monospace',
            theme: {
                background: '#1a1a1a',
                foreground: '#f0f0f0',
                cursor: '#48D096',
                cursorAccent: '#060f11',
                selectionBackground: 'rgba(72, 208, 150, 0.3)',
                black: '#1a1a1a',
                red: '#FF5555',
                green: '#48D096',
                yellow: '#FFB86C',
                blue: '#8BE9FD',
                magenta: '#FF79C6',
                cyan: '#48D096',
                white: '#f0f0f0',
                brightBlack: '#6272A4',
                brightRed: '#FF6E6E',
                brightGreen: '#69F0AE',
                brightYellow: '#FFD180',
                brightBlue: '#80D8FF',
                brightMagenta: '#FF80AB',
                brightCyan: '#69F0AE',
                brightWhite: '#FFFFFF'
            },
            cols: 80,
            rows: 24,
            scrollback: 1000,
            convertEol: false
        });

        // Create and attach FitAddon for responsive sizing
        this.fitAddon = new window.FitAddon.FitAddon();
        this.term.loadAddon(this.fitAddon);

        // Open terminal in container
        this.term.open(container);
        this.fitAddon.fit();

        // Setup event listeners
        this.setupEventListeners();

        // Terminal Ready State Indication (User Story 2)
        // Display welcome message to indicate terminal is ready for serial I/O.
        // This provides immediate visual feedback that:
        // - The terminal component initialized successfully
        // - UART communication is available at memory addresses $A000-$A003
        // - User can start typing to send characters to the 6502 emulator
        // - 6502 programs can write to $A000 to output text here
        this.write('6502 Serial Terminal Ready\r\n');
        this.write('UART: $A000-$A003\r\n');
        this.write('\r\n');
    }

    /**
     * Setup terminal event listeners
     * Handles user input and window resize events
     */
    setupEventListeners() {
        // Handle user input - dispatch CustomEvent for app to handle
        this.term.onData((data) => {
            // Dispatch custom event with terminal data
            document.dispatchEvent(new CustomEvent('terminal-data', {
                detail: { data }
            }));
        });

        // Handle window resize - fit terminal to container
        window.addEventListener('resize', () => {
            this.fitAddon.fit();
        });
    }

    /**
     * Write text to the terminal
     * @param {string} text - Text to display
     */
    write(text) {
        this.term.write(text);
    }

    /**
     * Clear the terminal display
     */
    clear() {
        this.term.clear();
    }

    /**
     * Fit terminal to container size
     */
    fit() {
        this.fitAddon.fit();
    }
}
