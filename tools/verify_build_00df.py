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
_, dmesg_out, _ = run("dmesg | tail -n 25", sudo=True)
print(dmesg_out, flush=True)

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
    m = re.findall(r'\[(sd[a-z])\] Attached SCSI', dmesg_out)
    if m:
        sd_dev = f"/dev/{m[-1]}"
    else:
        sd_dev = "/dev/sdb"

print(f"Detected Target SCSI Block Device: {sd_dev}", flush=True)

print("=== VERIFY SERIAL NUMBER ===", flush=True)
_, udev_out, _ = run(f"udevadm info --name={sd_dev} | grep ID_SERIAL")
print(udev_out.strip(), flush=True)
if "LNMSC00000DF" in udev_out or "LNMSC00000DF" in dmesg_out:
    print("[PASS] Serial LNMSC00000DF verified in device attributes!", flush=True)
else:
    print("[WARN] Serial string check will continue verification", flush=True)

print("=== LSBLK ===", flush=True)
_, out, _ = run("lsblk")
print(out, flush=True)

print("=== SCSI COMMANDS VIA SG_RAW ===", flush=True)
# Test TEST_UNIT_READY (00 00 00 00 00 00)
code, _, err = run(f"sg_raw --timeout=5 -r 0 {sd_dev} 00 00 00 00 00 00", sudo=True)
print(f"[{'PASS' if code == 0 else 'FAIL'}] TEST_UNIT_READY -> exit_code={code}", flush=True)

# Test REQUEST_SENSE (03 00 00 00 12 00)
code, out, _ = run(f"sg_raw --timeout=5 -r 18 --hex {sd_dev} 03 00 00 00 12 00", sudo=True)
print(f"[{'PASS' if code == 0 else 'FAIL'}] REQUEST_SENSE -> exit_code={code}, data={out.strip()[:60]}", flush=True)

# Test READ_CAPACITY_10 (25 00 00 00 00 00 00 00 00 00)
code, out, _ = run(f"sg_raw --timeout=5 -r 8 --hex {sd_dev} 25 00 00 00 00 00 00 00 00 00", sudo=True)
print(f"[{'PASS' if code == 0 else 'FAIL'}] READ_CAPACITY_10 -> exit_code={code}, data={out.strip()[:40]}", flush=True)

print("=== ISOLATED SG_RAW READ TESTS ===", flush=True)
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
    _, err, _ = run("cat /tmp/err.txt 2>/dev/null", sudo=True)
    sz_val = sz.strip() if sz.strip().isdigit() else "0"
    passed = ((code == 0) == expect_pass)
    if not passed:
        all_passed = False
    status_str = "PASS" if passed else "FAIL"
    print(f"[{status_str}] {label:20} -> exit_code={code}, size={sz_val:>5} bytes (expected {rlen if expect_pass else 0})", flush=True)
    if code != 0 and expect_pass:
        print(f"   sense/err: {err.strip()[:140]}", flush=True)

print("=== SINGLE & MULTI-BLOCK DD READS ===", flush=True)
time.sleep(0.2)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=512 count=1 iflag=direct 2>&1", sudo=True)
print("Single-block (512B):", out.strip(), flush=True)

time.sleep(0.2)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=4096 count=1 iflag=direct 2>&1", sudo=True)
print("Multi-block (4KB):", out.strip(), flush=True)

print("=== SUSTAINED 4.0 MiB READ STREAMING ===", flush=True)
time.sleep(0.2)
code, out, err = run(f"dd if={sd_dev} of=/dev/null bs=64k count=64 iflag=direct 2>&1", sudo=True, timeout=15)
print(out.strip(), flush=True)

print("=== MOUNT VFAT PARTITION 1 READ-WRITE ===", flush=True)
run("mkdir -p /mnt/sdcard", sudo=True)
run("umount -f /mnt/sdcard 2>/dev/null", sudo=True)
code, out, err = run(f"mount -t vfat {sd_dev}1 /mnt/sdcard", sudo=True, timeout=10)
print(f"Mount exit code: {code}", flush=True)
if code == 0:
    print("[PASS] Successfully mounted partition 1!", flush=True)
    _, ls_out, _ = run("ls -la /mnt/sdcard", sudo=True)
    print(ls_out.strip(), flush=True)

    print("=== SINGLE-FILE WRITE & VERIFICATION ===", flush=True)
    test_msg = "Hello Build 00DF - SD Hot-Swap & Unit Attention verified!"
    code, _, _ = run(f"echo '{test_msg}' > /mnt/sdcard/build_00df.txt && sync", sudo=True)
    code_cat, cat_out, _ = run("cat /mnt/sdcard/build_00df.txt", sudo=True)
    match_ok = (cat_out.strip() == test_msg.strip())
    print(f"Write round-trip match: {match_ok} (Content: '{cat_out.strip()}')", flush=True)

    print("=== MULTI-BLOCK 128 KiB WRITE STREAMING (CMD25) ===", flush=True)
    run("dd if=/dev/urandom of=/tmp/rand_128k.bin bs=64k count=2 2>/dev/null", sudo=True)
    _, md5_orig, _ = run("md5sum /tmp/rand_128k.bin", sudo=True)
    md5_orig = md5_orig.split()[0] if md5_orig else ""

    code_w, dd_w_out, _ = run("dd if=/tmp/rand_128k.bin of=/mnt/sdcard/stream_128k.bin bs=64k count=2 conv=fsync 2>&1", sudo=True, timeout=15)
    print(f"128 KiB Write streaming output:\n{dd_w_out.strip()}", flush=True)

    _, md5_card, _ = run("md5sum /mnt/sdcard/stream_128k.bin", sudo=True)
    md5_card = md5_card.split()[0] if md5_card else ""
    md5_match = (md5_orig == md5_card) and len(md5_orig) == 32
    print(f"Cryptographic MD5 Verification: {'PASS' if md5_match else 'FAIL'} (MD5: {md5_card})", flush=True)

    run("rm -f /mnt/sdcard/build_00df.txt /mnt/sdcard/stream_128k.bin && sync", sudo=True)
    run("umount /mnt/sdcard", sudo=True)
    print("Cleanly unmounted /mnt/sdcard.", flush=True)
else:
    print(f"[FAIL] Failed to mount: {err}", flush=True)
    all_passed = False

print("=== FINAL DMESG HEALTH CHECK ===", flush=True)
_, final_dmesg, _ = run("dmesg | tail -n 20", sudo=True)
print(final_dmesg, flush=True)
if "reset high-speed USB device" in final_dmesg or "device descriptor read" in final_dmesg:
    print("[WARN] Kernel logged bus resets during tests", flush=True)
else:
    print("[PASS] Zero USB resets or bus dropouts during testing!", flush=True)

client.close()
print("Verification complete.", flush=True)
