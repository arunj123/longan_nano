import os
import subprocess
import shutil
import sys
import re
import urllib.request
import zipfile
import tarfile
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed

def _download_and_extract_tool(url: str, archive_path: str, extract_dir: str, final_check_path: str, rename_map: dict = None):
    """
    Downloads, extracts, and optionally renames a tool archive.

    Args:
        url (str): The URL to download the tool archive from.
        archive_path (str): The local path where the downloaded archive will be saved.
        extract_dir (str): The directory where the archive will be extracted.
        final_check_path (str): A path to a key file or directory inside the extracted
                                contents to verify successful setup.
        rename_map (dict, optional): A map of {original_name: new_name} for renaming
                                     extracted directories. Defaults to None.
    """
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

class Builder:
    """
    Encapsulates all logic for building, cleaning, and programming the project.
    Receives configuration dynamically and supports incremental C/C++ builds.
    """

    def __init__(self, config_module, project_name, variant="all"):
        """Initializes the builder, sets up toolchain paths, and constructs build flags."""
        self.config = config_module
        self.project_name = project_name
        self.variant = (variant or "all").lower()
        # Create a project-specific build directory, e.g., 'build/prj_usb_serial'
        self.build_dir = os.path.join(self.config.BUILD_DIR, self.project_name)

        self.has_rust = os.path.isfile(os.path.join(self.project_name, "Cargo.toml"))

        self._ensure_tools_are_present()
        if self.config.TOOLCHAIN_PATH and self.config.TOOLCHAIN_PATH not in os.environ['PATH']:
            os.environ['PATH'] = self.config.TOOLCHAIN_PATH + os.pathsep + os.environ['PATH']

        self.cc = self.config.TOOLCHAIN_PREFIX + "gcc"
        self.cpp = self.config.TOOLCHAIN_PREFIX + "g++"
        self.asm = self.config.TOOLCHAIN_PREFIX + "gcc"
        self.cp = self.config.TOOLCHAIN_PREFIX + "objcopy"
        self.sz = self.config.TOOLCHAIN_PREFIX + "size"

        self._collect_sources_and_includes()
        self.has_cpp = bool(self.c_sources or self.cpp_sources or self.asm_sources)
        self._construct_flags()

    def _ensure_tools_are_present(self):
        """Checks for required toolchains and downloads them if missing."""
        tools_dir = "tools"
        os.makedirs(tools_dir, exist_ok=True)

        # 1. Check for RISC-V GCC Toolchain.
        # Using a specific version (v14.2.0-3) ensures a consistent build environment.
        if not os.path.isdir(self.config.TOOLCHAIN_PATH):
            print("INFO: RISC-V GCC toolchain not found. Attempting to download and set up...")
            url = "https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/v14.2.0-3/xpack-riscv-none-elf-gcc-14.2.0-3-win32-x64.zip"
            archive_path = os.path.join(tools_dir, "gcc.zip")
            _download_and_extract_tool(url=url, archive_path=archive_path, extract_dir=tools_dir, final_check_path=self.config.TOOLCHAIN_PATH)

        # 2. Check for OpenOCD.
        # Using a specific version (v0.12.0) ensures compatibility with the target and debugger.
        if not os.path.isfile(self.config.OPENOCD_PATH):
            print("INFO: OpenOCD not found. Attempting to download and set up...")
            url = "https://github.com/openocd-org/openocd/releases/download/v0.12.0/openocd-v0.12.0-i686-w64-mingw32.tar.gz"
            archive_path = os.path.join(tools_dir, 'openocd.tar.gz')
            # The tarball extracts to a folder named 'openocd-0.12.0'. We will rename it.
            extracted_folder_name = 'openocd-0.12.0'
            final_folder_name = os.path.basename(os.path.dirname(os.path.dirname(self.config.OPENOCD_PATH)))
            rename_map = {extracted_folder_name: final_folder_name}
            _download_and_extract_tool(url=url, archive_path=archive_path, extract_dir=tools_dir, final_check_path=self.config.OPENOCD_PATH, rename_map=rename_map)

    def _collect_sources_and_includes(self):
        """Iterates through components in config.py and collects all active source files and include paths."""
        self.c_sources = []
        self.cpp_sources = []
        self.asm_sources = []
        self.include_paths = []

        print("Analyzing project components...")
        for name, component in self.config.COMPONENTS.items():
            if component.get("enabled", False):
                print(f"  - Enabling component: {name}")
                module = component.get("module", self.project_name)

                # Prepend the project directory to all relative paths from the config
                self.c_sources.extend([os.path.join(module, p) for p in component.get("c_sources", [])])
                self.cpp_sources.extend([os.path.join(module, p) for p in component.get("cpp_sources", [])])
                self.asm_sources.extend([os.path.join(module, p) for p in component.get("asm_sources", [])])

                for inc_path in component.get("include_paths", []):
                    if inc_path.startswith("-I"):
                        # Reconstruct the include flag with the project path
                        path = inc_path[2:]                        
                        self.include_paths.append(f"-I{os.path.join(module, path)}")
                    else: # Fallback for paths not prefixed with -I, though current format uses it.
                        self.include_paths.append(f"-I{os.path.join(module, inc_path)}")
            else:
                print(f"  - Disabling component: {name}")
        
        self.is_cpp_project = bool(self.cpp_sources)
        self.include_paths = sorted(list(set(self.include_paths)))

    def _construct_flags(self):
        """Builds the final lists of CFLAGS, ASFLAGS, CPPFLAGS, and LDFLAGS."""
        # Add global C definitions to the base flags for all compiler invocations.
        base_flags = self.config.CPU_FLAGS + [
            self.config.OPTIMIZATION,
            "-Wall", # Enable all standard warnings.
            "-ffunction-sections", # Place each function in its own section.
            "-fdata-sections"
        ] + self.config.COMMON_WARNING_FLAGS + self.config.GLOBAL_C_DEFINES

        # C Flags
        self.cflags = base_flags + [self.config.C_STANDARD] + self.config.C_WARNING_FLAGS + self.include_paths + ["-MMD", "-MP"]
        if self.config.DEBUG_MODE:
            self.cflags.extend(["-g", "-gdwarf-2"])

        # C++ Flags
        self.cppflags = base_flags + [self.config.CPP_STANDARD] + self.config.CPP_WARNING_FLAGS + self.config.CPP_EMBEDDED_FLAGS + self.include_paths + ["-MMD", "-MP"]
        if self.config.DEBUG_MODE:
            self.cppflags.extend(["-g", "-gdwarf-2"])

        # Assembler Flags (Definitions are also needed here for C preprocessor directives in .S files)
        self.asflags = self.config.CPU_FLAGS + [self.config.OPTIMIZATION] + self.include_paths + self.config.GLOBAL_C_DEFINES

        # Linker Flags
        linker_script_path = self.config.LINKER_SCRIPT
        lto_flags = [f for f in self.config.COMMON_WARNING_FLAGS if f in ("-flto", "-fuse-linker-plugin")]
        self.ldflags = self.config.CPU_FLAGS + [
            self.config.OPTIMIZATION,
            "-nostartfiles",
            f"-T{linker_script_path}",
            "--specs=nano.specs", # Use newlib-nano for reduced code size.
            "-Xlinker", "--gc-sections", # Allow the linker to remove unused sections.
            f"-Wl,-Map={os.path.join(self.build_dir, self.config.TARGET_NAME)}.map"
        ] + lto_flags

    @staticmethod
    def run_command(cmd):
        """Executes a shell command, prints it, and exits on failure."""
        cmd_str = ' '.join([str(arg).replace('\\', '/') for arg in cmd])
        print(f"Executing: {cmd_str}")
        try:
            subprocess.run(cmd, check=True, shell=isinstance(cmd, str))
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            print(f"❌ Error: Command failed: {e}", file=sys.stderr)
            sys.exit(1)

    def clean(self):
        """Removes the build directory and cleans Cargo artifacts."""
        if self.variant in ("cpp", "all"):
            print(f"🧹 Cleaning C++ build directory: {self.build_dir}")
            if os.path.isdir(self.build_dir):
                shutil.rmtree(self.build_dir)
            print("C++ clean complete.")

        if self.variant in ("rust", "all") and self.has_rust:
            print(f"🧹 Cleaning Rust build artifacts for {self.project_name}...")
            try:
                subprocess.run(["cargo", "clean", "-p", self.project_name], check=False)
            except Exception:
                pass
            rust_dir = os.path.join(self.build_dir, "rust")
            if os.path.isdir(rust_dir):
                shutil.rmtree(rust_dir)
            print("Rust clean complete.")

    def _get_obj_path(self, src_file):
        """
        Generates the path for an object file inside the project's build directory,
        preserving the source file's relative directory structure without escaping.

        Examples:
            - src_file: "prj_usb_serial/src/main.cpp"
            - Returns: "build/prj_usb_serial/src/main.cpp.o"

            - src_file: "gd32/Firmware/RISCV/drivers/n200_func.c"
            - Returns: "build/prj_usb_serial/gd32/Firmware/RISCV/drivers/n200_func.c.o"

            - src_file: "lib/system/init.c"
            - Returns: "build/prj_usb_serial/lib/system/init.c.o"
        """
        norm_src = os.path.normpath(src_file)
        norm_proj = os.path.normpath(self.project_name)
        if norm_src.startswith(norm_proj + os.sep):
            rel_path = norm_src[len(norm_proj) + 1:]
        elif norm_src == norm_proj:
            rel_path = os.path.basename(norm_src)
        else:
            rel_path = norm_src
        return os.path.join(self.build_dir, rel_path + '.o')

    def _parse_dependencies(self, dep_file):
        """
        Parses a GCC-generated dependency file (.d) to extract all header dependencies.
        This is used for incremental build checks.
        """
        if not os.path.exists(dep_file):
            return []
        with open(dep_file, 'r') as f:
            content = f.read()
        return re.findall(r'([^\s\\]+\.(?:h|inc|hpp))', content)

    def _is_rebuild_needed(self, src_file, obj_file):
        """
        Checks if a source file needs to be recompiled.
        A rebuild is needed if:
        1. The object file does not exist.
        2. The source file is newer than the object file.
        3. Any of its header dependencies are newer than the object file.
        """
        if not os.path.exists(obj_file):
            return True

        obj_mtime = os.path.getmtime(obj_file)
        if os.path.getmtime(src_file) > obj_mtime:
            return True

        # For C/C++ files, check their header dependencies.
        if src_file.endswith((".c", ".cpp", ".cc", ".cxx")):
            dep_file = obj_file.replace('.o', '.d')
            if not os.path.exists(dep_file):
                return True

            dependencies = self._parse_dependencies(dep_file)
            for dep in dependencies:
                if os.path.exists(dep) and os.path.getmtime(dep) > obj_mtime:
                    return True
        return False

    def compile_sources(self):
        """Compiles all C, C++, and Assembly sources into object files in parallel, skipping unchanged files."""
        print("Analyzing sources...")
        object_files = []
        cpp_extensions = (".cpp", ".cc", ".cxx")

        all_sources = self.c_sources + self.cpp_sources + self.asm_sources
        tasks = []

        for src in all_sources:
            obj_path = self._get_obj_path(src)
            object_files.append(obj_path)

            if not self._is_rebuild_needed(src, obj_path):
                continue

            os.makedirs(os.path.dirname(obj_path), exist_ok=True)

            if src.endswith(".c"):
                cmd = [self.cc] + self.cflags + ["-c", src, "-o", obj_path]
            elif src.endswith(cpp_extensions):
                cmd = [self.cpp] + self.cppflags + ["-c", src, "-o", obj_path]
            else: # Assumes .S
                cmd = [self.asm, "-x", "assembler-with-cpp"] + self.asflags + ["-c", src, "-o", obj_path]
            
            tasks.append((src, cmd))

        if not tasks:
            print("  -> All object files are up to date.")
            return object_files

        print(f"Compiling {len(tasks)} source file(s) in parallel...")
        print_lock = threading.Lock()
        failures = []

        def _compile(item):
            s, c = item
            with print_lock:
                print(f"  [CC] {s}")
            try:
                proc = subprocess.run(c, capture_output=True, text=True)
                if proc.returncode != 0:
                    with print_lock:
                        print(f"❌ Error compiling {s}:", file=sys.stderr)
                        if proc.stdout:
                            print(proc.stdout, file=sys.stderr)
                        if proc.stderr:
                            print(proc.stderr, file=sys.stderr)
                    return s, proc.stderr
                elif proc.stderr:
                    with print_lock:
                        print(proc.stderr, file=sys.stderr)
            except Exception as e:
                with print_lock:
                    print(f"❌ Error compiling {s}: {e}", file=sys.stderr)
                return s, str(e)
            return None

        workers = min(os.cpu_count() or 4, len(tasks))
        with ThreadPoolExecutor(max_workers=workers) as executor:
            future_to_src = {executor.submit(_compile, t): t[0] for t in tasks}
            for future in as_completed(future_to_src):
                res = future.result()
                if res is not None:
                    failures.append(res)

        if failures:
            print(f"\n❌ Build failed: {len(failures)} file(s) failed to compile.", file=sys.stderr)
            sys.exit(1)

        return object_files

    def link_objects(self, object_files):
        """Links all compiled object files into a single .elf executable."""
        linker = self.cpp if self.is_cpp_project else self.cc
        print(f"Linking objects (using {os.path.basename(linker)})...")
        
        elf_path = os.path.join(self.build_dir, f"{self.config.TARGET_NAME}.elf")
        libs = []
        if self.is_cpp_project:
            libs.append("-lstdc++")
        libs.extend(self.config.LIBRARIES)

        cmd = [linker] + self.ldflags + object_files + ["-Wl,--start-group"] + libs + ["-Wl,--end-group", "-o", elf_path]
        self.run_command(cmd)

        print("Calculating size...")
        self.run_command([self.sz, elf_path])
        return elf_path

    def create_binaries(self, elf_path):
        """Creates .hex and .bin files from the .elf file for programming."""
        print("Creating final binaries...")
        hex_path = elf_path.replace(".elf", ".hex")
        bin_path = elf_path.replace(".elf", ".bin")
        self.run_command([self.cp, "-O", "ihex", elf_path, hex_path])
        self.run_command([self.cp, "-O", "binary", "-S", elf_path, bin_path])
        print(f"Successfully created binaries in {self.build_dir}/")

    def build_cpp(self):
        """Runs the C/C++ compilation and linking pipeline."""
        print(f"\n⚙️  Building C++ variant for {self.project_name}...")
        objects = self.compile_sources()
        elf_file = self.link_objects(objects)
        self.create_binaries(elf_file)

    def build_rust(self):
        """Compiles the Rust variant using Cargo and generates .hex and .bin binaries."""
        print(f"\n🦀 Building Rust variant for {self.project_name}...")
        cargo_toml = os.path.join(self.project_name, "Cargo.toml")
        if not os.path.isfile(cargo_toml):
            print(f"❌ Error: Cargo manifest not found at '{cargo_toml}'", file=sys.stderr)
            sys.exit(1)

        cargo_cmd = ["cargo", "build", "--release", "-p", self.project_name]
        print(f"Executing: {' '.join(cargo_cmd)}")
        try:
            subprocess.run(cargo_cmd, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            print(f"❌ Error: Cargo build failed: {e}", file=sys.stderr)
            sys.exit(1)

        rust_build_dir = os.path.join(self.build_dir, "rust")
        os.makedirs(rust_build_dir, exist_ok=True)

        possible_bins = [
            os.path.join("target", "riscv32imac-unknown-none-elf", "release", "firmware_rust"),
            os.path.join("target", "riscv32imac-unknown-none-elf", "release", self.project_name),
        ]
        target_bin = None
        for b in possible_bins:
            if os.path.isfile(b):
                target_bin = b
                break
            if os.path.isfile(b + ".exe"):
                target_bin = b + ".exe"
                break

        if not target_bin:
            print(f"❌ Error: Could not locate built Rust binary in target/riscv32imac-unknown-none-elf/release/", file=sys.stderr)
            sys.exit(1)

        dest_elf = os.path.join(rust_build_dir, "firmware_rust.elf")
        dest_hex = os.path.join(rust_build_dir, "firmware_rust.hex")
        dest_bin = os.path.join(rust_build_dir, "firmware_rust.bin")

        shutil.copyfile(target_bin, dest_elf)

        self.run_command([self.cp, "-O", "ihex", dest_elf, dest_hex])
        self.run_command([self.cp, "-O", "binary", "-S", dest_elf, dest_bin])

        print("Calculating Rust binary size...")
        self.run_command([self.sz, dest_elf])
        print(f"Successfully created Rust binaries in {rust_build_dir}/")

    def build_all(self):
        """Runs the build process for C++, Rust, or both variants depending on self.variant."""
        built_any = False
        if self.variant in ("cpp", "all"):
            if self.has_cpp:
                self.build_cpp()
                built_any = True
            elif self.variant == "cpp":
                print(f"❌ Error: Project '{self.project_name}' has no C/C++ sources configured.", file=sys.stderr)
                sys.exit(1)

        if self.variant in ("rust", "all"):
            if self.has_rust:
                self.build_rust()
                built_any = True
            elif self.variant == "rust":
                print(f"❌ Error: Project '{self.project_name}' has no Cargo.toml or Rust sources.", file=sys.stderr)
                sys.exit(1)

        if not built_any:
            print(f"❌ Error: No sources found to build for variant '{self.variant}'.", file=sys.stderr)
            sys.exit(1)

        print("\n✅ Build complete.")

    def get_hex_path(self):
        """Resolves the .hex file path based on the selected variant or timestamps."""
        cpp_hex = os.path.join(self.build_dir, f"{self.config.TARGET_NAME}.hex")
        rust_hex = os.path.join(self.build_dir, "rust", "firmware_rust.hex")

        if self.variant == "rust":
            target = rust_hex
        elif self.variant == "cpp":
            target = cpp_hex
        else:
            rust_exists = os.path.isfile(rust_hex)
            cpp_exists = os.path.isfile(cpp_hex)
            if rust_exists and cpp_exists:
                if os.path.getmtime(rust_hex) > os.path.getmtime(cpp_hex):
                    print("ℹ️ Auto-selected newer Rust firmware binary for programming.")
                    target = rust_hex
                else:
                    print("ℹ️ Auto-selected newer C++ firmware binary for programming.")
                    target = cpp_hex
            elif rust_exists:
                target = rust_hex
            else:
                target = cpp_hex

        full_path = os.path.join(os.getcwd(), target).replace('\\', '/')
        if not os.path.exists(full_path):
            print(f"❌ Error: '{full_path}' not found. Please run 'build' first.", file=sys.stderr)
            sys.exit(1)
        return full_path

    def get_bin_path(self):
        """Resolves the .bin file path based on the selected variant or timestamps."""
        cpp_bin = os.path.join(self.build_dir, f"{self.config.TARGET_NAME}.bin")
        rust_bin = os.path.join(self.build_dir, "rust", "firmware_rust.bin")

        if self.variant == "rust":
            target = rust_bin
        elif self.variant == "cpp":
            target = cpp_bin
        else:
            rust_exists = os.path.isfile(rust_bin)
            cpp_exists = os.path.isfile(cpp_bin)
            if rust_exists and cpp_exists:
                if os.path.getmtime(rust_bin) > os.path.getmtime(cpp_bin):
                    print("ℹ️ Auto-selected newer Rust firmware binary for programming.")
                    target = rust_bin
                else:
                    print("ℹ️ Auto-selected newer C++ firmware binary for programming.")
                    target = cpp_bin
            elif rust_exists:
                target = rust_bin
            else:
                target = cpp_bin

        full_path = os.path.join(os.getcwd(), target).replace('\\', '/')
        if not os.path.exists(full_path):
            print(f"❌ Error: '{full_path}' not found. Please run 'build' first.", file=sys.stderr)
            sys.exit(1)
        return full_path

    def program_dfu(self):
        """Programs the target using dfu-util."""
        bin_path = self.get_bin_path()
        print(f"🔌 Programming with dfu-util ({bin_path})...")
        cmd = [self.config.DFU_UTIL_PATH, "-a", "0", "-s", "0x08000000:leave", "-D", bin_path]
        self.run_command(cmd)
        print("✅ DFU Programming complete.")

    def debug_session(self):
        """Starts an interactive GDB debug session."""
        print("❌ Error: GDB debug session functionality is not yet implemented.", file=sys.stderr)
        sys.exit(1)

    def program_openocd(self):
        """Programs the target using a generic OpenOCD configuration."""
        hex_path = self.get_hex_path()
        print(f"🔌 Programming with OpenOCD ({hex_path})...")
        config_file = os.path.join("tools", "config", "openocd-sipeed-libusb.cfg")
        program_cmd = f'program "{hex_path}" verify; halt; reg pc 0x08000000; resume; shutdown'
        cmd = [self.config.OPENOCD_PATH, '-f', config_file, '-c', program_cmd]
        self.run_command(cmd)
        print("✅ OpenOCD Programming complete.")

    def program_openocd_nucleus(self):
        """Programs the target using the Nuclei-specific OpenOCD configuration."""
        hex_path = self.get_hex_path()
        print(f"🔌 Programming with Nuclei OpenOCD ({hex_path})...")
        board_cfg = os.path.join("tools", "config", "openocd_gd32vf103.cfg")
        program_cmd = f'program "{hex_path}" verify; reset; shutdown'
        cmd = [
            self.config.NUCLEUS_OPENOCD_PATH,
            '-f', board_cfg,
            '-c', "debug_level 1",
            '-c', "reset halt; flash protect 0 0 last off;",
            '-c', program_cmd
        ]
        self.run_command(cmd)
        print("✅ Nucleus OpenOCD Programming complete.")