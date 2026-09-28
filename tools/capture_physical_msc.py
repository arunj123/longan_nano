import paramiko
import sys
import time

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

def run(cmd, sudo=False):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

# Find current sdb / device number
code, out, _ = run("lsblk | grep sdb")
print("lsblk output:", out.strip())

code, dmesg_out, _ = run("dmesg | grep 'New USB device found' | tail -n 1", sudo=True)
print("Latest device:", dmesg_out.strip())

# Start usbmon capture to /tmp/mon.txt
print("Starting usbmon background capture...")
run("rm -f /tmp/mon.txt")
client.exec_command(f"echo '{PASS}' | sudo -S cat /sys/kernel/debug/usb/usbmon/1u > /tmp/mon.txt")
time.sleep(0.5)

# Trigger 512-byte read
print("Executing dd if=/dev/sdb of=/dev/null bs=512 count=1 iflag=direct...")
run("dd if=/dev/sdb of=/dev/null bs=512 count=1 iflag=direct", sudo=True)
time.sleep(1.0)

# Stop usbmon
print("Stopping usbmon capture...")
run("pkill -9 -f 'cat /sys/kernel/debug/usb/usbmon/1u'", sudo=True)
time.sleep(0.5)

# Read capture
_, mon_out, _ = run("cat /tmp/mon.txt", sudo=True)
client.close()

# Filter for Bulk endpoint (1:xxx:1)
lines = [l for l in mon_out.splitlines() if ":1 " in l or " s " in l]
print(f"\nCaptured {len(lines)} usbmon lines:")
for l in lines[-40:]:
    print(l)
