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

client = create_ssh_client(timeout=10)

cmd = f"echo '{PASS}' | sudo -S bash -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'"
stdin, stdout, stderr = client.exec_command(cmd)
print("STDOUT:", stdout.read().decode())
print("STDERR:", stderr.read().decode())

time.sleep(1)
_, o, _ = client.exec_command(f"echo '{PASS}' | sudo -S dmesg | tail -n 20")
print("DMESG:\n", o.read().decode())
client.close()
