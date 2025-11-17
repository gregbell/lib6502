/**
 * Example Selector Component
 * Load pre-written example programs
 */

export class ExampleSelector {
    constructor(editor) {
        this.editor = editor;
        this.examples = this.getExamples();
        this.render();
        this.setupEventListeners();
    }

    getExamples() {
        return [
            {
                id: 'counter',
                name: 'Counter',
                description: 'Simple counter from 0 to 255',
                code: `; Simple Counter
; Counts from 0 to 255 in the accumulator

        LDA #$00        ; Start at 0
loop:
        CLC             ; Clear carry
        ADC #$01        ; Add 1
        BNE loop        ; Loop if not zero
        BRK             ; Done`
            },
            {
                id: 'fibonacci',
                name: 'Fibonacci',
                description: 'Calculate Fibonacci sequence',
                code: `; Fibonacci Sequence
; Calculates Fibonacci numbers

        LDA #$00        ; F(0) = 0
        STA $00
        LDA #$01        ; F(1) = 1
        STA $01

loop:
        LDA $00         ; Load F(n)
        CLC
        ADC $01         ; Add F(n+1)
        STA $02         ; Store F(n+2)

        LDA $01         ; Shift: F(n) = F(n+1)
        STA $00
        LDA $02         ; F(n+1) = F(n+2)
        STA $01

        BCC loop        ; Continue if no overflow
        BRK             ; Done`
            },
            {
                id: 'stack',
                name: 'Stack Demo',
                description: 'Demonstrate stack operations',
                code: `; Stack Operations Demo
; Shows PHA/PLA stack usage

        LDA #$42        ; Load value
        PHA             ; Push A to stack

        LDA #$00        ; Clear A

        PLA             ; Pull from stack back to A

        TAX             ; Transfer to X
        TAY             ; Transfer to Y

        BRK             ; Done`
            }
        ];
    }

    render() {
        const container = document.querySelector('.editor-panel');
        if (!container) return;

        const selectorHtml = `
            <div class="example-selector">
                <label for="example-select">Examples:</label>
                <select id="example-select">
                    <option value="">-- Select an example --</option>
                    ${this.examples.map(ex =>
                        `<option value="${ex.id}">${ex.name} - ${ex.description}</option>`
                    ).join('')}
                </select>
            </div>
        `;

        // Insert before editor container
        const editorContainer = document.getElementById('editor-container');
        editorContainer.insertAdjacentHTML('beforebegin', selectorHtml);
    }

    setupEventListeners() {
        const select = document.getElementById('example-select');
        if (!select) return;

        select.addEventListener('change', (e) => {
            const exampleId = e.target.value;
            if (!exampleId) return;

            const example = this.examples.find(ex => ex.id === exampleId);
            if (example) {
                this.editor.setValue(example.code);
                document.dispatchEvent(new CustomEvent('example-loaded', { detail: example }));
                console.log(`✓ Loaded example: ${example.name}`);
            }
        });
    }
}
