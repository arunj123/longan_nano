import os
import sys
import paramiko

sys.stdout.reconfigure(encoding='utf-8')

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

def run(cmd, sudo=False, timeout=10):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S timeout {timeout} {cmd}"
    else:
        cmd = f"timeout {timeout} {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd, timeout=timeout+5)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

print("=== DMESG (Last 50 lines) ===", flush=True)
_, out, _ = run("dmesg | tail -n 50", sudo=True)
print(out, flush=True)

print("=== LSBLK ===", flush=True)
_, out, _ = run("lsblk")
print(out, flush=True)

print("=== HUNG PROCESSES ===", flush=True)
_, out, _ = run("ps aux | grep -E 'dd|sg_raw|usb-storage'")
print(out, flush=True)

client.close()
