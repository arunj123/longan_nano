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

print("1. Unsticking USB port...", flush=True)
run("echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable", sudo=True, timeout=10)

print("2. Waiting 3.5 seconds for enumeration...", flush=True)
time.sleep(3.5)

print("=== LSUSB ===", flush=True)
_, out, _ = run("lsusb | grep 28e9")
print(out, flush=True)

print("=== DMESG ===", flush=True)
_, out, _ = run("dmesg | tail -n 25", sudo=True)
print(out, flush=True)

# Find active SCSI disk name from dmesg / sysfs
_, dev_out, _ = run("ls -d /sys/block/sd* 2>/dev/null")
sd_dev = None
for d in dev_out.strip().split():
    d_name = d.split('/')[-1]
    _, model_out, _ = run(f"cat /sys/block/{d_name}/device/model 2>/dev/null")
    if "Longan" in model_out:
        sd_dev = f"/dev/{d_name}"
        break

if not sd_dev:
    m = re.findall(r'\[(sd[a-z])\] Attached SCSI', out)
    if m:
        sd_dev = f"/dev/{m[-1]}"
    else:
        sd_dev = "/dev/sdb"

print(f"Detected Target SCSI Block Device: {sd_dev}", flush=True)

print("=== LSBLK ===", flush=True)
_, out, _ = run("lsblk")
print(out, flush=True)

print("=== ISOLATED SG_RAW READ TESTS ===", flush=True)
tests = [
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512, True),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096, True),
    ("LBA 528 (8 blks)", "28 00 00 00 02 10 00 00 08 00", 4096, True),
    ("LBA 560 (8 blks)", "28 00 00 00 02 30 00 00 08 00", 4096, True),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512, True),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096, True),
    # Out of bounds / overflow fuzz test: LBA 0xFFFFFFF0 + 32 blks (should be rejected cleanly with Check Condition)
    ("OOB LBA 0xFFFFFFF0", "28 00 FF FF FF F0 00 00 20 00", 512, False),
]

for label, cdb, rlen, expect_pass in tests:
    time.sleep(0.2)
    run("rm -f /tmp/out.bin /tmp/err.txt", sudo=True)
    cmd = f"sg_raw --timeout=5 -r {rlen} -b {sd_dev} {cdb} > /tmp/out.bin 2> /tmp/err.txt"
    code, _, _ = run(cmd, sudo=True, timeout=8)
    _, sz, _ = run("stat -c %s /tmp/out.bin 2>/dev/null", sudo=True)
    _, err, _ = run("cat /tmp/err.txt 2>/dev/null", sudo=True)
    sz_val = sz.strip() if sz.strip().isdigit() else "0"
    status_str = "PASS" if ((code == 0) == expect_pass) else "FAIL"
    print(f"[{status_str}] {label:20} -> exit_code={code}, size={sz_val:>5} bytes (expected {rlen if expect_pass else 0})", flush=True)
    if code != 0:
        print(f"   sense/err: {err.strip()[:140]}", flush=True)

print(f"\n=== DD BS=512 COUNT=1 on {sd_dev} ===", flush=True)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=512 count=1 iflag=direct 2>&1", sudo=True, timeout=8)
print(f"code: {code}\n{out}", flush=True)

print(f"=== DD BS=4096 COUNT=1 on {sd_dev} ===", flush=True)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=4096 count=1 iflag=direct 2>&1", sudo=True, timeout=8)
print(f"code: {code}\n{out}", flush=True)

print(f"=== SUSTAINED STREAMING: DD BS=64K COUNT=64 (4 MiB) on {sd_dev} ===", flush=True)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=64k count=64 iflag=direct 2>&1", sudo=True, timeout=30)
print(f"code: {code}\n{out}", flush=True)

part_dev = f"{sd_dev}1"
print(f"=== PARTITION 1 MOUNT & RW VERIFICATION on {part_dev} ===", flush=True)
run("mkdir -p /mnt/sdcard", sudo=True)
run("umount /mnt/sdcard 2>/dev/null", sudo=True)
code, out, err = run(f"mount -t vfat {part_dev} /mnt/sdcard 2>&1", sudo=True, timeout=10)
print(f"mount (RW) code: {code}\n{out}", flush=True)
if code == 0:
    _, ls_out, _ = run("ls -la /mnt/sdcard | head -n 15")
    print("Files on SD card (first 15):", flush=True)
    print(ls_out, flush=True)

    # Test writing a file through the new 32-bit FIFO write path
    test_str = "Build 00D8 32-bit FIFO write verification payload."
    print("Writing verification file to /mnt/sdcard/build_00d8.txt...", flush=True)
    w_code, _, _ = run(f"echo {repr(test_str)} > /mnt/sdcard/build_00d8.txt && sync", sudo=True, timeout=10)
    print(f"Write exit code: {w_code}", flush=True)

    r_code, read_back, _ = run("cat /mnt/sdcard/build_00d8.txt", sudo=True, timeout=10)
    print(f"Read back content: {read_back.strip()}", flush=True)
    if test_str in read_back:
        print("[PASS] 32-bit FIFO write + read round-trip verified successfully!", flush=True)
    else:
        print("[FAIL] Read back content mismatch!", flush=True)

    # Clean up test file
    run("rm -f /mnt/sdcard/build_00d8.txt && sync", sudo=True, timeout=10)
    run("umount /mnt/sdcard", sudo=True)

print("=== FINAL DMESG TAIL ===", flush=True)
_, out, _ = run("dmesg | tail -n 20", sudo=True)
print(out, flush=True)

client.close()
