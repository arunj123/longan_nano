# display_manager.py
"""
Main application for the Longan Nano Dynamic Display.

This script acts as the central controller. It performs the following tasks:
1.  Connects to the Longan Nano device over a custom HID interface.
2.  Periodically fetches live weather data.
3.  Generates a user interface image displaying time, date, and weather.
4.  Compares the new image with the previous one to find changed areas.
5.  Transmits only the changed portions (diffs) of the image to the device's
    LCD, minimizing data transfer and flicker.
6.  Handles device connection and disconnection gracefully.
7.  Uses a separate thread to listen for device events (like button presses)
    for immediate responsiveness.
"""
import time
import os
from datetime import datetime
from PIL import Image, ImageChops
import hid
import sys
import threading

# --- Make imports robust by adding the script's directory to the path ---
# This allows the script to be run from any directory.
_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if _SCRIPT_DIR not in sys.path:
    sys.path.insert(0, _SCRIPT_DIR)

import config
import weather
import ui_generator

# A mutable list is used as a simple, thread-safe way for the listener
# thread to signal the main thread. The main thread checks this flag,
# and the listener thread sets it.
theme_change_requested = [False]

class DeviceManager:
    """Manages low-level HID communication with the Longan Nano device."""
    def __init__(self):
        """Initializes the device manager."""
        self.device = None
        self.sequence_number = 0

    def connect(self) -> bool:
        """
        Scans for and connects to the custom HID device.

        It identifies the correct interface by its Vendor ID, Product ID, and
        a specific usage page (0xFF00) defined in the firmware.

        Returns:
            bool: True if connection was successful, False otherwise.
        """
        if self.device: return True
        try:
            for device_info in hid.enumerate(config.VID, config.PID):
                if device_info['usage_page'] == 0xFF00:
                    self.device = hid.device()
                    self.device.open_path(device_info['path'])
                    self.device.set_nonblocking(1)
                    self.sequence_number = 0
                    print("--- Device Connected ---")
                    return True
            print("Device not found. Retrying...", end='\r')
            return False
        except Exception as e:
            print(f"Connection error: {e}")
            self.device = None
            return False

    def send_data_payload(self, data: bytearray):
        """
        Sends a raw data payload, splitting it into HID report-sized chunks.

        This function is the second part of the two-step update process. It sends
        the raw pixel data that was described by a preceding `CMD_DRAW_RECT` command.
        The data is chunked to fit into 64-byte HID reports.

        Args:
            data (bytearray): The raw byte data to send.
        
        Raises:
            OSError: If a HID write operation fails, indicating a likely disconnection.
        """
        sent_bytes = 0
        # The actual payload size per report is the report length minus the overhead
        # for the Report ID (1 byte) and the Command ID (1 byte).
        payload_size = config.REPORT_LENGTH - 1
        while sent_bytes < len(data):
            chunk = data[sent_bytes : sent_bytes + payload_size]
            packet = bytearray([config.REPORT_ID, config.CMD_IMAGE_DATA])
            packet.extend(chunk)
            packet.extend([0] * (config.REPORT_LENGTH - len(packet)))
            bytes_written = self.device.write(packet)
            if bytes_written < 0: raise OSError("HID write failed. Device may be disconnected.")
            sent_bytes += len(chunk)
            # A small delay is crucial for flow control. It gives the firmware time
            # to process the incoming packet and write it to the LCD's DMA buffer
            # before the next packet arrives.
            time.sleep(0.001) 

    def send_rect_update(self, image: Image.Image, bbox: tuple[int, int, int, int]):
        """
        Sends a rectangular portion of an image to the device.

        This function implements the core display update protocol:
        1.  If the update area is too large for the device's buffer, it is
            recursively split into smaller, horizontal chunks.
        2.  A `CMD_DRAW_RECT` command packet is sent, containing the
            coordinates (x, y), dimensions (w, h), and a sequence number.
        3.  The corresponding image data is cropped, converted from RGB to
            RGB565 format, and sent as a data payload.

        Args:
            image (Image.Image): The full PIL Image object.
            bbox (tuple): The bounding box (x1, y1, x2, y2) of the area to update.
        
        Raises:
            OSError: If a HID write operation fails, indicating a likely disconnection.
        """
        if not bbox: return
        x1, y1, x2, y2 = bbox
        width, height = x2 - x1, y2 - y1
        if width <= 0 or height <= 0: return
        if width * height > config.MAX_PIXELS_PER_CHUNK:
            rows_per_chunk = config.MAX_PIXELS_PER_CHUNK // width or 1
            # Iterate through the large bounding box in horizontal strips.
            for y_offset in range(0, height, rows_per_chunk):
                chunk_height = min(rows_per_chunk, height - y_offset)
                new_bbox = (x1, y1 + y_offset, x2, y1 + y_offset + chunk_height)
                # Recursively call this function for each smaller chunk.
                self.send_rect_update(image, new_bbox)
            return
        print(f"--- Sending Tile #{self.sequence_number} at ({x1},{y1}) size {width}x{height} ---")
        
        # Crop the image and convert from 8-bits-per-channel RGB to 16-bit RGB565.
        img_data = image.crop(bbox).tobytes("raw", "RGB")
        pixel_data_rgb565 = bytearray()
        for i in range(0, len(img_data), 3):
            r, g, b = img_data[i:i+3]
            # Bitwise operations to pack the 24-bit color into 16 bits.
            # RRRRRGGG GGGBBBBB
            rgb565 = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)
            # Append the 16-bit value as two bytes (little-endian).
            pixel_data_rgb565.extend(rgb565.to_bytes(2, 'little'))
        
        # Construct the command packet payload.
        seq_lsb = self.sequence_number & 0xFF
        seq_msb = (self.sequence_number >> 8) & 0xFF
        payload = bytearray([x1, y1, width, height, seq_lsb, seq_msb])
        
        # Prepend Report ID and Command ID, then pad to the required report length.
        command_packet = bytearray([config.REPORT_ID, config.CMD_DRAW_RECT])
        command_packet.extend(payload)
        command_packet.extend([0] * (config.REPORT_LENGTH - len(command_packet)))
        bytes_written = self.device.write(command_packet)
        if bytes_written < 0: raise OSError("HID write failed. Device may be disconnected.")
        time.sleep(0.005) # Wait for firmware to process the command.
        self.send_data_payload(pixel_data_rgb565)
        self.sequence_number = (self.sequence_number + 1) & 0xFFFF

    def close(self):
        """Closes the connection to the HID device."""
        if self.device:
            self.device.close()
            self.device = None
            print("--- Device Disconnected ---")


