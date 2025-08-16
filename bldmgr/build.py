#!/usr/bin/env python3
import sys
import os
import importlib

# Add the project root to the path to allow importing build_logic and project configs.
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from build_logic import Builder

def find_projects(root_dir='.'):
    """Finds all project directories (prefixed with 'prj_') in the root directory."""
    return [
        item for item in os.listdir(root_dir)
        if os.path.isdir(os.path.join(root_dir, item)) and item.startswith("prj_")
    ]

def print_usage(projects):
    """Prints the available commands and their descriptions."""
    print("\nUsage: python bldmgr/build.py <project_name> [command]")
    print("\nAvailable projects:")
    if projects:
        for proj in sorted(projects):
            print(f"  - {proj}")
    else:
        print("  No projects found (expected directories in the root folder starting with 'prj_').")

    print("\nCommands:")
    print("  build    (default) Incrementally builds the selected project.")
    print("  rebuild  Cleans and then builds the project from scratch.")
    print("  clean    Removes the project's build directory.")
    print("  flash    Builds and programs the device using OpenOCD.")
    print("  program  Programs an already-built binary using OpenOCD.")
    print("  nucleus  Builds and programs using Nuclei-specific OpenOCD.")
    print("  dfu      Builds and programs using DFU-util.")
    sys.exit(1)

def main():
    """Parses command-line arguments and executes the corresponding build process."""
    project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
    available_projects = find_projects(project_root)

    if len(sys.argv) < 2 or sys.argv[1] not in available_projects:
        print("\n❌ Error: Project name not specified or not found.")
        print_usage(available_projects)
        return

    project_name = sys.argv[1]

    # Dynamically import the project's config file
    try:
        config_module_path = f"{project_name}.config"
        config = importlib.import_module(config_module_path)
    except ImportError as e:
        print(f"\n❌ Error: Could not import configuration for project '{project_name}'.", file=sys.stderr)
        print(f"   Reason: {e}", file=sys.stderr)
        print(f"   Ensure '{project_name}/config.py' exists and is valid.", file=sys.stderr)
        sys.exit(1)

    # Pass both the loaded config module and the project name to the builder
    builder = Builder(config, project_name)
    command = "build"
    if len(sys.argv) > 2:
        command = sys.argv[2]

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
        print_usage(available_projects)
        sys.exit(1)

if __name__ == "__main__":
    main()