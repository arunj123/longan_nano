import os
import sys

# To import 'config' from the parent's 'tools' subfolder, we must modify the system path.
# 1. Get the directory of the current file (bldmgr/config.py).
_CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
# 2. Get the root folder by going one level up from the current directory.
_ROOT = os.path.abspath(os.path.join(_CURRENT_DIR, '..'))
# 3. Add the root folder to the system path.
sys.path.insert(0, _ROOT)

# Import the toolchain configuration from the 'tools' directory at the project root.
# This now works because we've added the project root to the system path above.
from tools import config
from gd32.components import components

# ==============================================================================
# Project & Target Configuration
# ==============================================================================
# Name of the final output file (e.g., firmware.elf, firmware.bin).
TARGET_NAME = "firmware"

# Directory where build artifacts will be stored.
BUILD_DIR = "build"

# Set to 1 to include debug symbols (-g), 0 for release.
DEBUG_MODE = 1

# ==============================================================================
# Hardware & Project-Specific Flags
# ==============================================================================
# --- Global Preprocessor Definitions ---
# These are specific to the GD32VF103 target and project setup.
GLOBAL_C_DEFINES = [
    "-DGD32VF103",
    "-D__NUCLEI_N200",
    "-DGD32VF103C_START",
]

# --- CPU & ABI Flags ---
# These flags define the specific RISC-V architecture of the target MCU.
CPU_FLAGS = [
    "-march=rv32imac_zicsr", "-mabi=ilp32",
    "-msmall-data-limit=8", "-mno-save-restore",
]

# --- Linker Script ---
# Path to the linker script specific to this MCU's memory layout.
LINKER_SCRIPT = r"src/system/GD32VF103xB.lds"

# ==============================================================================
# Project Components
# ==============================================================================
# Enable or disable parts of the firmware by toggling the 'enabled' flag.

gd32_components = {}
for component_name in ['riscv_drivers', 'syscall_stubs', 'gd32_std_peripheral_lib',
                       'usb_driver_core', 'usb_driver_device', 'usb_device_core',
                       'usb_device_ustd',]:
    gd32_components[component_name] = components[component_name].copy()
    gd32_components[component_name]['enabled'] = True
    gd32_components[component_name]['module'] = "gd32"

COMPONENTS = { 
    **gd32_components,
    "core_startup": {
        "c_sources": [r"src/system/system_gd32vf103.c", r"src/system/init.c",
                       r"src/system/handlers.c", r"src/system/systick.c",],
        "cpp_sources": [],
        "asm_sources": [r"src/system/entry.S", r"src/system/start.S"],
        "include_paths": [r"-Isrc/system"],
        "enabled": True,
    },
    "usb_serial": {
        "c_sources": [
            r"src/usb_serial/interrupts.c",
        ],
        "cpp_sources": [r"src/usb_serial/usb.cpp",],
        "asm_sources": [],
        "include_paths": [r"-Isrc/usb_serial"],
        "enabled": False,
    },
    "usb_composite": {
        "c_sources": [r"src/usb_composite/gd32vf103_it.cpp",
                      r"src/usb_composite/usb_device.cpp",
                      r"src/usb_composite/usbd_descriptors.cpp",],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Isrc/usb_composite"],
        "enabled": True,
    },
    "application": {
        "c_sources": [r"src/gd32vf103_hw.c",],
        "cpp_sources": [r"src/main.cpp", r"src/usbd_msc_mem.cpp", r"src/board.cpp",],
        "asm_sources": [],
        "include_paths": [r"-Isrc"],
        "enabled": True
    },
}

# ==============================================================================
# Expose Tool Config to Build Logic
# ==============================================================================
# Make the imported tool configuration variables available to build_logic.py.
# This avoids changing the build script, which expects these variables to be
# directly in the 'config' module's namespace.
TOOLCHAIN_PATH = config.TOOLCHAIN_PATH
TOOLCHAIN_PREFIX = config.TOOLCHAIN_PREFIX
NUCLEUS_OPENOCD_PATH = config.NUCLEUS_OPENOCD_PATH
OPENOCD_PATH = config.OPENOCD_PATH
DFU_UTIL_PATH = config.DFU_UTIL_PATH
OPTIMIZATION = config.OPTIMIZATION
C_STANDARD = config.C_STANDARD
CPP_STANDARD = config.CPP_STANDARD
COMMON_WARNING_FLAGS = config.COMMON_WARNING_FLAGS
C_WARNING_FLAGS = config.C_WARNING_FLAGS
CPP_WARNING_FLAGS = config.CPP_WARNING_FLAGS.copy()
CPP_WARNING_FLAGS.remove("-Wold-style-cast")  # Not needed for this project
CPP_EMBEDDED_FLAGS = config.CPP_EMBEDDED_FLAGS
LIBRARIES = config.LIBRARIES