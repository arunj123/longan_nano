import os
import sys
import paramiko

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    from ssh_util import HOST, USER, PASS, KEY_PATH, create_ssh_client
except ImportError:
    HOST = os.environ.get("LONGAN_SSH_HOST", "192.168.0.63")
    USER = os.environ.get("LONGAN_SSH_USER", "arun")
    PASS = os.environ.get("LONGAN_SSH_PASS", "arun")
    KEY_PATH = os.environ.get("LONGAN_SSH_KEY", None)
    def create_ssh_client(timeout=15, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout)
        return client

def remote_exec(cmd, sudo=False):
    client = create_ssh_client(timeout=15)
    
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
        
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    exit_code = stdout.channel.recv_exit_status()
    client.close()
    return exit_code, out, err

if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    if len(sys.argv) < 2:
        print("Usage: python remote_exec.py [--sudo] <command>")
        sys.exit(1)
    args = sys.argv[1:]
    use_sudo = False
    if args[0] in ("--sudo", "-s"):
        use_sudo = True
        args = args[1:]
    cmd = " ".join(args)
    code, out, err = remote_exec(cmd, sudo=use_sudo)
    if out:
        sys.stdout.write(out)
    if err:
        sys.stderr.write(err)
    sys.exit(code)
