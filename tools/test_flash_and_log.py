import paramiko
import time
import threading

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

def listen_uart():
    c2 = paramiko.SSHClient()
    c2.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    c2.connect(HOST, username=USER, password=PASS, timeout=10)
    cmd = "python3 -u -c \"import serial; s = serial.Serial('/dev/ttyUSB1', 115200); [print(s.readline().decode('latin1', errors='replace'), end='') for _ in range(50)]\""
    stdin, stdout, stderr = c2.exec_command(cmd)
    for line in stdout:
        print("[UART]", line.strip())
    c2.close()

t = threading.Thread(target=listen_uart, daemon=True)
t.start()
time.sleep(1)

# Reset board via openocd
openocd_cmd = "openocd -f /home/arun/longan_nano_tools/openocd.cfg -c 'init; halt; reg pc 0x08000000; resume; shutdown'"
stdin, stdout, stderr = client.exec_command(openocd_cmd)
stdout.read()

time.sleep(4)
client.close()
