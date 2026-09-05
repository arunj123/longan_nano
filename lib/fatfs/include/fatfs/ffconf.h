/*---------------------------------------------------------------------------/
/  Configurations of FatFs Module (R0.15 w/patch2)
/---------------------------------------------------------------------------*/

#define FFCONF_DEF	80286	/* Revision ID */

/*---------------------------------------------------------------------------/
/ Function Configurations
/---------------------------------------------------------------------------*/

#define FF_FS_READONLY	0
/* This option switches read-only configuration. (0:Read/Write or 1:Read-only)
/  Read-only configuration removes writing API functions, f_write(), f_sync(),
/  f_unlink(), f_mkdir(), f_chmod(), f_rename(), f_truncate(), f_getfree()
/  and optional writing functions as well. */


#define FF_FS_MINIMIZE	0
/* This option defines minimization level to remove some basic API functions.
/
/   0: Basic functions are fully enabled.
/   1: f_stat(), f_getfree(), f_unlink(), f_mkdir(), f_truncate() and f_rename()
/      are removed.
/   2: f_opendir(), f_readdir() and f_closedir() are removed in addition to 1.
/   3: f_lseek() function is removed in addition to 2. */


#define FF_USE_FIND		0
/* This option switches filtered directory read functions, f_findfirst() and
/  f_findnext(). (0:Disable, 1:Enable 2:Enable with matching altname[] too) */


#define FF_USE_MKFS		0
/* This option switches f_mkfs() function. (0:Disable or 1:Enable) */


#define FF_USE_FASTSEEK	0
/* This option switches fast seek function. (0:Disable or 1:Enable) */


#define FF_USE_EXPAND	0
/* This option switches f_expand function. (0:Disable or 1:Enable) */


#define FF_USE_CHMOD	0
/* This option switches attribute manipulation functions, f_chmod() and f_utime().
/  (0:Disable or 1:Enable) Also FF_FS_READONLY needs to be 0 to enable this option. */


#define FF_USE_LABEL	0
/* This option switches volume label functions, f_getlabel() and f_setlabel().
/  (0:Disable or 1:Enable) */


#define FF_USE_FORWARD	0
/* This option switches f_forward() function. (0:Disable or 1:Enable) */


#define FF_USE_STRFUNC	0
#define FF_PRINT_LLI	0
#define FF_PRINT_FLOAT	0
#define FF_STRF_ENCODE	3
/* FF_USE_STRFUNC switches string functions, f_gets(), f_putc(), f_puts() and f_printf().
/   0: Disable.
/   1: Enable without LF-CRLF conversion.
/   2: Enable with LF-CRLF conversion. */


/*---------------------------------------------------------------------------/
/ Locale and Namespace Configurations
/---------------------------------------------------------------------------*/

#define FF_CODE_PAGE	437
/* This option specifies the OEM code page to be used on the target system.
/   437 - U.S. (Standard Latin ASCII - minimal footprint) */


#define FF_USE_LFN		0
#define FF_MAX_LFN		255
/* The FF_USE_LFN switches the support for LFN (long file name).
/   0: Disable LFN. Compact 8.3 format. Zero dynamic memory allocation.
/   1: Enable LFN with static working buffer on the BSS. Always NOT thread-safe.
/   2: Enable LFN with dynamic working buffer on the STACK.
/   3: Enable LFN with dynamic working buffer on the HEAP. */


#define FF_LFN_UNICODE	0
/* This option switches the character encoding on the API when LFN is enabled.
/   0: ANSI/OEM in current CP (TCHAR = char) */


#define FF_LFN_BUF		255
#define FF_SFN_BUF		12


#define FF_FS_RPATH		0
/* This option configures support for relative path.
/   0: Disable relative path and remove related functions. */


/*---------------------------------------------------------------------------/
/ Drive/Volume Configurations
/---------------------------------------------------------------------------*/

#define FF_VOLUMES		1
/* Number of volumes (logical drives) to be used. (1-10) */


#define FF_STR_VOLUME_ID	0
#define FF_VOLUME_STRS		"RAM","NAND","CF","SD","SD2","USB","USB2","USB3"


#define FF_MULTI_PARTITION	0
/* 0: Each logical drive number is bound to the same physical drive number */


#define FF_MIN_SS		512
#define FF_MAX_SS		512
/* Range of sector size to be supported. (512, 1024, 2048 or 4096)
/  512 is standard for SD/MMC cards. */


#define FF_LBA64		0
/* This option switches support for 64-bit LBA. (0:Disable or 1:Enable)
/  0: Native 32-bit LBA (DWORD) - Single-cycle 32-bit integer arithmetic on RV32/Cortex-M.
/  1: 64-bit LBA (QWORD) for drives >= 2TB (requires exFAT). */


#define FF_MIN_GPT		0x10000000
/* Minimum number of sectors to switch GPT as partitioning format. */


#define FF_USE_TRIM		0
/* 0: Disable ATA-TRIM */


/*---------------------------------------------------------------------------/
/ System Configurations
/---------------------------------------------------------------------------*/

#define FF_FS_TINY		1
/* This option switches tiny buffer configuration. (0:Normal or 1:Tiny)
/  1: Tiny Mode: Shared sector buffer in FATFS object; shrinks FIL object to 36 bytes.
/  Crucial optimization for microcontrollers with <= 32 KB SRAM. */


#define FF_FS_EXFAT		0
/* This option switches support for exFAT filesystem. (0:Disable or 1:Enable) */


#define FF_FS_NORTC		0
#define FF_NORTC_MON	9
#define FF_NORTC_MDAY	5
#define FF_NORTC_YEAR	2026
/* FF_FS_NORTC = 0 uses get_fattime() provided by the user application. */


#define FF_FS_NOFSINFO	0
/* 0: Trust free cluster count in FSINFO */


#define FF_FS_LOCK		0
/* 0: Disable file lock function */


#define FF_FS_REENTRANT	0
#define FF_FS_TIMEOUT	1000
/* 0: Re-entrancy disabled for single-threaded bare-metal.
/  1: Enable re-entrancy for 32-bit RTOS (FreeRTOS/Zephyr/RT-Thread). */

/*--- End of configuration options ---*/
