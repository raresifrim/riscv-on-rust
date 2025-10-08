
#define TARGET_MICROBLAZE 1

#define UART0_BASE 0x10000000 //for qemu virt machine 
#define UARTLITE_BASE 0x40600000 //for amd-microblaze-v
#define UARTLITE_TX_OFFSET 0x4   //for amd-microblaze-v
#define UARTLITE_RX_OFFSET 0x0   //for amd-microblaze-v
#define UART0_TxRxFIFO0 ((unsigned int *) (UART0_BASE + 0x0)) //for qemu virt machine
#define UARTLITE_TxFIFO ((unsigned int *) (UARTLITE_BASE + UARTLITE_TX_OFFSET)) //for amd-microblaze-v
#define UARTLITE_RxFIFO ((unsigned int *) (UARTLITE_BASE + UARTLITE_RX_OFFSET)) //for amd-microblaze-v

#if TARGET_MICROBLAZE == 1
  volatile unsigned int * const TX_UART = UARTLITE_TxFIFO;
#else
  volatile unsigned int * const TX_UART = UART0_TxRxFIFO0;
#endif

void print_uart(const char *s) 
{
   while(*s != '\0') {     /* Loop until end of string */
    *TX_UART = (unsigned int)(*s); /* Transmit char */
    s++; /* Next char */
  }
}

int main(void)
{ 
  print_uart("Hello World!\n");
  return 0;
}
