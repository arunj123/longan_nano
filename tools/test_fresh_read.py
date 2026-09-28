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
run("rm -f /tmp/msc_fresh.pcap")

print("2. Starting tcpdump...")
client.exec_command(f"echo '{PASS}' | sudo -S tcpdump -U -i usbmon1 -w /tmp/msc_fresh.pcap")
time.sleep(0.5)

print("3. Power cycling xHCI port 1...")
run("sh -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'", sudo=True)

print("4. Waiting for enumeration (3.5s)...")
time.sleep(3.5)

print("5. Stopping tcpdump...")
run("pkill -2 tcpdump", sudo=True)
time.sleep(0.5)

print("6. Checking dmesg...")
_, dmesg_out, _ = run("dmesg | tail -n 25", sudo=True)
print(dmesg_out)

print("7. Downloading pcap...")
sftp = client.open_sftp()
with sftp.open('/tmp/msc_fresh.pcap', 'rb') as f:
    pcap_data = f.read()
sftp.close()
client.close()

with open("build/prj_usb_msc/msc_fresh.pcap", "wb") as f:
    f.write(pcap_data)
print(f"Saved {len(pcap_data)} bytes to build/prj_usb_msc/msc_fresh.pcap")

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
        event_type = chr(pkt_data[8])
        epnum = pkt_data[10]
        devnum = pkt_data[11]
        status = struct.unpack_from('<i', pkt_data, 28)[0]
        urb_len = struct.unpack_from('<I', pkt_data, 32)[0]
        data_len = struct.unpack_from('<I', pkt_data, 36)[0]
        payload = pkt_data[64:] if len(pkt_data) > 64 else b""
        payload_hex = " ".join(f"{b:02x}" for b in payload[:16])
        ep_dir = "IN " if epnum & 0x80 else "OUT"
        ep_id = epnum & 0x7F
        print(f"#{pkt_idx:03d} [{ts_sec}.{ts_usec:06d}] Dev {devnum:2d} EP{ep_id} {ep_dir} {event_type} | stat={status:4d} len={urb_len:4d} dlen={data_len:4d} | hex: {payload_hex}")
