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

print("Cleaning up old pcaps and processes...")
run("pkill -9 tcpdump", sudo=True)
run("pkill -9 dd", sudo=True)
run("rm -f /tmp/msc.pcap")

print("Starting tcpdump on usbmon1...")
client.exec_command(f"echo '{PASS}' | sudo -S tcpdump -U -i usbmon1 -w /tmp/msc.pcap")
time.sleep(1.0)

print("Triggering 512B read via dd with 5s timeout...")
code, out, err = run("timeout 5 dd if=/dev/sdb of=/tmp/sec.bin bs=512 count=1 iflag=direct", sudo=True)
print(f"dd return code: {code}")
if err:
    print(f"dd stderr: {err.strip()}")

time.sleep(1.0)
print("Stopping tcpdump...")
run("pkill -2 tcpdump", sudo=True)
time.sleep(1.0)

print("Downloading /tmp/msc.pcap...")
sftp = client.open_sftp()
with sftp.open('/tmp/msc.pcap', 'rb') as f:
    pcap_data = f.read()
sftp.close()
client.close()

local_pcap = "build/prj_usb_msc/msc.pcap"
with open(local_pcap, "wb") as f:
    f.write(pcap_data)
print(f"Saved {len(pcap_data)} bytes to {local_pcap}")

# Parse pcap file
# PCAP global header: 24 bytes
# magic (4), v_maj (2), v_min (2), thiszone (4), sigfigs (4), snaplen (4), network (4)
if len(pcap_data) < 24:
    print("Pcap too small!")
    sys.exit(1)

magic = struct.unpack('<I', pcap_data[:4])[0]
print(f"PCAP magic: 0x{magic:08x}")

offset = 24
pkt_idx = 0
while offset + 16 <= len(pcap_data):
    ts_sec, ts_usec, incl_len, orig_len = struct.unpack('<IIII', pcap_data[offset:offset+16])
    offset += 16
    pkt_data = pcap_data[offset : offset + incl_len]
    offset += incl_len
    pkt_idx += 1
    
    # USBmon Linux header (48 or 64 bytes)
    # Typically 64 bytes on 64-bit Linux (DLT_USB_LINUX_MMAPPED = 220)
    if len(pkt_data) >= 48:
        # Check URB type (Submit 'S', Complete 'C', Error 'E')
        # In Linux usbmon mmapped:
        # id: 8 bytes, type: 1 byte, xfer_type: 1 byte, epnum: 1 byte, devnum: 1 byte
        # busnum: 2 bytes, flag_setup: 1 byte, flag_data: 1 byte, ts_sec: 8 bytes, ts_usec: 4 bytes
        # status: 4 bytes, length: 4 bytes, len_cap: 4 bytes
        urb_id = struct.unpack_from('<Q', pkt_data, 0)[0]
        event_type = chr(pkt_data[8])
        xfer_type = pkt_data[9]
        epnum = pkt_data[10]
        devnum = pkt_data[11]
        busnum = struct.unpack_from('<H', pkt_data, 12)[0]
        status = struct.unpack_from('<i', pkt_data, 28)[0]
        urb_len = struct.unpack_from('<I', pkt_data, 32)[0]
        data_len = struct.unpack_from('<I', pkt_data, 36)[0]
        payload = pkt_data[64:] if len(pkt_data) > 64 else b""
        payload_hex = " ".join(f"{b:02x}" for b in payload[:16])
        if epnum in (0x81, 0x01, 1, 129):
            ep_dir = "IN " if epnum & 0x80 else "OUT"
            ep_id = epnum & 0x7F
            print(f"#{pkt_idx:04d} [{ts_sec}.{ts_usec:06d}] Dev {devnum} EP{ep_id} {ep_dir} {event_type} | stat={status:4d} len={urb_len:4d} dlen={data_len:4d} | data: {payload_hex}")
