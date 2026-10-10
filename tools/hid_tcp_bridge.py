#!/usr/bin/env python3
"""
Bidirectional TCP Bridge for Longan Nano Current Monitor
Bridges remote testbed /dev/hidraw0 (via SSH to 192.168.0.63) to a local TCP socket (localhost:5055).
Also works with local hidapi if device is plugged into Windows.
"""

import sys
import time
import socket
import select
import threading
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"
LOCAL_PORT = 5055

def run_ssh_bridge():
    print(f"[*] Starting SSH HID bridge to {USER}@{HOST}...")
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        client.connect(HOST, username=USER, password=PASS, timeout=10)
    except Exception as e:
        print(f"[!] SSH connection error: {e}")
        return

    remote_cmd = """python3 -u -c '
import os, sys, select

try:
    fd = os.open("/dev/hidraw0", os.O_RDWR)
except Exception as e:
    sys.stderr.write(f"OPEN_FAIL:{e}\\n")
    sys.stderr.flush()
    sys.exit(1)

sys.stderr.write("DEVICE_READY\\n")
sys.stderr.flush()

while True:
    r, _, _ = select.select([sys.stdin.buffer, fd], [], [], 1.0)
    if fd in r:
        data = os.read(fd, 64)
        if not data:
            break
        sys.stdout.buffer.write(data)
        sys.stdout.buffer.flush()
    if sys.stdin.buffer in r:
        cmd = sys.stdin.buffer.read(64)
        if not cmd:
            break
        os.write(fd, cmd)
'"""

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", LOCAL_PORT))
    server.listen(1)
    print(f"[*] Local TCP bridge listening on 127.0.0.1:{LOCAL_PORT}...")

    stdin, stdout, stderr = client.exec_command(remote_cmd)
    ready_line = stderr.readline().strip()
    if "DEVICE_READY" not in ready_line:
        print(f"[!] Remote error: {ready_line}")
        client.close()
        server.close()
        return

    print("[*] Remote /dev/hidraw0 open and ready. Waiting for GUI client to connect...")

    while True:
        try:
            conn, addr = server.accept()
            print(f"[*] GUI client connected from {addr}")
            conn.setblocking(False)

            # Bidirectional pump
            while True:
                # 1. From remote SSH stdout -> TCP conn
                if stdout.channel.recv_ready():
                    chunk = stdout.channel.recv(4096)
                    if chunk:
                        conn.sendall(chunk)
                    else:
                        break

                # 2. From TCP conn -> remote SSH stdin
                r, _, _ = select.select([conn], [], [], 0.01)
                if r:
                    try:
                        cmd = conn.recv(64)
                        if not cmd:
                            print("[*] GUI client disconnected")
                            break
                        stdin.channel.sendall(cmd)
                        print(f"[*] Forwarded {len(cmd)} byte command to hardware (0x{cmd[0]:02X})")
                    except BlockingIOError:
                        pass
                    except Exception as e:
                        print(f"[!] TCP read error: {e}")
                        break

                time.sleep(0.005)

            conn.close()
        except KeyboardInterrupt:
            print("\n[*] Exiting bridge...")
            break
        except Exception as e:
            print(f"[!] Bridge loop error: {e}")
            break

    client.close()
    server.close()

if __name__ == "__main__":
    run_ssh_bridge()
