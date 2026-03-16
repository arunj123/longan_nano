import hid
import struct
import time
import sys

# USB Device Parameters
VID = 0x28E9
PID = 0x1234

def main():
    try:
        # Open the device by VID/PID
        device = hid.device()
        device.open(VID, PID)
        device.set_nonblocking(True)

        print(f"Connected to INA219 Monitor (VID: 0x{VID:04X}, PID: 0x{PID:04X})")
        print("Press Ctrl+C to stop.")
        print("-" * 50)
        print(f"{'Voltage (mV)':>15} | {'Current (mA)':>15} | {'Power (mW)':>15}")
        print("-" * 50)

        while True:
            # Read a report (9 bytes)
            report = device.read(64)
            if report:
                if report[0] == 0x01 and len(report) >= 7:
                    # Packet: [ID, V_L, V_H, C_L, C_H, P_L, P_H, ...]
                    # Using format '<HhH' for Little-Endian: uint16, int16, uint16
                    v, c, p = struct.unpack('<HhH', bytes(report[1:7]))
                    
                    # Print formatted data on the same line
                    sys.stdout.write(f"\r{v:>15} | {c:>15} | {p:>15}")
                    sys.stdout.flush()
            
            # Small sleep to prevent high CPU usage
            time.sleep(0.01)

    except KeyboardInterrupt:
        print("\nStopping monitor...")
    except Exception as e:
        print(f"\nError: {e}")
    finally:
        if 'device' in locals():
            device.close()

if __name__ == "__main__":
    main()
