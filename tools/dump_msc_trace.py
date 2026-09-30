import paramiko
import sys
import struct

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15, banner_timeout=30)

# Dump 0x20000200 - 0x20000ea0 (3232 bytes)
cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/msc_trace.bin 0x20000200 3232; resume; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
out = stdout.read().decode('utf-8', errors='replace')
err = stderr.read().decode('utf-8', errors='replace')

sftp = client.open_sftp()
with sftp.open('/tmp/msc_trace.bin', 'rb') as f:
    data = f.read()
sftp.close()
client.close()

with open('scratch/msc_trace.bin', 'wb') as f:
    f.write(data)

tail = data[0x20000205 - 0x20000200]
head = data[0x20000206 - 0x20000200]
print(f"MSC Trace: tail={tail}, head={head}")

type_names = {
    1: "CBW_RECV",
    2: "SCSI_RES",
    3: "CSW_SENT",
    4: "ABORT_XF",
    5: "CLASS_RQ"
}

trace_base = 0x200008a8 - 0x20000200

entry_size = 24

cur = tail
while cur != head:
    offset = trace_base + cur * entry_size
    t_type, opcode, val8, status, val32_1, val32_2 = struct.unpack_from('<BBBBI I', data, offset)
    cdb = data[offset+12 : offset+22]
    tname = type_names.get(t_type, f"TYPE_{t_type}")
    cdb_hex = " ".join(f"{b:02x}" for b in cdb)
    print(f"[{cur:02d}] {tname:10s} | op=0x{opcode:02x} val8=0x{val8:02x} status=0x{status:02x} | v1=0x{val32_1:08x} ({val32_1}) v2=0x{val32_2:08x} ({val32_2}) | cdb={cdb_hex}")
    cur = (cur + 1) % 64
