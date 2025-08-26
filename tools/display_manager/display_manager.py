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
"""
import time
import os
from datetime import datetime
from PIL import Image, ImageChops
import hid
import sys

# --- Make imports robust by adding the script's directory to the path ---
# This allows the script to be run from any directory.
_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if _SCRIPT_DIR not in sys.path:
    sys.path.insert(0, _SCRIPT_DIR)

import config
import weather
import ui_generator

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
        if self.device:
            return True
        try:
            # Enumerate all HID devices to find our specific custom interface.
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

        This is used to transmit the pixel data for an image update, following
        a `CMD_DRAW_RECT` command.

        Args:
            data (bytearray): The raw byte data to send.
        """
        sent_bytes = 0
        payload_size = config.REPORT_LENGTH - 1  # Report ID (1) + Command Byte (1)

        while sent_bytes < len(data):
            chunk = data[sent_bytes : sent_bytes + payload_size]
            
            # Construct the packet: [Report ID, Command, Data..., Padding...]
            packet = bytearray([config.REPORT_ID, config.CMD_IMAGE_DATA])
            packet.extend(chunk)
            packet.extend([0] * (config.REPORT_LENGTH - len(packet)))
            
            # Check the return value of write(). It returns -1 on error.
            bytes_written = self.device.write(packet)
            if bytes_written < 0:
                raise OSError("HID write failed. Device may be disconnected.")

            sent_bytes += len(chunk)
            # A small delay is crucial for flow control with the firmware.
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
        """
        if not bbox:
            return
        
        x1, y1, x2, y2 = bbox
        width, height = x2 - x1, y2 - y1

        if width <= 0 or height <= 0:
            return

        # If the update is too large, split it into smaller horizontal strips.
        if width * height > config.MAX_PIXELS_PER_CHUNK:
            # Calculate how many rows can fit in one chunk.
            rows_per_chunk = config.MAX_PIXELS_PER_CHUNK // width or 1
            for y_offset in range(0, height, rows_per_chunk):
                chunk_height = min(rows_per_chunk, height - y_offset)
                new_bbox = (x1, y1 + y_offset, x2, y1 + y_offset + chunk_height)
                self.send_rect_update(image, new_bbox)
            return

        print(f"--- Sending Tile #{self.sequence_number} at ({x1},{y1}) size {width}x{height} ---")

        # 1. Prepare pixel data: Crop, get raw RGB bytes, convert to RGB565.
        img_data = image.crop(bbox).tobytes("raw", "RGB")
        pixel_data_rgb565 = bytearray()
        for i in range(0, len(img_data), 3):
            r, g, b = img_data[i:i+3]
            # Convert 24-bit RGB to 16-bit RGB565
            rgb565 = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)
            pixel_data_rgb565.extend(rgb565.to_bytes(2, 'little'))

        # 2. Send the command packet to tell the device where to draw.
        seq_lsb = self.sequence_number & 0xFF
        seq_msb = (self.sequence_number >> 8) & 0xFF
        payload = bytearray([x1, y1, width, height, seq_lsb, seq_msb])
        
        command_packet = bytearray([config.REPORT_ID, config.CMD_DRAW_RECT])
        command_packet.extend(payload)
        command_packet.extend([0] * (config.REPORT_LENGTH - len(command_packet)))
        
        # Check the return value of write(). It returns -1 on error.
        bytes_written = self.device.write(command_packet)
        if bytes_written < 0:
            raise OSError("HID write failed. Device may be disconnected.")
        time.sleep(0.005) # Wait for firmware to process the command.

        # 3. Send the actual pixel data.
        self.send_data_payload(pixel_data_rgb565)
        self.sequence_number = (self.sequence_number + 1) & 0xFFFF

    def close(self):
        """Closes the connection to the HID device."""
        if self.device:
            self.device.close()
            self.device = None
            print("--- Device Disconnected ---")


def main():
    """Main execution function."""
    # Clean up previous state file on startup
    if os.path.exists(config.STATE_IMAGE_PATH):
        os.remove(config.STATE_IMAGE_PATH)

    manager = DeviceManager()
    # The main application loop. This outer loop handles device disconnections
    # and automatically attempts to reconnect.
    try:
        while True:
            try:
                # 1. Connect to the device. If not found, sleep and retry.
                if not manager.connect():
                    time.sleep(2)
                    continue

                # 2. Reset state for a fresh start after connection.
                previous_image = None
                current_weather = None
                last_weather_check = 0

                # 3. Inner loop for continuous display updates.
                while True:
                    # Fetch weather data at the specified interval
                    if (time.time() - last_weather_check) > config.WEATHER_UPDATE_INTERVAL_SECONDS:
                        current_weather = weather.get_weather(config.LOCATION_LAT, config.LOCATION_LON)
                        last_weather_check = time.time()
                    
                    # Generate the new UI image
                    now = datetime.now()
                    time_string = now.strftime("%H:%M")
                    date_string = now.strftime("%a %b %d")
                    new_image = ui_generator.create_image(time_string, date_string, current_weather)

                    # Determine update type: full or partial
                    if not previous_image:
                        # On first run, send the entire screen
                        print("\n--- Sending Full Initial Image ---")
                        manager.send_rect_update(new_image, (0, 0, config.LCD_WIDTH, config.LCD_HEIGHT))
                    else:
                        # On subsequent runs, find the difference and send only that
                        diff = ImageChops.difference(previous_image, new_image)
                        bbox = diff.getbbox()
                        if bbox:
                            manager.send_rect_update(new_image, bbox)

                    # Save the new image as the state for the next comparison
                    new_image.save(config.STATE_IMAGE_PATH)
                    previous_image = new_image
                    
                    # Wait before generating the next frame
                    time.sleep(5)
            except OSError as e:
                print(f"\nDevice error or disconnection: {e}")
                manager.close() # Properly close the handle, sets internal device to None
                print("Attempting to reconnect in 3 seconds...")
                time.sleep(3)
    except KeyboardInterrupt:
        print("\nExiting.")
    finally:
        if manager:
            manager.close()

if __name__ == "__main__":
    main()