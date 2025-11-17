/**
 * Code Editor Component
 * Simple textarea with syntax highlighting for 6502 assembly
 */

export class CodeEditor {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.textarea = null;
        this.highlighted = null;
        this.render();
        this.setupEventListeners();
    }

    render() {
        this.container.innerHTML = `
            <div class="editor-wrapper">
                <textarea id="code-editor" class="code-input" spellcheck="false" placeholder="Type your 6502 assembly code here...

Example:
    LDA #$42    ; Load $42 into accumulator
    TAX         ; Transfer A to X
    TAY         ; Transfer A to Y
    BRK         ; Break (halt)"></textarea>
                <pre id="code-highlight" class="code-highlight"><code id="code-display"></code></pre>
            </div>
        `;

        this.textarea = document.getElementById('code-editor');
        this.highlighted = document.getElementById('code-display');
    }

    setupEventListeners() {
        this.textarea.addEventListener('input', () => {
            this.updateHighlighting();
            document.dispatchEvent(new CustomEvent('code-changed'));
        });

        this.textarea.addEventListener('scroll', () => {
            this.highlighted.parentElement.scrollTop = this.textarea.scrollTop;
            this.highlighted.parentElement.scrollLeft = this.textarea.scrollLeft;
        });

        this.textarea.addEventListener('keydown', (e) => {
            // Tab key support
            if (e.key === 'Tab') {
                e.preventDefault();
                const start = this.textarea.selectionStart;
                const end = this.textarea.selectionEnd;
                const value = this.textarea.value;
                this.textarea.value = value.substring(0, start) + '    ' + value.substring(end);
                this.textarea.selectionStart = this.textarea.selectionEnd = start + 4;
                this.updateHighlighting();
            }
        });
    }

    updateHighlighting() {
        const code = this.textarea.value;
        const highlighted = this.highlightSyntax(code);
        this.highlighted.innerHTML = highlighted + '\n';
    }

    highlightSyntax(code) {
        // 6502 instruction set
        const instructions = [
            'ADC', 'AND', 'ASL', 'BCC', 'BCS', 'BEQ', 'BIT', 'BMI', 'BNE', 'BPL', 'BRK', 'BVC', 'BVS',
            'CLC', 'CLD', 'CLI', 'CLV', 'CMP', 'CPX', 'CPY',
            'DEC', 'DEX', 'DEY',
            'EOR',
            'INC', 'INX', 'INY',
            'JMP', 'JSR',
            'LDA', 'LDX', 'LDY', 'LSR',
            'NOP',
            'ORA',
            'PHA', 'PHP', 'PLA', 'PLP',
            'ROL', 'ROR', 'RTI', 'RTS',
            'SBC', 'SEC', 'SED', 'SEI', 'STA', 'STX', 'STY',
            'TAX', 'TAY', 'TSX', 'TXA', 'TXS', 'TYA'
        ];

        const instructionPattern = new RegExp(`\\b(${instructions.join('|')})\\b`, 'gi');

        return code
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .split('\n')
            .map(line => {
                // Highlight comments
                if (line.includes(';')) {
                    const parts = line.split(';');
                    const code = parts[0];
                    const comment = parts.slice(1).join(';');
                    return this.highlightLine(code) + '<span class="asm-comment">; ' + comment + '</span>';
                }
                return this.highlightLine(line);
            })
            .join('\n');
    }

    highlightLine(line) {
        // Label (starts at beginning of line, ends with colon)
        line = line.replace(/^(\w+):/, '<span class="asm-label">$1:</span>');

        // Instructions
        line = line.replace(/\b(ADC|AND|ASL|BCC|BCS|BEQ|BIT|BMI|BNE|BPL|BRK|BVC|BVS|CLC|CLD|CLI|CLV|CMP|CPX|CPY|DEC|DEX|DEY|EOR|INC|INX|INY|JMP|JSR|LDA|LDX|LDY|LSR|NOP|ORA|PHA|PHP|PLA|PLP|ROL|ROR|RTI|RTS|SBC|SEC|SED|SEI|STA|STX|STY|TAX|TAY|TSX|TXA|TXS|TYA)\b/gi,
            '<span class="asm-instruction">$1</span>');

        // Hex numbers
        line = line.replace(/\$[0-9A-Fa-f]+/g, '<span class="asm-hex">$&</span>');

        // Immediate mode marker
        line = line.replace(/#/g, '<span class="asm-immediate">#</span>');

        return line;
    }

    getValue() {
        return this.textarea.value;
    }

    setValue(code) {
        this.textarea.value = code;
        this.updateHighlighting();
    }
}
