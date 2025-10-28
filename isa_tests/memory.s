.global _start
.section .text.init

_start:	
	li a1, 0x80010000 #RAM memory
	addi a0, x0, 0x68
	sb a0, 0(a1) # 'h'

	addi a0, x0, 0x65
	sb a0, 1(a1) # 'e'

	addi a0, x0, 0x6C
	sb a0, 2(a1) # 'l'

	addi a0, x0, 0x6C
	sb a0, 3(a1) # 'l'

	addi a0, x0, 0x6F
	sb a0, 4(a1) # 'o'

	li a0, 0x6F77
	sh a0, 5(a1) # 'wo'

	li a0, 0x21646C72
	sw a0, 7(a1) # 'rld!'

	li a2, 0x40600004 #UART TX IO
	lb a0, 0(a1)
	sb a0, (a2) # 'h'

	lb a0, 1(a1)
	sb a0, (a2) # 'e'

	lb a0, 2(a1)
	sb a0, (a2) # 'l'

	lb a0, 3(a1)
	sb a0, (a2) # 'l'

	lb a0, 4(a1)
	sb a0, (a2) # 'o'

	lh a0, 5(a1)
	sb a0, (a2) # 'w'
	srl a0, a0, 8
	sb a0, (a2) # 'o'

	lw a0, 7(a1)
	sb a0, (a2) # 'r'
	srl a0, a0, 8
	sb a0, (a2) # 'l'
	srl a0, a0, 8
	sb a0, (a2) # 'd'
	srl a0, a0, 8
	sb a0, (a2) # '!'

loop: j loop
