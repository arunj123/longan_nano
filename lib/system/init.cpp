#include <cstdint>
#include <unistd.h>
#include "hal/eclic.hpp"
#include "hal/uart.hpp"
#include "hal/core.hpp"
#include "system_gd32vf103.h"

extern "C" {

extern uint32_t disable_mcycle_minstret(void);
extern void initialise_debug_uart(void);

void fault_puts(const char *s);
void fault_puthex(uint32_t h);
void __attribute__((interrupt, nothrow)) _unassigned_interrupts_handler(void);

#define WEAK_ALIAS(f) __attribute__ ((weak, alias("_unassigned_interrupts_handler"), interrupt, nothrow))

/* Core N200 Interrupts */
void eclic_msip_handler(void)       WEAK_ALIAS(eclic_msip_handler);
void eclic_mtip_handler(void)       WEAK_ALIAS(eclic_mtip_handler);
void eclic_bwei_handler(void)       WEAK_ALIAS(eclic_bwei_handler);
void eclic_pmovi_handler(void)      WEAK_ALIAS(eclic_pmovi_handler);

/* GD32VF103 Peripheral Interrupts */
void WWDGT_IRQHandler(void)         WEAK_ALIAS(WWDGT_IRQHandler);
void LVD_IRQHandler(void)           WEAK_ALIAS(LVD_IRQHandler);
void TAMPER_IRQHandler(void)        WEAK_ALIAS(TAMPER_IRQHandler);
void RTC_IRQHandler(void)           WEAK_ALIAS(RTC_IRQHandler);
void FMC_IRQHandler(void)           WEAK_ALIAS(FMC_IRQHandler);
void RCU_IRQHandler(void)           WEAK_ALIAS(RCU_IRQHandler);
void EXTI0_IRQHandler(void)         WEAK_ALIAS(EXTI0_IRQHandler);
void EXTI1_IRQHandler(void)         WEAK_ALIAS(EXTI1_IRQHandler);
void EXTI2_IRQHandler(void)         WEAK_ALIAS(EXTI2_IRQHandler);
void EXTI3_IRQHandler(void)         WEAK_ALIAS(EXTI3_IRQHandler);
void EXTI4_IRQHandler(void)         WEAK_ALIAS(EXTI4_IRQHandler);
void DMA0_Channel0_IRQHandler(void) WEAK_ALIAS(DMA0_Channel0_IRQHandler);
void DMA0_Channel1_IRQHandler(void) WEAK_ALIAS(DMA0_Channel1_IRQHandler);
void DMA0_Channel2_IRQHandler(void) WEAK_ALIAS(DMA0_Channel2_IRQHandler);
void DMA0_Channel3_IRQHandler(void) WEAK_ALIAS(DMA0_Channel3_IRQHandler);
void DMA0_Channel4_IRQHandler(void) WEAK_ALIAS(DMA0_Channel4_IRQHandler);
void DMA0_Channel5_IRQHandler(void) WEAK_ALIAS(DMA0_Channel5_IRQHandler);
void DMA0_Channel6_IRQHandler(void) WEAK_ALIAS(DMA0_Channel6_IRQHandler);
void ADC0_1_IRQHandler(void)        WEAK_ALIAS(ADC0_1_IRQHandler);
void CAN0_TX_IRQHandler(void)       WEAK_ALIAS(CAN0_TX_IRQHandler);
void CAN0_RX0_IRQHandler(void)      WEAK_ALIAS(CAN0_RX0_IRQHandler);
void CAN0_RX1_IRQHandler(void)      WEAK_ALIAS(CAN0_RX1_IRQHandler);
void CAN0_EWMC_IRQHandler(void)     WEAK_ALIAS(CAN0_EWMC_IRQHandler);
void EXTI5_9_IRQHandler(void)       WEAK_ALIAS(EXTI5_9_IRQHandler);
void TIMER0_BRK_IRQHandler(void)    WEAK_ALIAS(TIMER0_BRK_IRQHandler);
void TIMER0_UP_IRQHandler(void)     WEAK_ALIAS(TIMER0_UP_IRQHandler);
void TIMER0_TRG_CMT_IRQHandler(void) WEAK_ALIAS(TIMER0_TRG_CMT_IRQHandler);
void TIMER0_Channel_IRQHandler(void) WEAK_ALIAS(TIMER0_Channel_IRQHandler);
void TIMER1_IRQHandler(void)        WEAK_ALIAS(TIMER1_IRQHandler);
void TIMER2_IRQHandler(void)        WEAK_ALIAS(TIMER2_IRQHandler);
void TIMER3_IRQHandler(void)        WEAK_ALIAS(TIMER3_IRQHandler);
void I2C0_EV_IRQHandler(void)       WEAK_ALIAS(I2C0_EV_IRQHandler);
void I2C0_ER_IRQHandler(void)       WEAK_ALIAS(I2C0_ER_IRQHandler);
void I2C1_EV_IRQHandler(void)       WEAK_ALIAS(I2C1_EV_IRQHandler);
void I2C1_ER_IRQHandler(void)       WEAK_ALIAS(I2C1_ER_IRQHandler);
void SPI0_IRQHandler(void)          WEAK_ALIAS(SPI0_IRQHandler);
void SPI1_IRQHandler(void)          WEAK_ALIAS(SPI1_IRQHandler);
void USART0_IRQHandler(void)        WEAK_ALIAS(USART0_IRQHandler);
void USART1_IRQHandler(void)        WEAK_ALIAS(USART1_IRQHandler);
void USART2_IRQHandler(void)        WEAK_ALIAS(USART2_IRQHandler);
void EXTI10_15_IRQHandler(void)     WEAK_ALIAS(EXTI10_15_IRQHandler);
void RTC_Alarm_IRQHandler(void)     WEAK_ALIAS(RTC_Alarm_IRQHandler);
void USBFS_WKUP_IRQHandler(void)    WEAK_ALIAS(USBFS_WKUP_IRQHandler);
void TIMER4_IRQHandler(void)        WEAK_ALIAS(TIMER4_IRQHandler);
void SPI2_IRQHandler(void)          WEAK_ALIAS(SPI2_IRQHandler);
void UART3_IRQHandler(void)         WEAK_ALIAS(UART3_IRQHandler);
void UART4_IRQHandler(void)         WEAK_ALIAS(UART4_IRQHandler);
void TIMER5_IRQHandler(void)        WEAK_ALIAS(TIMER5_IRQHandler);
void TIMER6_IRQHandler(void)        WEAK_ALIAS(TIMER6_IRQHandler);
void DMA1_Channel0_IRQHandler(void) WEAK_ALIAS(DMA1_Channel0_IRQHandler);
void DMA1_Channel1_IRQHandler(void) WEAK_ALIAS(DMA1_Channel1_IRQHandler);
void DMA1_Channel2_IRQHandler(void) WEAK_ALIAS(DMA1_Channel2_IRQHandler);
void DMA1_Channel3_IRQHandler(void) WEAK_ALIAS(DMA1_Channel3_IRQHandler);
void DMA1_Channel4_IRQHandler(void) WEAK_ALIAS(DMA1_Channel4_IRQHandler);
void CAN1_TX_IRQHandler(void)       WEAK_ALIAS(CAN1_TX_IRQHandler);
void CAN1_RX0_IRQHandler(void)      WEAK_ALIAS(CAN1_RX0_IRQHandler);
void CAN1_RX1_IRQHandler(void)      WEAK_ALIAS(CAN1_RX1_IRQHandler);
void CAN1_EWMC_IRQHandler(void)     WEAK_ALIAS(CAN1_EWMC_IRQHandler);
void USBFS_IRQHandler(void)         WEAK_ALIAS(USBFS_IRQHandler);

void _init(void) {
    hal::eclic::Eclic::init();
    hal::eclic::Eclic::enable_mode();

    SystemCoreClockUpdate();
    initialise_debug_uart();
}

void fault_puts(const char *s) {
    if (!s) return;
    hal::uart::Uart0::print(s);
}

void fault_puthex(uint32_t h) {
    const char hex_chars[] = "0123456789ABCDEF";
    fault_puts("0x");
    for (int i = 28; i >= 0; i -= 4) {
        uint8_t nibble = static_cast<uint8_t>((h >> i) & 0xFU);
        hal::uart::Uart0::putc(hex_chars[nibble]);
    }
}

volatile int g_unhandled_interrupt_fired = 0;

void __attribute__((interrupt, nothrow)) _unassigned_interrupts_handler(void) {
    uint32_t cause = hal::csr::read<hal::csr::Csr::Mcause>();

    fault_puts("\n\n*** Unhandled Interrupt ***\nCause (mcause): ");
    fault_puthex(cause);
    fault_puts("\nSystem Halted.\n");

    g_unhandled_interrupt_fired = 1;

    while (1) {
    }
}

} // extern "C"
