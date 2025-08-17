/*
 * -----------------------------------------------------------------------------
 * -- FILE: entry.S
 * -----------------------------------------------------------------------------
 * --
 * -- DESCRIPTION:
 * -- Low-level entry points for exceptions (traps) and interrupts for the
 * -- GD32VF103 MCU, which uses a Nuclei N200 series RISC-V core.
 * --
 * -- This file provides the assembly-level "trampoline" code that:
 * --   1. Saves the CPU context (registers) upon entering an exception or ISR.
 * --   2. Calls the appropriate high-level C/C++ handler.
 * --   3. Restores the CPU context before returning to the interrupted code.
 * --
 * -----------------------------------------------------------------------------
 */

#include "riscv_encoding.h"
#include "riscv_bits.h"
#include "n200_eclic.h"

#ifndef ENTRY_S
#define ENTRY_S

// =============================================================================
// MACRO: DISABLE_MIE
// Disables Machine-level Interrupt Enable (MIE) bit in the mstatus CSR.
// This globally masks all machine-level interrupts.
// =============================================================================
.macro DISABLE_MIE
  csrc CSR_MSTATUS, MSTATUS_MIE
.endm


/*
 * =============================================================================
 * MACRO: SAVE_CONTEXT
 *
 * Saves all caller-saved general-purpose and floating-point registers to the
 * current stack, as defined by the RISC-V calling convention. This is
 * essential to preserve the state of the code that was interrupted.
 * =============================================================================
 */
.macro SAVE_CONTEXT
    /*
     * Allocate stack space for all registers that need to be saved.
     * The RISC-V ABI defines registers t0-t6 and a0-a7 as "caller-saved",
     * meaning an interrupt handler must preserve their values.
     * We also save `ra` (x1) and `tp` (x4).
     */
#ifdef __riscv_flen
  #if (__riscv_flen==64)
    // Allocate space for 20 GP registers and 20 FP registers for RV64D
    addi sp, sp, -20*REGBYTES - 20*FPREGBYTES
  #else
    // Allocate space for 20 GP registers (FP extension not handled here for RV32F)
    addi sp, sp, -20*REGBYTES
  #endif
#else
  // Allocate space for 20 GP registers
  addi sp, sp, -20*REGBYTES
#endif

  // Save caller-saved general-purpose registers
  // x1 (ra), x4 (tp), x5-x7 (t0-t2), x10-x15 (a0-a5)
  // x16-x17 (a6-a7), x28-x31 (t3-t6)
	STORE x1, 0*REGBYTES(sp)
	STORE x4, 1*REGBYTES(sp)
	STORE x5, 2*REGBYTES(sp)
	STORE x6, 3*REGBYTES(sp)
	STORE x7, 4*REGBYTES(sp)
	STORE x10, 5*REGBYTES(sp)
	STORE x11, 6*REGBYTES(sp)
	STORE x12, 7*REGBYTES(sp)
	STORE x13, 8*REGBYTES(sp)
	STORE x14, 9*REGBYTES(sp)
	STORE x15, 10*REGBYTES(sp)
#ifndef __riscv_32e // RV32E only has 16 registers (x0-x15)
  STORE x16, 11*REGBYTES(sp)
	STORE x17, 12*REGBYTES(sp)
	STORE x28, 13*REGBYTES(sp)
	STORE x29, 14*REGBYTES(sp)
	STORE x30, 15*REGBYTES(sp)
	STORE x31, 16*REGBYTES(sp)
#endif

  // Save caller-saved floating-point registers if FPU extension is present
