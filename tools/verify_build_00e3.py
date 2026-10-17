import paramiko
import time
import sys
import re

sys.stdout.reconfigure(encoding='utf-8')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

def run(cmd, sudo=False, timeout=10):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S timeout {timeout} bash -c { repr(cmd) }"
    else:
        cmd = f"timeout {timeout} bash -c { repr(cmd) }"
    stdin, stdout, stderr = client.exec_command(cmd, timeout=timeout+5)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

def uart_cmd(cmd_char):
    script = f"""import serial, time
s = serial.Serial('/dev/ttyUSB1', 115200, timeout=0.5)
s.reset_input_buffer()
s.write(b'{cmd_char}')
s.flush()
time.sleep(0.3)
out = s.read(s.in_waiting or 300).decode('utf-8', errors='replace')
print(out.strip())
s.close()
"""
    sftp = client.open_sftp()
    with sftp.file('/tmp/uart_cmd.py', 'w') as f:
        f.write(script)
    sftp.close()
    code, out, err = run("python3 /tmp/uart_cmd.py", sudo=True, timeout=5)
    return out.strip()

print("=== 1. UNSTICK USB PORT & WAIT FOR ENUMERATION ===", flush=True)
run("echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable", sudo=True, timeout=10)
time.sleep(3.5)

print("=== 2. LSUSB ===", flush=True)
_, out, _ = run("lsusb | grep 28e9")
print(out.strip(), flush=True)

print("=== 3. DMESG ===", flush=True)
_, dmesg_out, _ = run("dmesg | tail -n 25", sudo=True)
print(dmesg_out.strip(), flush=True)

# Find active SCSI disk name
_, dev_out, _ = run("ls -d /sys/block/sd* 2>/dev/null")
sd_dev = None
for d in dev_out.strip().split():
    d_name = d.split('/')[-1]
    _, model_out, _ = run(f"cat /sys/block/{d_name}/device/model 2>/dev/null")
    if "Longan" in model_out:
        sd_dev = f"/dev/{d_name}"
        break

if not sd_dev:
    m = re.findall(r'\[(sd[a-z])\] Attached SCSI', dmesg_out)
    if m:
        sd_dev = f"/dev/{m[-1]}"
    else:
        sd_dev = "/dev/sdb"

print(f"Detected Target SCSI Block Device: {sd_dev}", flush=True)

print("=== 4. VERIFY SERIAL NUMBER (BUILD 00E3) ===", flush=True)
_, udev_out, _ = run(f"udevadm info --name={sd_dev} | grep ID_SERIAL")
print(udev_out.strip(), flush=True)
if "LNMSC00000E3" in udev_out or "LNMSC00000E3" in dmesg_out:
    print("[PASS] Serial LNMSC00000E3 verified in device attributes!", flush=True)
else:
    print("[WARN] Serial check continued", flush=True)

print("=== 5. TEST UART0 WRITE-PROTECT & DISPLAY COMMANDS VIA /dev/ttyUSB1 ===", flush=True)
print("1. Query Help ('?'):")
h_out = uart_cmd('?')
print(h_out)
print("2. Initial Status ('s'):")
s_out = uart_cmd('s')
print(s_out)
print("3. Toggle Write-Protect via UART ('w'):")
w_out = uart_cmd('w')
print(w_out)
print("4. Verify Status after Toggle ('s'):")
s2_out = uart_cmd('s')
print(s2_out)
print("5. Simulate PA8 Double-click ('K'):")
k_out = uart_cmd('K')
print(k_out)
print("6. Verify Status after Simulated Double-Click ('s'):")
s3_out = uart_cmd('s')
print(s3_out)
print("7. Simulate PA8 Single-click ('k'):")
sk_out = uart_cmd('k')
print(sk_out)
print("8. Verify Rotation Status after Simulated Single-Click ('s'):")
s4_out = uart_cmd('s')
print(s4_out)
print("9. Restore Rotation via Simulated Single-Click ('k'):")
sk2_out = uart_cmd('k')
print(sk2_out)
print("10. Verify Restored Rotation ('s'):")
s5_out = uart_cmd('s')
print(s5_out)

print("=== 6. SCSI COMMANDS IN READ-WRITE MODE ===", flush=True)
# Ensure RW mode
st = uart_cmd('s')
if "RO" in st:
    uart_cmd('w')
