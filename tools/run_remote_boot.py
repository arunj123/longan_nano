import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

sftp = client.open_sftp()
with sftp.file('/home/arun/longan_nano_tools/get_boot.py', 'w') as f:
    f.write('''import serial
import subprocess
import time

s = serial.Serial('/dev/ttyUSB1', 115200, timeout=0.05)
s.reset_input_buffer()

subprocess.run(["openocd", "-f", "/home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg", "-c", "adapter speed 2000; init; halt; reg pc 0x08000000; resume; shutdown"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

buf = b""
t_end = time.time() + 3.0
while time.time() < t_end:
    chunk = s.read(1024)
    if chunk:
        buf += chunk

print(buf.decode('latin1', errors='replace'))
''')
sftp.close()

stdin, stdout, stderr = client.exec_command("python3 /home/arun/longan_nano_tools/get_boot.py")
print("=== Boot Telemetry ===")
print(stdout.read().decode('utf-8', errors='replace'))
client.close()
