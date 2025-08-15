#!/usr/bin/env python3
import sys
from build_logic import Builder

def print_usage():
    """Prints the available commands and their descriptions."""
    print("\nUsage: python build.py [command]")
    print("Commands:")
    print("  build    (default) Incrementally builds the project (.elf, .hex, .bin).")
    print("  rebuild  Cleans and then builds the entire project from scratch.")
    print("  clean    Removes the build directory.")
    print("  flash    Incrementally builds and then programs the device using OpenOCD.")
    print("  program  Programs an already-built binary to the device using OpenOCD.")
    print("  nucleus  Incrementally builds and programs using Nuclei-specific OpenOCD.")
    print("  dfu      Incrementally builds and programs using DFU-util.")

def main():
    """Parses command-line arguments and executes the corresponding build process."""
    builder = Builder()

    command = "build"
    if len(sys.argv) > 1:
        command = sys.argv[1]

    if command == "build":
        builder.build_all()
    elif command == "rebuild":
        builder.clean()
        builder.build_all()
    elif command == "clean":
        builder.clean()
    elif command == "flash":
        builder.build_all()
        builder.program_openocd()
    elif command == "program":
        # Note: This command does not build first.
        builder.program_openocd()
    elif command == "nucleus":
        builder.build_all()
        builder.program_openocd_nucleus()
    elif command == "dfu":
        builder.build_all()
        builder.program_dfu()
    else:
        print(f"\n❌ Error: Unknown command '{command}'")
        print_usage()
        sys.exit(1)

if __name__ == "__main__":
    main()