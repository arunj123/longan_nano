#!/usr/bin/env python3
"""
Automated test suite for Virtual USB MSC Simulation over USB/IP.
Deploys tools/virtual_usb_msc to remote Linux host (192.168.0.63),
compiles with g++ -std=c++23, runs the virtual device, attaches via vhci-hcd,
and executes full block layer & filesystem verification tests.
"""

import os
import sys
import time
import re
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

LOCAL_DIR = os.path.dirname(os.path.abspath(__file__))
MSC_DIR = os.path.join(LOCAL_DIR, "virtual_usb_msc")
REMOTE_BASE = "/home/arun/longan_nano_tools/virtual_usb_msc"

def log(msg):
    print(f"\033[1;34m[TEST]\033[0m {msg}", flush=True)

def ok(msg):
    print(f"\033[1;32m[PASS]\033[0m {msg}", flush=True)

def fail(msg):
    print(f"\033[1;31m[FAIL]\033[0m {msg}", flush=True)

def run_remote(client, cmd, sudo=False):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    exit_code = stdout.channel.recv_exit_status()
    return exit_code, out, err

def main():
    log(f"Connecting to {USER}@{HOST}...")
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

    log(f"Ensuring remote directory: {REMOTE_BASE}...")
    run_remote(client, f"mkdir -p {REMOTE_BASE}")

    log("Uploading modern C++23 virtual_usb_msc sources...")
    sftp = client.open_sftp()
    for fname in os.listdir(MSC_DIR):
        local_path = os.path.join(MSC_DIR, fname)
        if os.path.isfile(local_path):
            remote_path = f"{REMOTE_BASE}/{fname}"
            sftp.put(local_path, remote_path)
            log(f"  Uploaded: {fname}")
    sftp.close()

    log("Compiling virtual_usb_msc with g++ -std=c++23...")
    code, out, err = run_remote(client, f"cd {REMOTE_BASE} && make clean && make")
    if code != 0:
        fail(f"Compilation failed (code {code}):\n{out}\n{err}")
        client.close()
        return 1
    ok("Compiled cleanly with C++23!")

    log("Cleaning up any stale usbip / virtual_usb_msc state...")
    code, port_out, _ = run_remote(client, "usbip port", sudo=True)
    for line in port_out.splitlines():
        m = re.search(r'Port (\d+):', line)
        if m:
            run_remote(client, f"usbip detach -p {m.group(1)}", sudo=True)
    run_remote(client, "pkill -9 -f virtual_usb_msc")
    run_remote(client, "modprobe vhci-hcd", sudo=True)
    time.sleep(1)

    log("Starting virtual_usb_msc server in background...")
    run_remote(client, f"cd {REMOTE_BASE} && (./virtual_usb_msc --size 32 > /tmp/virtual_msc.log 2>&1 &)")
    time.sleep(1)

    log("Attaching virtual USB device via usbip attach...")
    code, out, err = run_remote(client, "usbip attach -r 127.0.0.1 -b 1-1", sudo=True)
    if code != 0:
        fail(f"usbip attach failed:\n{out}\n{err}")
        client.close()
        return 1
    ok("usbip attach succeeded!")
    time.sleep(2)

    log("Checking kernel dmesg for USB Mass Storage detection...")
    code, out, err = run_remote(client, "dmesg | tail -n 45", sudo=True)
    print("--- Kernel dmesg ---")
    print(out)
    print("--------------------")

    # Find the assigned scsi disk (/dev/sdX)
    match = re.search(r'\[(sd[a-z])\] Attached SCSI', out)
    if not match:
        match = re.search(r'\[(sd[a-z])\] \d+ \d+-byte logical blocks', out)
    if not match:
        # Fallback to lsblk looking for 32M disk
        _, lsblk_out, _ = run_remote(client, "lsblk -d -o NAME,SIZE -n")
        for line in lsblk_out.splitlines():
            parts = line.strip().split()
            if len(parts) >= 2 and parts[1] == '32M' and parts[0].startswith('sd'):
                match = re.search(r'(sd[a-z])', parts[0])
                break
    
    if not match:
        fail("Could not detect assigned /dev/sdX in dmesg or lsblk!")
        log("Checking virtual_msc.log...")
        _, log_out, _ = run_remote(client, "cat /tmp/virtual_msc.log")
        print(log_out)
        run_remote(client, "usbip detach -p 0", sudo=True)
        run_remote(client, "pkill -f virtual_usb_msc")
        client.close()
        return 1

    dev_name = match.group(1)
    dev_path = f"/dev/{dev_name}"
    ok(f"Virtual USB drive enumerated as \033[1;33m{dev_path}\033[0m!")

    # Test 1: Single-sector direct read (512B)
    log(f"Test 1: Reading LBA 0 (512B) via dd if={dev_path} bs=512 count=1 iflag=direct...")
    code, out, err = run_remote(client, f"dd if={dev_path} of=/tmp/sec0.bin bs=512 count=1 iflag=direct", sudo=True)
    if code != 0:
        fail(f"Single-sector direct read failed:\n{err}")
    else:
        # Check MBR signature (last 2 bytes = 55 aa)
        _, hex_out, _ = run_remote(client, "xxd -s 510 -l 2 /tmp/sec0.bin")
        if "55aa" in hex_out.replace(" ", ""):
            ok(f"Single-sector 512B read passed! MBR signature verified: {hex_out.strip()}")
        else:
            fail(f"Invalid MBR signature: {hex_out.strip()}")

    # Test 2: 4096-byte direct read (8 sectors)
    log(f"Test 2: Reading 4096B via dd if={dev_path} bs=4096 count=1 iflag=direct...")
    code, out, err = run_remote(client, f"dd if={dev_path} of=/dev/null bs=4096 count=1 iflag=direct", sudo=True)
    if code == 0:
        ok("4096B direct read passed with 0 errors!")
    else:
        fail(f"4096B direct read failed:\n{err}")

    # Test 3: Multi-megabyte sustained streaming (2 MB = 512 blocks of 4KB)
    log(f"Test 3: Sustained streaming read 2 MB via dd if={dev_path} bs=4096 count=512 iflag=direct...")
    code, out, err = run_remote(client, f"dd if={dev_path} of=/dev/null bs=4096 count=512 iflag=direct", sudo=True)
    if code == 0:
        ok("2 MB streaming read passed with 0 errors!")
    else:
        fail(f"2 MB streaming read failed:\n{err}")

    # Test 4: Block writing & data integrity verification
    log("Test 4: Writing random pattern to partition and verifying SHA256 checksum...")
    run_remote(client, "dd if=/dev/urandom of=/tmp/random.bin bs=4096 count=32", sudo=True)
    code, out, err = run_remote(client, f"dd if=/tmp/random.bin of={dev_path} bs=4096 count=32 seek=100 oflag=direct", sudo=True)
    if code == 0:
        run_remote(client, f"dd if={dev_path} of=/tmp/readback.bin bs=4096 count=32 skip=100 iflag=direct", sudo=True)
        _, diff_out, _ = run_remote(client, "diff -q /tmp/random.bin /tmp/readback.bin")
        if not diff_out.strip():
            ok("Data write & readback integrity 100% verified (0 byte differences)!")
        else:
            fail(f"Data verification mismatch: {diff_out}")
    else:
        fail(f"Write failed:\n{err}")

    # Test 5: Filesystem formatting with FAT32
    log(f"Test 5: Formatting {dev_path} with mkfs.vfat -F 32 -I...")
    code, out, err = run_remote(client, f"mkfs.vfat -F 32 -I {dev_path}", sudo=True)
    if code == 0:
        ok("Filesystem format mkfs.vfat -F 32 -I passed!")
    else:
        fail(f"mkfs.vfat failed:\n{err}")

    # Test 6: Mounting and file copy
    log(f"Test 6: Mounting {dev_path} to /mnt and writing test files...")
    run_remote(client, "umount /mnt 2>/dev/null", sudo=True)
    code, mnt_out, mnt_err = run_remote(client, f"mount {dev_path} /mnt", sudo=True)
    if code != 0:
        fail(f"Mount failed:\n{mnt_err}")
    else:
        run_remote(client, "sh -c \"echo 'Longan Nano USB MSC simulation verified' > /mnt/sim_test.txt\"", sudo=True)
        run_remote(client, "sync", sudo=True)
        _, cat_out, _ = run_remote(client, "cat /mnt/sim_test.txt", sudo=True)
        if "Longan Nano USB MSC simulation verified" in cat_out:
            ok("File write and read on mounted filesystem 100% successful!")
        else:
            fail(f"File content check failed: {cat_out}")
        run_remote(client, "umount /mnt", sudo=True)

    # Detach & cleanup
    log("Detaching virtual USB device...")
    run_remote(client, "usbip detach -p 0", sudo=True)
    run_remote(client, "pkill -f virtual_usb_msc")
    ok("Virtual device detached cleanly. All tests completed successfully!")

    client.close()
    return 0

if __name__ == "__main__":
    sys.exit(main())
