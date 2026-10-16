import paramiko
import time
import threading

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

# Start usbmon capture in background on host
run("echo '' | sudo tee /tmp/usbmon.log", sudo=True)
c_mon = paramiko.SSHClient()
c_mon.set_missing_host_key_policy(paramiko.AutoAddPolicy())
c_mon.connect(HOST, username=USER, password=PASS, timeout=10)
c_mon.exec_command(f"echo '{PASS}' | sudo -S cat /sys/kernel/debug/usb/usbmon/1u > /tmp/usbmon.log")

time.sleep(0.5)

# Test single block LBA 1024 (1 sector = 512B)
print("=== Testing LBA 1024: 1 block (512B) ===")
code, out, err = run("sg_raw -r 512 -b /dev/sdb 28 00 00 00 04 00 00 00 01 00 > /tmp/out512.bin 2>&1", sudo=True)
print(f"1 block exit code: {code}, output: {out.strip()}")

# Test 2 blocks LBA 1024 (2 sectors = 1024B)
print("=== Testing LBA 1024: 2 blocks (1024B) ===")
code, out, err = run("sg_raw -r 1024 -b /dev/sdb 28 00 00 00 04 00 00 00 02 00 > /tmp/out1024.bin 2>&1", sudo=True)
print(f"2 blocks exit code: {code}, output: {out.strip()}")

time.sleep(0.5)
c_mon.close()

# Read back usbmon log
code, out, err = run("cat /tmp/usbmon.log | grep -E 'Bi:1:001:1|Bo:1:001:1|Co:1:001:0|Ci:1:001:0' | tail -n 60", sudo=True)
print("--- usbmon trace (filtered) ---")
print(out)

code, dmesg_tail, _ = run("dmesg | tail -n 20", sudo=True)
print("--- dmesg tail ---")
print(dmesg_tail)

client.close()
