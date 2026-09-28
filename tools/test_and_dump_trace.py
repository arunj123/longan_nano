import paramiko
import sys
import time
import struct

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

def run(cmd, sudo=False):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

print("1. Cleaning up...")
run("pkill -9 tcpdump", sudo=True)
run("pkill -9 dd", sudo=True)
run("rm -f /tmp/msc_fresh.pcap /tmp/ep1_debug.bin /tmp/msc_trace.bin")

print("2. Starting tcpdump...")
client.exec_command(f"echo '{PASS}' | sudo -S tcpdump -U -i usbmon1 -w /tmp/msc_fresh.pcap")
time.sleep(0.5)

print("3. Power cycling xHCI port 1...")
run("sh -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'", sudo=True)

print("4. Waiting for Tag 4 (2.3s)...")
time.sleep(2.3)

print("5. Halting CPU via OpenOCD to capture exact RAM state...")
cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/ep1_debug.bin 0x200001f8 12000; resume; shutdown"'
run(cmd)

print("6. Stopping tcpdump...")
run("pkill -2 tcpdump", sudo=True)
time.sleep(0.5)

print("7. Downloading pcap and ep1_debug.bin...")
sftp = client.open_sftp()
with sftp.open('/tmp/msc_fresh.pcap', 'rb') as f:
    pcap_data = f.read()
try:
    with sftp.open('/tmp/ep1_debug.bin', 'rb') as f:
        debug_data = f.read()
except Exception as e:
    debug_data = None
    print("Failed to download ep1_debug.bin:", e)
sftp.close()
client.close()

with open("scratch/msc_fresh.pcap", "wb") as f:
    f.write(pcap_data)

# Parse PCAP
print("\n--- PCAP URBS ---")
offset = 24
pkt_idx = 0
while offset + 16 <= len(pcap_data):
    ts_sec, ts_usec, incl_len, orig_len = struct.unpack('<IIII', pcap_data[offset:offset+16])
    offset += 16
    pkt_data = pcap_data[offset : offset + incl_len]
    offset += incl_len
    pkt_idx += 1
    if len(pkt_data) >= 48:
        epnum = pkt_data[10]
        devnum = pkt_data[11]
        ev = chr(pkt_data[8])
        status = struct.unpack_from('<i', pkt_data, 28)[0]
        urb_len = struct.unpack_from('<I', pkt_data, 32)[0]
        data_len = struct.unpack_from('<I', pkt_data, 36)[0]
        payload = pkt_data[64:] if len(pkt_data) > 64 else b""
        payload_hex = " ".join(f"{b:02x}" for b in payload[:16])
        ep_dir = "IN " if epnum & 0x80 else "OUT"
        ep_id = epnum & 0x7F
        if ep_id == 1:
            print(f"#{pkt_idx:03d} [{ts_sec}.{ts_usec:06d}] Dev {devnum:2d} EP{ep_id} {ep_dir} {ev} | stat={status:4d} len={urb_len:4d} dlen={data_len:4d} | hex: {payload_hex}")

# Parse EP1 debug
if debug_data:
    print("\n--- EP1 DEBUG LOG ---")
    idx = struct.unpack_from('<I', debug_data, 0)[0]
    print(f"Total debug entries: {idx}")
    base_offset = 0x20000ec0 - 0x200001f8
    step_names = {1: "IN_XFER (poll)", 2: "EPIN_ENT (isr)", 4: "IN_TRSC (isr)", 5: "CSW_SEND", 7: "TXFE_WR"}
    num_entries = 256
    start_idx = max(0, idx - 40)
    for i in range(start_idx, idx):
        slot = i % num_entries
        off = base_offset + slot * 32
        step, val1, dieplen, dieptfstat, xfer_count, xfer_len, epctl, t = struct.unpack_from('<8I', debug_data, off)
        sname = step_names.get(step, f"STEP_{step}")
        pcnt = (dieplen >> 19) & 0x7F
        tlen = dieplen & 0x7FFFF
        ieptfs = dieptfstat & 0xFFFF
        nak = (epctl >> 17) & 1
        dpid = (epctl >> 16) & 1
        epen = (epctl >> 31) & 1
        print(f"#{i:04d} [{t}] {sname:16s} | val1=0x{val1:08x} | dieplen={dieplen:08x} (pcnt={pcnt:2d}, tlen={tlen:5d}) | fifo_free={ieptfs:3d}w | cnt={xfer_count:5d}/{xfer_len:5d} | EPEN={epen} NAK={nak} DPID={dpid}")
