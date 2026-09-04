#include <cstddef>
#include <cstdint>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include "hal/time.hpp"

extern "C" {

extern char _end[];
extern char _heap_end[];
extern int _write(int file, char *ptr, int len);

__attribute__((used)) void *_sbrk(ptrdiff_t incr) {
    static char *curbrk = _end;

    if ((curbrk + incr < _end) || (curbrk + incr > _heap_end)) {
        return reinterpret_cast<void *>(-1);
    }

    char *prev = curbrk;
    curbrk += incr;
    return prev;
}

__attribute__((used, noreturn)) void _exit(int status) {
    (void)status;
    while (true) {
        asm volatile("wfi");
    }
}

__attribute__((used)) int _close(int fd) {
    (void)fd;
    return -1;
}

__attribute__((used)) int _fstat(int fd, struct stat *st) {
    (void)fd;
    if (st) {
        st->st_mode = S_IFCHR;
    }
    return 0;
}

__attribute__((used)) int _isatty(int fd) {
    (void)fd;
    return 1;
}

__attribute__((used)) off_t _lseek(int fd, off_t ptr, int dir) {
    (void)fd;
    (void)ptr;
    (void)dir;
    return 0;
}

__attribute__((used)) ssize_t _read(int fd, void *ptr, size_t len) {
    (void)fd;
    (void)ptr;
    (void)len;
    return 0;
}

void write_hex(int fd, unsigned long int hex) {
    char prefix[] = "0x";
    _write(fd, prefix, 2);
    for (int i = sizeof(unsigned long int) * 2; i > 0; --i) {
        int shift = (i - 1) * 4;
        uint8_t digit = (hex >> shift) & 0x0F;
        char c = (digit < 10) ? static_cast<char>('0' + digit) : static_cast<char>('A' + (digit - 10));
        _write(fd, &c, 1);
    }
}

// Hardware-backed mtime tick counter (replaces vendor get_timer_value)
__attribute__((used)) uint64_t get_timer_value(void) {
    return hal::time::get_raw_ticks();
}

__attribute__((used)) uint32_t get_timer_freq(void) {
    extern uint32_t SystemCoreClock;
    return SystemCoreClock / 4;
}

}