#ifdef __riscv_flen
  #if (__riscv_flen == 64)
    // f0-f7 (ft0-ft7), f10-f17 (fa0-fa7), f28-f31 (ft8-ft11)
    FPSTORE f0, (20*REGBYTES + 0*FPREGBYTES)(sp)
    FPSTORE f1, (20*REGBYTES + 1*FPREGBYTES)(sp)
    FPSTORE f2, (20*REGBYTES + 2*FPREGBYTES)(sp)
    FPSTORE f3, (20*REGBYTES + 3*FPREGBYTES)(sp)
    FPSTORE f4, (20*REGBYTES + 4*FPREGBYTES)(sp)
    FPSTORE f5, (20*REGBYTES + 5*FPREGBYTES)(sp)
    FPSTORE f6, (20*REGBYTES + 6*FPREGBYTES)(sp)
    FPSTORE f7, (20*REGBYTES + 7*FPREGBYTES)(sp)
    FPSTORE f10, (20*REGBYTES + 8*FPREGBYTES)(sp)
    FPSTORE f11, (20*REGBYTES + 9*FPREGBYTES)(sp)
    FPSTORE f12, (20*REGBYTES + 10*FPREGBYTES)(sp)
    FPSTORE f13, (20*REGBYTES + 11*FPREGBYTES)(sp)
    FPSTORE f14, (20*REGBYTES + 12*FPREGBYTES)(sp)
    FPSTORE f15, (20*REGBYTES + 13*FPREGBYTES)(sp)
    FPSTORE f16, (20*REGBYTES + 14*FPREGBYTES)(sp)
    FPSTORE f17, (20*REGBYTES + 15*FPREGBYTES)(sp)
    FPSTORE f28, (20*REGBYTES + 16*FPREGBYTES)(sp)
    FPSTORE f29, (20*REGBYTES + 17*FPREGBYTES)(sp)
    FPSTORE f30, (20*REGBYTES + 18*FPREGBYTES)(sp)
    FPSTORE f31, (20*REGBYTES + 19*FPREGBYTES)(sp)
  #endif
#endif
.endm


/*
 * =============================================================================
 * MACRO: RESTORE_CONTEXT
 *
 * Restores all registers that were saved by the SAVE_CONTEXT macro.
 * =============================================================================
 */
.macro RESTORE_CONTEXT
  // Restore general-purpose registers from the stack
  LOAD x1, 0*REGBYTES(sp)
	LOAD x4, 1*REGBYTES(sp)
	LOAD x5, 2*REGBYTES(sp)
	LOAD x6, 3*REGBYTES(sp)
	LOAD x7, 4*REGBYTES(sp)
	LOAD x10, 5*REGBYTES(sp)
	LOAD x11, 6*REGBYTES(sp)
	LOAD x12, 7*REGBYTES(sp)
	LOAD x13, 8*REGBYTES(sp)
	LOAD x14, 9*REGBYTES(sp)
	LOAD x15, 10*REGBYTES(sp)
#ifndef __riscv_32e
  LOAD x16, 11*REGBYTES(sp)
	LOAD x17, 12*REGBYTES(sp)
	LOAD x28, 13*REGBYTES(sp)
	LOAD x29, 14*REGBYTES(sp)
	LOAD x30, 15*REGBYTES(sp)
	LOAD x31, 16*REGBYTES(sp)
#endif

  // Restore floating-point registers
#ifdef __riscv_flen
  #if (__riscv_flen==64)
	FPLOAD f0, (20*REGBYTES + 0*FPREGBYTES)(sp)
	FPLOAD f1, (20*REGBYTES + 1*FPREGBYTES)(sp)
	FPLOAD f2, (20*REGBYTES + 2*FPREGBYTES)(sp)
	FPLOAD f3, (20*REGBYTES + 3*FPREGBYTES)(sp)
	FPLOAD f4, (20*REGBYTES + 4*FPREGBYTES)(sp)
	FPLOAD f5, (20*REGBYTES + 5*FPREGBYTES)(sp)
	FPLOAD f6, (20*REGBYTES + 6*FPREGBYTES)(sp)
	FPLOAD f7, (20*REGBYTES + 7*FPREGBYTES)(sp)
	FPLOAD f10, (20*REGBYTES + 8*FPREGBYTES)(sp)
	FPLOAD f11, (20*REGBYTES + 9*FPREGBYTES)(sp)
	FPLOAD f12, (20*REGBYTES + 10*FPREGBYTES)(sp)
	FPLOAD f13, (20*REGBYTES + 11*FPREGBYTES)(sp)
	FPLOAD f14, (20*REGBYTES + 12*FPREGBYTES)(sp)
	FPLOAD f15, (20*REGBYTES + 13*FPREGBYTES)(sp)
	FPLOAD f16, (20*REGBYTES + 14*FPREGBYTES)(sp)
	FPLOAD f17, (20*REGBYTES + 15*FPREGBYTES)(sp)
	FPLOAD f28, (20*REGBYTES + 16*FPREGBYTES)(sp)
	FPLOAD f29, (20*REGBYTES + 17*FPREGBYTES)(sp)
	FPLOAD f30, (20*REGBYTES + 18*FPREGBYTES)(sp)
	FPLOAD f31, (20*REGBYTES + 19*FPREGBYTES)(sp)
  #endif
