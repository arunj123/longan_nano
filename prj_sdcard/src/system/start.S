/*
 * -----------------------------------------------------------------------------
 * -- FILE: start.S
 * -----------------------------------------------------------------------------
 * --
 * -- DESCRIPTION:
 * -- MCU startup file for the GD32VF103. This is the first code executed
 * -- after a processor reset.
 * --
 * -- RESPONSIBILITIES:
 * --   - Provides the interrupt vector table structure.
 * --   - Sets up core CSRs for exception and interrupt handling (ECLIC).
 * --   - Initializes the stack pointer (sp) and global pointer (gp).
 * --   - Initializes the C runtime environment by:
 * --     - Copying the .data section from Flash to SRAM.
 * --     - Clearing the .bss section in SRAM.
 * --   - Calls the application's main() function.
 * --
 * -- NOTE: The default implementations for the interrupt handlers are now
 * --       provided via weak aliases in `init.c`. This file only contains
 * --       the vector table itself.
 * -----------------------------------------------------------------------------
 */

#include "riscv_encoding.h"

// =============================================================================
// SECTION: .init
// Contains the startup code and interrupt vector table.
// =============================================================================
    .section .init
    .align 2

// =============================================================================
// Interrupt Vector Table for ECLIC
// =============================================================================
// =============================================================================
// Interrupt Vector Table for ECLIC
//
// This table contains the addresses of the interrupt service routines (ISRs).
// The ECLIC uses this table in vectored mode to directly jump to the
// corresponding ISR when an interrupt occurs.
// =============================================================================
.global vector_base
vector_base:
    // Entry 0: Reset Vector. Jumps to the startup code entry point.
    j _start
    .align 2
    
    // Core Interrupts (N200 Specific)
    .word     0                                  // Reserved
    .word     0                                  // Reserved
    .word     eclic_msip_handler                 // IRQ 3: Machine Software Interrupt
    .word     0, 0, 0                            // Reserved
    .word     eclic_mtip_handler                 // IRQ 7: Machine Timer Interrupt
    .word     0, 0, 0, 0, 0, 0, 0, 0, 0          // Reserved
    .word     eclic_bwei_handler                 // IRQ 17: Bus Error/Write Error
    .word     eclic_pmovi_handler                // IRQ 18: Performance Monitor Overflow
    
    // Peripheral Interrupts (GD32VF103 Specific)
    .word     WWDGT_IRQHandler                   // IRQ 19: Window Watchdog Timer
    .word     LVD_IRQHandler                     // IRQ 20: Low Voltage Detector
    .word     TAMPER_IRQHandler                  // IRQ 21: Tamper
    .word     RTC_IRQHandler                     // IRQ 22: RTC global
    .word     FMC_IRQHandler                     // IRQ 23: FMC global
    .word     RCU_IRQHandler                     // IRQ 24: RCU global
    .word     EXTI0_IRQHandler                   // IRQ 25: EXTI Line 0
    .word     EXTI1_IRQHandler                   // IRQ 26: EXTI Line 1
    .word     EXTI2_IRQHandler                   // IRQ 27: EXTI Line 2
    .word     EXTI3_IRQHandler                   // IRQ 28: EXTI Line 3
    .word     EXTI4_IRQHandler                   // IRQ 29: EXTI Line 4
    .word     DMA0_Channel0_IRQHandler           // IRQ 30: DMA0 Channel 0
    .word     DMA0_Channel1_IRQHandler           // IRQ 31: DMA0 Channel 1
    .word     DMA0_Channel2_IRQHandler           // IRQ 32: DMA0 Channel 2
    .word     DMA0_Channel3_IRQHandler           // IRQ 33: DMA0 Channel 3
    .word     DMA0_Channel4_IRQHandler           // IRQ 34: DMA0 Channel 4
    .word     DMA0_Channel5_IRQHandler           // IRQ 35: DMA0 Channel 5
    .word     DMA0_Channel6_IRQHandler           // IRQ 36: DMA0 Channel 6
    .word     ADC0_1_IRQHandler                  // IRQ 37: ADC0 and ADC1 global
    .word     CAN0_TX_IRQHandler                 // IRQ 38: CAN0 TX
    .word     CAN0_RX0_IRQHandler                // IRQ 39: CAN0 RX0
    .word     CAN0_RX1_IRQHandler                // IRQ 40: CAN0 RX1
    .word     CAN0_EWMC_IRQHandler               // IRQ 41: CAN0 EWMC
    .word     EXTI5_9_IRQHandler                 // IRQ 42: EXTI Line 9..5
    .word     TIMER0_BRK_IRQHandler              // IRQ 43: TIMER0 Break
    .word     TIMER0_UP_IRQHandler               // IRQ 44: TIMER0 Update
    .word     TIMER0_TRG_CMT_IRQHandler          // IRQ 45: TIMER0 Trigger and Commutation
    .word     TIMER0_Channel_IRQHandler          // IRQ 46: TIMER0 Channel Capture/Compare
    .word     TIMER1_IRQHandler                  // IRQ 47: TIMER1
    .word     TIMER2_IRQHandler                  // IRQ 48: TIMER2
    .word     TIMER3_IRQHandler                  // IRQ 49: TIMER3
    .word     I2C0_EV_IRQHandler                 // IRQ 50: I2C0 Event
    .word     I2C0_ER_IRQHandler                 // IRQ 51: I2C0 Error
    .word     I2C1_EV_IRQHandler                 // IRQ 52: I2C1 Event
    .word     I2C1_ER_IRQHandler                 // IRQ 53: I2C1 Error
    .word     SPI0_IRQHandler                    // IRQ 54: SPI0
    .word     SPI1_IRQHandler                    // IRQ 55: SPI1
    .word     USART0_IRQHandler                  // IRQ 56: USART0
    .word     USART1_IRQHandler                  // IRQ 57: USART1
    .word     USART2_IRQHandler                  // IRQ 58: USART2
    .word     EXTI10_15_IRQHandler               // IRQ 59: EXTI Line 15..10
    .word     RTC_Alarm_IRQHandler               // IRQ 60: RTC Alarm through EXTI Line
    .word     USBFS_WKUP_IRQHandler              // IRQ 61: USBFS Wakeup from suspend
    .word     0, 0, 0, 0, 0, 0, 0                // Reserved
    .word     TIMER4_IRQHandler                  // IRQ 69: TIMER4
    .word     SPI2_IRQHandler                    // IRQ 70: SPI2
    .word     UART3_IRQHandler                   // IRQ 71: UART3
    .word     UART4_IRQHandler                   // IRQ 72: UART4
    .word     TIMER5_IRQHandler                  // IRQ 73: TIMER5
    .word     TIMER6_IRQHandler                  // IRQ 74: TIMER6
    .word     DMA1_Channel0_IRQHandler           // IRQ 75: DMA1 Channel 0
    .word     DMA1_Channel1_IRQHandler           // IRQ 76: DMA1 Channel 1
    .word     DMA1_Channel2_IRQHandler           // IRQ 77: DMA1 Channel 2
    .word     DMA1_Channel3_IRQHandler           // IRQ 78: DMA1 Channel 3
    .word     DMA1_Channel4_IRQHandler           // IRQ 79: DMA1 Channel 4
    .word     0, 0                               // Reserved
    .word     CAN1_TX_IRQHandler                 // IRQ 82: CAN1 TX
    .word     CAN1_RX0_IRQHandler                // IRQ 83: CAN1 RX0
    .word     CAN1_RX1_IRQHandler                // IRQ 84: CAN1 RX1
    .word     CAN1_EWMC_IRQHandler               // IRQ 85: CAN1 EWMC
    .word     USBFS_IRQHandler                   // IRQ 86: USBFS global

