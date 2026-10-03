import paramiko
import time

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect("192.168.0.63", username="arun", password="arun")

cmd = "echo arun | sudo -S bash -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'"
stdin, stdout, stderr = client.exec_command(cmd)
print("STDOUT:", stdout.read().decode())
print("STDERR:", stderr.read().decode())

time.sleep(1)
_, o, _ = client.exec_command("echo arun | sudo -S dmesg | tail -n 20")
print("DMESG:\n", o.read().decode())
client.close()
