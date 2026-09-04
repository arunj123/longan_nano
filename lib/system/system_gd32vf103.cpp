#include <cstdint>
#include "hal/pmu.hpp"
#include "hal/core.hpp"

extern "C" {
#include "gd32vf103.h"

#define HXTAL_STARTUP_TIMEOUT   ((uint16_t)0xFFFF)

#if defined(__SYSTEM_CLOCK_108M_PLL_HXTAL)
uint32_t SystemCoreClock = 108000000U;
#else
// Default 96 MHz for USB applications (generates exact 48 MHz USB clock)
uint32_t SystemCoreClock = 96000000U;
#endif

uint32_t gd32vf103_firmware_version_get(void) {
    return 0x01050000U; // V1.5.0
}

void pmu_to_deepsleepmode(uint32_t ldo, uint8_t deepsleepmodecmd) {
    const auto ldo_mode = (ldo != 0) ? hal::pmu::LdoMode::LowPower : hal::pmu::LdoMode::Normal;
    const auto cmd = (deepsleepmodecmd == 0) ? hal::pmu::SleepCommand::Wfi : hal::pmu::SleepCommand::Wfe;
    hal::pmu::Pmu::to_deepsleep_mode(ldo_mode, cmd);
}

static void system_clock_config(void) {
    uint32_t timeout = 0;

    // Enable high-speed external crystal oscillator (8 MHz on Sipeed Longan Nano)
    RCU_CTL |= RCU_CTL_HXTALEN;
    while (((RCU_CTL & RCU_CTL_HXTALSTB) == 0) && (++timeout < HXTAL_STARTUP_TIMEOUT)) {}

    if ((RCU_CTL & RCU_CTL_HXTALSTB) == 0) {
        // HXTAL failed to stabilize: fallback to IRC8M
        SystemCoreClock = 8000000U;
        return;
    }

    // Bus prescalers: AHB = SYSCLK, APB2 = AHB/1, APB1 = AHB/2 (max APB1 is 54 MHz)
    RCU_CFG0 |= RCU_AHB_CKSYS_DIV1 | RCU_APB2_CKAHB_DIV1 | RCU_APB1_CKAHB_DIV2;

    // PREDV0 = HXTAL (8 MHz) / 2 = 4 MHz
    RCU_CFG1 &= ~(RCU_CFG1_PREDV0SEL | RCU_CFG1_PREDV1 | RCU_CFG1_PLL1MF | RCU_CFG1_PREDV0);
    RCU_CFG1 |= (RCU_PREDV0SRC_HXTAL | RCU_PREDV0_DIV2);

#if defined(__SYSTEM_CLOCK_108M_PLL_HXTAL)
    // CK_PLL = 4 MHz * 27 = 108 MHz
    RCU_CFG0 &= ~(RCU_CFG0_PLLMF | RCU_CFG0_PLLMF_4);
    RCU_CFG0 |= (RCU_PLLSRC_HXTAL | RCU_PLL_MUL27);
    SystemCoreClock = 108000000U;
#else
    // CK_PLL = 4 MHz * 24 = 96 MHz (default, exact 48 MHz for USBFS)
    RCU_CFG0 &= ~(RCU_CFG0_PLLMF | RCU_CFG0_PLLMF_4);
    RCU_CFG0 |= (RCU_PLLSRC_HXTAL | RCU_PLL_MUL24);
    SystemCoreClock = 96000000U;
#endif

    // Enable PLL
    RCU_CTL |= RCU_CTL_PLLEN;
    while ((RCU_CTL & RCU_CTL_PLLSTB) == 0) {}

    // Switch system clock source to PLL
    RCU_CFG0 = (RCU_CFG0 & ~RCU_CFG0_SCS) | RCU_CKSYSSRC_PLL;
    while ((RCU_CFG0 & RCU_SCSS_PLL) == 0) {}
}

void SystemInit(void) {
    // Enable IRC8M internal oscillator
    RCU_CTL |= RCU_CTL_IRC8MEN;
    while ((RCU_CTL & RCU_CTL_IRC8MSTB) == 0) {}

    // Reset clock configuration registers to default known state
    RCU_CFG0 &= ~(RCU_CFG0_SCS | RCU_CFG0_AHBPSC | RCU_CFG0_APB1PSC | RCU_CFG0_APB2PSC |
                  RCU_CFG0_ADCPSC | RCU_CFG0_ADCPSC_2 | RCU_CFG0_CKOUT0SEL);
    RCU_CTL &= ~(RCU_CTL_HXTALEN | RCU_CTL_CKMEN | RCU_CTL_PLLEN | RCU_CTL_HXTALBPS);
    RCU_CFG0 &= ~(RCU_CFG0_PLLSEL | RCU_CFG0_PREDV0_LSB | RCU_CFG0_PLLMF |
                  RCU_CFG0_USBFSPSC | RCU_CFG0_PLLMF_4);
    RCU_CFG1 = 0x00000000U;
    RCU_CTL &= ~(RCU_CTL_PLLEN | RCU_CTL_PLL1EN | RCU_CTL_PLL2EN | RCU_CTL_CKMEN | RCU_CTL_HXTALEN);
    RCU_INT = 0x00FF0000U;

    // Configure system clock
    system_clock_config();
}

void SystemCoreClockUpdate(void) {
    const uint32_t scss = GET_BITS(RCU_CFG0, 2, 3);
    if (scss == 0) {
        SystemCoreClock = 8000000U; // IRC8M
    } else if (scss == 1) {
        SystemCoreClock = 8000000U; // HXTAL
    } else if (scss == 2) {
        // PLL source
        const uint32_t pllmf = (RCU_CFG0 & RCU_CFG0_PLLMF) >> 18;
        const uint32_t pllmf4 = (RCU_CFG0 & RCU_CFG0_PLLMF_4) ? 16 : 0;
        const uint32_t mul = pllmf + pllmf4 + 2;
        // PREDV0 divider
        const uint32_t predv0 = (RCU_CFG1 & RCU_CFG1_PREDV0) + 1;
        const uint32_t base_clock = 8000000U / predv0;
        SystemCoreClock = base_clock * mul;
    }
}

} // extern "C"