time.sleep(0.3)

# Test TEST_UNIT_READY (00 00 00 00 00 00)
code, out, err = run(f"sg_raw --timeout=5 -r 0 {sd_dev} 00 00 00 00 00 00", sudo=True)
print(f"[{'PASS' if code == 0 else 'FAIL'}] TEST_UNIT_READY -> exit_code={code}", flush=True)

# Test MODE_SENSE_6 in RW mode (byte 2 bit 7 must be 0)
code, out, err = run(f"sg_raw --timeout=5 -r 4 {sd_dev} 1a 00 00 00 04 00", sudo=True)
combined_ms6 = out + "\n" + err
print(f"MODE_SENSE_6 (RW mode): exit_code={code}, data=\n{combined_ms6.strip()}", flush=True)
m = re.search(r'00\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})', combined_ms6)
if m:
    bytes_ms6 = [int(m.group(i), 16) for i in range(1, 5)]
    print(f"Parsed MODE_SENSE_6 bytes: {[hex(b) for b in bytes_ms6]}", flush=True)
    if (bytes_ms6[2] & 0x80) == 0:
        print("[PASS] MODE_SENSE_6 WP bit is 0 in RW mode!", flush=True)
    else:
        print(f"[FAIL] MODE_SENSE_6 WP bit is 1 in RW mode: {hex(bytes_ms6[2])}", flush=True)
else:
    print("[FAIL] Could not parse MODE_SENSE_6 bytes", flush=True)

# Test MODE_SENSE_10 in RW mode (byte 3 bit 7 must be 0)
code, out, err = run(f"sg_raw --timeout=5 -r 8 {sd_dev} 5a 00 00 00 00 00 00 00 08 00", sudo=True)
combined_ms10 = out + "\n" + err
print(f"MODE_SENSE_10 (RW mode): exit_code={code}, data=\n{combined_ms10.strip()}", flush=True)
m10 = re.search(r'00\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})', combined_ms10)
if m10:
    bytes_ms10 = [int(m10.group(i), 16) for i in range(1, 9)]
    print(f"Parsed MODE_SENSE_10 bytes: {[hex(b) for b in bytes_ms10]}", flush=True)
    if (bytes_ms10[3] & 0x80) == 0:
        print("[PASS] MODE_SENSE_10 WP bit is 0 in RW mode!", flush=True)
    else:
        print(f"[FAIL] MODE_SENSE_10 WP bit is 1 in RW mode: {hex(bytes_ms10[3])}", flush=True)
else:
    print("[FAIL] Could not parse MODE_SENSE_10 bytes", flush=True)

print("=== 7. SWITCH TO READ-ONLY MODE & VERIFY WP PROTECTION ===", flush=True)
# Toggle to RO via UART 'w'
w_res = uart_cmd('w')
print(f"Toggled WP via UART: {w_res}", flush=True)
time.sleep(0.3)

# Test MODE_SENSE_6 in RO mode (byte 2 bit 7 must be 1 -> 0x80)
code, out, err = run(f"sg_raw --timeout=5 -r 4 {sd_dev} 1a 00 00 00 04 00", sudo=True)
combined_ro6 = out + "\n" + err
print(f"MODE_SENSE_6 (RO mode): exit_code={code}, data=\n{combined_ro6.strip()}", flush=True)
m_ro6 = re.search(r'00\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})', combined_ro6)
if m_ro6:
    bytes_ro6 = [int(m_ro6.group(i), 16) for i in range(1, 5)]
    print(f"Parsed MODE_SENSE_6 bytes: {[hex(b) for b in bytes_ro6]}", flush=True)
    if (bytes_ro6[2] & 0x80) == 0x80:
        print("[PASS] MODE_SENSE_6 WP bit is 1 (0x80) in RO mode!", flush=True)
    else:
        print(f"[FAIL] MODE_SENSE_6 WP bit expected 0x80, got {hex(bytes_ro6[2])}", flush=True)
else:
    print("[FAIL] Could not parse MODE_SENSE_6 RO bytes", flush=True)

