import paramiko
import time
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=10)

# Kill any lingering fdisk/dd/sg_raw
print("Killing lingering disk processes...", flush=True)
client.exec_command(f'echo {PASS} | sudo -S killall -9 fdisk dd sg_raw 2>/dev/null')
time.sleep(0.5)

# Cycle xHCI port disable via sysfs
print("Cycling xHCI root hub port 1...", flush=True)
unstick_cmd = 'echo 1 | sudo tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 | sudo tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'
_, stdout, stderr = client.exec_command(f'echo {PASS} | sudo -S sh -c "{unstick_cmd}"')
print("Unstick Output:", stdout.read().decode('utf-8', 'replace').strip(), flush=True)

time.sleep(2)
_, stdout, _ = client.exec_command(f'echo {PASS} | sudo -S dmesg | tail -n 20')
print("\nDMESG AFTER UNSTICK:", flush=True)
print(stdout.read().decode('utf-8', 'replace'), flush=True)

client.close()
