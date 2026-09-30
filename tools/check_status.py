import paramiko
import struct

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/ctx_dump.bin 0x20000690 656; resume; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
code = stdout.channel.recv_exit_status()

sftp = client.open_sftp()
sftp.get('/tmp/ctx_dump.bin', 'build/prj_usb_msc/ctx_dump.bin')
sftp.close()
client.close()

raw = open('build/prj_usb_msc/ctx_dump.bin', 'rb').read()
cbw = raw[512:512+31]
csw = raw[576:576+13]
state, status = struct.unpack('<II', raw[592:600])
lba, total_blocks, rem_bytes, residue = struct.unpack('<IIII', raw[608:624])
sub_off, sub_rem = struct.unpack('<HH', raw[648:652])

print(f"CBW: {cbw.hex(' ')}")
print(f"CSW: {csw.hex(' ')}")
print(f"State: {state}, Status: {status}")
print(f"LBA: {lba}, TotalBlocks: {total_blocks}, Remaining: {rem_bytes}, Residue: {residue}")
print(f"SubOffset: {sub_off}, SubRemaining: {sub_rem}")
