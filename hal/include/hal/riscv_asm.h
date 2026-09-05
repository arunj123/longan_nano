/*
 * -----------------------------------------------------------------------------
 * -- FILE: riscv_asm.h
 * -----------------------------------------------------------------------------
 * -- Minimal, clean RISC-V RV32IMAC assembly definitions specifically for
 * -- the Nuclei Bumblebee N200 core (GD32VF103).
 * -- Replaces legacy riscv_encoding.h, riscv_bits.h, and n200_eclic.h.
 * -----------------------------------------------------------------------------
 */

#ifndef HAL_RISCV_ASM_H
#define HAL_RISCV_ASM_H

// Register size for RV32
#define REGBYTES            4

// Assembly memory access pseudo-instructions for 32-bit registers
#define STORE               sw
#define LOAD                lw
#define store               sw
#define load                lw

// Core RISC-V Standard Control and Status Registers (CSRs)
#define CSR_MSTATUS         0x300
#define CSR_MISA            0x301
#define CSR_MIE             0x304
#define CSR_MTVEC           0x305
#define CSR_MCOUNTINHIBIT   0x320
#define CSR_MSCRATCH        0x340
#define CSR_MEPC            0x341
#define CSR_MCAUSE          0x342
#define CSR_MTVAL           0x343
#define CSR_MIP             0x344

// Nuclei N200 Custom Machine CSRs
#define CSR_MSUBM           0x7C4   // Machine Sub-Mode
#define CSR_MMISC_CTL       0x7D0   // Machine Miscellaneous Control
#define CSR_MTVT            0x307   // Machine Trap Vector Table base address
#define CSR_MTVT2           0x7EC   // Machine Trap Vector Table 2 (non-vectored IRQ entry)
#define CSR_JALMNXTI        0x7ED   // ECLIC jump-and-link next interrupt
#define CSR_PUSHMCAUSE      0x7EE   // Push mcause during fast interrupt
#define CSR_PUSHMEPC        0x7EF   // Push mepc during fast interrupt
#define CSR_PUSHMSUBM       0x7EB   // Push msubm during fast interrupt

// mstatus register bit masks
#define MSTATUS_UIE         0x00000001
#define MSTATUS_SIE         0x00000002
#define MSTATUS_MIE         0x00000008
#define MSTATUS_UPIE        0x00000010
#define MSTATUS_SPIE        0x00000020
#define MSTATUS_MPIE        0x00000080
#define MSTATUS_SPP         0x00000100
#define MSTATUS_MPP         0x00001800
#define MSTATUS_FS          0x00006000
#define MSTATUS_XS          0x00018000
#define MSTATUS_MPRV        0x00020000
#define MSTATUS_SUM         0x00040000
#define MSTATUS_MXR         0x00080000
#define MSTATUS_TVM         0x00100000
#define MSTATUS_TW          0x00200000
#define MSTATUS_TSR         0x00400000
#define MSTATUS_SD          0x80000000

#endif // HAL_RISCV_ASM_H
