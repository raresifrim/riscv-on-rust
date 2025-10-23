.global _start
.section .text.init 

_start: addi x1, x1, 1
        addi x2, x2, 2
        add  x3, x2, x1
