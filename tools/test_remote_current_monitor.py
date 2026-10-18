import os
import struct
import time
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
    def create_ssh_client(timeout=10, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout)
        return client

def main():
    print("==================================================")
    print(" Longan Nano INA219 Current Monitor Remote Test")
    print("==================================================")
    
    try:
        ssh = create_ssh_client(timeout=10)
    except Exception as e:
        print(f"[FAIL] SSH Connection Failed: {e}")
        sys.exit(1)
    
    # Python script executed on the remote host to read active /dev/hidraw* node
    remote_code = '''python3 -c "
import os, struct, time, sys, glob

try:
    fd = -1
    for node in sorted(glob.glob('/dev/hidraw*'), reverse=True):
        try:
            cand = os.open(node, os.O_RDONLY)
            d = os.read(cand, 64)
            if d and len(d) >= 7 and d[0] == 0x01:
                fd = cand
                print(f'CONNECTED on {node}')
                sys.stdout.flush()
                break
            os.close(cand)
        except Exception:
            pass
    if fd < 0:
        print('ERROR: No active Longan Nano hidraw device found')
        sys.exit(1)
    
    for i in range(30):
        data = os.read(fd, 64)
        if data and len(data) >= 7 and data[0] == 0x01:
            v, c, p = struct.unpack('<HhH', data[1:7])
            print(f'REPORT #{i+1:02d}: V={v:5d} mV | I={c:5d} mA | P={p:5d} mW | len={len(data)}')
            sys.stdout.flush()
    os.close(fd)
except Exception as e:
    print('ERROR:', e)
"'''
    
    stdin, stdout, stderr = ssh.exec_command(f'echo "{PASS}" | sudo -S ' + remote_code)
    
    for line in stdout:
        line_str = line.strip()
        if line_str and not line_str.startswith('[sudo]'):
            print("  [REMOTE]", line_str)
            
    err = stderr.read().decode().strip()
    if err and not '[sudo]' in err:
        print("  [STDERR]", err)
        
    ssh.close()
    print("==================================================")
    print(" Test Complete")
    print("==================================================")

if __name__ == "__main__":
    main()
