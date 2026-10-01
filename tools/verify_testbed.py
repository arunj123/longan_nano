#!/usr/bin/env python3
"""
Testbed Verification Script:
Flashes selected firmware projects and verifies UART logging and USB enumeration.
"""

import sys
import os
import time
import paramiko

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"
REMOTE_DIR = "/home/arun/longan_nano_tools"
REMOTE_CFG = f"{REMOTE_DIR}/openocd-sipeed-libusb.cfg"

def get_ssh():
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=10)
    return client

def flash_hex(hex_path):
    if not os.path.exists(hex_path):
        print(f"❌ Error: {hex_path} does not exist!")
        return False

    client = get_ssh()
    sftp = client.open_sftp()
    remote_hex = f"{REMOTE_DIR}/test_firmware.hex"
    sftp.put(hex_path, remote_hex)
    sftp.close()

    program_cmd = f'program "{remote_hex}" verify; halt; reg pc 0x08000000; resume; shutdown'
    full_cmd = f"openocd -f {REMOTE_CFG} -c '{program_cmd}'"
    stdin, stdout, stderr = client.exec_command(full_cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    client.close()

    if code == 0 and "Verified OK" in (out + err):
        print(f"  [FLASH] Programmed & Verified successfully!")
        return True
    else:
        print(f"  [FLASH] FAILED with code {code}!")
        print(out + err)
        return False

def setup_capture_script(client, duration=6.0):
    remote_script = f"""
import serial, time, glob, os
candidates = [p for p in sorted(glob.glob('/dev/ttyUSB*'), reverse=True) if p != '/dev/ttyUSB0']
port = candidates[0] if candidates else '/dev/ttyUSB1'
ser = serial.Serial(port, 115200, timeout=0.1)
t0 = time.time()
with open('/tmp/uart_capture.log', 'w', encoding='utf-8', errors='replace') as out:
    while time.time() - t0 < {duration}:
        line = ser.readline()
        if line:
            try:
                txt = line.decode('utf-8', errors='replace').strip()
                if txt:
                    out.write(txt + '\\n')
                    out.flush()
            except Exception:
                pass
ser.close()
"""
    sftp = client.open_sftp()
    with sftp.open(f"{REMOTE_DIR}/capture_uart.py", "w") as f:
        f.write(remote_script)
    sftp.close()

def check_usb_device():
    client = get_ssh()
    stdin, stdout, stderr = client.exec_command("lsusb")
    lsusb_out = stdout.read().decode('utf-8', errors='replace')
    stdin, stdout, stderr = client.exec_command("echo arun | sudo -S dmesg | tail -n 12")
    dmesg_out = stdout.read().decode('utf-8', errors='replace')
    client.close()
    return lsusb_out, dmesg_out

def verify_project(name, hex_path, check_usb=False, expected_vid_pid=None):
    print(f"\n=======================================================")
    print(f" Verifying: {name}")
    print(f" Hex: {hex_path}")
    print(f"=======================================================")

    client = get_ssh()
    setup_capture_script(client, duration=7.0)

    # Start background capture
    print("  [UART] Starting background UART listener...")
    client.exec_command(f"python3 {REMOTE_DIR}/capture_uart.py > /dev/null 2>&1 &")
    time.sleep(0.3)
    client.close()

    # Flash firmware
    if not flash_hex(hex_path):
        return False

    print("  [UART] Waiting for boot sequence and logs to collect...")
    time.sleep(3.5)

    client = get_ssh()
    stdin, stdout, stderr = client.exec_command("cat /tmp/uart_capture.log")
    logs = stdout.read().decode('utf-8', errors='replace').strip().splitlines()
    client.close()

    if logs:
        print(f"  [UART] Captured {len(logs)} log lines:")
        for line in logs[:20]:
            print(f"    | {line}")
        if len(logs) > 20:
            print(f"    | ... ({len(logs) - 20} more lines)")
    else:
        print("  [UART] No serial logs received.")

    if check_usb:
        time.sleep(1.0)
        lsusb_out, dmesg_out = check_usb_device()
        found = (expected_vid_pid.lower() in lsusb_out.lower()) if expected_vid_pid else ("28e9:" in lsusb_out)
        print(f"  [USB] Enumeration Check ({expected_vid_pid or '28e9:*'}): {'PASS' if found else 'NOT FOUND'}")
        if found:
            for line in lsusb_out.splitlines():
                if "28e9:" in line or (expected_vid_pid and expected_vid_pid in line):
                    print(f"    | {line}")
        print("  [USB] Kernel dmesg:")
        for line in dmesg_out.splitlines()[-6:]:
            print(f"    | {line}")

    return True

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "all"
    if target in ("uart", "all"):
        verify_project("prj_uart_test", "build/prj_uart_test/firmware.hex", check_usb=False)
    if target in ("serial", "all"):
        verify_project("prj_usb_serial", "build/prj_usb_serial/firmware.hex", check_usb=True, expected_vid_pid="28e9:018a")
    if target in ("sdcard", "all"):
        verify_project("prj_sdcard_test", "build/prj_sdcard_test/firmware.hex", check_usb=False)
    if target in ("fatfs", "all"):
        verify_project("prj_sdcard_fs_test", "build/prj_sdcard_fs_test/firmware.hex", check_usb=False)
    if target in ("composite", "all"):
        verify_project("prj_usb_composite", "build/prj_usb_composite/firmware.hex", check_usb=True, expected_vid_pid="28e9:abe8")
    if target in ("monitor", "all"):
        verify_project("prj_current_monitor", "build/prj_current_monitor/current_monitor.hex", check_usb=True, expected_vid_pid="28e9:1234")
    if target in ("lcd", "all"):
        verify_project("prj_lcd_test", "build/prj_lcd_test/firmware.hex", check_usb=False)
    if target in ("msc", "all"):
        verify_project("prj_usb_msc", "build/prj_usb_msc/firmware.hex", check_usb=True, expected_vid_pid="28e9:aba3")
