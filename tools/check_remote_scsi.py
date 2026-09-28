import paramiko
import sys
import struct

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=15)

# Python script to run on remote host to issue SG_IO
remote_script = '''
import fcntl
import struct
import sys
import os

# Find sg device for Longan Nano (Vendor Sipeed)
sg_dev = None
for i in range(16):
    path = f"/dev/sg{i}"
    if os.path.exists(path):
        try:
            # Inquiry
            cdb = bytes([0x12, 0, 0, 0, 36, 0])
            buf = bytearray(36)
            sense = bytearray(32)
            # sg_io_hdr_t
            # int interface_id, int dxfer_direction, unsigned char cmd_len, unsigned char mx_sb_len,
            # unsigned short iovec_count, unsigned int dxfer_len, void *dxferp, unsigned char *cmdp,
            # unsigned char *sbp, unsigned int timeout, unsigned int flags, int pack_id, void * usr_ptr,
            # unsigned char status, unsigned char masked_status, unsigned char msg_status,
            # unsigned short sb_len_wr, unsigned short host_status, unsigned short driver_status,
            # int resid, unsigned int duration, unsigned int info
            hdr = struct.pack('iiBBHI4P4I6H2i2I',
                ord('S'), -3, len(cdb), len(sense),
                0, len(buf), id(buf), id(cdb), id(sense),
                5000, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
                0, 0, 0)
        except Exception:
            pass

# Alternatively use sg_raw command
'''

# Let's check what tools are on the remote host: sg_raw, sg3-utils, dd
stdin, stdout, stderr = client.exec_command("which sg_raw || which dd")
print("Available tools on remote host:", stdout.read().decode('utf-8'))

# Check current block devices
stdin, stdout, stderr = client.exec_command("lsblk")
print("Block devices:\n", stdout.read().decode('utf-8'))

client.close()
