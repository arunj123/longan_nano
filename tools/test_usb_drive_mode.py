#!/usr/bin/env python3
"""
Targeted Verification Script for Phase 5 Task 5.1:
Dedicated USB Drive Mode on Screen 6 for prj_current_monitor.
Connects to Linux testbed (192.168.0.63) over SSH and exercises:
1. Ensuring clean initial state (Screen 1 Hero Mode, USB HID 28e9:1234).
2. Switching to Screen 6 (Standby mode) via UART ('6').
3. Toggling Standby -> Active via simulated PA8 double-click ('K') or UART ('u').
4. Verifying USB re-enumeration as MSC (28e9:abd4) and block device (/dev/sdb1).
5. Verifying SCSI block read via sg_raw AND mounting /dev/sdb1 to read LOG_*.CSV files.
6. Unmounting /dev/sdb1 and exiting Active mode via simulated PA8 single-click ('k').
7. Verifying USB re-enumeration back to HID (28e9:1234).
8. Verifying automatic SD re-probe and log file rotation to next sequential LOG_XXXX.CSV.
9. Verifying resumption of 10 Hz USB HID telemetry streaming.
"""

import sys
import time
import json
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

REMOTE_SCRIPT = r"""
import sys
import time
import json
import struct
import serial
import subprocess
import glob
import os
import select

results = {}

def run_cmd(cmd):
    return subprocess.getoutput(cmd)

def log_step(name, status, details=""):
    results[name] = {"pass": bool(status), "details": str(details)}
    print(f"[{'PASS' if status else 'FAIL'}] {name}: {details}", flush=True)

# 1. Connect to UART0
try:
    ser = serial.Serial('/dev/ttyUSB1', 115200, timeout=0.4)
    ser.reset_input_buffer()
    log_step("open_uart", True, "Opened /dev/ttyUSB1 @ 115200")
except Exception as e:
    log_step("open_uart", False, str(e))
    print("FINAL_RESULTS:" + json.dumps(results), flush=True)
    sys.exit(1)

def send_cmd(cmd_str, expected_substrs, timeout=4.0):
    ser.reset_input_buffer()
    ser.write(cmd_str.encode())
    ser.flush()
    t0 = time.time()
    captured = []
    found = False
    while time.time() - t0 < timeout:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if line:
            captured.append(line)
            full = "\n".join(captured)
            if all(s in full for s in expected_substrs):
                found = True
                break
    return found, captured

# Ensure we start cleanly in Screen 1 (Hero Mode, HID)
send_cmd('1', ['[UI] Screen mode: 1: HERO'], timeout=3.0)
for _ in range(10):
    if "28e9:1234" in run_cmd("lsusb"):
        break
    time.sleep(0.5)

# Record initial active log file from session summary ('s')
ok_s0, lines_s0 = send_cmd('s', ['--- SESSION TELEMETRY SUMMARY ---', '[JSON]'], timeout=2.5)
initial_file = None
for l in lines_s0:
    if '[JSON] ' in l:
        try:
            d = json.loads(l.split('[JSON] ', 1)[1])
            initial_file = d.get('sd_file')
        except Exception:
            pass
log_step("baseline_hero_mode", ok_s0 and ("28e9:1234" in run_cmd("lsusb")), f"Initial file: {initial_file}")

# Step 1: Switch to Screen 6 (USB Drive Mode Standby)
ok, lines = send_cmd('6', ['[UI] Screen mode: 6: USB DRIVE (Mass Storage)'], timeout=2.5)
log_step("switch_to_screen_6", ok, " | ".join(lines))

# Verify Standby keeps HID 28e9:1234 active
time.sleep(0.4)
lsusb_out = run_cmd("lsusb")
hid_present = "28e9:1234" in lsusb_out
log_step("standby_hid_present", hid_present, "HID 28e9:1234 remains active in Standby")

# Step 2: Toggle Standby -> Active using simulated PA8 double-click ('K')
ok, lines = send_cmd('K', ['[USB-DRIVE] Detaching USB HID and initializing USB MSC...', 'USB Drive ACTIVE'], timeout=4.0)
log_step("double_click_to_active", ok, " | ".join(lines))

# Step 3: Wait for Linux xHCI to enumerate MSC device (28e9:abd4) and /dev/sdb
msc_found = False
for _ in range(12):
    time.sleep(0.5)
    if "28e9:abd4" in run_cmd("lsusb"):
        msc_found = True
        break
log_step("msc_enumeration", msc_found, "MSC 28e9:abd4 enumerated on Linux host")

# Step 4: Verify SCSI read access via sg_raw AND FAT32 filesystem mount
sg_ok = False
sg_details = ""
fs_ok = False
fs_details = ""

if msc_found:
    # Wait up to 3s for /dev/sg1 or /dev/sdb1 to appear
    for _ in range(6):
        if os.path.exists("/dev/sdb1"):
            break
        time.sleep(0.5)

    # Find Longan /dev/sg* device
    sg_dev = None
    for node in sorted(glob.glob("/dev/sg*")):
        out = run_cmd(f"echo arun | sudo -S timeout -k 2 3 sg_inq {node} 2>/dev/null")
        if "Longan" in out:
            sg_dev = node
            break

    if sg_dev:
        cmd = f"echo arun | sudo -S timeout -k 2 4 sg_raw -b -r 512 {sg_dev} 28 00 00 00 00 00 00 00 01 00"
        ret = subprocess.run(cmd, shell=True, capture_output=True)
        if ret.returncode == 0 and len(ret.stdout) == 512:
            sg_ok = True
            sg_details = f"Read 512B LBA 0 from {sg_dev} (MBR sig: {ret.stdout[510:512].hex()})"
        else:
            sg_details = f"sg_raw rc={ret.returncode}, len={len(ret.stdout)}"
    else:
        sg_details = "No /dev/sg* matched Longan"

    # Mount /dev/sdb1 read-only and verify LOG_*.CSV files exist
    if os.path.exists("/dev/sdb1"):
        run_cmd("echo arun | sudo -S mkdir -p /mnt/usbdrive_test")
        run_cmd("echo arun | sudo -S umount /mnt/usbdrive_test 2>/dev/null")
        mnt_rc = subprocess.call("echo arun | sudo -S timeout -k 2 5 mount -o ro /dev/sdb1 /mnt/usbdrive_test", shell=True)
        if mnt_rc == 0:
            csv_files = sorted(glob.glob("/mnt/usbdrive_test/LOG_*.CSV"))
            fs_ok = len(csv_files) > 0
            fs_details = f"Mounted /dev/sdb1 RO, found {len(csv_files)} CSV files (latest: {os.path.basename(csv_files[-1]) if csv_files else 'none'})"
            run_cmd("echo arun | sudo -S timeout -k 2 5 umount /mnt/usbdrive_test")
        else:
            fs_details = f"mount /dev/sdb1 failed with rc={mnt_rc}"
    else:
        fs_details = "/dev/sdb1 node not present"

log_step("scsi_block_read", sg_ok, sg_details)
log_step("fat32_host_mount", fs_ok, fs_details)

# Step 5: Single-click PA8 ('k') while in Active USB Drive mode
# Should automatically exit Active MSC mode, restore HID, re-probe SD, rotate log file, and advance to Screen 1 (Hero)
ok, lines = send_cmd('k', [
    '[USB-DRIVE] Detaching USB MSC and restoring USB HID...',
    'Resuming logging in new file: LOG_',
    '[BUTTON] Single-click: Exited USB Drive -> Advance Screen -> HERO'
], timeout=4.5)
log_step("single_click_auto_exit", ok, " | ".join(lines))

# Step 6: Wait for HID re-enumeration (28e9:1234)
hid_restored = False
for _ in range(12):
    time.sleep(0.5)
    if "28e9:1234" in run_cmd("lsusb"):
        hid_restored = True
        break
log_step("hid_restoration", hid_restored, "HID 28e9:1234 restored on Linux host")

# Step 7: Verify HID telemetry stream resumes (non-blocking read with select)
hid_telemetry_ok = False
hid_reading = {}
if hid_restored:
    time.sleep(0.8)
    for node in sorted(glob.glob('/dev/hidraw*'), reverse=True):
        try:
            fd = os.open(node, os.O_RDONLY | os.O_NONBLOCK)
            try:
                rlist, _, _ = select.select([fd], [], [], 0.5)
                if rlist:
                    d = os.read(fd, 64)
                    if len(d) >= 9 and d[0] == 0x01:
                        rid, v, i, p, flg, seq = struct.unpack('<BHhHBB', d[:9])
                        hid_reading = {'node': node, 'v_mv': v, 'i_tenth_ma': i, 'p_mw': p, 'seq': seq}
                        hid_telemetry_ok = True
                        break
            finally:
                os.close(fd)
        except Exception:
            pass
log_step("hid_telemetry_stream", hid_telemetry_ok, str(hid_reading))

# Step 8: Verify Session Summary ('s') shows rotated log file
ok_s1, lines_s1 = send_cmd('s', ['--- SESSION TELEMETRY SUMMARY ---', '[JSON]'], timeout=2.5)
rotated_file = None
for l in lines_s1:
    if '[JSON] ' in l:
        try:
            d = json.loads(l.split('[JSON] ', 1)[1])
            rotated_file = d.get('sd_file')
        except Exception:
            pass
file_rotated_ok = ok_s1 and (rotated_file is not None) and (rotated_file != initial_file)
log_step("file_rotation_verified", file_rotated_ok, f"{initial_file} -> {rotated_file}")

ser.close()
print("FINAL_RESULTS:" + json.dumps(results), flush=True)
"""