// =============================================================================
// Startup Code Entry Point (_start)
// =============================================================================
	.globl _start
	.type _start,@function
_start:

	// Globally disable machine interrupts until the system is initialized.
	csrc CSR_MSTATUS, MSTATUS_MIE

	/*
	 * Ensure the code runs from the correct physical address. The GD32VF103
	 * main flash is mapped at 0x08000000. This code snippet ensures that if
	 * the program counter (PC) is in a different memory region (e.g., an
	 * alias), it jumps to the correct absolute address in the flash region.
	 */
    la		a0,	_start
    li		a1,	1
	slli	a1,	a1, 29      // a1 = 0x20000000
    bleu	a1, a0, _start_flash_address // If PC >= 0x20000000, jump
    srli	a1,	a1, 2       // a1 = 0x08000000
    bleu	a1, a0, _start_flash_address // If PC >= 0x08000000, jump
    la		a0,	_start_flash_address
    add		a0, a0, a1      // a0 = _start_flash_address + 0x08000000
	jr      a0

_start_flash_address:

    // --- Initialize Core Control and Status Registers (CSRs) ---

    /*
     * Configure MMISC_CTL (Machine Miscellaneous Control), a Nuclei-specific CSR.
     * Setting bit 9 (0x200) makes the NMI entry point share the same base
     * address as exceptions, which is set in MTVEC.
     */
    li t0, 0x200
    csrs CSR_MMISC_CTL, t0

	/*
	 * Set MTVT (Machine Trap Vector Table) to point to our vector_base table.
	 * This register holds the base address for vectored interrupt handlers.
	 */
    la t0, vector_base
    csrw CSR_MTVT, t0

	/*
	 * Set MTVT2 (Machine Trap Vector Table 2) to point to the common IRQ entry.
	 * This is used for non-vectored interrupt handling. Setting bit 0 enables it.
	 */
    la t0, irq_entry
    csrw CSR_MTVT2, t0
    csrs CSR_MTVT2, 0x1 // Set enable bit

    /*
     * Set MTVEC (Machine Trap Vector Base Address) for exceptions (traps)
     * and NMIs. This points to the non-vectored trap handler.
     */
    la t0, trap_entry
    csrw CSR_MTVEC, t0

    // --- Initialize Floating-Point Unit (FPU) if present ---
