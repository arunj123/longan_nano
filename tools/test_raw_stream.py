import paramiko, struct

c = paramiko.SSHClient()
c.set_missing_host_key_policy(paramiko.AutoAddPolicy())
c.connect('192.168.0.63', username='arun', password='arun')

remote_cmd = """python3 -u -c '
import os, sys
fd = os.open("/dev/hidraw0", os.O_RDONLY)
for _ in range(5):
    pkt = os.read(fd, 64)
    sys.stdout.buffer.write(pkt[:9])
sys.stdout.buffer.flush()
'"""

stdin, stdout, stderr = c.exec_command(remote_cmd)
raw = stdout.read()
print(f"Read {len(raw)} bytes: {raw.hex()}")
for i in range(len(raw)//9):
    chunk = raw[i*9:(i+1)*9]
    rid, v, cur, p, flg, seq = struct.unpack('<BHhHBB', chunk)
    print(f"  Pkt {i+1}: RID=0x{rid:02X}, V={v}mV, I={cur/10.0}mA, P={p}mW, Flags=0x{flg:02X}, Seq={seq}")

c.close()