# Test MODE_SENSE_10 in RO mode (byte 3 bit 7 must be 1 -> 0x80)
code, out, err = run(f"sg_raw --timeout=5 -r 8 {sd_dev} 5a 00 00 00 00 00 00 00 08 00", sudo=True)
combined_ro10 = out + "\n" + err
print(f"MODE_SENSE_10 (RO mode): exit_code={code}, data=\n{combined_ro10.strip()}", flush=True)
m_ro10 = re.search(r'00\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})\s+([0-9a-fA-F]{2})', combined_ro10)
if m_ro10:
    bytes_ro10 = [int(m_ro10.group(i), 16) for i in range(1, 9)]
    print(f"Parsed MODE_SENSE_10 bytes: {[hex(b) for b in bytes_ro10]}", flush=True)
    if (bytes_ro10[3] & 0x80) == 0x80:
        print("[PASS] MODE_SENSE_10 WP bit is 1 (0x80) in RO mode!", flush=True)
    else:
        print(f"[FAIL] MODE_SENSE_10 WP bit expected 0x80, got {hex(bytes_ro10[3])}", flush=True)
else:
    print("[FAIL] Could not parse MODE_SENSE_10 RO bytes", flush=True)

# Test WRITE_10 rejection in RO mode (LBA 1000, 1 block)
run("dd if=/dev/zero of=/tmp/zero_512.bin bs=512 count=1 2>/dev/null", sudo=True)
code, out, err = run(f"sg_raw --timeout=5 -s 512 -i /tmp/zero_512.bin {sd_dev} 2a 00 00 00 03 e8 00 00 01 00", sudo=True)
combined_w = out + "\n" + err
print(f"WRITE_10 rejection in RO mode -> exit_code={code}, output:\n{combined_w.strip()}", flush=True)
if code != 0 or "Check Condition" in combined_w:
    print("[PASS] WRITE_10 was correctly rejected in Read-Only mode!", flush=True)
else:
    print("[FAIL] WRITE_10 was NOT rejected in Read-Only mode!", flush=True)

if "Sense key: Data Protect" in combined_w and "Write protected" in combined_w:
    print("[PASS] SCSI Sense key: Data Protect, Additional sense: Write protected verified!", flush=True)
else:
    # Query manual REQUEST_SENSE if not already fetched
    code, out, err = run(f"sg_raw --timeout=5 -r 18 {sd_dev} 03 00 00 00 12 00", sudo=True)
    combined_sense = out + "\n" + err
    print(f"REQUEST_SENSE fallback data:\n{combined_sense.strip()}", flush=True)

print("=== 8. SWITCH BACK TO RW MODE & VERIFY READ/WRITE OPERATION ===", flush=True)
# Toggle back to RW via UART 'w'
w_res2 = uart_cmd('w')
print(f"Toggled back to RW via UART: {w_res2}", flush=True)
time.sleep(0.3)

# Verify isolated SG_RAW reads
tests = [
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512, True),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096, True),
    ("LBA 528 (8 blks)", "28 00 00 00 02 10 00 00 08 00", 4096, True),
    ("LBA 560 (8 blks)", "28 00 00 00 02 30 00 00 08 00", 4096, True),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512, True),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096, True),
    ("OOB LBA 0xFFFFFFF0", "28 00 FF FF FF F0 00 00 20 00", 512, False),
]

all_passed = True
for label, cdb, rlen, expect_pass in tests:
    time.sleep(0.15)
    run("rm -f /tmp/out.bin /tmp/err.txt", sudo=True)
    cmd = f"sg_raw --timeout=5 -r {rlen} -b {sd_dev} {cdb} > /tmp/out.bin 2> /tmp/err.txt"
    code, _, _ = run(cmd, sudo=True, timeout=8)
    _, sz, _ = run("stat -c %s /tmp/out.bin 2>/dev/null", sudo=True)
    sz_val = sz.strip() if sz.strip().isdigit() else "0"
    passed = ((code == 0) == expect_pass)
    if not passed:
        all_passed = False
    status_str = "PASS" if passed else "FAIL"
    print(f"[{status_str}] {label:20} -> exit_code={code}, size={sz_val:>5} bytes (expected {rlen if expect_pass else 0})", flush=True)

print("=== 9. SUSTAINED 4.0 MiB READ STREAMING ===", flush=True)
time.sleep(0.2)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=64k count=64 iflag=direct 2>&1", sudo=True, timeout=15)
print(out.strip(), flush=True)

