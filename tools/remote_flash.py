#!/usr/bin/env python3
"""
Remote OpenOCD programmer for Longan Nano over SSH to Linux host (192.168.0.63).
Transfers the given .hex binary and programs the GD32VF103 via FT2232 JTAG.
"""

import sys
import os
import paramiko

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    from ssh_util import HOST, USER, PASS, KEY_PATH, create_ssh_client
except ImportError:
    HOST = os.environ.get("LONGAN_SSH_HOST", "192.168.0.63")
    USER = os.environ.get("LONGAN_SSH_USER", "arun")
    PASS = os.environ.get("LONGAN_SSH_PASS", "arun")
    KEY_PATH = os.environ.get("LONGAN_SSH_KEY", None)
    def create_ssh_client(timeout=15, banner_timeout=30, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout, banner_timeout=banner_timeout)
        return client

REMOTE_DIR = "/home/arun/longan_nano_tools"
REMOTE_CFG = f"{REMOTE_DIR}/openocd-sipeed-libusb.cfg"

def flash_remote(hex_path):
    if not os.path.exists(hex_path):
        print(f"❌ Error: File not found: {hex_path}", file=sys.stderr)
        sys.exit(1)

    print(f"[REMOTE FLASH] Connecting to {USER}@{HOST}...")
    try:
        client = create_ssh_client(timeout=15, banner_timeout=30)
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
    program_cmd = f'adapter speed 2000; program "{remote_hex}" verify; halt; reg pc 0x08000000; resume; shutdown'
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
