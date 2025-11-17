; Simple Counter
; Counts from 0 to 255 in the accumulator

        LDA #$00        ; Start at 0
loop:
        CLC             ; Clear carry
        ADC #$01        ; Add 1
        BNE loop        ; Loop if not zero
        BRK             ; Done
