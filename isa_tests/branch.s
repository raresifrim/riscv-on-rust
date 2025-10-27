.global _start
.section .text.init 

_start: addi x10, x10, 1
        addi x11, x11, 2
        bne x10, x11, _neq
        add  x12, x11, x10

_neq:   addi x12, x12, 2
        beq x12, x11, _start 
        add  x13, x13, x12
