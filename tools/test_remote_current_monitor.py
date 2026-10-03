import os
import struct
import time
import sys
import paramiko

def main():
    print("==================================================")
    print(" Longan Nano INA219 Current Monitor Remote Test")
    print("==================================================")
    
    ssh = paramiko.SSHClient()
    ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    ssh.connect('192.168.0.63', username='arun', password='arun')
    
    # Python script executed on the remote host to read /dev/hidraw0
    remote_code = '''python3 -c "
import os, struct, time, sys

try:
    fd = os.open('/dev/hidraw0', os.O_RDONLY)
    print('CONNECTED')
    sys.stdout.flush()
    
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
    
    stdin, stdout, stderr = ssh.exec_command('echo arun | sudo -S ' + remote_code)
    
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
