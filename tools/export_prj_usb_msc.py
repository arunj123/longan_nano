#!/usr/bin/env python3
"""
export_prj_usb_msc.py

Creates a consolidated text file containing all relevant files used to build
prj_usb_msc (or any specified project) in the longan_nano repository.

The output text file contains the relative file path followed by the exact file content,
separated by clear demarcators.
Included files:
  - Build system scripts (bldmgr/build.py, bldmgr/build_logic.py, tools/config.py)
  - Project configuration and component definitions (config.py, components.py)
  - Linker scripts (.lds)
  - All compiled C, C++, and Assembly source files
  - All direct and transitive header files (.h, .hpp) used by the project
"""

import os
import sys
import re
import glob
import argparse
import importlib

def resolve_repo_root():
    """Locates the repository root directory regardless of current working directory."""
    current_dir = os.path.dirname(os.path.abspath(__file__))
    # If this script is in tools/, root is parent directory
    if os.path.basename(current_dir) == "tools":
        return os.path.abspath(os.path.join(current_dir, ".."))
    # Otherwise check if bldmgr exists in current dir
    if os.path.isdir(os.path.join(current_dir, "bldmgr")):
        return current_dir
    # Check if bldmgr exists in parent
    parent_dir = os.path.abspath(os.path.join(current_dir, ".."))
    if os.path.isdir(os.path.join(parent_dir, "bldmgr")):
        return parent_dir
    return os.getcwd()


