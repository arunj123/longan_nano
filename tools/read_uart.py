import paramiko
import sys

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

cmd = """python3 -c "import serial; s = serial.Serial('/dev/ttyUSB1', 115200, timeout=1); print(s.read(2000).decode('latin1', errors='replace'))" """
stdin, stdout, stderr = client.exec_command(cmd)
print(stdout.read().decode('utf-8', errors='replace'))
client.close()