def main():
    print("=" * 70, flush=True)
    print(" Running Phase 5 Task 5.1 Hardware Verification: USB Drive Mode", flush=True)
    print(f" Target Testbed: {USER}@{HOST}", flush=True)
    print("=" * 70, flush=True)

    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        client.connect(HOST, username=USER, password=PASS, timeout=10)
    except Exception as e:
        print(f"[FAIL] SSH Connection Failed: {e}", flush=True)
        sys.exit(1)

    print("[1/2] Deploying test script to remote testbed...", flush=True)
    sftp = client.open_sftp()
    remote_path = "/home/arun/longan_nano_tools/test_usb_drive_mode.py"
    with sftp.file(remote_path, "w") as f:
        f.write(REMOTE_SCRIPT)
    sftp.close()

    print("[2/2] Executing remote test against GD32VF103 hardware...", flush=True)
    stdin, stdout, stderr = client.exec_command(f"python3 -u {remote_path}")

    output_lines = []
    raw_results = None
    for line in iter(stdout.readline, ""):
        line = line.strip()
        if not line:
            continue
        if line.startswith("FINAL_RESULTS:"):
            raw_results = line[len("FINAL_RESULTS:"):]
        else:
            print(line, flush=True)
            output_lines.append(line)

    err = stderr.read().decode().strip()
    client.close()

    if not raw_results:
        print("[FAIL] Test harness did not return final results JSON!", flush=True)
        if err:
            print(f"Stderr: {err}", flush=True)
        sys.exit(1)

    res = json.loads(raw_results)
    print("\n" + "=" * 70, flush=True)
    print(" SUMMARY OF TEST RESULTS", flush=True)
    print("=" * 70, flush=True)

    all_pass = True
    for test_name, test_data in res.items():
        p = test_data.get("pass", False)
        all_pass &= p
        status_str = "PASS" if p else "FAIL"
        print(f" [{status_str}] {test_name}: {test_data.get('details', '')}", flush=True)

    print("=" * 70, flush=True)
    if all_pass:
        print(">>> ALL TASK 5.1 TESTS PASSED SUCCESSFULLY! <<<", flush=True)
        sys.exit(0)
    else:
        print(">>> SOME TESTS FAILED! <<<", flush=True)
        sys.exit(1)

if __name__ == "__main__":
    main()
