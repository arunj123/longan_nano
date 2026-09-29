import paramiko
import sys
import time
import struct

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

print("1. Cleaning up existing tcpdump/dd...")
run("pkill -9 tcpdump", sudo=True)
run("pkill -9 dd", sudo=True)
run("rm -f /tmp/test_read.pcap", sudo=True)

print("2. Starting tcpdump on usbmon1...")
client.exec_command(f"echo '{PASS}' | sudo -S nohup tcpdump -U -i usbmon1 -w /tmp/test_read.pcap >/dev/null 2>&1 &")
time.sleep(0.5)

print("3. Running dd read of 8 sectors (4096 bytes) on /dev/sdb with timeout...")
# Run dd with timeout 5s so it won't block forever
code, out, err = run("timeout 5s dd if=/dev/sdb of=/dev/null bs=512 count=8 iflag=direct", sudo=True)
print(f"dd exit code: {code}")
if out.strip(): print("dd stdout:", out.strip())
if err.strip(): print("dd stderr:", err.strip())

print("4. Stopping tcpdump...")
run("pkill -2 tcpdump", sudo=True)
time.sleep(0.5)

print("5. Downloading pcap...")
run("chmod 666 /tmp/test_read.pcap", sudo=True)
sftp = client.open_sftp()
with sftp.open('/tmp/test_read.pcap', 'rb') as f:
    pcap_data = f.read()
sftp.close()
client.close()

with open("build/prj_usb_msc/test_read.pcap", "wb") as f:
    f.write(pcap_data)
print(f"Saved {len(pcap_data)} bytes to build/prj_usb_msc/test_read.pcap")

# Parse PCAP
offset = 24
pkt_idx = 0
while offset + 16 <= len(pcap_data):
    ts_sec, ts_usec, incl_len, orig_len = struct.unpack('<IIII', pcap_data[offset:offset+16])
    offset += 16
    pkt_data = pcap_data[offset : offset + incl_len]
    offset += incl_len
    pkt_idx += 1
    if len(pkt_data) >= 48:
        devnum = pkt_data[11]
        epnum = pkt_data[10]
        event_type = chr(pkt_data[8])
        status = struct.unpack_from('<i', pkt_data, 28)[0]
        urb_len = struct.unpack_from('<I', pkt_data, 32)[0]
        data_len = struct.unpack_from('<I', pkt_data, 36)[0]
        payload = pkt_data[64:] if len(pkt_data) > 64 else b""
        payload_hex = " ".join(f"{b:02x}" for b in payload[:16])
        ep_dir = "IN " if epnum & 0x80 else "OUT"
        ep_id = epnum & 0x7F
        # Filter for bulk endpoints (EP1) or control (EP0)
        if ep_id in (0, 1):
            print(f"#{pkt_idx:03d} [{ts_sec}.{ts_usec:06d}] Dev {devnum:2d} EP{ep_id} {ep_dir} {event_type} | stat={status:4d} len={urb_len:4d} dlen={data_len:4d} | hex: {payload_hex}")
