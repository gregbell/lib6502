; Stack Operations Demo
; Shows PHA/PLA stack usage

        LDA #$42        ; Load value
        PHA             ; Push A to stack

        LDA #$00        ; Clear A

        PLA             ; Pull from stack back to A

        TAX             ; Transfer to X
        TAY             ; Transfer to Y

        BRK             ; Done
