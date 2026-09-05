import os
import sys

# To import 'config' from the parent's 'tools' subfolder, we must modify the system path.
_CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.abspath(os.path.join(_CURRENT_DIR, '..'))
sys.path.insert(0, _ROOT)

# Import the toolchain configuration from the 'tools' directory at the project root.
from tools import config
from lib.components import components as lib
from hal.components import components as hal
from bsp.components import components as bsp
from drivers.components import components as drivers


# ==============================================================================
# Project & Target Configuration
# ==============================================================================
TARGET_NAME = "current_monitor"
BUILD_DIR = "build"
DEBUG_MODE = 1

# ==============================================================================
# Hardware & Project-Specific Flags
# ==============================================================================
GLOBAL_C_DEFINES = [
    "-DGD32VF103",
    "-D__NUCLEI_N200",
    "-DGD32VF103C_START",
]

CPU_FLAGS = [
    "-march=rv32imac_zicsr", "-mabi=ilp32",
    "-msmall-data-limit=8", "-mno-save-restore",
]

LINKER_SCRIPT = r"lib/system/GD32VF103xB.lds"

lib_components = {}
for component_name in ['system', 'debug_uart0', 'gd32_lcd']:
    lib_components[component_name] = lib[component_name].copy()
    lib_components[component_name]['module'] = 'lib'

modern_components = {
    "hal": {**hal["hal"], "module": "hal"},
    "bsp": {**bsp["bsp"], "module": "bsp"},
    "drivers": {**drivers["drivers"], "module": "drivers"},
    "usb_driver": {**drivers["usb_driver"], "module": "drivers"},
}

COMPONENTS = { 
    **lib_components,
    **modern_components,
    "usb_hid": {
        "c_sources": [],
        "cpp_sources": [r"src/usb_hid/gd32vf103_it.cpp",
                        r"src/usb_hid/usb_device.cpp",
                        r"src/usb_hid/usbd_descriptors.cpp",],
        "asm_sources": [],
        "include_paths": [r"-Isrc/usb_hid"],
        "enabled": True,
    },
    "application": {
        "c_sources": [],
        "cpp_sources": [r"src/i2c_hw.cpp", r"src/ina219.cpp", r"src/main.cpp", r"src/board.cpp", r"src/display_manager.cpp"],
        "asm_sources": [],
        "include_paths": [r"-Isrc"],
        "enabled": True
    },
}

# ==============================================================================
# Expose Tool Config to Build Logic
# ==============================================================================
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
if "-Wold-style-cast" in CPP_WARNING_FLAGS:
    CPP_WARNING_FLAGS.remove("-Wold-style-cast")
CPP_EMBEDDED_FLAGS = config.CPP_EMBEDDED_FLAGS
LIBRARIES = config.LIBRARIES