def collect_project_files(repo_root: str, project_name: str = "prj_usb_msc") -> list[str]:
    """
    Analyzes project configuration, components, sources, and header dependencies
    to produce a complete list of relative file paths required to build the project.
    """
    if repo_root not in sys.path:
        sys.path.insert(0, repo_root)

    config_module_name = f"{project_name}.config"
    try:
        config = importlib.import_module(config_module_name)
    except ImportError as e:
        print(f"Error: Could not import configuration for project '{project_name}': {e}", file=sys.stderr)
        sys.exit(1)

    collected_paths = set()

    def add_file(rel_or_abs):
        if not rel_or_abs:
            return
        abs_p = rel_or_abs if os.path.isabs(rel_or_abs) else os.path.join(repo_root, rel_or_abs)
        abs_p = os.path.normpath(abs_p)
        if os.path.isfile(abs_p):
            rel_p = os.path.relpath(abs_p, repo_root).replace("\\", "/")
            collected_paths.add(rel_p)

    # 1. Build manager and global tools configuration
    build_scripts = [
        "bldmgr/build.py",
        "bldmgr/build_logic.py",
        "tools/config.py",
    ]
    for bs in build_scripts:
        add_file(bs)

    # 2. Project config file
    add_file(f"{project_name}/config.py")

    # 3. Component definitions (.py)
    standard_modules = ["lib", "hal", "bsp", "drivers"]
    for comp_name, comp_data in getattr(config, "COMPONENTS", {}).items():
        if isinstance(comp_data, dict):
            mod = comp_data.get("module")
            if mod and mod not in standard_modules:
                standard_modules.append(mod)

    for mod in standard_modules:
        comp_py = os.path.join(mod, "components.py")
        if os.path.isfile(os.path.join(repo_root, comp_py)):
            add_file(comp_py)

    # 4. Linker script(s)
    linker_script = getattr(config, "LINKER_SCRIPT", None)
    if linker_script:
        add_file(linker_script)
        # Check if linker script includes other scripts
        lds_abs = os.path.join(repo_root, linker_script)
        if os.path.isfile(lds_abs):
            try:
                with open(lds_abs, "r", encoding="utf-8", errors="ignore") as f:
                    for line in f:
                        m = re.match(r'^\s*INCLUDE\s+["\']?([^"\'\s]+)["\']?', line)
                        if m:
                            inc_lds = m.group(1)
                            add_file(os.path.join(os.path.dirname(linker_script), inc_lds))
            except Exception:
                pass

    # 5. Collect active source files and include directories from enabled components
    sources = []
    include_dirs = []
    for comp_name, comp in getattr(config, "COMPONENTS", {}).items():
        if not comp.get("enabled", False):
            continue
        module = comp.get("module", project_name)

        comp_sources = comp.get("c_sources", []) + comp.get("cpp_sources", []) + comp.get("asm_sources", [])
        for src in comp_sources:
            src_rel = os.path.normpath(os.path.join(module, src)).replace("\\", "/")
            src_abs = os.path.join(repo_root, src_rel)
            if os.path.isfile(src_abs):
                sources.append(src_abs)
                add_file(src_rel)

        for inc in comp.get("include_paths", []):
            raw_inc = inc[2:] if inc.startswith("-I") else inc
            inc_abs = os.path.normpath(os.path.join(repo_root, module, raw_inc))
            include_dirs.append(inc_abs)

    include_dirs = sorted(list(set(include_dirs)))

    # 6. Header discovery via recursive C/C++/ASM include scan
    include_pattern = re.compile(r'^\s*#\s*include\s*["<]([^">]+)[">]')
    to_scan = list(sources)
    visited_files = set(to_scan)

    while to_scan:
        current_file = to_scan.pop()
        if not os.path.isfile(current_file):
            continue
        try:
            with open(current_file, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
        except Exception:
            continue

        file_dir = os.path.dirname(current_file)
        search_dirs = [file_dir] + include_dirs
        for line in lines:
            match = include_pattern.match(line)
            if match:
                header_name = match.group(1)
                for sdir in search_dirs:
                    candidate = os.path.normpath(os.path.join(sdir, header_name))
                    if os.path.isfile(candidate):
                        if candidate not in visited_files:
                            visited_files.add(candidate)
                            add_file(os.path.relpath(candidate, repo_root))
                            to_scan.append(candidate)
                        break

    # 7. Header discovery via compiler dependency (.d) files if project was built
    d_pattern = os.path.join(repo_root, "build", project_name, "**", "*.d")
    for d_file in glob.glob(d_pattern, recursive=True):
        try:
            with open(d_file, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
            clean_content = content.replace("\\\n", " ")
            for line in clean_content.splitlines():
                if ":" in line:
                    _, dep_str = line.split(":", 1)
                    for token in dep_str.strip().split():
                        dep_abs = os.path.normpath(os.path.join(repo_root, token))
                        if os.path.isfile(dep_abs):
                            add_file(os.path.relpath(dep_abs, repo_root))
        except Exception:
            pass

    return sorted(list(collected_paths))


def write_bundle_file(repo_root: str, file_list: list[str], output_path: str):
    """
    Writes all collected files into a single consolidated text file.
    Format:
      Relative file path header followed by exact content.
    """
    abs_output = output_path if os.path.isabs(output_path) else os.path.join(repo_root, output_path)
    os.makedirs(os.path.dirname(abs_output) or ".", exist_ok=True)

    file_count = len(file_list)
    total_bytes = 0
    total_lines = 0

    with open(abs_output, "w", encoding="utf-8", errors="replace") as out_file:
        for idx, rel_path in enumerate(file_list, start=1):
            abs_path = os.path.join(repo_root, rel_path)

            try:
                with open(abs_path, "r", encoding="utf-8") as in_file:
                    content = in_file.read()
            except UnicodeDecodeError:
                with open(abs_path, "r", encoding="latin-1") as in_file:
                    content = in_file.read()

            file_lines = content.count("\n") + (1 if content and not content.endswith("\n") else 0)
            file_bytes = len(content.encode("utf-8"))
            total_lines += file_lines
            total_bytes += file_bytes

            separator = "=" * 80
            out_file.write(f"{separator}\n")
            out_file.write(f"FILE: {rel_path}\n")
            out_file.write(f"{separator}\n")
            out_file.write(content)
            if not content.endswith("\n"):
                out_file.write("\n")
            out_file.write("\n")

    return abs_output, file_count, total_lines, total_bytes


def main():
    parser = argparse.ArgumentParser(
        description="Bundle all relevant source, header, config, and linker files for building a Longan Nano project."
    )
    parser.add_argument(
        "-p", "--project",
        default="prj_usb_msc",
        help="Target project name (default: prj_usb_msc)"
    )
    parser.add_argument(
        "-o", "--output",
        default=None,
        help="Output text file path (default: <project>_all_files.txt in repo root)"
    )
    args = parser.parse_args()

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    repo_root = resolve_repo_root()
    output_filename = args.output or f"{args.project}_all_files.txt"

    print(f"Repository root: {repo_root}")
    print(f"Target project:  {args.project}")
    print("Collecting relevant files...")

    files = collect_project_files(repo_root, args.project)
    print(f"Found {len(files)} relevant files.")

    output_path, count, lines, nbytes = write_bundle_file(repo_root, files, output_filename)
    rel_out = os.path.relpath(output_path, repo_root).replace("\\", "/")

    print(f"\n[SUCCESS] Successfully generated bundle: {rel_out}")
    print(f"   - Total files: {count}")
    print(f"   - Total lines: {lines}")
    print(f"   - Total size:  {nbytes / 1024:.1f} KB")


if __name__ == "__main__":
    main()
