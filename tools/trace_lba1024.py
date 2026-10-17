import paramiko
import time

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

remote_code = """
import os, time, subprocess, select

pipe = subprocess.Popen(["cat", "/sys/kernel/debug/usb/usbmon/1u"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
time.sleep(0.1)

# Start sg_raw for LBA 1024
proc = subprocess.Popen(["sg_raw", "--timeout=2", "-r", "4096", "-b", "/dev/sdb", "28", "00", "00", "00", "04", "00", "00", "00", "08", "00"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

lines = []
t_end = time.time() + 2.5
while time.time() < t_end:
    r, _, _ = select.select([pipe.stdout], [], [], 0.05)
    if r:
        line = pipe.stdout.readline()
        if line:
            l = line.decode('utf-8', errors='replace').strip()
            if "Bi:1:" in l or "Bo:1:" in l:
                lines.append(l)

pipe.terminate()
try:
    pipe.wait(timeout=0.2)
except:
    pipe.kill()

try:
    proc.terminate()
    proc.wait(timeout=0.2)
except:
    proc.kill()

for l in lines:
    print(l)

out, err = proc.communicate()
print("sg_raw returncode:", proc.returncode)
print("sg_raw stderr:", err.decode('utf-8', errors='replace'))
"""

sftp = client.open_sftp()
with sftp.file('/home/arun/longan_nano_tools/trace_1024.py', 'w') as f:
    f.write(remote_code)
sftp.close()

stdin, stdout, stderr = client.exec_command("echo arun | sudo -S python3 /home/arun/longan_nano_tools/trace_1024.py")
print("=== Filtered USB Bulk Trace (LBA 1024) ===")
print(stdout.read().decode('utf-8', errors='replace'))
print("=== Stderr ===")
print(stderr.read().decode('utf-8', errors='replace'))
client.close()
