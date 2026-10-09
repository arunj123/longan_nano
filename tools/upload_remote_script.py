import paramiko

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect('192.168.0.63', username='arun', password='arun', timeout=10)

script_content = """#!/usr/bin/env python3
import os, struct, time, sys

DEV = '/dev/hidraw0'

def main():
    print('========================================================')
    print('  Longan Nano INA219 Live Current & Power Monitor')
    print('========================================================')
    
    if not os.path.exists(DEV):
        print(f'Error: {DEV} not found. Is Longan Nano connected via USB?')
        sys.exit(1)
        
    try:
        fd = os.open(DEV, os.O_RDONLY)
        print(f'Connected to {DEV}. Streaming live reports (Ctrl+C to stop)...\\n')
        print(f'{"#":>5} | {"Voltage":>10} | {"Current":>10} | {"Power":>10}')
        print('-' * 46)
        
        idx = 0
        while True:
            data = os.read(fd, 64)
            if data and len(data) >= 7 and data[0] == 0x01:
                idx += 1
                v_mv, c_ma, p_mw = struct.unpack('<HhH', data[1:7])
                v_str = f'{v_mv / 1000.0:.3f} V'
                c_str = f'{c_ma:.1f} mA'
                p_str = f'{p_mw:.1f} mW'
                print(f'{idx:>5} | {v_str:>10} | {c_str:>10} | {p_str:>10}')
                sys.stdout.flush()
    except KeyboardInterrupt:
        print('\\nExiting...')
    except Exception as e:
        print(f'Error reading {DEV}: {e}')
    finally:
        try: os.close(fd)
        except: pass

if __name__ == '__main__':
    main()
"""

sftp = client.open_sftp()
rem_path1 = '/home/arun/longan_nano_tools/test_remote_current_monitor.py'
rem_path2 = '/home/arun/longan_nano_tools/monitor.py'
with sftp.file(rem_path1, 'w') as f:
    f.write(script_content)
with sftp.file(rem_path2, 'w') as f:
    f.write(script_content)
sftp.close()

client.exec_command(f'chmod +x {rem_path1} {rem_path2}')
print('Uploaded test_remote_current_monitor.py and monitor.py successfully!')
client.close()