#endif

  // De-allocate the stack space used for saving context
#ifdef __riscv_flen
  #if(__riscv_flen == 64)
    addi sp, sp, 20*REGBYTES + 20*FPREGBYTES
  #else
    addi sp, sp, 20*REGBYTES
  #endif
#else
  addi sp, sp, 20*REGBYTES
#endif
.endm

###############################################
###############################################
#restore caller registers
.macro RESTORE_CONTEXT_EXCPT_X5
  LOAD x1,  0*REGBYTES(sp)
  LOAD x6,  2*REGBYTES(sp)
  LOAD x7,  3*REGBYTES(sp)
  LOAD x10, 4*REGBYTES(sp)
  LOAD x11, 5*REGBYTES(sp)
  LOAD x12, 6*REGBYTES(sp)
  LOAD x13, 7*REGBYTES(sp)
  LOAD x14, 8*REGBYTES(sp)
  LOAD x15, 9*REGBYTES(sp)
#ifndef __riscv_32e
  LOAD x16, 10*REGBYTES(sp)
  LOAD x17, 11*REGBYTES(sp)
  LOAD x28, 12*REGBYTES(sp)
  LOAD x29, 13*REGBYTES(sp)
  LOAD x30, 14*REGBYTES(sp)
  LOAD x31, 15*REGBYTES(sp)
#endif
.endm

###############################################
###############################################
#restore caller registers
.macro RESTORE_CONTEXT_ONLY_X5
  LOAD x5,  1*REGBYTES(sp)
.endm

###############################################
###############################################
/*
 * =============================================================================
 * MACRO: SAVE_EPC_STATUS
 *
 * Saves critical Control and Status Registers (CSRs) to the stack. These
 * are required to correctly return from an exception or interrupt.
 * =============================================================================
 */
.macro SAVE_EPC_STATUS
  csrr x5, CSR_MEPC      // Read Machine Exception Program Counter
  STORE x5,  16*REGBYTES(sp)
  csrr x5, CSR_MSTATUS    // Read Machine Status Register
  STORE x5,  17*REGBYTES(sp)
  csrr x5, CSR_MSUBM      // Read Machine Sub-mode Register (Nuclei specific)
  STORE x5,  18*REGBYTES(sp)
.endm

/*
 * =============================================================================
 * MACRO: RESTORE_EPC_STATUS
 *
 * Restores the critical CSRs from the stack.
 * =============================================================================
 */
.macro RESTORE_EPC_STATUS
  LOAD x5,  16*REGBYTES(sp)
  csrw CSR_MEPC, x5      // Write Machine Exception Program Counter
  LOAD x5,  17*REGBYTES(sp)
  csrw CSR_MSTATUS, x5   // Write Machine Status Register
  LOAD x5,  18*REGBYTES(sp)
  csrw CSR_MSUBM, x5     // Write Machine Sub-mode Register
.endm

