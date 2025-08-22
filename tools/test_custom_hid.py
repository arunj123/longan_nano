import hid
import time
import sys

VID = 0x28E9
PID = 0xABCD

# Protocol commands must match the firmware
CMD_START_QUADRANT_TRANSFER = 0x05
CMD_IMAGE_DATA     = 0x02
QUADRANT_BYTES = 160 * 20 * 2 # 6400 bytes

# The HID descriptor mandates that all output reports are this size.
REPORT_LENGTH = 64
# The default Report ID for our custom interface is 0x00
REPORT_ID = 0x00

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

def send_quadrant(device_handle, quadrant_index, quadrant_data):
    """Sends a single quadrant, prepending the Report ID to all packets."""
    print(f"\n--- Sending Quadrant {quadrant_index} ({len(quadrant_data)} bytes) ---")
    
    # --- Prepend the Report ID (0x00) to the start command ---
    start_packet = bytearray([REPORT_ID, CMD_START_QUADRANT_TRANSFER, quadrant_index])
    start_packet.extend([0] * (REPORT_LENGTH - len(start_packet))) # Pad to 64 bytes
    
    device_handle.write(start_packet)
    time.sleep(0.0005) 

    bytes_sent = 0
    # --- Adjust chunk size to account for Report ID + Command Byte ---
    chunk_size = REPORT_LENGTH - 2 
    
    while bytes_sent < len(quadrant_data):
        chunk = quadrant_data[bytes_sent : bytes_sent + chunk_size]
        
        # --- Prepend Report ID to the data packet ---
        packet = bytearray([REPORT_ID, CMD_IMAGE_DATA])
        packet.extend(chunk)
        
        if len(packet) < REPORT_LENGTH:
            packet.extend([0] * (REPORT_LENGTH - len(packet)))

        device_handle.write(packet)
        bytes_sent += len(chunk)

        time.sleep(0.0001) 
        print(f"Quadrant {quadrant_index}: Sent {bytes_sent} / {len(quadrant_data)} bytes, {len(chunk)}", end='\r')
    
    print(f"\n--- Quadrant {quadrant_index} Sent ---{' '*20}")


def send_full_image(device_handle, image_path):
    """Reads a full image, splits it into 4 quadrants, and sends them."""
    try:
        with open(image_path, "rb") as f:
            full_image_data = f.read()
        if len(full_image_data) != QUADRANT_BYTES * 4:
            print(f"ERROR: Image file size is incorrect! Expected {QUADRANT_BYTES*4} bytes.")
            return False
    except FileNotFoundError:
        print(f"\n\nERROR: Image file not found at '{image_path}'")
        print("Please run 'prepare_image.py' first to generate it.")
        return False

    quadrants = [full_image_data[i:i + QUADRANT_BYTES] for i in range(0, len(full_image_data), QUADRANT_BYTES)]

    for i, quad_data in enumerate(quadrants):
        send_quadrant(device_handle, i, quad_data)
        time.sleep(0.2) # A longer delay between quadrants to let the device DMA catch up
        
    return True

def main():
    print("--------------------------------------------------")
    print(" Longan Nano Custom HID Image Transfer Script")
    print("--------------------------------------------------")
    device_handle = None

    # Main application loop handles connection, disconnection, and reconnection.
    while True:
        try:
            if not device_handle:
                device_path = find_custom_hid_device()
                if device_path:
                    device_handle = hid.device()
                    device_handle.open_path(device_path)
                    print("\n--- Device Connected ---")
                    
                    if send_full_image(device_handle, "image_160x80.bin"):
                        print("\nImage sent successfully. Script will now exit.")
                    else:
                        print("\nImage sending failed. Exiting.")
                    
                    # Exit after the transfer attempt is complete.
                    break 

                else:
                    # If no device is found, wait and then restart the loop to try again.
                    print("Device not found. Retrying in 3 seconds...", end='\r')
                    time.sleep(3)
        except OSError as e:
            print(f"\n--- Device Disconnected ---")
            if device_handle: device_handle.close()
            device_handle = None
            time.sleep(2)
        except KeyboardInterrupt:
            print("\nExiting.")
            break

    if device_handle:
        device_handle.close()

if __name__ == '__main__':
    main()