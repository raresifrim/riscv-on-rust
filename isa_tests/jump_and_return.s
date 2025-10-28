.global _start
.section .text.init 

_start: 
    addi a0, a0, 1
    addi a1, a1, 2
    addi a2, a2, 3
    call _func 
    addi a0, a0, 1

_no_exit:
    j _no_exit
        
_func:
    add a4, a0, a1
    slli a5, a2, 1
    sub a6, a5, a5
    ori a0, a6, 4   
    ret
