/**
 * Code Editor Component
 * CodeMirror 6 via unpkg ESM modules with regex-based highlighting
 */

import {
    EditorState,
    RangeSetBuilder,
    Decoration,
    EditorView,
    ViewPlugin,
    highlightActiveLine,
    keymap,
    placeholder,
    highlightActiveLineGutter,
    lineNumbers,
    history,
    historyKeymap,
    defaultKeymap,
    indentWithTab
} from '../vendor/codemirror.js';

const MNEMONICS = [
    'ADC', 'AND', 'ASL', 'BCC', 'BCS', 'BEQ', 'BIT', 'BMI', 'BNE', 'BPL', 'BRK', 'BVC', 'BVS',
    'CLC', 'CLD', 'CLI', 'CLV', 'CMP', 'CPX', 'CPY', 'DEC', 'DEX', 'DEY', 'EOR', 'INC', 'INX',
    'INY', 'JMP', 'JSR', 'LDA', 'LDX', 'LDY', 'LSR', 'NOP', 'ORA', 'PHA', 'PHP', 'PLA', 'PLP',
    'ROL', 'ROR', 'RTI', 'RTS', 'SBC', 'SEC', 'SED', 'SEI', 'STA', 'STX', 'STY', 'TAX', 'TAY',
    'TSX', 'TXA', 'TXS', 'TYA'
];

