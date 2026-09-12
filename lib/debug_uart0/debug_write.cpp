#include <cstdint>
#include <cstddef>
#include "hal/uart.hpp"
#include "hal/eclic.hpp"

extern "C" {
int _write(int file, char *ptr, int len);
void initialise_debug_uart(void);
void enable_debug_uart_interrupt(void);
void USART0_IRQHandler(void);
}

namespace {

constexpr size_t kTxBufSize = 2048;
static_assert((kTxBufSize & (kTxBufSize - 1)) == 0, "Buffer size must be a power of 2");

alignas(4) volatile uint8_t s_tx_buf[kTxBufSize];
volatile uint32_t s_tx_head = 0;
volatile uint32_t s_tx_tail = 0;
volatile bool s_async_enabled = false;

static inline bool is_interrupts_enabled() {
    uint32_t mstatus;
    asm volatile("csrr %0, mstatus" : "=r"(mstatus));
    return (mstatus & (1U << 3)) != 0; // MIE bit
}

} // namespace

void initialise_debug_uart(void) {
    hal::uart::Uart0::init(115200);
}

void enable_debug_uart_interrupt(void) {
    hal::eclic::Eclic::enable(hal::eclic::Irq::Usart0, 1, 1, false);
    s_async_enabled = true;
}

extern "C" void USART0_IRQHandler(void) {
    using Uart = hal::uart::Uart0;
    uint32_t stat = Uart::RegSTAT::read();
    uint32_t ctl0 = Uart::RegCTL0::read();

    if ((stat & Uart::STAT_TBE) && (ctl0 & Uart::CTL0_TBEIE)) {
        uint32_t tail = s_tx_tail;
        if (tail != s_tx_head) {
            Uart::RegDATA::write(s_tx_buf[tail]);
            s_tx_tail = (tail + 1) & (kTxBufSize - 1);
        } else {
            Uart::RegCTL0::clear_bits(Uart::CTL0_TBEIE);
        }
    }
}

__attribute__((used)) int _write(int file, char *ptr, int len) {
    (void)file;
    if (!ptr || len <= 0) return 0;

    using Uart = hal::uart::Uart0;

    if (!s_async_enabled || !is_interrupts_enabled()) {
        for (int i = 0; i < len; ++i) {
            Uart::putc(ptr[i]);
        }
        return len;
    }

    for (int i = 0; i < len; ++i) {
        uint32_t next_head = (s_tx_head + 1) & (kTxBufSize - 1);
        if (next_head == s_tx_tail) {
            // Buffer full: drop remaining bytes to avoid blocking real-time USB loop
            break;
        }
        s_tx_buf[s_tx_head] = static_cast<uint8_t>(ptr[i]);
        s_tx_head = next_head;
    }

    Uart::RegCTL0::set_bits(Uart::CTL0_TBEIE);
    return len;
}