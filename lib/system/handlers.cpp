#include <cstdint>
#include <unistd.h>

extern "C" {

__attribute__((weak)) uintptr_t handle_nmi(void) {
    write(1, "nmi\n", 4);
    _exit(1);
    return 0;
}

__attribute__((weak)) uintptr_t handle_trap(uintptr_t mcause, uintptr_t sp) {
    (void)sp;
    if ((mcause & 0xFFFU) == 0xFFFU) {
        return handle_nmi();
    }
    write(1, "trap\n", 5);
    _exit(static_cast<int>(mcause));
    return 0;
}

} // extern "C"
