import paramiko
import time

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect("192.168.0.63", username="arun", password="arun")

# Start reading usbmon into a file on remote host
client.exec_command("echo arun | sudo -S bash -c 'cat /sys/kernel/debug/usb/usbmon/1u > /tmp/usbmon.log &'")
time.sleep(0.5)

# Reset port 1-1
client.exec_command("echo arun | sudo -S bash -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 0.5; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'")
time.sleep(2)

# Kill cat
client.exec_command("echo arun | sudo -S pkill -f 'cat /sys/kernel/debug/usb/usbmon/1u'")
time.sleep(0.5)

# Print log
_, o, _ = client.exec_command("cat /tmp/usbmon.log | head -n 40")
print("USBMON TRACE:\n", o.read().decode())
client.close()