const PATTERNS = {
    label: /^\s*[A-Za-z_][A-Za-z0-9_]*:/g,
    constant: /^\s*[A-Za-z_][A-Za-z0-9_]*\s*=/g,
    directive: /\.(byte|word|org|ascii|string|fill|align|db|dw|ds|equ)/gi,
    mnemonic: new RegExp(`\\b(${MNEMONICS.join('|')})\\b`, 'gi'),
    immediate: /#(?:\$[0-9A-Fa-f]+|%[01]+|0x[0-9A-Fa-f]+|\d+)/g,
    hexNumber: /\$[0-9A-Fa-f]+\b/g,
    binaryNumber: /%[01]+\b/g,
    decimalNumber: /\b\d+\b/g,
    string: /"[^"\\]*(?:\\.[^"\\]*)*"?/g
};

const HIGHLIGHT_DECOS = {
    comment: Decoration.mark({ class: 'cm-asm-comment' }),
    label: Decoration.mark({ class: 'cm-asm-label' }),
    constant: Decoration.mark({ class: 'cm-asm-constant' }),
    directive: Decoration.mark({ class: 'cm-asm-directive' }),
    mnemonic: Decoration.mark({ class: 'cm-asm-mnemonic' }),
    immediate: Decoration.mark({ class: 'cm-asm-immediate' }),
    number: Decoration.mark({ class: 'cm-asm-number' }),
    stringLiteral: Decoration.mark({ class: 'cm-asm-string' })
};

function collectRegexMatches(matches, regex, deco, lineStart, text) {
    regex.lastIndex = 0;
    let match;
    while ((match = regex.exec(text)) !== null) {
        matches.push({ from: lineStart + match.index, to: lineStart + match.index + match[0].length, deco });
    }
}

// Build regex-based highlights while skipping everything after ';' comments.
function buildAsmDecorations(view) {
    const allMatches = [];

    for (const { from, to } of view.visibleRanges) {
        let line = view.state.doc.lineAt(from);

        while (true) {
            const lineStart = line.from;
            const lineText = line.text;
            const commentIndex = lineText.indexOf(';');
            const codeText = commentIndex === -1 ? lineText : lineText.slice(0, commentIndex);

            if (commentIndex !== -1) {
                allMatches.push({ from: lineStart + commentIndex, to: lineStart + lineText.length, deco: HIGHLIGHT_DECOS.comment });
            }

            collectRegexMatches(allMatches, PATTERNS.label, HIGHLIGHT_DECOS.label, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.constant, HIGHLIGHT_DECOS.constant, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.directive, HIGHLIGHT_DECOS.directive, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.mnemonic, HIGHLIGHT_DECOS.mnemonic, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.immediate, HIGHLIGHT_DECOS.immediate, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.hexNumber, HIGHLIGHT_DECOS.number, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.binaryNumber, HIGHLIGHT_DECOS.number, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.decimalNumber, HIGHLIGHT_DECOS.number, lineStart, codeText);
            collectRegexMatches(allMatches, PATTERNS.string, HIGHLIGHT_DECOS.stringLiteral, lineStart, codeText);

            if (line.to >= to) break;
            line = view.state.doc.line(line.number + 1);
        }
    }

    // Sort by position (required by RangeSetBuilder)
    allMatches.sort((a, b) => a.from - b.from || a.to - b.to);

    const builder = new RangeSetBuilder();
    for (const { from, to, deco } of allMatches) {
        builder.add(from, to, deco);
    }
    return builder.finish();
}

const asmHighlighter = ViewPlugin.fromClass(class {
    constructor(view) {
        this.decorations = buildAsmDecorations(view);
    }

    update(update) {
        if (update.docChanged || update.viewportChanged) {
            this.decorations = buildAsmDecorations(update.view);
        }
    }
}, {
    decorations: plugin => plugin.decorations
});

const asmTheme = EditorView.theme({
    '&': {
        backgroundColor: 'var(--bg-secondary)',
        color: 'var(--text-primary)',
        border: '1px solid var(--border)',
        borderRadius: '4px',
        fontFamily: '\'JetBrains Mono\', monospace',
        fontSize: '13px',
        height: '400px'
    },
    '.cm-scroller': {
        lineHeight: '1.5',
        overflow: 'auto'
    },
    '.cm-content': {
        caretColor: 'var(--text-primary)'
    },
    '.cm-activeLine': {
        backgroundColor: 'rgba(72, 208, 150, 0.08)'
    },
    '.cm-gutters': {
        backgroundColor: 'var(--bg-secondary)',
        color: 'var(--text-secondary)',
        borderRight: '1px solid var(--border)'
    },
    '.cm-activeLineGutter': {
        backgroundColor: 'rgba(72, 208, 150, 0.08)',
        color: 'var(--text-primary)'
    },
    '.cm-selectionBackground': {
        backgroundColor: 'rgba(74, 158, 255, 0.25)'
    },
    '.cm-asm-mnemonic': { color: '#FF79C6', fontWeight: '600' },
    '.cm-asm-label': { color: '#50FA7B', fontWeight: '600' },
    '.cm-asm-constant': { color: '#50FA7B', fontWeight: '600' },
    '.cm-asm-directive': { color: '#8BE9FD' },
    '.cm-asm-number': { color: '#BD93F9' },
    '.cm-asm-immediate': { color: '#FFB86C' },
    '.cm-asm-string': { color: '#F8F8F2' },
    '.cm-asm-comment': { color: '#6272A4', fontStyle: 'italic' }
}, { dark: true });

export class CodeEditor {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.view = null;
        this.suppressEvents = false;
        this.render();
    }

    render() {
        this.container.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'editor-wrapper';
        this.container.appendChild(wrapper);

        const state = EditorState.create({
            doc: '',
            extensions: [
                lineNumbers(),
                highlightActiveLineGutter(),
                history(),
                keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
                highlightActiveLine(),
                EditorView.lineWrapping,
                placeholder('Type your 6502 assembly code here...'),
                asmHighlighter,
                asmTheme,
                EditorView.updateListener.of((update) => {
                    if (update.docChanged && !this.suppressEvents) {
                        document.dispatchEvent(new CustomEvent('code-changed'));
                    }
                })
            ]
        });

        this.view = new EditorView({
            state,
            parent: wrapper
        });
    }

    getValue() {
        return this.view ? this.view.state.doc.toString() : '';
    }

    setValue(code) {
        if (!this.view) return;

        this.suppressEvents = true;
        this.view.dispatch({
            changes: { from: 0, to: this.view.state.doc.length, insert: code }
        });
        this.suppressEvents = false;
        this.view.focus();
    }

    focus() {
        this.view?.focus();
    }
}
