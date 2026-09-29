import paramiko
import sys
import time

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username=USER, password=PASS, timeout=15)

for s in [0, 1, 32, 64, 128, 256, 512, 550, 559, 560, 561, 562, 563, 564, 565, 566, 567]:
    t0 = time.time()
    try:
        _, so, se = client.exec_command(f'echo {PASS} | sudo -S dd if=/dev/sdb of=/tmp/test_s.bin bs=512 skip={s} count=1 iflag=direct', timeout=5)
        out = so.read().decode('utf-8', errors='replace')
        err = se.read().decode('utf-8', errors='replace')
        dt = time.time() - t0
        success = 'copied' in err
        status_str = "OK" if success else "FAIL"
        print(f"Sector {s:4d}: {status_str} (time={dt:.3f}s)")
        if not success:
            print("  stderr:", err.strip())
            break
    except Exception as e:
        dt = time.time() - t0
        print(f"Sector {s:4d}: TIMEOUT/EXCEPTION ({e}) (time={dt:.3f}s)")
        break

client.close()
