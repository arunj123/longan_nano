import paramiko
import sys
import struct

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/stats.bin 0x20000f20 64; resume; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
stdout.channel.recv_exit_status()

sftp = client.open_sftp()
with sftp.open('/tmp/stats.bin', 'rb') as f:
    data = f.read()
sftp.close()
client.close()

# struct MscDiskStats {
#     uint32_t sectors_read{0};        // 0
#     uint32_t sectors_written{0};     // 4
#     uint32_t last_sector{0};         // 8
#     bool is_active{false};           // 12 (+3 pad -> 16 if Instant is 64-bit aligned, or at 13)
#     hal::time::Instant last_activity;// 16 (uint64_t ticks: 8 bytes -> 16..24)
#     uint32_t last_sd_result{0};      // 24
#     uint32_t last_sd_error_lba{0};   // 28
# };

fields = struct.unpack('<IIIIIIII', data[:32])
for i, val in enumerate(fields):
    print(f"word[{i}] (offset +{i*4:02d}): 0x{val:08x} ({val})")

sd_results = [
    "Success", "Timeout", "NoResponse", "Cmd0Fail", "Cmd8Fail",
    "Acmd41Timeout", "Cmd58Fail", "ReadTokenTimeout", "WriteError", "NotInitialized"
]

sec_read = fields[0]
sec_write = fields[1]
last_res = fields[6]
last_err_lba = fields[7]

res_str = sd_results[last_res] if last_res < len(sd_results) else f"Unknown({last_res})"
print(f"Sectors read: {sec_read}")
print(f"Sectors written: {sec_write}")
print(f"Last SD result: {res_str} (code {last_res})")
print(f"Last SD error LBA: {last_err_lba}")