/*
 * =============================================================================
 * SECTION: .text.trap
 *
 * Entry point for synchronous exceptions (e.g., illegal instruction,
 * memory access fault). This is a non-vectored handler; the address of
 * `trap_entry` is placed in the MTVEC CSR during startup.
 * =============================================================================
 */
  .section .text.trap
  .align 6 // Align to a 64-byte boundary, required by ECLIC in vectored mode.
  .global trap_entry
.weak trap_entry
trap_entry:
  // 1. Save all caller-saved registers to the stack.
  SAVE_CONTEXT

  // 2. Save essential CSRs (mepc, mstatus, msubm) to the stack.
  SAVE_EPC_STATUS

  // 3. Prepare arguments and call the C-level trap handler.
  // According to the ABI, a0 and a1 are the first two argument registers.
  csrr a0, mcause   // Pass mcause (the cause of the trap) as the first argument.
  mv a1, sp         // Pass the stack pointer as the second argument.
  call handle_trap  // Call the high-level C function to handle the trap.

  // 4. Restore CSRs from the stack.
  RESTORE_EPC_STATUS

  // 5. Restore all caller-saved registers.
  RESTORE_CONTEXT

  // 6. Return from the trap. mret restores the program counter from mepc
  // and restores the previous interrupt-enable state from mstatus.
  mret


/*
 * =============================================================================
 * SECTION: .text.irq
 *
 * Common entry point for ECLIC interrupts in non-vectored mode. The address
 * of `irq_entry` is placed in the MTVT2 CSR during startup.
 * =============================================================================
 */
  .section .text.irq
  .align 2
  .global irq_entry
.weak irq_entry
irq_entry: // This label's address is written to the MTVT2 register at startup.

  // 1. Save caller-saved registers.
  SAVE_CONTEXT

  // 2. Use Nuclei-specific CSRs for hardware-accelerated context saving.
  // This is faster than manually reading with csrr and then storing.
  // The immediate value is a word-offset from the stack pointer (sp).
  // csrrwi rd, csr, imm  =>  rd = CSR; CSR = (rd | imm)
  // Since rd is x0 (zero), this effectively becomes: read CSR, write 0.
  // However, for these custom CSRs, it's a special "push" operation.
  csrrwi  x0, CSR_PUSHMCAUSE, 17  // Hardware pushes mcause to sp + 17*REGBYTES
  csrrwi  x0, CSR_PUSHMEPC, 18    // Hardware pushes mepc to sp + 18*REGBYTES
  csrrwi  x0, CSR_PUSHMSUBM, 19   // Hardware pushes msubm to sp + 19*REGBYTES

// This loop services all pending, enabled interrupts. The ECLIC hardware
// handles preemption and tail-chaining automatically.
service_pending_interrupts:
  // 3. Atomically claim and jump to the next pending interrupt handler.
  // This is the core of the ECLIC's non-vectored handling mechanism.
  // The CSR_JALMNXTI operation does the following:
  //  - Finds the highest priority pending interrupt.
  //  - If found, it enables global interrupts (sets mstatus.MIE).
  //  - Jumps to the interrupt's service routine (address from MTVT).
  //  - Puts the return address (here, the next instruction) into the `ra` register.
  //  - If no interrupt is pending, it does nothing and execution continues.
  csrrw ra, CSR_JALMNXTI, ra

  // Upon returning from the ISR (via `ret`), execution resumes here.
  // The loop is implicitly handled by the JALMNXTI hardware until no more
  // interrupts are pending to be claimed.

  // 4. Once all interrupts are serviced, restore the original context.
  // Interrupts must be disabled to safely restore CSRs.
  DISABLE_MIE

  // Restore the original (pre-interrupt) CSR values from the stack.
  LOAD x5,  19*REGBYTES(sp)
  csrw CSR_MSUBM, x5
  LOAD x5,  18*REGBYTES(sp)
  csrw CSR_MEPC, x5
  LOAD x5,  17*REGBYTES(sp)
  csrw CSR_MCAUSE, x5

  // Restore the general-purpose and floating-point registers.
  RESTORE_CONTEXT

  // 5. Return from interrupt.
  mret

#endif // ENTRY_S