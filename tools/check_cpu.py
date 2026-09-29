import paramiko
import sys

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/ep1_debug_now.bin 0x20000ed0 8192; dump_image /tmp/idx_now.bin 0x20000208 4; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
out = stdout.read().decode('utf-8', errors='replace')
err = stderr.read().decode('utf-8', errors='replace')
sftp = client.open_sftp()
sftp.get('/tmp/ep1_debug_now.bin', 'build/prj_usb_msc/ep1_debug_now.bin')
sftp.get('/tmp/idx_now.bin', 'build/prj_usb_msc/idx_now.bin')
sftp.close()
client.close()

import struct
idx = struct.unpack('<I', open('build/prj_usb_msc/idx_now.bin', 'rb').read())[0]
data = open('build/prj_usb_msc/ep1_debug_now.bin', 'rb').read()
print(f"Total entries: {idx}")
entries = [struct.unpack('<8I', data[i*32:(i+1)*32]) for i in range(min(idx, 256))]
for i in range(min(idx, 256)):
    step, val1, dieplen, dieptfstat, cnt, xlen, epctl, t = struct.unpack('<8I', data[i*32:(i+1)*32])
    print(f"[{i:03d}] step={step:2d} val1={val1:#010x} dieplen={dieplen:#010x} tfstat={dieptfstat:#06x} cnt={cnt:3d} len={xlen:3d} epctl={epctl:#010x} t={t}")
