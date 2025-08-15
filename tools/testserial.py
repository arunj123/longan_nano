import serial
import serial.tools.list_ports
import time

# --- Configuration ---
BAUD_RATE = 115200
# A part of the description of the COM port for the Longan Nano.
# This helps the script find the board automatically.
# Common names include "USB Serial Port", "JLink CDC UART Port", or "Serial".
# You can find the exact name in the Windows Device Manager.
TARGET_DEVICE_DESCRIPTION = "Serial Device"

def find_serial_port():
    """Scans available COM ports and finds the one matching the target device."""
    print("Scanning for serial ports...")
    ports = serial.tools.list_ports.comports()
    for port in ports:
        # On Windows, port.device is the COM port name (e.g., "COM3")
        # port.description contains a human-readable name.
        print(f" - Found port: {port.device} ({port.description})")
        if TARGET_DEVICE_DESCRIPTION in port.description:
            print(f"\n✅ Target device found on {port.device}!")
            return port.device
    return None

def test_usb_serial_echo(port_name):
    """
    Opens the specified serial port, sends a test message, and checks for an echo.
    """
    ser = None  # Initialize to None
    try:
        # --- 1. Open the Serial Port ---
        # The timeout parameter is crucial. It sets the maximum time (in seconds)
        # that the read() function will wait for data before returning.
        print(f"Opening {port_name} at {BAUD_RATE} baud...")
        ser = serial.Serial(port_name, BAUD_RATE, timeout=1)
        print("Port opened successfully.")

        # --- 2. Prepare and Send Data ---
        # The 'b' prefix creates a "bytes" string, which is what the serial library expects.
        test_message = b"Hello from Python! This is a test.\n"
        print(f"Sending message: {test_message.decode('utf-8').strip()}")
        ser.write(test_message)

        # Give the device a moment to process and send the echo back.
        time.sleep(0.1)

        # --- 3. Read the Echoed Data ---
        # We expect the device to echo the exact same number of bytes we sent.
        print("Waiting for echo...")
        received_data = ser.read(len(test_message))
        
        if not received_data:
            print("\n❌ TEST FAILED: No data received. (Timeout)")
            return

        print(f"Received message: {received_data.decode('utf-8').strip()}")

        # --- 4. Verify the Data ---
        if received_data == test_message:
            print("\n✅ TEST PASSED: Received data matches sent data.")
        else:
            print("\n❌ TEST FAILED: Received data does NOT match sent data.")
            print(f"   Expected: {test_message}")
            print(f"   Got:      {received_data}")

    except serial.SerialException as e:
        print(f"\n❌ An error occurred: {e}")
        print("   Please ensure the COM port is correct and not in use by another program (like PuTTY).")

    finally:
        # --- 5. Close the Port ---
        # This is critical to release the COM port for other applications.
        if ser and ser.is_open:
            ser.close()
            print("\nPort closed.")

if __name__ == "__main__":
    target_port = find_serial_port()
    if target_port:
        test_usb_serial_echo(target_port)
    else:
        print("\n❌ Could not find the target device.")
        print(f"   Make sure your device is connected and its description contains '{TARGET_DEVICE_DESCRIPTION}'.")
        print("   You can check the correct name in the Windows Device Manager under 'Ports (COM & LPT)'.")