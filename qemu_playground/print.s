  .global print_ecall
  .section .text.print

print_ecall:
  addi sp, sp, -12
  sw ra, 8(sp)
  sw a0, 4(sp) #arg pointer to string
  sw a1, 0(sp) #arg size of string 
  
  li a7, 0x4442434E
  li a6, 0x00
  lw a1, 4(sp) #second is pointer to strin in this ecall
  lw a0, 0(sp) #first is size in this ecall 
  li a2, 0
  ecall
  
  lw ra, 8(sp)
  addi sp, sp, 12
  ret

