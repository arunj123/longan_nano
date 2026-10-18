import os
import sys
import paramiko
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    from ssh_util import HOST, USER, PASS, KEY_PATH, create_ssh_client
except ImportError:
    HOST = os.environ.get("LONGAN_SSH_HOST", "192.168.0.63")
    USER = os.environ.get("LONGAN_SSH_USER", "arun")
    PASS = os.environ.get("LONGAN_SSH_PASS", "arun")
    KEY_PATH = os.environ.get("LONGAN_SSH_KEY", None)
    def create_ssh_client(timeout=10, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout)
        return client

def unstick():
    client = create_ssh_client(timeout=10)

    unstick_cmd = (
        f'echo "{PASS}" | sudo -S bash -c "'
        'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; '
        'sleep 1; '
        'echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'
        '"'
    )
    stdin, stdout, stderr = client.exec_command(unstick_cmd)
    stdout.read()
    time.sleep(2)

    stdin, stdout, stderr = client.exec_command(f'echo "{PASS}" | sudo -S dmesg | tail -n 25')
    print(stdout.read().decode('utf-8', errors='replace'))

    stdin, stdout, stderr = client.exec_command("lsusb")
    print("=== lsusb ===")
    print(stdout.read().decode('utf-8', errors='replace'))
    client.close()

if __name__ == "__main__":
    unstick()
