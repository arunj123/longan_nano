#include <cstdint>
#include "hal/pmu.hpp"
#include "hal/core.hpp"
#include "hal/rcu.hpp"
#include "hal/registers.hpp"
#include "system_gd32vf103.h"

#if defined(__SYSTEM_CLOCK_108M_PLL_HXTAL)
uint32_t SystemCoreClock = 108000000U;
#else
// Default 96 MHz for USB applications (generates exact 48 MHz USB clock)
uint32_t SystemCoreClock = 96000000U;
#endif

uint32_t gd32vf103_firmware_version_get(void) {
    return 0x01050000U; // V1.5.0 baseline
}

static void system_clock_config(void) {
    using namespace hal::reg::rcu;
    using RegCTL = hal::rcu::RegCTL;
    using RegCFG0 = hal::rcu::RegCFG0;
    using RegCFG1 = hal::rcu::RegCFG1;

    constexpr uint32_t HXTAL_STARTUP_TIMEOUT = 0xFFFFU;
    uint32_t timeout = 0;

    // Enable high-speed external crystal oscillator (8 MHz on Sipeed Longan Nano)
    RegCTL::set_bits(CTL_HXTALEN);
    while (((RegCTL::read() & CTL_HXTALSTB) == 0) && (++timeout < HXTAL_STARTUP_TIMEOUT)) {}

    if ((RegCTL::read() & CTL_HXTALSTB) == 0) {
        // HXTAL failed to stabilize: fallback to IRC8M
        SystemCoreClock = 8000000U;
        return;
    }

    // Bus prescalers: AHB = SYSCLK, APB2 = AHB/1, APB1 = AHB/2 (max APB1 is 54 MHz)
    RegCFG0::set_bits(CFG0_AHB_DIV1 | CFG0_APB2_DIV1 | CFG0_APB1_DIV2);

    // PREDV0 = HXTAL (8 MHz) / 2 = 4 MHz
    RegCFG1::write((RegCFG1::read() & ~0x0001FFFFU) | (CFG1_PREDV0SRC_HXTAL | CFG1_PREDV0_DIV2));

#if defined(__SYSTEM_CLOCK_108M_PLL_HXTAL)
    // CK_PLL = 4 MHz * 27 = 108 MHz (MUL27: pllmf = 0b1001 << 18, pllmf4 = 1 << 29)
    RegCFG0::modify((0xFU << 18) | (1U << 29), (1U << 16) | (0x9U << 18) | (1U << 29));
    SystemCoreClock = 108000000U;
#else
    // CK_PLL = 4 MHz * 24 = 96 MHz (default: MUL24: pllmf = 0b0110 << 18, pllmf4 = 1 << 29)
    RegCFG0::modify((0xFU << 18) | (1U << 29), (1U << 16) | (0x6U << 18) | (1U << 29));
    SystemCoreClock = 96000000U;
#endif

    // Enable PLL
    RegCTL::set_bits(CTL_PLLEN);
    while ((RegCTL::read() & CTL_PLLSTB) == 0) {}

    // Switch system clock source to PLL (SCS = 0b10)
    RegCFG0::modify(CFG0_SCS_MASK, CFG0_SCS_PLL);
    while ((RegCFG0::read() & (0x3U << 2)) != CFG0_SCSS_PLL) {}
}

extern "C" void SystemInit(void) {
    using namespace hal::reg::rcu;
    using RegCTL = hal::rcu::RegCTL;
    using RegCFG0 = hal::rcu::RegCFG0;
    using RegCFG1 = hal::rcu::RegCFG1;
    using RegINT = hal::rcu::RegINT;

    // Enable IRC8M internal oscillator
    RegCTL::set_bits(CTL_IRC8MEN);
    while ((RegCTL::read() & CTL_IRC8MSTB) == 0) {}

    // Reset clock configuration registers to default known state
    RegCFG0::clear_bits(0x08FF0FFFU);
    RegCTL::clear_bits(CTL_HXTALEN | CTL_CKMEN | CTL_PLLEN | CTL_HXTALBPS);
    RegCFG0::clear_bits(0x203F0000U);
    RegCFG1::write(0x00000000U);
    RegCTL::clear_bits(CTL_PLLEN | CTL_PLL1EN | CTL_PLL2EN | CTL_CKMEN | CTL_HXTALEN);
    RegINT::write(0x00FF0000U);

    // Configure system clock
    system_clock_config();
}

void SystemCoreClockUpdate(void) {
    using RegCFG0 = hal::rcu::RegCFG0;
    using RegCFG1 = hal::rcu::RegCFG1;

    const uint32_t scss = (RegCFG0::read() >> 2) & 0x3U;
    if (scss == 0) {
        SystemCoreClock = 8000000U; // IRC8M
    } else if (scss == 1) {
        SystemCoreClock = 8000000U; // HXTAL
    } else if (scss == 2) {
        // PLL source
        const uint32_t cfg0 = RegCFG0::read();
        const uint32_t pllmf = (cfg0 >> 18) & 0xFU;
        const uint32_t pllmf4 = (cfg0 & (1U << 29)) ? 16 : 0;
        const uint32_t mul = pllmf + pllmf4 + 2;
        // PREDV0 divider
        const uint32_t predv0 = (RegCFG1::read() & 0xFU) + 1;
        const uint32_t base_clock = 8000000U / predv0;
        SystemCoreClock = base_clock * mul;
    }
}
