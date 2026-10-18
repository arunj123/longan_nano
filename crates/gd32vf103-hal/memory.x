/* GD32VF103CBT6 Linker Memory Map & Section Reconciliation
 *
 * Architecture Reference:
 *   Flash: 128 KB (0x08000000 - 0x08020000)
 *   SRAM:   32 KB (0x20000000 - 0x20008000)
 *
 * Section Breakdown ([MON-MEM-02]):
 *   - Flash:
 *       .text   : Executable machine code instructions
 *       .rodata : Read-only data, lookup tables, and string constants
 *   - SRAM:
 *       .data   : Initialized static/global variables
 *       .bss    : Zero-initialized static/global variables (e.g. 8 KB LBA cache)
 *       .heap   : Heap space for allocators (0 bytes in bare-metal no_std)
 *       .stack  : Runtime call stack allocated by riscv-rt from end of .bss
 *                 to 0x20008000 (_stack_start).
 *
 * Note on Berkeley `size` vs System V `size -A`:
 *   Standard Berkeley `size` (text data bss) aggregates all NOBITS sections
 *   (.bss + .stack) into the `bss` column, reporting bss = 32,768 bytes (100% SRAM).
 *   System V format (`riscv-none-elf-size -A`) separates these sections:
 *     - Static .bss: ~8,196 bytes (25.0% SRAM)
 *     - Stack headroom: ~24,572 bytes (75.0% SRAM)
 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  RAM   : ORIGIN = 0x20000000, LENGTH = 32K
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_STACK", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
