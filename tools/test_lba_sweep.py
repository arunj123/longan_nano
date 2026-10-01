import paramiko
import sys
import time

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=15)

def run_dd(name, bs, skip, count, timeout=5):
    cmd = f"echo {PASS} | sudo -S dd if=/dev/sdb of=/tmp/dd_test.bin bs={bs} skip={skip} count={count} iflag=direct"
    t0 = time.time()
    try:
        stdin, stdout, stderr = client.exec_command(cmd, timeout=timeout)
        out = stdout.read().decode('utf-8', errors='replace')
        err = stderr.read().decode('utf-8', errors='replace')
        code = stdout.channel.recv_exit_status()
        dt = time.time() - t0
        success = (code == 0 and "copied" in err)
        print(f"[{'PASS' if success else 'FAIL'}] {name:<25s}: code={code}, time={dt:.3f}s", flush=True)
        if not success:
            print("  stderr:", err.strip(), flush=True)
        return success
    except Exception as e:
        dt = time.time() - t0
        print(f"[FAIL] {name:<25s}: EXCEPTION ({e}) in {dt:.3f}s", flush=True)
        return False

tests = [
    ("Sector 0 (512B)", 512, 0, 1),
    ("Sector 32 (512B)", 512, 32, 1),
    ("Sector 64 (512B)", 512, 64, 1),
    ("Sector 128 (512B)", 512, 128, 1),
    ("Sector 256 (512B)", 512, 256, 1),
    ("Sector 496 (512B)", 512, 496, 1),
    ("Sector 512 (512B)", 512, 512, 1),
    ("Sector 528 (512B)", 512, 528, 1),
    ("Sector 560 (512B)", 512, 560, 1),
    ("LBA 560 2-sec (1024B)", 512, 560, 2),
    ("LBA 560 4-sec (2048B)", 512, 560, 4),
    ("LBA 560 8-sec (4096B)", 512, 560, 8),
]

print("=== Running Direct I/O LBA Sweep Test ===", flush=True)
for name, bs, skip, count in tests:
    ok = run_dd(name, bs, skip, count)
    if not ok:
        print("Stopping sweep on first unexpected failure.", flush=True)
        break
    time.sleep(0.05)

client.close()
