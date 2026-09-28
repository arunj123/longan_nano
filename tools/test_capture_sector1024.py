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
run("rm -f /tmp/msc_s1024.pcap /tmp/ep1_debug_s1024.bin /tmp/idx_s1024.bin", sudo=True)

print("2. Starting tcpdump...")
client.exec_command(f"echo '{PASS}' | sudo -S tcpdump -U -i usbmon1 -w /tmp/msc_s1024.pcap")
time.sleep(1.0)

print("3. Power cycling xHCI port 1...")
run("sh -c 'echo 1 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 > /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable'", sudo=True)

# Port disable sleep 1s + enable -> enumeration takes ~1.2s -> total ~2.2s from disable end
print("4. Waiting 2.2s for partition recognition and Sector 1024 attempt...")
time.sleep(2.2)

print("5. Halting MCU and dumping trace immediately...")
cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/ep1_debug_s1024.bin 0x20000ec0 8192; dump_image /tmp/idx_s1024.bin 0x200001f8 4; resume; shutdown"'
run(cmd, sudo=False)

print("6. Stopping tcpdump...")
run("pkill -2 tcpdump", sudo=True)
time.sleep(0.5)

print("7. Downloading trace files...")
sftp = client.open_sftp()
with sftp.open('/tmp/msc_s1024.pcap', 'rb') as f:
    pcap_data = f.read()
with sftp.open('/tmp/ep1_debug_s1024.bin', 'rb') as f:
    ep1_data = f.read()
with sftp.open('/tmp/idx_s1024.bin', 'rb') as f:
    idx_data = f.read()
sftp.close()

_, dmesg_tail, _ = run("dmesg | tail -n 25", sudo=True)
client.close()

with open('scratch/msc_s1024.pcap', 'wb') as f: f.write(pcap_data)
with open('scratch/ep1_debug_s1024.bin', 'wb') as f: f.write(ep1_data)
with open('scratch/idx_s1024.bin', 'wb') as f: f.write(idx_data)

idx = struct.unpack('<I', idx_data)[0]
print(f"Downloaded: pcap={len(pcap_data)}B, ep1={len(ep1_data)}B, total entries recorded={idx}")
print("\ndmesg tail:\n", dmesg_tail)
