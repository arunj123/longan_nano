#!/usr/bin/env python3
"""
One-Click Launcher for Longan Nano Native 3D Current & Power Monitor Suite
Automatically detects hardware location:
  1. Local USB HID -> Launches native app with direct HID
  2. Remote Testbed (192.168.0.63) -> Launches local TCP bridge and connects native app
  3. No Hardware -> Launches native app in Simulation Mode
"""

import os
import sys
import time
import socket
import subprocess

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
_path_target_msvc = os.path.join(
    PROJECT_ROOT,
    "tools",
    "current_monitor_gui",
    "target",
    "x86_64-pc-windows-msvc",
    "release",
    "current_monitor_gui.exe"
)
_path_target_default = os.path.join(
    PROJECT_ROOT,
    "tools",
    "current_monitor_gui",
    "target",
    "release",
    "current_monitor_gui.exe"
)
EXE_PATH = _path_target_msvc if os.path.exists(_path_target_msvc) else _path_target_default

BRIDGE_SCRIPT = os.path.join(SCRIPT_DIR, "hid_tcp_bridge.py")

def check_local_hid():
    try:
        import hid
        devs = hid.enumerate(0x28E9, 0x1234)
        return len(devs) > 0
    except Exception:
        return False

def check_remote_host(host="192.168.0.63", port=22, timeout=1.5):
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(timeout)
        res = sock.connect_ex((host, port))
        sock.close()
        return res == 0
    except Exception:
        return False

def main():
    print("=" * 68)
    print(" ⚡ Longan Nano Native 3D Current & Power Monitor Launcher")
    print("=" * 68)

    if not os.path.exists(EXE_PATH):
        print(f"[!] Error: Executable not found at {EXE_PATH}")
        print("Please build it first: cd tools/current_monitor_gui && cargo build --release")
        sys.exit(1)

    bridge_proc = None
    args = [EXE_PATH]

    # Explicit CLI override
    if "--sim" in sys.argv:
        print("[*] Launching in Simulation Mode...")
        args.append("--sim")
    elif "--hid" in sys.argv:
        print("[*] Launching with Direct USB HID...")
        args.append("--hid")
    elif "--tcp" in sys.argv:
        idx = sys.argv.index("--tcp")
        addr = sys.argv[idx + 1] if idx + 1 < len(sys.argv) else "127.0.0.1:5055"
        print(f"[*] Launching with TCP bridge at {addr}...")
        args.extend(["--tcp", addr])
    else:
        # Auto-detect
        if check_local_hid():
            print("[+] Detected Longan Nano connected to Local USB! Using Direct HID.")
            args.append("--hid")
        elif check_remote_host():
            print("[+] Detected Remote Testbed at 192.168.0.63!")
            print("[*] Starting background SSH HID bridge (localhost:5055)...")
            bridge_proc = subprocess.Popen(
                [sys.executable, BRIDGE_SCRIPT],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )
            time.sleep(1.8) # Allow bridge to establish SSH channel
            args.extend(["--tcp", "127.0.0.1:5055"])
        else:
            print("[i] No hardware detected locally or on remote network.")
            print("[*] Launching in High-Performance Simulation Mode...")
            args.append("--sim")

    print(f"[*] Launching: {' '.join(args)}")
    try:
        app_proc = subprocess.run(args)
    finally:
        if bridge_proc:
            print("[*] Shutting down background bridge...")
            bridge_proc.terminate()
            try:
                bridge_proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                bridge_proc.kill()

    print("[*] Application session closed.")

if __name__ == "__main__":
    main()
