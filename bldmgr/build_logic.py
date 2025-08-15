import os
import subprocess
import shutil
import config
import sys
import re
import urllib.request
import zipfile
import tarfile

def _download_and_extract_tool(url, archive_path, extract_dir, final_check_path, rename_map=None):
    """Helper to download, extract, and set up a single tool."""
    print(f"    -> Downloading from {url}")
    try:
        with urllib.request.urlopen(url) as response, open(archive_path, 'wb') as out_file:
            total_size_str = response.getheader('Content-Length')
            if total_size_str:
                total_size = int(total_size_str)
                downloaded_size = 0
                chunk_size = 8192  # 8 KB chunks

                while True:
                    chunk = response.read(chunk_size)
                    if not chunk:
                        break
                    out_file.write(chunk)
                    downloaded_size += len(chunk)

                    # Draw progress bar
                    progress = downloaded_size / total_size
                    bar_length = 40
                    filled_length = int(bar_length * progress)
                    bar = '█' * filled_length + '-' * (bar_length - filled_length)
                    percent = progress * 100
                    print(f"\r      [{bar}] {percent:.1f}% ({downloaded_size/1024/1024:.1f}/{total_size/1024/1024:.1f} MB)", end="")
                print()  # Newline after the progress bar is complete
            else:
                # Fallback if Content-Length is not provided
                print("    -> Downloading (size unknown)...")
                shutil.copyfileobj(response, out_file)
                print("    -> Download complete.")
    except Exception as e:
        print(f"\n❌ Error: Failed to download tool. Reason: {e}", file=sys.stderr)
        print("Please check your internet connection or download it manually as per README.md.", file=sys.stderr)
        sys.exit(1)

    print(f"    -> Extracting {os.path.basename(archive_path)}...")
    try:
        if archive_path.endswith(".zip"):
            with zipfile.ZipFile(archive_path, 'r') as zip_ref:
                zip_ref.extractall(extract_dir)
        elif archive_path.endswith(".tar.gz"):
            with tarfile.open(archive_path, 'r:gz') as tar_ref:
                tar_ref.extractall(extract_dir)
    except Exception as e:
        print(f"❌ Error: Failed to extract tool. Reason: {e}", file=sys.stderr)
        print("Please ensure you have permissions and the archive is not corrupt.", file=sys.stderr)
        sys.exit(1)
    finally:
        os.remove(archive_path)

    if rename_map:
        for src, dst in rename_map.items():
            src_path = os.path.join(extract_dir, src)
            dst_path = os.path.join(extract_dir, dst)
            print(f"    -> Renaming '{src_path}' to '{dst_path}'...")
            try:
                os.rename(src_path, dst_path)
            except OSError as e:
                print(f"❌ Error: Failed to rename extracted directory. Reason: {e}", file=sys.stderr)
                sys.exit(1)

    if not (os.path.isdir(final_check_path) or os.path.isfile(final_check_path)):
        print(f"❌ Error: Tool setup failed. Expected path '{final_check_path}' not found after download.", file=sys.stderr)
        sys.exit(1)
    
    print("    -> Tool setup successful.")

def ensure_tools_are_present():
    """Checks for required toolchains and downloads them if missing."""
    tools_dir = "tools"
    os.makedirs(tools_dir, exist_ok=True)

    # 1. Check for RISC-V GCC Toolchain
    if not os.path.isdir(config.TOOLCHAIN_PATH):
        print("⚠️  RISC-V GCC toolchain not found. Attempting to download and set up...")
        url = "https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/v14.2.0-3/xpack-riscv-none-elf-gcc-14.2.0-3-win32-x64.zip"
        archive_path = os.path.join(tools_dir, "gcc.zip")
        _download_and_extract_tool(url=url, archive_path=archive_path, extract_dir=tools_dir, final_check_path=config.TOOLCHAIN_PATH)

    # 2. Check for OpenOCD
    if not os.path.isfile(config.OPENOCD_PATH):
        print("⚠️  OpenOCD not found. Attempting to download and set up...")
        url = "https://github.com/openocd-org/openocd/releases/download/v0.12.0/openocd-v0.12.0-i686-w64-mingw32.tar.gz"
        archive_path = os.path.join(tools_dir, 'openocd.tar.gz')
        extract_dir = os.path.join(tools_dir, "OpenOCD_0.12.0_")
        _download_and_extract_tool(url=url, archive_path=archive_path, extract_dir=extract_dir, final_check_path=config.OPENOCD_PATH, rename_map={"": "../OpenOCD_0.12.0"})

