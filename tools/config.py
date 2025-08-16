"""
This file contains the configuration for the development toolchain and
generic compiler/linker flags. These settings are generally specific to the
development environment and not the project itself.
"""

# ==============================================================================
# Toolchain Configuration
# ==============================================================================
# Path to the 'bin' directory of the RISC-V GCC toolchain.
TOOLCHAIN_PATH = "tools/xpack-riscv-none-elf-gcc-14.2.0-3/bin"

# The prefix for the toolchain executables (e.g., 'riscv-none-elf-').
TOOLCHAIN_PREFIX = "riscv-none-elf-"

# Path to the Nuclei-specific OpenOCD executable.
NUCLEUS_OPENOCD_PATH = "C:/src/arunj/.platformio/packages/tool-openocd-nuclei/bin/openocd.exe"

# Path to the standard OpenOCD executable.
OPENOCD_PATH = "tools/OpenOCD_0.12.0/bin/openocd.exe"

# Path to the dfu-util executable (can be just the name if it's in the system PATH).
DFU_UTIL_PATH = "dfu-util"


# ==============================================================================
# Generic Compiler & Linker Flags
# ==============================================================================
# Optimization level. -Os for size, -O2 for speed, -O0 for debugging.
OPTIMIZATION = "-Os"

# C and C++ Language Standards.
# Use GNU standards for common extensions in embedded code.
C_STANDARD = "-std=gnu17"
CPP_STANDARD = "-std=gnu++20"

# Common warning flags for both C and C++.
COMMON_WARNING_FLAGS = [
    "-Wall", "-Wextra", "-Wpedantic", "-Wshadow",
    "-Wconversion", "-Wsign-conversion",
]

# C-specific warning flags.
C_WARNING_FLAGS = ["-Wmissing-prototypes", "-Wstrict-prototypes"]

# C++-specific warning flags.
CPP_WARNING_FLAGS = ["-Wnon-virtual-dtor", "-Wold-style-cast"]

# Flags to disable C++ features not typically used in bare-metal embedded systems.
CPP_EMBEDDED_FLAGS = ["-fno-exceptions", "-fno-rtti", "-fno-threadsafe-statics"]

# Standard libraries to link against.
LIBRARIES = ["-lc", "-lm", "-lnosys"]