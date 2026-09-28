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

cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/ep1_debug.bin 0x200001f8 12000; resume; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
out = stdout.read().decode('utf-8', errors='replace')
err = stderr.read().decode('utf-8', errors='replace')

sftp = client.open_sftp()
with sftp.open('/tmp/ep1_debug.bin', 'rb') as f:
    data = f.read()
sftp.close()
client.close()

with open('scratch/ep1_debug.bin', 'wb') as f:
    f.write(data)

idx = struct.unpack_from('<I', data, 0)[0]
print(f"Total debug entries logged: {idx}")

base_offset = 0x20000ec0 - 0x200001f8

step_names = {
    1: "IN_XFER (poll)",
    2: "EPIN_ENT (isr)",
    4: "IN_TRSC (isr)",
    5: "CSW_SEND",
    7: "TXFE_WR"
}

num_entries = 256
start_idx = max(0, idx - 80)
for i in range(start_idx, idx):
    slot = i % num_entries
    offset = base_offset + slot * 32
    step, val1, dieplen, dieptfstat, xfer_count, xfer_len, epctl, t = struct.unpack_from('<8I', data, offset)
    sname = step_names.get(step, f"STEP_{step}")
    pcnt = (dieplen >> 19) & 0x7F
    tlen = dieplen & 0x7FFFF
    ieptfs = dieptfstat & 0xFFFF
    nak = (epctl >> 17) & 1
    dpid = (epctl >> 16) & 1
    epen = (epctl >> 31) & 1
    epctl_str = f"EPEN={epen} NAK={nak} DPID={dpid}"
    print(f"#{i:04d} [{t}] {sname:16s} | val1=0x{val1:08x} | dieplen={dieplen:08x} (pcnt={pcnt:2d}, tlen={tlen:5d}) | fifo_free={ieptfs:3d}w | cnt={xfer_count:5d}/{xfer_len:5d} | {epctl_str} (0x{epctl:08x})")
