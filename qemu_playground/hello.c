extern void print_ecall(char*, unsigned int);
int main(void) {
    char message[] = "Hello from RISC-V UART inside an ecall!\n";
    print_ecall(message, sizeof(message)/sizeof(char)); 
    return 0;
}
