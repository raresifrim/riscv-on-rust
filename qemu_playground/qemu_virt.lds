MEMORY {
  program (rwx) : ORIGIN = 0x80200000, LENGTH = 64M
}

SECTIONS { 
  .text : {
    *(.text*)
  } > program

  .rodata : ALIGN(4) {
    *(.rodata .rodata.* .srodata);
  }

  .data : ALIGN(4) {
    *(.data .data.*);
  }

  /* Small initialized data */
  .sdata : {
    *(.sdata)
  } > program

  /* BSS section with explicit symbols */
  .bss : {
    _bss_start = .;  /* Define BSS start symbol */
    *(.bss)
    *(COMMON)
    . = ALIGN(4);
    _bss_end = .;    /* Define BSS end symbol */
  } > program

  /* Small BSS section */
  .sbss : {
    _sbss_start = .;
    *(.sbss)
    *(.sbss.*)
    . = ALIGN(4);
    _sbss_end = .;
  } > program

  /* End marker for heap start */
  . = ALIGN(4);
  _heap_start = .; /* Heap starts here and grows upwards */

  . = ALIGN(4);
  . += 2 * 1024 * 1024; /* 2M */
  _stack_top = .;
}


