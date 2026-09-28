import paramiko
import sys

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

tests = [
    ("LBA 0 (1 blk = 512B)", "28 00 00 00 00 00 00 00 01 00", 512),
    ("LBA 0 (8 blks = 4096B)", "28 00 00 00 00 00 00 00 08 00", 4096),
    ("LBA 528 (8 blks = 4096B)", "28 00 00 00 02 10 00 00 08 00", 4096),
    ("LBA 560 (8 blks = 4096B)", "28 00 00 00 02 30 00 00 08 00", 4096),
    ("LBA 1024 (8 blks = 4096B)", "28 00 00 00 04 00 00 00 08 00", 4096),
]

for label, cdb, rlen in tests:
    print(f"\n--- Testing {label} ---")
    cmd = f"sg_raw -r {rlen} -b /dev/sdb {cdb} > /tmp/sg_out.bin 2> /tmp/sg_err.txt"
    code, _, _ = run(cmd, sudo=True)
    _, err_txt, _ = run("cat /tmp/sg_err.txt", sudo=True)
    _, sz_txt, _ = run("stat -c %s /tmp/sg_out.bin", sudo=True)
    sz = sz_txt.strip()
    print(f"Exit code: {code}, Output size: {sz} bytes (expected {rlen})")
    if err_txt.strip():
        print(f"Stderr: {err_txt.strip()}")
    # Check dmesg for any new error
    _, dmesg_tail, _ = run("dmesg | tail -n 5", sudo=True)
    for line in dmesg_tail.strip().splitlines():
        if "sdb" in line or "usb" in line:
            print(f"  dmesg: {line}")

client.close()
