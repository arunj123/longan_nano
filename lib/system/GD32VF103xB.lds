OUTPUT_ARCH( "riscv" )

ENTRY( _start )

MEMORY
{ 
	flash (rx)  : ORIGIN = 0x08000000, LENGTH = 128k
	ram   (rwx) : ORIGIN = 0x20000000, LENGTH = 32K 
}

PHDRS
{
	flash_pt PT_LOAD FLAGS(5); /* RX: Read + Execute */
	ram_pt   PT_LOAD FLAGS(6); /* RW: Read + Write */
}

SECTIONS
{
  __stack_size = DEFINED(__stack_size) ? __stack_size : 2K;

  .init           :
  {
    KEEP (*(SORT_NONE(.init)))
  } >flash AT>flash :flash_pt

  .ilalign         :
  {
    . = ALIGN(4);
    PROVIDE( _ilm_lma = . );
  } >flash AT>flash :flash_pt

  .ialign         :
  {
    PROVIDE( _ilm = . );
  } >flash AT>flash :flash_pt

  .text           :
  {
    *(.text.unlikely .text.unlikely.*)
    *(.text.startup .text.startup.*)
    *(.text .text.*)
    *(.gnu.linkonce.t.*)
  } >flash AT>flash :flash_pt

  .rodata         :
  {
    *(.rdata)
    *(.rodata .rodata.*)
    *(.gnu.linkonce.r.*)
    . = ALIGN(8);
    *(.srodata.cst16)
    *(.srodata.cst8)
    *(.srodata.cst4)
    *(.srodata.cst2)
    *(.srodata .srodata.*)
  } >flash AT>flash :flash_pt

  .fini           :
  {
    KEEP (*(SORT_NONE(.fini)))
  } >flash AT>flash :flash_pt

  . = ALIGN(4);

  PROVIDE (__etext = .);
  PROVIDE (_etext = .);
  PROVIDE (etext = .);
  PROVIDE( _eilm = . );

  .preinit_array  :
  {
    PROVIDE_HIDDEN (__preinit_array_start = .);
    KEEP (*(.preinit_array))
    PROVIDE_HIDDEN (__preinit_array_end = .);
  } >flash AT>flash :flash_pt

  .init_array     :
  {
    PROVIDE_HIDDEN (__init_array_start = .);
    KEEP (*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
    KEEP (*(.init_array EXCLUDE_FILE (*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o ) .ctors))
    PROVIDE_HIDDEN (__init_array_end = .);
  } >flash AT>flash :flash_pt

  .fini_array     :
  {
    PROVIDE_HIDDEN (__fini_array_start = .);
    KEEP (*(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)))
    KEEP (*(.fini_array EXCLUDE_FILE (*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o ) .dtors))
    PROVIDE_HIDDEN (__fini_array_end = .);
  } >flash AT>flash :flash_pt

  .ctors          :
  {
    KEEP (*crtbegin.o(.ctors))
    KEEP (*crtbegin?.o(.ctors))
    KEEP (*(EXCLUDE_FILE (*crtend.o *crtend?.o ) .ctors))
    KEEP (*(SORT(.ctors.*)))
    KEEP (*(.ctors))
  } >flash AT>flash :flash_pt

  .dtors          :
  {
    KEEP (*crtbegin.o(.dtors))
    KEEP (*crtbegin?.o(.dtors))
    KEEP (*(EXCLUDE_FILE (*crtend.o *crtend?.o ) .dtors))
    KEEP (*(SORT(.dtors.*)))
    KEEP (*(.dtors))
  } >flash AT>flash :flash_pt

  . = ALIGN(4);
  PROVIDE( _data_lma = . );

  .data           : AT( _data_lma )
  {
    . = ALIGN(4);
    PROVIDE( _data = . );
    *(.data .data.*)
    *(.gnu.linkonce.d.*)
    . = ALIGN(8);
    PROVIDE( __global_pointer$ = . + 0x800); 
    *(.sdata .sdata.*)
    *(.gnu.linkonce.s.*)
    . = ALIGN(4);
    PROVIDE( _edata = . );
    PROVIDE( edata = . );
  } >ram :ram_pt

  .bss (NOLOAD)   :
  {
    . = ALIGN(4);
    PROVIDE( _fbss = . );
    PROVIDE( __bss_start = . );
    *(.sbss*)
    *(.gnu.linkonce.sb.*)
    *(.bss .bss.*)
    *(.gnu.linkonce.b.*)
    *(COMMON)
    . = ALIGN(4);
    PROVIDE( _end = . );
    PROVIDE( end = . );
  } >ram :ram_pt

  .stack ORIGIN(ram) + LENGTH(ram) - __stack_size (NOLOAD) :
  {
    . = ALIGN(8);
    PROVIDE( _heap_end = . ); 
    . = __stack_size;  
    . = ALIGN(8);
    PROVIDE( _sp = . ); 
  } >ram :ram_pt

  ASSERT(_end <= _heap_end, "Error: RAM overflowed by .data + .bss!")
  ASSERT(_sp <= ORIGIN(ram) + LENGTH(ram), "Error: Stack pointer exceeds RAM bounds!")
}