print("=== 10. MOUNT VFAT PARTITION 1 & TEST RW FILE I/O ===", flush=True)
run("mkdir -p /mnt/sdcard", sudo=True)
run("umount -f /mnt/sdcard 2>/dev/null", sudo=True)
code, out, err = run(f"mount -t vfat {sd_dev}1 /mnt/sdcard", sudo=True, timeout=10)
print(f"Mount exit code: {code}", flush=True)
if code == 0:
    print("[PASS] Successfully mounted partition 1 read-write!", flush=True)

    print("=== 11. SINGLE-FILE WRITE & VERIFICATION ===", flush=True)
    test_msg = "Hello Build 00E3 - Non-Breaking Stream Cache Lookup Policy verified!"
    code, _, _ = run(f"echo '{test_msg}' > /mnt/sdcard/build_00e3.txt && sync", sudo=True)
    code_cat, cat_out, _ = run("cat /mnt/sdcard/build_00e3.txt", sudo=True)
    match_ok = (cat_out.strip() == test_msg.strip())
    print(f"Write round-trip match: {match_ok} (Content: '{cat_out.strip()}')", flush=True)

    print("=== 12. MULTI-BLOCK 128 KiB WRITE STREAMING (CMD25) ===", flush=True)
    run("dd if=/dev/urandom of=/tmp/rand_128k.bin bs=64k count=2 2>/dev/null", sudo=True)
    _, md5_orig, _ = run("md5sum /tmp/rand_128k.bin", sudo=True)
    md5_orig = md5_orig.split()[0] if md5_orig else ""

    code_w, dd_w_out, _ = run("dd if=/tmp/rand_128k.bin of=/mnt/sdcard/stream_128k.bin bs=64k count=2 conv=fsync 2>&1", sudo=True, timeout=15)
    print(f"128 KiB Write streaming output:\n{dd_w_out.strip()}", flush=True)

    _, md5_card, _ = run("md5sum /mnt/sdcard/stream_128k.bin", sudo=True)
    md5_card = md5_card.split()[0] if md5_card else ""
    md5_match = (md5_orig == md5_card) and len(md5_orig) == 32
    print(f"Cryptographic MD5 Verification: {'PASS' if md5_match else 'FAIL'} (MD5: {md5_card})", flush=True)

    run("rm -f /mnt/sdcard/build_00e3.txt /mnt/sdcard/stream_128k.bin && sync", sudo=True)
    run("umount /mnt/sdcard", sudo=True)
    print("Cleanly unmounted /mnt/sdcard.", flush=True)
else:
    print(f"[FAIL] Failed to mount: {err}", flush=True)

print("=== 13. TEST BKP WRITE-PROTECT PERSISTENCE ACROSS REBOOT ===", flush=True)
# Set to RO mode in BKP6
w_ro = uart_cmd('w')
print("Toggled to RO mode:", w_ro, flush=True)
s_ro = uart_cmd('s')
print("Before reset status:", s_ro, flush=True)

# Re-trigger reset via OpenOCD
print("Re-triggering reset via OpenOCD...", flush=True)
run("openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c 'adapter speed 2000; init; halt; reg pc 0x08000000; resume; shutdown'", timeout=10)
time.sleep(2.5)

# Check status on boot
s_boot = uart_cmd('s')
print("After reset boot status:", s_boot, flush=True)
if "RO" in s_boot and "0x5701" in s_boot:
    print("[PASS] BKP Register 6 Write-Protect RO state persisted across reboot!", flush=True)
else:
    print(f"[FAIL] BKP Register 6 state not persisted: {s_boot}", flush=True)

# Restore to RW mode
w_rw = uart_cmd('w')
print("Toggled to RW mode:", w_rw, flush=True)
s_rw = uart_cmd('s')
print("Restored status:", s_rw, flush=True)

print("=== 14. FINAL DMESG HEALTH CHECK ===", flush=True)
_, final_dmesg, _ = run("dmesg | tail -n 20", sudo=True)
print(final_dmesg.strip(), flush=True)
if "reset high-speed USB device" in final_dmesg or "device descriptor read" in final_dmesg:
    print("[WARN] Kernel logged bus resets during tests", flush=True)
else:
    print("[PASS] Zero USB resets or bus dropouts during testing!", flush=True)

client.close()
print("Verification complete.", flush=True)
