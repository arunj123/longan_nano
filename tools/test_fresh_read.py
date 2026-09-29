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
run("pkill -9 -f uart_monitor", sudo=True)
run("pkill -9 dd", sudo=True)
run("rm -f /tmp/msc_fresh.pcap /tmp/msc_uart.log /tmp/ep1_debug.bin /tmp/idx.bin", sudo=True)

print("2. Starting tcpdump & uart_monitor in background...")
client.exec_command(f"echo '{PASS}' | sudo -S nohup tcpdump -U -i usbmon1 -w /tmp/msc_fresh.pcap >/dev/null 2>&1 &")
client.exec_command("nohup python3 -u /home/arun/longan_nano_tools/uart_monitor.py > /tmp/msc_uart.log 2>&1 &")
time.sleep(1.0)

print("3. Power cycling xHCI port 1...")
run("sh -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'", sudo=True)

print("4. Waiting for enumeration (6.0s)...")
time.sleep(6.0)

code, out, _ = run("ls /sys/block/ | grep -E '^sd[b-z]$' | head -n 1")
dev_name = out.strip()
if not dev_name:
    dev_name = "sdb"
dev_path = f"/dev/{dev_name}"
print(f"Detected target block device: {dev_path}")

print(f"5. Testing multi-sector read with dd on {dev_path} (sectors 0-2048 = 1MB)...")
code, dd_out, dd_err = run(f"echo 'arun' | sudo -S dd if={dev_path} of=/dev/null bs=512 count=2048 iflag=direct", sudo=False)
print(f"dd exit code: {code}")
if dd_out.strip(): print(f"dd stdout: {dd_out.strip()}")
if dd_err.strip(): print(f"dd stderr: {dd_err.strip()}")

print("6. Stopping tcpdump & uart_monitor...")
run("pkill -2 tcpdump", sudo=True)
run("pkill -9 -f uart_monitor", sudo=True)
time.sleep(0.5)

print("7. Checking dmesg...")
_, dmesg_out, _ = run("dmesg | tail -n 25", sudo=True)
print(dmesg_out)

print("8. Checking UART logs...")
_, uart_out, _ = run("cat /tmp/msc_uart.log", sudo=False)
print("=== UART LOGS ===")
print(uart_out)
print("=================")

print("9. Downloading pcap...")
run("chmod 666 /tmp/msc_fresh.pcap", sudo=True)
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
