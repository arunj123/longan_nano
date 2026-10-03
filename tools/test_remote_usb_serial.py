import paramiko
import sys

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect("192.168.0.63", username="arun", password="arun")

cmd = """python3 -c "import serial, time
s = serial.Serial('/dev/ttyACM1', 115200, timeout=2)
s.write(b'Hello GD32VF103!')
time.sleep(0.1)
recv = s.read(30)
print('ECHO RECV:', recv)
" """

stdin, stdout, stderr = client.exec_command(cmd)
print("STDOUT:", stdout.read().decode())
print("STDERR:", stderr.read().decode())
client.close()
