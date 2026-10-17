import paramiko
import time

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=10)

def run(cmd, sudo=False):
    if sudo:
        cmd = f"echo '{PASS}' | sudo -S {cmd}"
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace')
    err = stderr.read().decode('utf-8', errors='replace')
    code = stdout.channel.recv_exit_status()
    return code, out, err

tests = [
    ("INQUIRY", "12 00 00 00 24 00", 36),
    ("TEST UNIT READY", "00 00 00 00 00 00", 0),
    ("READ CAPACITY", "25 00 00 00 00 00 00 00 00 00", 8),
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512),
    ("LBA 0 (2 blks)", "28 00 00 00 00 00 00 00 02 00", 1024),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096),
    ("LBA 32 (8 blks)", "28 00 00 00 00 20 00 00 08 00", 4096),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096),
]

# Find sg device for Longan
code, out, _ = run("lsscsi -g | grep Longan", sudo=True)
print("SCSI Device info:", out.strip())

for label, cdb, rlen in tests:
    cmd = f"sg_raw -t 2 -r {rlen} -b /dev/sg1 {cdb} > /tmp/sg_out.bin 2> /tmp/sg_err.txt"
    code, _, _ = run(cmd, sudo=True)
    _, sz, _ = run("stat -c %s /tmp/sg_out.bin 2>/dev/null || echo 0", sudo=True)
    _, err, _ = run("cat /tmp/sg_err.txt 2>/dev/null", sudo=True)
    sz_val = sz.strip()
    status = "OK" if code == 0 and (rlen == 0 or sz_val == str(rlen)) else "FAIL"
    print(f"[{status}] {label:20} -> code={code}, size={sz_val:>5} (exp {rlen})")
    if status == "FAIL":
        print(f"      err: {err.strip()[:100]}")
        break
    time.sleep(0.1)

client.close()
