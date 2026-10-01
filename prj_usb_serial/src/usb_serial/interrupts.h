#pragma once

#include "usbd_core.h"

/* function declarations */
/* this function handles USB wake-up interrupt handler */
void USBFS_WKUP_IRQHandler(void);
/* this function handles USBFS IRQ Handler */
void USBFS_IRQHandler(void);
