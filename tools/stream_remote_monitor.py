#!/usr/bin/env python3
"""
Live streaming tool for Longan Nano INA219 Current Monitor over SSH.
Connects to 192.168.0.63 and streams live Voltage, Current, and Power telemetry.
"""

import sys
import time
import struct
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
    def create_ssh_client(timeout=10, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout)
        return client

def main():
    count = int(sys.argv[1]) if len(sys.argv) > 1 else 30
    print("=" * 65)
    print(f" Connecting to {USER}@{HOST} to stream INA219 telemetry...")
    print("=" * 65)

    try:
        client = create_ssh_client(timeout=10)
    except Exception as e:
        print(f"[FAIL] SSH connection failed: {e}")
        sys.exit(1)

    remote_script = f"""python3 -u -c '
import os, struct, sys, glob

fd = -1
for node in sorted(glob.glob("/dev/hidraw*"), reverse=True):
    try:
        cand = os.open(node, os.O_RDONLY)
        d = os.read(cand, 64)
        if d and len(d) >= 7 and d[0] == 0x01:
            fd = cand
            break
        os.close(cand)
    except Exception:
        pass

if fd < 0:
    print("STATUS:ERROR:No Longan Nano hidraw device found", flush=True)
    sys.exit(1)

try:
    print("STATUS:CONNECTED", flush=True)
    for i in range({count}):
        data = os.read(fd, 64)
        if data and len(data) >= 7 and data[0] == 0x01:
            v_mv, c_tenth, p_mw = struct.unpack("<HhH", data[1:7])
            print(f"DATA:{{i+1}}|{{v_mv}}|{{c_tenth}}|{{p_mw}}", flush=True)
    os.close(fd)
    print("STATUS:DONE", flush=True)
except Exception as e:
    print(f"STATUS:ERROR:{{e}}", flush=True)
'"""

    stdin, stdout, stderr = client.exec_command(remote_script)

    print(f"{'#':>4} | {'Voltage':>10} | {'Current':>10} | {'Power':>10} | {'Status'}")
    print("-" * 65)

    idx = 0
    for line in stdout:
        line = line.strip()
        if not line or line.startswith("[sudo]"):
            continue
        if line.startswith("DATA:"):
            parts = line[5:].split("|")
            sample_num = int(parts[0])
            v_mv = int(parts[1])
            c_tenth = int(parts[2])
            p_mw = int(parts[3])
            v_v = v_mv / 1000.0
            c_ma = c_tenth / 10.0
            print(f"{sample_num:>4} | {v_v:>8.3f} V | {c_ma:>7.1f} mA | {p_mw:>7.1f} mW | OK")
            sys.stdout.flush()
        elif line.startswith("STATUS:"):
            status = line[7:]
            if status == "CONNECTED":
                print(">>> Connected to /dev/hidraw0. Receiving 10 Hz USB HID packets...")
            elif status == "DONE":
                print("-" * 65)
                print(">>> Stream completed successfully.")
            elif status.startswith("ERROR"):
                print(f"[ERROR] Remote error: {status}")

    client.close()
    print("=" * 65)

if __name__ == "__main__":
    main()
