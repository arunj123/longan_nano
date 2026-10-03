import paramiko
import time

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect('192.168.0.63', username='arun', password='arun')

client2 = paramiko.SSHClient()
client2.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client2.connect('192.168.0.63', username='arun', password='arun')

stdin, stdout, stderr = client.exec_command('stty -F /dev/ttyUSB2 115200 raw -echo && cat /dev/ttyUSB2')
time.sleep(0.3)

client2.exec_command('openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "halt; reg pc 0x08000000; resume; shutdown"')

t0 = time.time()
while time.time() - t0 < 5:
    line = stdout.readline()
    if not line:
        break
    print(line.strip())

client.close()
client2.close()
