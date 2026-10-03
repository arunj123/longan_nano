import os
import struct
import time
import sys
import paramiko

def main():
    print("==================================================")
    print(" Longan Nano USB Composite Remote Test")
    print("==================================================")

    ssh = paramiko.SSHClient()
    ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    ssh.connect('192.168.0.63', username='arun', password='arun')

    # Python script executed on the remote host to test /dev/hidraw1
    remote_code = '''python3 -c "
import os, struct, time, sys

try:
    # Open Custom HID interface /dev/hidraw1 for read/write
    fd = os.open('/dev/hidraw1', os.O_RDWR)
    print('Opened /dev/hidraw1 successfully!')
    sys.stdout.flush()

    # Send DRAW_RECT command [0x01, x=10, y=20, w=40, h=30, seq_l=1, seq_h=0] + padding
    rect_pkt = bytearray([0x01, 10, 20, 40, 30, 1, 0])
    rect_pkt.extend([0] * (64 - len(rect_pkt)))
    written = os.write(fd, rect_pkt)
    print(f'Sent DRAW_RECT packet: {written} bytes')
    sys.stdout.flush()

    time.sleep(0.1)

    # Send IMAGE_DATA packet [0x02, pixel data...] + padding
    img_pkt = bytearray([0x02] + [0x55] * 63)
    written = os.write(fd, img_pkt)
    print(f'Sent IMAGE_DATA packet: {written} bytes')
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