# --- START OF MODIFICATION ---
def device_listener(device_manager: DeviceManager, stop_event: threading.Event):
    """
    Listens for incoming HID reports from the device in a separate thread.

    This function runs in a background thread and performs blocking reads on the
    HID device. This is an efficient way to wait for device-initiated events
    (like a button press) without consuming CPU cycles in the main loop.

    Args:
        device_manager (DeviceManager): The active device manager instance.
        stop_event (threading.Event): An event to signal when the thread should exit.
    """
    print("--- Listener thread started ---")
    while not stop_event.is_set():
        try:
            if device_manager.device:
                # Use a blocking read with a timeout. This is efficient, as the
                # thread will sleep until a report arrives or 500ms passes.
                report = device_manager.device.read(64, timeout_ms=500)
                # Check for the specific report sent by the firmware's USER button press.
                # report[0] is the Report ID, report[1] is the button state.
                if report and report[0] == 0x01 and report[1] == 0x01:
                    print("--- Theme change request received from device ---")
                    theme_change_requested[0] = True
            else:
                # If the device is not connected, wait a bit before checking again.
                time.sleep(1)
        except OSError:
            # This can happen if the device disconnects while we are reading.
            # The main loop will handle the reconnection.
            time.sleep(1)
    print("--- Listener thread stopped ---")
