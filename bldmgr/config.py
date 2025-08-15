import os

# ==============================================================================
# Project & Target Configuration
# ==============================================================================
TARGET_NAME = "firmware"
BUILD_DIR = "build"
DEBUG_MODE = 1

# ==============================================================================
# Toolchain Configuration
# ==============================================================================
TOOLCHAIN_PATH = "tools/xpack-riscv-none-elf-gcc-14.2.0-3/bin"
TOOLCHAIN_PREFIX = "riscv-none-elf-"
NUCLEUS_OPENOCD_PATH = "C:/src/arunj/.platformio/packages/tool-openocd-nuclei/bin/openocd.exe"
OPENOCD_PATH = "tools/OpenOCD_0.12.0/bin/openocd.exe"
DFU_UTIL_PATH = "dfu-util"

# ==============================================================================
# Compiler & Linker Flags
# ==============================================================================
OPTIMIZATION = "-Os"

# C and C++ Language Standards
# Use the GNU standards to enable GCC-specific extensions like inline 'asm'
# and braced-group expressions, which are used in the low-level drivers.
C_STANDARD = "-std=gnu17"
CPP_STANDARD = "-std=gnu++20"

# --- Global Preprocessor Definitions ---
GLOBAL_C_DEFINES = [
    "-DGD32VF103",
    "-D__NUCLEI_N200",
    "-DGD32VF103C_START",
]

# --- Enhanced Compiler Warnings ---
COMMON_WARNING_FLAGS = [
    "-Wall", "-Wextra", "-Wpedantic", "-Wshadow",
    "-Wconversion", "-Wsign-conversion",
]
C_WARNING_FLAGS = ["-Wmissing-prototypes", "-Wstrict-prototypes"]
CPP_WARNING_FLAGS = ["-Wnon-virtual-dtor", "-Wold-style-cast"]
CPP_EMBEDDED_FLAGS = ["-fno-exceptions", "-fno-rtti", "-fno-threadsafe-statics"]

# Flags defining the CPU architecture and ABI.
CPU_FLAGS = [
    "-march=rv32imac_zicsr", "-mabi=ilp32",
    "-msmall-data-limit=8", "-mno-save-restore",
]

LINKER_SCRIPT = r"src/system/GD32VF103xB.lds"
LIBRARIES = ["-lc", "-lm", "-lnosys"]

# ==============================================================================
# Project Components
# ==============================================================================
COMPONENTS = {
    "core_startup": {
        "c_sources": [r"src/system/system_gd32vf103.c"],
        "cpp_sources": [],
        "asm_sources": [r"src/system/entry.S", r"src/system/start.S"],
        "include_paths": [r"-Isrc/system"],
        "enabled": True
    },
    "riscv_drivers": {
        "c_sources": [r"gd32/Firmware/RISCV/drivers/n200_func.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Igd32/Firmware/RISCV/drivers"],
        "enabled": True
    },
    "syscall_stubs": {
        "c_sources": [
            r"gd32/Firmware/RISCV/stubs/sbrk.c", r"gd32/Firmware/RISCV/stubs/lseek.c",
            r"gd32/Firmware/RISCV/stubs/fstat.c", r"gd32/Firmware/RISCV/stubs/close.c",
            r"gd32/Firmware/RISCV/stubs/_exit.c", r"gd32/Firmware/RISCV/stubs/write_hex.c",
            r"gd32/Firmware/RISCV/stubs/isatty.c", r"gd32/Firmware/RISCV/stubs/read.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Igd32/Firmware/RISCV/stubs"],
        "enabled": True
    },
    "eclipse_env": {
        "c_sources": [
            r"src/system/init.c",
            r"src/system/handlers.c",
            r"src/system/your_printf.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [],
        "enabled": True
    },
    "gd32_std_peripheral_lib": {
        "c_sources": [
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_rcu.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_gpio.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_usart.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_timer.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_eclic.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_exti.c",
            r"gd32/Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_pmu.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-Igd32/Firmware/GD32VF103_standard_peripheral",
            r"-Igd32/Firmware/GD32VF103_standard_peripheral/Include"
        ],
        "enabled": True
    },
    "usb_driver": {
        "c_sources": [
            r"gd32/Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_core.c",
            r"gd32/Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_dev.c",
            r"gd32/Firmware/GD32VF103_usbfs_library/driver/Source/drv_usbd_int.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Igd32/Firmware/GD32VF103_usbfs_library/driver/Include"],
        "enabled": True
    },
    "usb_device_core": {
        "c_sources": [
            r"gd32/Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_core.c",
            r"gd32/Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_enum.c",
            r"gd32/Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_transc.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Igd32/Firmware/GD32VF103_usbfs_library/device/core/Include"],
        "enabled": True
    },
    "usb_class_cdc": {
        "c_sources": [r"gd32/Firmware/GD32VF103_usbfs_library/device/class/cdc/Source/cdc_acm_core.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-Igd32/Firmware/GD32VF103_usbfs_library/device/class/cdc/Include",
            r"-Igd32/Firmware/GD32VF103_usbfs_library/ustd/common",
            r"-Igd32/Firmware/GD32VF103_usbfs_library/ustd/class/cdc",
        ],
        "enabled": True
    },
    "application": {
        "c_sources": [
            r"src/main.c",
            r"src/gd32vf103_hw.c",
            r"src/gd32vf103_it.c",
            r"src/systick.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Isrc", r"-IUtilities"],
        "enabled": True
    },
    "cpp_example": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [],
        "enabled": False
    },
}