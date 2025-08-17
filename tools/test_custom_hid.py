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
    We identify it by its Vendor ID, Product ID, and the vendor-defined
    Usage Page (0xFF00).
    """
    print("Searching for Custom HID device (VID=0x{:04X}, PID=0x{:04X})...".format(VID, PID))
    
    # The usage page for our custom interface is 0xFF00
    # hid.enumerate() returns a list of dictionaries
    for device_dict in hid.enumerate():
        if device_dict['vendor_id'] == VID and \
           device_dict['product_id'] == PID and \
           device_dict['usage_page'] == 0xFF00:
            print("Found Custom HID device!")
            return device_dict['path']
            
    return None

def main():
    device_path = find_custom_hid_device()

    if not device_path:
        print("\nERROR: Device not found. Make sure it's plugged in and has enumerated correctly.")
        print("Check Device Manager. If the device has a yellow icon, there might be a driver issue or firmware problem.")
        sys.exit(1)

    try:
        # Open the device
        h = hid.device()
        h.open_path(device_path)
        h.set_nonblocking(1) # Important for a responsive loop

        print("\n--- Custom HID Test ---")
        print("Device opened successfully.")
        print("-> Press the USER BUTTON on your Longan Nano board.")
        print("   You should see 'Received data:' messages here.")
        print("-> This script will now cycle the RGB LED on the board.")
        print("Press Ctrl+C to exit.")

        led_state = 0
        while True:
            # --- 1. Try to read data from the device ---
            # This is the data sent from the EXTI5_9_IRQHandler in your firmware
            data = h.read(64) # Read up to 64 bytes
            if data:
                # The first byte is the Report ID (0x15), the second is the counter
                print(f"Received data: Report ID=0x{data[0]:02X}, Counter={data[1]}")

            # --- 2. Send data to the device to control LEDs ---
            print("\nCycling LEDs...")

            # Turn RED on
            print("  - RED ON")
            h.write([0x11, 0x01]) # Report ID 0x11, Value 1 (ON)
            time.sleep(0.5)

            # Turn RED off, GREEN on
            print("  - RED OFF, GREEN ON")
            h.write([0x11, 0x00]) # Report ID 0x11, Value 0 (OFF)
            h.write([0x12, 0x01]) # Report ID 0x12, Value 1 (ON)
            time.sleep(0.5)

            # Turn GREEN off, BLUE on
            print("  - GREEN OFF, BLUE ON")
            h.write([0x12, 0x00]) # Report ID 0x12, Value 0 (OFF)
            h.write([0x13, 0x01]) # Report ID 0x13, Value 1 (ON)
            time.sleep(0.5)
            
            # Turn BLUE off
            print("  - BLUE OFF")
            h.write([0x13, 0x00]) # Report ID 0x13, Value 0 (OFF)
            time.sleep(0.5)


    except KeyboardInterrupt:
        print("\nExiting.")
    except Exception as e:
        print(f"\nAn error occurred: {e}")
    finally:
        if 'h' in locals() and h:
            print("Closing device.")
            h.close()

if __name__ == '__main__':
    print("--------------------------------------------------")
    print(" Longan Nano Custom HID Communication Test Script")
    print("--------------------------------------------------")
    main()