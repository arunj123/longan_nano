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
    ("LBA 0 (1 blk)", "28 00 00 00 00 00 00 00 01 00", 512),
    ("LBA 0 (8 blks)", "28 00 00 00 00 00 00 00 08 00", 4096),
    ("LBA 528 (8 blks)", "28 00 00 00 02 10 00 00 08 00", 4096),
    ("LBA 560 (8 blks)", "28 00 00 00 02 30 00 00 08 00", 4096),
    ("LBA 1024 (1 blk)", "28 00 00 00 04 00 00 00 01 00", 512),
    ("LBA 1024 (8 blks)", "28 00 00 00 04 00 00 00 08 00", 4096),
]

for label, cdb, rlen in tests:
    time.sleep(0.5)
    cmd = f"sg_raw --timeout=5 -r {rlen} -b /dev/sdb {cdb} > /tmp/out.bin 2> /tmp/err.txt"
    code, _, _ = run(cmd, sudo=True)
    _, sz, _ = run("stat -c %s /tmp/out.bin", sudo=True)
    _, err, _ = run("cat /tmp/err.txt", sudo=True)
    print(f"{label:20} -> code={code}, size={sz.strip():>5} bytes (expected {rlen})")
    if code != 0:
        print(f"   err: {err.strip()[:100]}")

client.close()
