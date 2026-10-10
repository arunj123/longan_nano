#!/usr/bin/env python3
"""
Remote USB HID TCP Bridge for Longan Nano Current Monitor
Runs on Linux testbed (192.168.0.63).
Bridges /dev/hidraw0 (VID: 0x28E9, PID: 0x1234) to a lightweight TCP server (port 5055).
Allows desktop GUI applications across the network to stream telemetry and send HID OUT commands.
"""

import os
import sys
import time
import socket
import select
import threading

TCP_PORT = 5055
HID_PATH = "/dev/hidraw0"

def find_hidraw():
    # If /dev/hidraw0 exists, check if it matches or scan /sys/class/hidraw
    if os.path.exists(HID_PATH):
        return HID_PATH
    for entry in sorted(os.listdir("/sys/class/hidraw")):
        path = f"/dev/{entry}"
        return path
    return None

def client_worker(conn, addr, hid_fd):
    print(f"[BRIDGE] Client connected from {addr}")
    conn.setblocking(False)
    
    # Track buffer
    while True:
        try:
            rlist, _, _ = select.select([conn, hid_fd], [], [], 1.0)
            if not rlist:
                continue

            # Read from HID -> Send to TCP client
            if hid_fd in rlist:
                try:
                    data = os.read(hid_fd, 64)
                    if not data:
                        print("[BRIDGE] HID device read returned empty (disconnected?)")
                        break
                    conn.sendall(data)
                except BlockingIOError:
                    pass
                except Exception as e:
                    print(f"[BRIDGE] HID read error: {e}")
                    break

            # Read from TCP client -> Write to HID
            if conn in rlist:
                try:
                    cmd_data = conn.recv(64)
                    if not cmd_data:
                        print(f"[BRIDGE] Client {addr} disconnected")
                        break
                    os.write(hid_fd, cmd_data)
                    print(f"[BRIDGE] Forwarded command to HID: {len(cmd_data)} bytes (opcode 0x{cmd_data[0]:02X})")
                except BlockingIOError:
                    pass
                except Exception as e:
                    print(f"[BRIDGE] Client socket error: {e}")
                    break

        except Exception as e:
            print(f"[BRIDGE] Worker exception: {e}")
            break

    try:
        conn.close()
    except Exception:
        pass
    print(f"[BRIDGE] Client {addr} session closed")

def main():
    hid_path = find_hidraw()
    if not hid_path:
        print(f"[BRIDGE] Error: No hidraw device found!")
        sys.exit(1)

    print(f"[BRIDGE] Opening HID device at {hid_path}...")
    try:
        # Open in non-blocking mode
        hid_fd = os.open(hid_path, os.O_RDWR | os.O_NONBLOCK)
    except Exception as e:
        print(f"[BRIDGE] Failed to open {hid_path}: {e}")
        print("Note: Try running with appropriate udev permissions or sudo.")
        sys.exit(1)

    server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server_sock.bind(("0.0.0.0", TCP_PORT))
    server_sock.listen(2)
    print(f"[BRIDGE] Listening on 0.0.0.0:{TCP_PORT} (bridging {hid_path}). Press Ctrl+C to stop.")

    try:
        while True:
            conn, addr = server_sock.accept()
            # Handle one active client session
            client_worker(conn, addr, hid_fd)
    except KeyboardInterrupt:
        print("\n[BRIDGE] Stopping bridge...")
    finally:
        os.close(hid_fd)
        server_sock.close()
        print("[BRIDGE] Cleaned up and exited.")

if __name__ == "__main__":
    main()
