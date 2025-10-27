.global _start
.section .text.init 

_start: addi x1, x1, 1
        addi x2, x2, 2
        add  x3, x2, x1
        addi x4, x3, -4
        addi x5, x4, -4
        add  x6, x5, x3
        sub  x7, x6, x5