#ifdef __riscv_flen
	/*
	 * Enable the FPU by setting the FS (Floating-point Status) bits
	 * in the mstatus register to 'Initial' (0b01).
	 * Also, clear the fcsr (Floating-Point Control and Status Register).
	 */
	li t0, MSTATUS_FS
	csrs mstatus, t0
	csrw fcsr, x0
#endif

    // --- Initialize Global and Stack Pointers ---
.option push
.option norelax
	// Set the global pointer (gp) register. Required for accessing global variables.
	// The __global_pointer$ symbol is defined by the linker.
	la gp, __global_pointer$
.option pop
	// Set the stack pointer (sp). The _sp symbol is defined in the linker script
	// to be at the top of the allocated RAM for the stack.
	la sp, _sp

	// --- Initialize C Runtime Memory (.data and .bss) ---

	/*
	 * Copy the .data section from its Load Memory Address (LMA) in Flash
	 * to its Virtual Memory Address (VMA) in SRAM.
	 */
	la a0, _data_lma  // Source address in Flash
	la a1, _data      // Destination address in SRAM
	la a2, _edata     // End address in SRAM
	bgeu a1, a2, data_init_done
data_init_loop:
	lw t0, (a0)
	sw t0, (a1)
	addi a0, a0, 4
	addi a1, a1, 4
	bltu a1, a2, data_init_loop
data_init_done:

	/*
	 * Clear the .bss section in SRAM. This section contains uninitialized
	 * global and static variables, which must be zeroed before main() is called.
	 */
	la a0, __bss_start // Start of .bss
	la a1, _end        // End of .bss
	bgeu a0, a1, bss_clear_done
bss_clear_loop:
	sw zero, (a0)      // Write zero to memory
	addi a0, a0, 4
	bltu a0, a1, bss_clear_loop
bss_clear_done:

	/*
	 * Enable the machine cycle and instruction-retired counters.
	 * MCOUNTINHIBIT CSR: bit 0 inhibits mcycle, bit 2 inhibits minstret.
	 * 0x5 is 0b101. Clearing these bits enables the counters.
	 */
    csrci CSR_MCOUNTINHIBIT, 0x5

	// --- Call C++ / System Init Functions ---
	call __libc_init_array // Call global constructors for C++
    call _init              // Call any system-level init function

	// --- Call main() ---
	// Set argc (a0) and argv (a1) to 0, as is standard for embedded systems.
	li a0, 0
	li a1, 0
	call main

	// --- Post-main() ---
	// If main returns, call exit, which should ideally not return.
	// `tail` is an optimization to turn the `call` into a `jump`.
	tail exit

// A fallback infinite loop in case `exit` returns or something goes wrong.
halt_loop:
	j halt_loop

// =============================================================================
// Helper Functions to control performance counters
// =============================================================================
	.global disable_mcycle_minstret
disable_mcycle_minstret:
        csrsi CSR_MCOUNTINHIBIT, 0x5 // Set bits to disable counters
	ret

	.global enable_mcycle_minstret
enable_mcycle_minstret:
        csrci CSR_MCOUNTINHIBIT, 0x5 // Clear bits to enable counters
	ret