# --- END OF MODIFICATION ---


def main():
    """
    Main execution function.

    Sets up the device manager and listener thread, then enters a main loop
    to handle UI updates, data fetching, and device disconnections.
    """
    if os.path.exists(config.STATE_IMAGE_PATH):
        os.remove(config.STATE_IMAGE_PATH)

    manager = DeviceManager()
    
    # --- START OF MODIFICATION ---
    stop_event = threading.Event()
    listener_thread = None
    # --- END OF MODIFICATION ---

    try:
        # This outer loop handles device connection and reconnection.
        while True:
            try:
                # 1. Connect to the device. If not found, sleep and retry.
                if not manager.connect():
                    time.sleep(2)
                    continue

                # 2. Start the listener thread once the device is connected.
                if not listener_thread or not listener_thread.is_alive():
                    stop_event.clear()
                    theme_change_requested[0] = False
                    listener_thread = threading.Thread(
                        target=device_listener,
                        args=(manager, stop_event),
                        daemon=True  # Allows main program to exit even if thread is running
                    )
                    listener_thread.start()

                # 3. Reset state for a fresh start after connection.
                previous_image = None
                previous_time_string = ""
                current_weather = None
                last_weather_check = 0

                # 4. Inner loop for continuous display updates.
                while True:
                    # --- Event Handling ---
                    # Check if the listener thread has requested a theme change.
                    if theme_change_requested[0]:
                        config.cycle_theme()
                        previous_image = None  # Force a full redraw on the next iteration
                        theme_change_requested[0] = False # Reset the flag
                    
                    # If the device has disconnected, break to the outer loop to reconnect.
                    if not manager.device:
                        raise OSError("Device disconnected")

                    # --- Data Fetching ---
                    # Fetch weather data at the specified interval.
                    if (time.time() - last_weather_check) > config.WEATHER_UPDATE_INTERVAL_SECONDS:
                        current_weather = weather.get_weather(config.LOCATION_LAT, config.LOCATION_LON)
                        last_weather_check = time.time()
                    
                    now = datetime.now()
                    time_string = now.strftime("%H:%M")
                    
                    # --- UI Generation and Diffing ---
                    # Optimization: Only redraw the screen if the time has changed
                    # or a full redraw has been forced (previous_image is None).
                    if time_string == previous_time_string and previous_image is not None:
                        time.sleep(1) # Check again in one second.
                        continue

                    # Generate the new UI image.
                    date_string = now.strftime("%a %b %d")
                    new_image = ui_generator.create_image(time_string, date_string, current_weather)

                    # Determine update type: full or partial.
                    if not previous_image:
                        print("\n--- Sending Full Initial Image ---")
                        manager.send_rect_update(new_image, (0, 0, config.LCD_WIDTH, config.LCD_HEIGHT))
                    else:
                        # Find the difference between the old and new images.
                        diff = ImageChops.difference(previous_image, new_image)
                        bbox = diff.getbbox()
                        if bbox:
                            manager.send_rect_update(new_image, bbox)

                    # Save the new image as the state for the next comparison.
                    new_image.save(config.STATE_IMAGE_PATH)
                    previous_image = new_image
                    previous_time_string = time_string
                    # Wait a short time before checking for updates again.
                    time.sleep(1)

            except OSError as e:
                print(f"\nDevice error or disconnection: {e}")
                if listener_thread:
                    stop_event.set() # Signal the listener thread to stop
                    listener_thread.join(timeout=1) # Wait for it to exit
                manager.close()
                print("Attempting to reconnect in 3 seconds...")
                time.sleep(3)

    except KeyboardInterrupt:
        print("\nExiting.")
    finally:
        # Ensure resources are cleaned up on exit.
        if listener_thread:
            stop_event.set()
        if manager:
            manager.close()

if __name__ == "__main__":
    main()