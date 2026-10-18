#!/usr/bin/env python3
"""
uart_monitor.py - Resilient local UART monitor for Longan Nano.

Auto-reconnects when the target device or FT2232D programmer is reset or power-cycled.
Supports Windows (COM ports) and Linux (/dev/ttyUSB*).
Port resolution priority:
  1. CLI argument: python tools/uart_monitor.py COM13 (or /dev/ttyUSB1)
  2. Environment variable: LONGAN_UART_PORT
  3. Auto-detected FTDI UART port (Interface 1 / USB Serial Converter B)
  4. Platform default (COM13 on Windows, /dev/ttyUSB1 on Linux)
"""

import os
import sys
import time
import serial
import serial.tools.list_ports

BAUDRATE = 115200

def detect_port():
    if len(sys.argv) > 1:
        return sys.argv[1]
    if os.environ.get("LONGAN_UART_PORT"):
        return os.environ.get("LONGAN_UART_PORT")

    # Scan connected serial ports for FTDI FT2232D Interface 1 (VID: 0403, PID: 6010)
    try:
        ports = serial.tools.list_ports.comports()
        for p in ports:
            # Check for FT2232D Interface 1 (UART channel B)
            hwid = (p.hwid or "").upper()
            desc = (p.description or "").lower()
            if "0403:6010" in hwid or "converter b" in desc or "uart" in desc:
                return p.device
    except Exception:
        pass

    return "COM13" if sys.platform == "win32" else "/dev/ttyUSB1"

def main():
    target_port = detect_port()
    print(f"[UART MONITOR] Target port: {target_port} @ {BAUDRATE} baud (auto-reconnect enabled)")
    print(f"[UART MONITOR] Press Ctrl+C to exit.\n")

    while True:
        try:
            ser = serial.Serial(target_port, BAUDRATE, timeout=0.1)
            print(f"[UART MONITOR] Connected to {target_port}. Streaming logs...")
            sys.stdout.flush()

            while True:
                # Poll port health; raises exception if FTDI connection reset
                _ = ser.in_waiting
                line = ser.readline()
                if line:
                    try:
                        text = line.decode('utf-8', errors='replace').strip()
                    except Exception:
                        text = str(line)
                    if text:
                        timestamp = time.strftime("%H:%M:%S")
                        print(f"[{timestamp}] {text}")
                        sys.stdout.flush()
                time.sleep(0.01)
        except KeyboardInterrupt:
            print("\n[UART MONITOR] Exiting.")
            break
        except Exception:
            # Reconnection backoff
            time.sleep(1.0)

if __name__ == "__main__":
    main()
