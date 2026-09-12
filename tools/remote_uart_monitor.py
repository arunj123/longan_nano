#!/usr/bin/env python3
"""
Remote UART Monitor for Longan Nano on Linux host (192.168.0.63).
Streams serial output from /dev/ttyUSB1 at 115200 baud in real time.
"""

import sys
import time
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"
PORT = "/dev/ttyUSB1"
BAUDRATE = 115200

def stream_remote_uart():
    print(f"[REMOTE UART] Connecting to {USER}@{HOST}...")
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        client.connect(HOST, username=USER, password=PASS, timeout=10)
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
