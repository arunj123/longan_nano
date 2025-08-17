
/* This file contains the C-level system initialization routines and fault
 * handlers for the GD32VF103 MCU. It is responsible for preparing the C
 * runtime environment and providing robust debugging handlers for system faults.
 */
#include <gd32vf103.h>
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>
#include "riscv_encoding.h"
#include "n200_func.h"

extern uint32_t disable_mcycle_minstret(void);

/* --- Forward Declarations for Fault Handling Functions --- */
void fault_puts(const char *s);
void fault_puthex(uint32_t h);
void _unassigned_interrupts_handler(void);

/**
 * @brief  System-level initialization routine.
 * @note   This function is called by the startup code in `start.S` immediately
 *         before `main()`. It is responsible for essential low-level hardware
 *         setup required for the C runtime and application to function correctly.
 *         This includes:
 *         - Initializing the system clock via `SystemInit()`.
 *         - Configuring the Core-Local Interrupt Controller (ECLIC).
 *         - Disabling performance counters to conserve power.
 *         - Updating the global `SystemCoreClock` variable.
 */
void _init(void)
{
	SystemInit();

	// Initialize the Core-Local Interrupt Controller (ECLIC)
	eclic_init(ECLIC_NUM_INTERRUPTS);
	eclic_mode_enable();

	// NOTE: The following code is an example of how to configure the PMP
	// (Physical Memory Protection) and switch to user mode. It is disabled
	// by default for this application.
	//pmp_open_all_space();
	//switch_m2u_mode();
	
	// Disable machine cycle and instruction-retired counters by default to save power.
	disable_mcycle_minstret();

	SystemCoreClockUpdate();
}

/**
 * @brief A safe, polled, non-interrupt-driven function to send a string.
 * @note  This function is safe to call from an interrupt or fault handler because
 *        it does not rely on interrupts or complex buffering like printf.
 *        It assumes the corresponding UART has already been initialized.
 * @param s The null-terminated string to print.
 */
void fault_puts(const char *s) {
    if (!s) return;
    while (*s) {
        // Wait until the transmit data register is empty (TBE flag is set)
        while (RESET == usart_flag_get(USART0, USART_FLAG_TBE));
        usart_data_transmit(USART0, (uint8_t)(*s++));
    }
    // Optional: Wait for the last character to be fully transmitted (TC flag)
    while (RESET == usart_flag_get(USART0, USART_FLAG_TC));
}

/**
 * @brief A safe, polled function to print a 32-bit number in hexadecimal.
 * @note  This is safe to call from a fault handler. It assumes the UART
 *        is already initialized.
 * @param h The 32-bit unsigned integer to print.
 */
void fault_puthex(uint32_t h) {
    char hex_chars[] = "0123456789ABCDEF";
    fault_puts("0x");
    // Iterate through each nibble (4 bits) of the 32-bit number, from MSB to LSB
    for (int i = 28; i >= 0; i -= 4) {
        // Isolate the nibble
        uint8_t nibble = (h >> i) & 0xF;
        // Transmit the corresponding ASCII character for the hex digit
        while (RESET == usart_flag_get(USART0, USART_FLAG_TBE));
        usart_data_transmit(USART0, hex_chars[nibble]);
    }
}

/* A global flag to indicate that an unhandled interrupt has occurred. */
/* 'volatile' is crucial to ensure the compiler does not optimize away accesses to it. */
volatile int g_unhandled_interrupt_fired = 0;

/**
 * @brief  C-level default handler for unassigned interrupts.
 * @note   This function is designed to be the target for all weak interrupt
 *         handler symbols that have not been implemented by the user. It
 *         provides a safe and debuggable failure state.
 *
 * @warning Do NOT use complex, non-reentrant, or blocking functions like
 *          `printf` here. An interrupt can occur at any time, including
 *          while the main application is in the middle of using `printf`,
 *          which would lead to a deadlock or state corruption.
 *
 * @note    The `__attribute__((interrupt))` tells the compiler to generate
 *          the special prolog and epilog code required for an ISR. This
 *          includes saving all necessary registers on entry and using the
 *          `mret` instruction on exit. This makes the function a valid
 *          target for the hardware's interrupt vector table.
 */
void __attribute__((interrupt)) _unassigned_interrupts_handler(void)
{
	// Read the mcause register to find out which interrupt occurred.
	// The MSB being 1 indicates an interrupt. The lower bits are the interrupt ID.
	uint32_t cause = read_csr(mcause);

	// Use our safe, polled printing functions to send a detailed message.
	fault_puts("\n\n*** Unhandled Interrupt ***\nCause (mcause): ");
	fault_puthex(cause);
	fault_puts("\nSystem Halted.\n");

	// Set a flag that can be easily observed in a hardware debugger session.
	g_unhandled_interrupt_fired = 1;

	// Halt the processor in a predictable infinite loop. This is a "safe crash"
	// that prevents further execution and preserves the state for debugging.
	while (1) {
		// To debug with a hardware debugger (the best method):
		// 1. Connect GDB and interrupt the execution (Ctrl+C).
		// 2. The Program Counter (PC) will be at this `while(1)` loop.
		// 3. Use `p/x $mcause` in GDB to re-check the cause register.
		// 4. Use `bt` (backtrace) to see the call stack.
		// 5. Inspect peripheral registers (e.g., ECLIC_IP) for more details.
	}
}