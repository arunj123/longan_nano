import hid
import time
import sys

# Vendor and Product ID must match your device's descriptors
# From usbd_descriptors.cpp:
#   #define USBD_VID                     0x28E9
#   #define USBD_PID                     0xABCD
VID = 0x28E9
PID = 0xABCD

def find_custom_hid_device():
    """
    Finds the specific custom HID interface of the composite device.
    This function performs a single scan. The main loop will handle retries.
    """
    # The usage page for our custom interface is 0xFF00
    for device_dict in hid.enumerate():
        if device_dict['vendor_id'] == VID and \
           device_dict['product_id'] == PID and \
           device_dict['usage_page'] == 0xFF00:
            print(f"Found Custom HID device at path: {device_dict['path']}")
            return device_dict['path']
    return None

def main():
    print("--------------------------------------------------")
    print(" Longan Nano Custom HID Communication Test Script")
    print("--------------------------------------------------")
    print("This script will automatically search for and connect to the device.")
    print("Press Ctrl+C to exit.")

    device_handle = None

    # Main application loop handles connection, disconnection, and reconnection.
    while True:
        try:
            # --- STAGE 1: CONNECTION ---
            # If the device is not connected, search for it.
            if not device_handle:
                device_path = find_custom_hid_device()
                if device_path:
                    print("Device found. Attempting to open...")
                    device_handle = hid.device()
                    device_handle.open_path(device_path)
                    device_handle.set_nonblocking(1)
                    print("\n--- Device Connected ---")
                    print("-> Press the USER BUTTON on your Longan Nano board to send data.")
                    print("-> This script will now cycle the RGB LED on the board.")
                else:
                    # If no device is found, wait and then restart the loop to try again.
                    print("Device not found. Retrying in 3 seconds...", end='\r')
                    time.sleep(3)
                    continue

            # --- STAGE 2: COMMUNICATION ---
            # If we have a valid device handle, proceed with reading and writing.

            # 1. Try to read data from the device (user button presses)
            report = device_handle.read(64)
            if report:
                # The first byte is the Report ID (0x15), the second is the counter
                print(f"-> Received IN-Report: Report ID=0x{report[0]:02X}, Counter={report[1]}")

            # 2. Send data to the device to control LEDs
            print("Cycling LEDs (Sending OUT-Reports)...", end='\r')
            
            # Turn RED on
            device_handle.write([0x11, 0x01]) # Report ID 0x11, Value 1 (ON)
            time.sleep(0.5)

            # Turn RED off, GREEN on
            device_handle.write([0x11, 0x00]) # Report ID 0x11, Value 0 (OFF)
            device_handle.write([0x12, 0x01]) # Report ID 0x12, Value 1 (ON)
            time.sleep(0.5)

            # Turn GREEN off, BLUE on
            device_handle.write([0x12, 0x00]) # Report ID 0x12, Value 0 (OFF)
            device_handle.write([0x13, 0x01]) # Report ID 0x13, Value 1 (ON)
            time.sleep(0.5)
            
            # Turn BLUE off
            device_handle.write([0x13, 0x00]) # Report ID 0x13, Value 0 (OFF)
            time.sleep(0.5)

        # CORRECTED EXCEPTION TYPE: This is the key fix.
        except OSError as e:
            print(f"\n\n--- Device Disconnected or Communication Error ---")
            print(f"Error details: {e}")
            print("Closing handle and starting rescan...")
            if device_handle:
                # We do not need to call close() on a handle that has already errored out.
                # Setting it to None is sufficient to trigger the reconnection logic.
                pass
            device_handle = None
            # A short delay gives the OS time to process the disconnection
            time.sleep(2)

        except KeyboardInterrupt:
            print("\n\nExiting by user request.")
            break

        except Exception as e:
            print(f"\nAn unexpected error occurred: {e}")
            print("Resetting connection.")
            if device_handle:
                device_handle.close()
            device_handle = None
            time.sleep(2)

    # --- STAGE 3: CLEANUP ---
    # Ensure the device is closed on exit.
    if device_handle:
        print("Closing device.")
        device_handle.close()

if __name__ == '__main__':
    main()