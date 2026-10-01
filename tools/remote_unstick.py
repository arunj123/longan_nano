import paramiko
import time

def unstick():
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect("192.168.0.63", username="arun", password="arun")

    unstick_cmd = (
        'echo arun | sudo -S bash -c "'
        'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; '
        'sleep 1; '
        'echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'
        '"'
    )
    stdin, stdout, stderr = client.exec_command(unstick_cmd)
    stdout.read()
    time.sleep(2)

    stdin, stdout, stderr = client.exec_command("echo arun | sudo -S dmesg | tail -n 25")
    print(stdout.read().decode('utf-8', errors='replace'))

    stdin, stdout, stderr = client.exec_command("lsusb")
    print("=== lsusb ===")
    print(stdout.read().decode('utf-8', errors='replace'))
    client.close()

if __name__ == "__main__":
    unstick()
