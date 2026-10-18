import os
import sys
import paramiko

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')

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

stdin, stdout, stderr = client.exec_command('ps aux | grep -E "(usbmon|sg_|python)" | grep -v grep')
print('=== Running processes ===\n' + stdout.read().decode())

stdin, stdout, stderr = client.exec_command(f'echo "{PASS}" | sudo -S dmesg | tail -n 25')
print('=== dmesg tail ===\n' + stdout.read().decode())

stdin, stdout, stderr = client.exec_command('lsblk')
print('=== lsblk ===\n' + stdout.read().decode())

client.close()
