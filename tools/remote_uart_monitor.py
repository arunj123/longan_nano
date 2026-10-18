#!/usr/bin/env python3
"""
Remote UART Monitor for Longan Nano on Linux host (192.168.0.63).
Streams serial output from /dev/ttyUSB1 at 115200 baud in real time.
"""

import os
import sys
import time
import paramiko

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    from ssh_util import HOST, USER, PASS, KEY_PATH, create_ssh_client
except ImportError:
    HOST = os.environ.get("LONGAN_SSH_HOST", "192.168.0.63")
    USER = os.environ.get("LONGAN_SSH_USER", "arun")
    PASS = os.environ.get("LONGAN_SSH_PASS", "arun")
    KEY_PATH = os.environ.get("LONGAN_SSH_KEY", None)
    def create_ssh_client(timeout=10, **kwargs):
        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        client.connect(HOST, username=USER, password=PASS, timeout=timeout)
        return client

PORT = "/dev/ttyUSB1"
BAUDRATE = 115200

def stream_remote_uart():
    print(f"[REMOTE UART] Connecting to {USER}@{HOST}...")
    try:
        client = create_ssh_client(timeout=10)
    except Exception as e:
        print(f"[REMOTE UART] SSH Connection failed: {e}")
        return

    print(f"[REMOTE UART] Connected to {HOST}. Streaming {PORT} @ {BAUDRATE} baud (Press Ctrl+C to stop)...")
    
    stdin, stdout, stderr = client.exec_command("python3 -u /home/arun/longan_nano_tools/uart_monitor.py", get_pty=True)
    
    try:
        for line in iter(stdout.readline, ""):
            if not line:
                break
            print(line.rstrip())
            sys.stdout.flush()
    except KeyboardInterrupt:
        print("\n[REMOTE UART] Exiting...")
    finally:
        client.close()

if __name__ == "__main__":
    stream_remote_uart()
