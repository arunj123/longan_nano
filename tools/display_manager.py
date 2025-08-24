import sys
import time
import hid
import os
from PIL import Image, ImageDraw, ImageFont, ImageChops

# -----------------------------------------------------------------------------
# -- DEVICE & PROTOCOL CONFIGURATION
# -----------------------------------------------------------------------------
VID = 0x28E9
PID = 0xABCD
REPORT_LENGTH = 64
REPORT_ID = 0x00

# Commands must match the firmware
# CMD_START_QUADRANT_TRANSFER is removed
CMD_IMAGE_DATA = 0x02
CMD_DRAW_RECT = 0x06

# --- NEW: Buffer configuration matching the device ---
DEVICE_BUFFER_SIZE = 4096
# Max pixels per transfer = 4096 bytes / 2 bytes_per_pixel
MAX_PIXELS_PER_CHUNK = DEVICE_BUFFER_SIZE // 2

# -----------------------------------------------------------------------------
# -- IMAGE & FONT CONFIGURATION
# -----------------------------------------------------------------------------
LCD_WIDTH = 160
LCD_HEIGHT = 80
STATE_IMAGE_PATH = "current_display.png"

try:
    FONT_LARGE = ImageFont.truetype("FreeSans.ttf", 16)
    FONT_SMALL = ImageFont.truetype("Ubuntu-L.ttf", 11)
except IOError:
    print("Error: Font file 'FreeSans.ttf' or 'Ubuntu-L.ttf' not found.")
    sys.exit(1)

def create_image(counter):
    """Generates the display image. The counter makes the content dynamic."""
    bg_color = (10, 20, 40)
    image = Image.new('RGB', (LCD_WIDTH, LCD_HEIGHT), color=bg_color)
    draw = ImageDraw.Draw(image)

    temp = 23.5 + (counter % 10) / 10.0
    text = f"Temp: {temp:.1f} C"
    
    draw.text((10, 10), "System Monitor", font=FONT_SMALL, fill=(200, 220, 255))
    draw.rectangle((8, 35, 152, 60), fill=(20, 40, 60))
    draw.text((15, 38), text, font=FONT_LARGE, fill=(255, 255, 100))
    
    return image

def get_image_diff(img1, img2):
    """Compares two images and returns the bounding box of the differences."""
    diff = ImageChops.difference(img1, img2)
    bbox = diff.getbbox()
    return bbox

class DeviceManager:
    def __init__(self):
        self.device = None
        self.sequence_number = 0

    def connect(self):
        if self.device:
            return True
        try:
            for device_dict in hid.enumerate(VID, PID):
                if device_dict['usage_page'] == 0xFF00:
                    print(f"Found device at path: {device_dict['path']}")
                    self.device = hid.device()
                    self.device.open_path(device_dict['path'])
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

    def wait_for_ack(self, timeout_ms=2000):
        """Waits for a specific ACK report from the device."""
        start_time = time.time()
        while (time.time() - start_time) * 1000 < timeout_ms:
            # <-- CRITICAL FIX: The timeout is the second POSITIONAL argument.
            report = self.device.read(64, 10) 
            if report:
                # Check for our ACK: Report ID 1, Data 0xAC (172)
                if report[0] == 1 and report[1] == 0xAC:
                    return True
        print("Error: Timed out waiting for ACK from device.")
        return False

    # (send_data_payload and send_rect_update from the previous turn are correct)
    def send_data_payload(self, data):
        """Sends a raw data payload in chunks."""
        bytes_sent = 0
        chunk_size = REPORT_LENGTH - 1
        while bytes_sent < len(data):
            chunk = data[bytes_sent : bytes_sent + chunk_size]
            packet = bytearray([0, CMD_IMAGE_DATA])
            packet.extend(chunk)
            packet.extend([0] * (REPORT_LENGTH - len(packet)))
            self.device.write(packet)
            bytes_sent += len(chunk)
            time.sleep(0.001)

    def send_rect_update(self, image, bbox):
        if not bbox: return
        x1, y1, x2, y2 = bbox
        w, h = x2 - x1, y2 - y1

        if w <= 0 or h <= 0: return

        if w * h > MAX_PIXELS_PER_CHUNK:
            rows_per_chunk = MAX_PIXELS_PER_CHUNK // w
            if rows_per_chunk == 0: rows_per_chunk = 1
            for y_offset in range(0, h, rows_per_chunk):
                chunk_h = min(rows_per_chunk, h - y_offset)
                chunk_bbox = (x1, y1 + y_offset, x2, y1 + y_offset + chunk_h)
                self.send_rect_update(image, chunk_bbox)
            return

        print(f"--- Sending Tile #{self.sequence_number} at ({x1},{y1}) size {w}x{h} ---")
        
        cropped_image = image.crop(bbox)
        image_data = cropped_image.tobytes("raw", "RGB")
        rgb565_data = bytearray()
        for i in range(0, len(image_data), 3):
            r, g, b = image_data[i:i+3]
            pixel = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)
            rgb565_data.extend(pixel.to_bytes(2, 'little'))
            
        seq_lsb = self.sequence_number & 0xFF
        seq_msb = (self.sequence_number >> 8) & 0xFF
        payload = bytearray([x1, y1, w, h, seq_lsb, seq_msb])
        cmd_packet = bytearray([0, CMD_DRAW_RECT])
        cmd_packet.extend(payload)
        cmd_packet.extend([0] * (REPORT_LENGTH - len(cmd_packet)))

        self.device.write(cmd_packet)
        self.send_data_payload(rgb565_data)
        
        #if not self.wait_for_ack():
        #    raise IOError(f"Device failed to acknowledge tile #{self.sequence_number}")

        self.sequence_number = (self.sequence_number + 1) & 0xFFFF

    def close(self):
        if self.device:
            self.device.close()
            print("--- Device Disconnected ---")


# (The main workflow remains the same)
if __name__ == "__main__":
    if os.path.exists(STATE_IMAGE_PATH):
        os.remove(STATE_IMAGE_PATH)
        print("Removed old state file to force a full initial transfer.")

    manager = DeviceManager()
    update_counter = 0
    try:
        while not manager.connect():
            time.sleep(2)

        previous_image = None
        while True:
            new_image = create_image(update_counter)
            
            if not previous_image:
                print("\n--- Sending Full Image (as synchronized tiles) ---")
                tile_w, tile_h = 40, 40
                for y in range(0, LCD_HEIGHT, tile_h):
                    for x in range(0, LCD_WIDTH, tile_w):
                        right = min(x + tile_w, LCD_WIDTH)
                        bottom = min(y + tile_h, LCD_HEIGHT)
                        bbox = (x, y, right, bottom)
                        manager.send_rect_update(new_image, bbox)
            else:
                bbox = get_image_diff(previous_image, new_image)
                if bbox:
                    manager.send_rect_update(new_image, bbox)

            previous_image = new_image
            previous_image.save(STATE_IMAGE_PATH)
            
            update_counter += 1
            time.sleep(2)

    except KeyboardInterrupt:
        print("\nExiting.")
    except Exception as e:
        print(f"\nAn unhandled error occurred: {e}")
    finally:
        manager.close()