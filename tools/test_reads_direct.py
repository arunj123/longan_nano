import sys
import paramiko
import time

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

ssh = paramiko.SSHClient()
ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
ssh.connect(HOST, username=USER, password=PASS, allow_agent=False, look_for_keys=False, timeout=10)

def test_sg(lba, blocks, expect_pass=True):
    cdb = f"28 00 {(lba>>24)&0xff:02x} {(lba>>16)&0xff:02x} {(lba>>8)&0xff:02x} {lba&0xff:02x} 00 {(blocks>>8)&0xff:02x} {blocks&0xff:02x} 00"
    nbytes = blocks * 512
    cmd = f"echo {PASS} | sudo -S timeout 3 sg_raw -r {nbytes} /dev/sdb {cdb}"
    t0 = time.time()
    stdin, stdout, stderr = ssh.exec_command(cmd)
    out = stdout.read()
    err = stderr.read().decode('utf-8', 'replace')
    dt = time.time() - t0
    code = stdout.channel.recv_exit_status()
    status_str = "PASS" if code == 0 and len(out) == nbytes else "FAIL"
    status_tag = f"[{status_str}]"
    print(f"{status_tag:6s} LBA {lba:5d} ({blocks} blks = {nbytes:4d}B): code={code} in {dt:.3f}s, out={len(out)}B", flush=True)
    if code != 0 and expect_pass:
        print(f"       stderr: {' '.join(err.strip().splitlines())}", flush=True)

print("=== 1. Single Sector (512B) Read Verification ===", flush=True)
for lba in [0, 32, 64, 128, 256, 496, 528, 560]:
    test_sg(lba, 1)

print("\n=== 2. Multi-Sector (8 blocks = 4096B) Verification ===", flush=True)
for lba in [0, 64, 256, 560]:
    test_sg(lba, 8)

print("\n=== 3. Flash AU Boundary Verification (LBA 1024 - Known Limitation) ===", flush=True)
print("Note: Sector 1024 hits ~3.5ms SD erase block latency; expected to fail in single-buffer mode.", flush=True)
test_sg(1024, 1, expect_pass=False)

ssh.close()
