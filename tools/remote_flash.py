#!/usr/bin/env python3
"""
Remote OpenOCD programmer for Longan Nano over SSH to Linux host (192.168.0.63).
Transfers the given .hex binary and programs the GD32VF103 via FT2232 JTAG.
"""

import sys
import os
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"
REMOTE_DIR = "/home/arun/longan_nano_tools"
REMOTE_CFG = f"{REMOTE_DIR}/openocd-sipeed-libusb.cfg"

def flash_remote(hex_path):
    if not os.path.exists(hex_path):
        print(f"❌ Error: File not found: {hex_path}", file=sys.stderr)
        sys.exit(1)

    print(f"[REMOTE FLASH] Connecting to {USER}@{HOST}...")
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15, banner_timeout=30)
    except Exception as e:
        print(f"❌ SSH connection failed: {e}", file=sys.stderr)
        sys.exit(1)

    filename = os.path.basename(hex_path)
    remote_hex = f"{REMOTE_DIR}/{filename}"

    print(f"[REMOTE FLASH] Uploading {filename} -> {remote_hex}...")
    sftp = client.open_sftp()
    sftp.put(hex_path, remote_hex)
    sftp.close()

    print(f"[REMOTE FLASH] Programming via OpenOCD...")
    program_cmd = f'program "{remote_hex}" verify; halt; reg pc 0x08000000; resume; shutdown'
    full_cmd = f"openocd -f {REMOTE_CFG} -c '{program_cmd}'"

    stdin, stdout, stderr = client.exec_command(full_cmd)
    exit_code = stdout.channel.recv_exit_status()
    out = stdout.read().decode()
    err = stderr.read().decode()

    if exit_code == 0 and "Verified OK" in (out + err):
        print("[OK] Flashing and verification successful!")
        print(out + err)
    else:
        print("[FAIL] Flashing failed:", file=sys.stderr)
        print(out, file=sys.stdout)
        print(err, file=sys.stderr)
        client.close()
        sys.exit(exit_code if exit_code != 0 else 1)

    client.close()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python tools/remote_flash.py <path_to_hex>")
        sys.exit(1)
    flash_remote(sys.argv[1])
