import paramiko
import time
import sys

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

print("=== DMESG ===", flush=True)
_, out, _ = run("dmesg | tail -n 30", sudo=True)
print(out, flush=True)

print("=== LSBLK ===", flush=True)
_, out, _ = run("lsblk")
print(out, flush=True)

print("=== ISOLATED SG_RAW TESTS ===", flush=True)
tests = [
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096),
    ("LBA 528 (8 blks)", "28 00 00 00 02 10 00 00 08 00", 4096),
    ("LBA 560 (8 blks)", "28 00 00 00 02 30 00 00 08 00", 4096),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096),
]

for label, cdb, rlen in tests:
    time.sleep(0.2)
    cmd = f"sg_raw --timeout=5 -r {rlen} -b /dev/sdb {cdb} > /tmp/out.bin 2> /tmp/err.txt"
    code, _, _ = run(cmd, sudo=True, timeout=8)
    _, sz, _ = run("stat -c %s /tmp/out.bin", sudo=True)
    _, err, _ = run("cat /tmp/err.txt", sudo=True)
    print(f"{label:20} -> code={code}, size={sz.strip():>5} bytes (expected {rlen})", flush=True)
    if code != 0:
        print(f"   err: {err.strip()[:120]}", flush=True)

print("=== DD BS=512 COUNT=1 ===", flush=True)
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=512 count=1 iflag=direct 2>&1", sudo=True, timeout=8)
print(f"code: {code}\n{out}", flush=True)

print("=== DD BS=4096 COUNT=1 ===", flush=True)
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=4096 count=1 iflag=direct 2>&1", sudo=True, timeout=8)
print(f"code: {code}\n{out}", flush=True)

print("=== SUSTAINED STREAMING: DD BS=64K COUNT=64 (4 MiB) ===", flush=True)
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=64k count=64 iflag=direct 2>&1", sudo=True, timeout=30)
print(f"code: {code}\n{out}", flush=True)

print("=== PARTITION 1 MOUNT TEST ===", flush=True)
run("mkdir -p /mnt/sdcard", sudo=True)
run("umount /mnt/sdcard 2>/dev/null", sudo=True)
code, out, err = run("mount -t vfat -o ro /dev/sdb1 /mnt/sdcard 2>&1", sudo=True, timeout=10)
print(f"mount code: {code}\n{out}", flush=True)
if code == 0:
    _, ls_out, _ = run("ls -la /mnt/sdcard")
    print("Files on SD card:", flush=True)
    print(ls_out, flush=True)
    run("umount /mnt/sdcard", sudo=True)

print("=== FINAL DMESG TAIL ===", flush=True)
_, out, _ = run("dmesg | tail -n 20", sudo=True)
print(out, flush=True)

client.close()
