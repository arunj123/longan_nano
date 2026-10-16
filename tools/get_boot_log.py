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

remote_script = """
import serial, subprocess, time

s = serial.Serial('/dev/ttyUSB1', 115200, timeout=0.1)
s.reset_input_buffer()

# Trigger reset
subprocess.run(["openocd", "-f", "/home/arun/longan_nano_tools/openocd.cfg", "-c", "init; halt; reg pc 0x08000000; resume; shutdown"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

buf = b""
t_end = time.time() + 3.0
while time.time() < t_end:
    chunk = s.read(1024)
    if chunk:
        buf += chunk

print(buf.decode('latin1', errors='replace'))
"""

stdin, stdout, stderr = client.exec_command(f"python3 -c \"{remote_script}\"")
print("=== Target Boot Telemetry ===")
print(stdout.read().decode('utf-8', errors='replace'))
print("=== Stderr ===")
print(stderr.read().decode('utf-8', errors='replace'))

client.close()
