#include <cstdint>
#include <unistd.h>

extern "C" {

static void print_hex(uintptr_t val) {
    char buf[10];
    buf[0] = '0';
    buf[1] = 'x';
    for (int i = 7; i >= 0; --i) {
        uint8_t nibble = (val >> (i * 4)) & 0x0F;
        buf[2 + (7 - i)] = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
    }
    write(1, buf, 10);
}

static void print_str(const char *str) {
    size_t len = 0;
    while (str[len]) len++;
    write(1, str, len);
}

__attribute__((weak)) uintptr_t handle_nmi(void) {
    write(1, "\n[NMI]\n", 7);
    _exit(1);
    return 0;
}

__attribute__((weak)) uintptr_t handle_trap(uintptr_t mcause, uintptr_t sp) {
    if ((mcause & 0xFFFU) == 0xFFFU) {
        return handle_nmi();
    }
    uintptr_t mepc = 0, mtval = 0;
    asm volatile("csrr %0, mepc" : "=r"(mepc));
    asm volatile("csrr %0, mtval" : "=r"(mtval));

    print_str("\n[TRAP] mcause=");
    print_hex(mcause);
    print_str(" mepc=");
    print_hex(mepc);
    print_str(" mtval=");
    print_hex(mtval);
    print_str(" sp=");
    print_hex(sp);
    print_str("\n");

    _exit(static_cast<int>(mcause));
    return 0;
}

} // extern "C"
