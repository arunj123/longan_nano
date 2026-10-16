import paramiko
import time

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

# We will use python on the remote host to capture usbmon for 2 seconds while resetting the USB port
remote_code = """
import os, time, subprocess, select

# Open usbmon
pipe = subprocess.Popen(["sudo", "cat", "/sys/kernel/debug/usb/usbmon/1u"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
time.sleep(0.2)

# Unstick / reset USB port
subprocess.run(["sudo", "sh", "-c", "echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 0.5; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable"])

# Read for 2.5 seconds
lines = []
t_end = time.time() + 2.5
while time.time() < t_end:
    r, _, _ = select.select([pipe.stdout], [], [], 0.1)
    if r:
        line = pipe.stdout.readline()
        if line:
            lines.append(line.decode('utf-8', errors='replace'))

pipe.terminate()
try:
    pipe.wait(timeout=0.5)
except:
    pipe.kill()

for l in lines:
    if "Bi:1:" in l or "Bo:1:" in l:
        print(l.strip())
"""

sftp = client.open_sftp()
with sftp.file('/home/arun/longan_nano_tools/trace.py', 'w') as f:
    f.write(remote_code)
sftp.close()

stdin, stdout, stderr = client.exec_command("echo arun | sudo -S python3 /home/arun/longan_nano_tools/trace.py")
print("=== Filtered USB Bulk Trace ===")
print(stdout.read().decode('utf-8', errors='replace'))
print("=== Stderr ===")
print(stderr.read().decode('utf-8', errors='replace'))
client.close()
