#pragma once

#include <cstdint>

/* system clock frequency (core clock) */
extern uint32_t SystemCoreClock;

/* function declarations */
/* initialize the system and update the SystemCoreClock variable (called from start.S) */
extern "C" void SystemInit(void);

/* update the SystemCoreClock with current core clock retrieved from cpu registers */
void SystemCoreClockUpdate(void);

/* get firmware version */
uint32_t gd32vf103_firmware_version_get(void);
