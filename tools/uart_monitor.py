import sys
import time
import serial

PORT = "COM13"
BAUDRATE = 115200

def main():
    while True:
        try:
            print(f"[UART MONITOR] Opening {PORT} at {BAUDRATE} baud...")
            ser = serial.Serial(PORT, BAUDRATE, timeout=0.1)
            print(f"[UART MONITOR] Connected to {PORT}. Listening for serial logs...")
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
            print("[UART MONITOR] Exiting...")
            break
        except Exception as e:
            time.sleep(1.0)

if __name__ == "__main__":
    main()
