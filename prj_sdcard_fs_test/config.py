import os
import sys

_CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.abspath(os.path.join(_CURRENT_DIR, '..'))
sys.path.insert(0, _ROOT)

from tools import config
from gd32.components import components as gd32
from lib.components import components as lib
from hal.components import components as hal
from bsp.components import components as bsp
from drivers.components import components as drivers

# ==============================================================================
# Project & Target Configuration
# ==============================================================================
TARGET_NAME = "firmware"
BUILD_DIR = "build"
DEBUG_MODE = 1

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

# ==============================================================================
# Project Components
# ==============================================================================
gd32_components = {}
for component_name in ['riscv_drivers', 'syscall_stubs', 'gd32_std_peripheral_lib']:
    gd32_components[component_name] = gd32[component_name].copy()
    gd32_components[component_name]['module'] = "gd32"

lib_components = {}
for component_name in ['debug_uart0', 'system', 'gd32_lcd', 'fatfs']:
    lib_components[component_name] = lib[component_name].copy()
    lib_components[component_name]['module'] = 'lib'

modern_components = {
    "hal": {**hal["hal"], "module": "hal"},
    "bsp": {**bsp["bsp"], "module": "bsp"},
    "drivers": {**drivers["drivers"], "module": "drivers"},
}

COMPONENTS = { 
    **gd32_components,
    **lib_components,
    **modern_components,
    "application": {
        "c_sources": [],
        "cpp_sources": [r"src/main.cpp"],
        "asm_sources": [],
        "include_paths": [r"-Isrc"],
        "enabled": True
    },
}

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