class Builder:
    """
    Encapsulates all logic for building, cleaning, and programming the project.
    Reads configuration from 'config.py' and supports incremental C/C++ builds.
    """

    def __init__(self):
        """Initializes the builder, sets up toolchain paths, and constructs build flags."""
        ensure_tools_are_present()
        if config.TOOLCHAIN_PATH and config.TOOLCHAIN_PATH not in os.environ['PATH']:
            os.environ['PATH'] = config.TOOLCHAIN_PATH + os.pathsep + os.environ['PATH']

        self.cc = config.TOOLCHAIN_PREFIX + "gcc"
        self.cpp = config.TOOLCHAIN_PREFIX + "g++"
        self.asm = config.TOOLCHAIN_PREFIX + "gcc"
        self.cp = config.TOOLCHAIN_PREFIX + "objcopy"
        self.sz = config.TOOLCHAIN_PREFIX + "size"

        self._collect_sources_and_includes()
        self._construct_flags()

    def _collect_sources_and_includes(self):
        """Iterates through components in config.py and collects all active source files and include paths."""
        self.c_sources = []
        self.cpp_sources = []
        self.asm_sources = []
        self.include_paths = []

        print("🔎 Analyzing project components...")
        for name, component in config.COMPONENTS.items():
            if component.get("enabled", False):
                print(f"  - Enabling component: {name}")
                self.c_sources.extend(component.get("c_sources", []))
                self.cpp_sources.extend(component.get("cpp_sources", []))
                self.asm_sources.extend(component.get("asm_sources", []))
                self.include_paths.extend(component.get("include_paths", []))
            else:
                print(f"  - Disabling component: {name}")
        
        self.is_cpp_project = bool(self.cpp_sources)
        self.include_paths = sorted(list(set(self.include_paths)))

    def _construct_flags(self):
        """Builds the final lists of CFLAGS, ASFLAGS, CPPFLAGS, and LDFLAGS."""
        # THIS IS THE FIX: Add GLOBAL_C_DEFINES to the base flags
        base_flags = config.CPU_FLAGS + [
            config.OPTIMIZATION,
            "-Wall",
            "-ffunction-sections",
            "-fdata-sections"
        ] + config.COMMON_WARNING_FLAGS + config.GLOBAL_C_DEFINES

        # C Flags
        self.cflags = base_flags + [config.C_STANDARD] + config.C_WARNING_FLAGS + self.include_paths + ["-MMD", "-MP"]
        if config.DEBUG_MODE:
            self.cflags.extend(["-g", "-gdwarf-2"])

        # C++ Flags
        self.cppflags = base_flags + [config.CPP_STANDARD] + config.CPP_WARNING_FLAGS + config.CPP_EMBEDDED_FLAGS + self.include_paths + ["-MMD", "-MP"]
        if config.DEBUG_MODE:
            self.cppflags.extend(["-g", "-gdwarf-2"])

        # Assembler Flags (Definitions are also needed here for C preprocessor directives in .S files)
        self.asflags = config.CPU_FLAGS + [config.OPTIMIZATION] + self.include_paths + config.GLOBAL_C_DEFINES

        # Linker Flags
        self.ldflags = config.CPU_FLAGS + [
            "-nostartfiles",
            f"-T{config.LINKER_SCRIPT}",
            "--specs=nano.specs",
            "-Xlinker", "--gc-sections",
            f"-Wl,-Map={os.path.join(config.BUILD_DIR, config.TARGET_NAME)}.map"
        ] + config.LIBRARIES
        
        if self.is_cpp_project:
            self.ldflags.append("-lstdc++")

    @staticmethod
    def run_command(cmd):
        """Executes a shell command, prints it, and exits on failure."""
        cmd_str = ' '.join([str(arg).replace('\\', '/') for arg in cmd])
        print(f"🚀 Executing: {cmd_str}")
        try:
            subprocess.run(cmd, check=True, shell=isinstance(cmd, str))
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            print(f"❌ Error: Command failed: {e}", file=sys.stderr)
            sys.exit(1)

    def clean(self):
        """Removes the build directory to ensure a fresh build."""
        print(f"🧹 Cleaning build directory: {config.BUILD_DIR}")
        if os.path.isdir(config.BUILD_DIR):
            shutil.rmtree(config.BUILD_DIR)
        print("Clean complete.")

    def _get_obj_path(self, src_file):
        """Generates the path for an object file, preserving the source directory structure."""
        return os.path.join(config.BUILD_DIR, os.path.normpath(src_file) + '.o')

    def _parse_dependencies(self, dep_file):
        """Parses a .d file to extract all header dependencies."""
        if not os.path.exists(dep_file):
            return []
        with open(dep_file, 'r') as f:
            content = f.read()
        return re.findall(r'([^\s\\]+\.(?:h|inc))', content)

    def _is_rebuild_needed(self, src_file, obj_file):
        """Checks if a source file needs to be recompiled based on modification times."""
        if not os.path.exists(obj_file):
            return True

        obj_mtime = os.path.getmtime(obj_file)
        if os.path.getmtime(src_file) > obj_mtime:
            return True

        if src_file.endswith((".c", ".cpp", ".cc", ".cxx")):
            dep_file = obj_file.replace('.o', '.d')
            if not os.path.exists(dep_file):
                return True

            dependencies = self._parse_dependencies(dep_file)
            for dep in dependencies:
                if os.path.exists(dep) and os.path.getmtime(dep) > obj_mtime:
                    print(f"    -> Dependency '{dep}' changed.")
                    return True
        return False

    def compile_sources(self):
        """Compiles all C, C++, and Assembly sources into object files, skipping unchanged files."""
        print("⚙️  Compiling sources...")
        object_files = []
        cpp_extensions = (".cpp", ".cc", ".cxx")

        all_sources = self.c_sources + self.cpp_sources + self.asm_sources
        for src in all_sources:
            obj_path = self._get_obj_path(src)
            object_files.append(obj_path)

            print(f"  - Checking '{src}'...")
            if not self._is_rebuild_needed(src, obj_path):
                print("    -> Up-to-date. Skipping.")
                continue

            print(f"    -> Compiling...")
            os.makedirs(os.path.dirname(obj_path), exist_ok=True)

            if src.endswith(".c"):
                cmd = [self.cc] + self.cflags + ["-c", src, "-o", obj_path]
            elif src.endswith(cpp_extensions):
                cmd = [self.cpp] + self.cppflags + ["-c", src, "-o", obj_path]
            else: # Assumes .S
                cmd = [self.asm, "-x", "assembler-with-cpp"] + self.asflags + ["-c", src, "-o", obj_path]
            
            self.run_command(cmd)

        return object_files

    def link_objects(self, object_files):
        """Links all compiled object files into a single .elf executable."""
        linker = self.cpp if self.is_cpp_project else self.cc
        print(f"🔗 Linking objects (using {os.path.basename(linker)})...")
        
        elf_path = os.path.join(config.BUILD_DIR, f"{config.TARGET_NAME}.elf")
        cmd = [linker] + self.ldflags + object_files + ["-o", elf_path]
        self.run_command(cmd)

        print("📊 Calculating size...")
        self.run_command([self.sz, elf_path])
        return elf_path

    def create_binaries(self, elf_path):
        """Creates .hex and .bin files from the .elf file for programming."""
        print("📦 Creating final binaries...")
        hex_path = elf_path.replace(".elf", ".hex")
        bin_path = elf_path.replace(".elf", ".bin")
        self.run_command([self.cp, "-O", "ihex", elf_path, hex_path])
        self.run_command([self.cp, "-O", "binary", "-S", elf_path, bin_path])
        print(f"Successfully created binaries in {config.BUILD_DIR}/")

    def build_all(self):
        """Runs the entire build process: compile (incrementally), link, and create binaries."""
        objects = self.compile_sources()
        elf_file = self.link_objects(objects)
        self.create_binaries(elf_file)
        print("\n✅ Build complete.")

    def program_dfu(self):
        """Builds and programs the target using dfu-util."""
        self.build_all()
        print("🔌 Programming with dfu-util...")
        bin_path = os.path.join(config.BUILD_DIR, f"{config.TARGET_NAME}.bin")
        if not os.path.exists(bin_path):
            print(f"❌ Error: {bin_path} not found. Please run 'build' first.", file=sys.stderr)
            sys.exit(1)

        cmd = [config.DFU_UTIL_PATH, "-a", "0", "-s", "0x08000000:leave", "-D", bin_path]
        self.run_command(cmd)
        print("✅ DFU Programming complete.")

    def program_openocd(self):
        """Builds and programs the target using a generic OpenOCD configuration."""
        self.build_all()
        print("🔌 Programming with OpenOCD...")
        hex_path = os.path.join(os.getcwd(), config.BUILD_DIR, f"{config.TARGET_NAME}.hex").replace('\\', '/')
        if not os.path.exists(hex_path):
            print(f"❌ Error: {hex_path} not found. Please run 'build' first.", file=sys.stderr)
            sys.exit(1)

        config_file = "tools/config/openocd-sipeed-libusb.cfg"
        program_cmd = f'program "{hex_path}" verify; reset; shutdown'
        cmd = [config.OPENOCD_PATH, '-f', config_file, '-c', program_cmd]
        self.run_command(cmd)
        print("✅ OpenOCD Programming complete.")

    def program_openocd_nucleus(self):
        """Builds and programs the target using the Nuclei-specific OpenOCD configuration."""
        self.build_all()
        print("🔌 Programming with Nuclei OpenOCD...")
        hex_path = os.path.join(os.getcwd(), config.BUILD_DIR, f"{config.TARGET_NAME}.hex").replace('\\', '/')
        if not os.path.exists(hex_path):
            print(f"❌ Error: {hex_path} not found. Please run 'build' first.", file=sys.stderr)
            sys.exit(1)

        board_cfg = 'C:/Users/arunj/.platformio/packages/framework-nuclei-sdk/SoC/gd32vf103/Board/gd32vf103c_longan_nano/openocd_gd32vf103.cfg'
        program_cmd = f'program "{hex_path}" verify; reset; shutdown'
        cmd = [
            config.NUCLEUS_OPENOCD_PATH,
            '-f', board_cfg,
            '-c', "debug_level 1",
            '-c', "reset halt; flash protect 0 0 last off;",
            '-c', program_cmd
        ]
        self.run_command(cmd)
        print("✅ Nucleus OpenOCD Programming complete.")