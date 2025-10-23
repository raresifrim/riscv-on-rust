MEMORY {
  ROM (rx) : ORIGIN = 0x80000000, LENGTH = 64K
  RAM (rw) : ORIGIN = 0x80010000, LENGTH = 64K
}

SECTIONS { 
  .text : {
    *(.text*)
  } > ROM

  .rodata : ALIGN(4) {
    *(.rodata .rodata.* .srodata);
  } > RAM

  .data : ALIGN(4) {
    *(.data .data.*);
  } > RAM

  /* Small initialized data */
  .sdata : {
    *(.sdata)
  } > RAM

  /* BSS section with explicit symbols */
  .bss : {
    _bss_start = .;  /* Define BSS start symbol */
    *(.bss)
    *(COMMON)
    . = ALIGN(4);
    _bss_end = .;    /* Define BSS end symbol */
  } > RAM

  /* Small BSS section */
  .sbss : {
    _sbss_start = .;
    *(.sbss)
    *(.sbss.*)
    . = ALIGN(4);
    _sbss_end = .;
  } > RAM

  /* End marker for heap start */
  . = ALIGN(4);
  _heap_start = .; /* Heap starts here and grows upwards */

  . = ALIGN(4);
  . += 32 * 1024; /* 32K */
  _stack_top = .;
}


