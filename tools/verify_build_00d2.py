import paramiko
import time

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

def run(cmd, sudo=False):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

print("1. Unsticking USB port...")
run("tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable <<< 1; sleep 1; tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable <<< 0", sudo=True)

print("2. Waiting 3 seconds for enumeration...")
time.sleep(3)

print("=== DMESG ===")
_, out, _ = run("dmesg | tail -n 30", sudo=True)
print(out)

print("=== LSBLK ===")
_, out, _ = run("lsblk")
print(out)

print("=== DD BS=512 COUNT=1 ===")
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=512 count=1 iflag=direct 2>&1", sudo=True)
print(f"code: {code}\n{out}")

print("=== DD BS=4096 COUNT=1 ===")
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=4096 count=1 iflag=direct 2>&1", sudo=True)
print(f"code: {code}\n{out}")

print("=== ISOLATED SG_RAW TESTS ===")
tests = [
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096),
    ("LBA 528 (8 blks)", "28 00 00 00 02 10 00 00 08 00", 4096),
    ("LBA 560 (8 blks)", "28 00 00 00 02 30 00 00 08 00", 4096),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096),
]

for label, cdb, rlen in tests:
    time.sleep(0.3)
    cmd = f"sg_raw --timeout=5 -r {rlen} -b /dev/sdb {cdb} > /tmp/out.bin 2> /tmp/err.txt"
    code, _, _ = run(cmd, sudo=True)
    _, sz, _ = run("stat -c %s /tmp/out.bin", sudo=True)
    _, err, _ = run("cat /tmp/err.txt", sudo=True)
    print(f"{label:20} -> code={code}, size={sz.strip():>5} bytes (expected {rlen})")
    if code != 0:
        print(f"   err: {err.strip()[:100]}")

print("=== SUSTAINED STREAMING: DD BS=64K COUNT=64 (4 MiB) ===")
code, out, err = run("dd if=/dev/sdb of=/dev/null bs=64k count=64 iflag=direct 2>&1", sudo=True)
print(f"code: {code}\n{out}")

print("=== PARTITION 1 MOUNT TEST ===")
run("mkdir -p /mnt/sdcard", sudo=True)
run("umount /mnt/sdcard 2>/dev/null", sudo=True)
code, out, err = run("mount -t vfat -o ro /dev/sdb1 /mnt/sdcard 2>&1", sudo=True)
print(f"mount code: {code}\n{out}")
if code == 0:
    _, ls_out, _ = run("ls -la /mnt/sdcard")
    print("Files on SD card:")
    print(ls_out)
    run("umount /mnt/sdcard", sudo=True)

client.close()
