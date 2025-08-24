import sys
import time
import hid
from PIL import Image, ImageDraw, ImageFont, ImageChops

# -----------------------------------------------------------------------------
# -- DEVICE & PROTOCOL CONFIGURATION
# -----------------------------------------------------------------------------
VID = 0x28E9
PID = 0xABCD
REPORT_LENGTH = 64
REPORT_ID = 0x00

# Commands must match the firmware
CMD_START_QUADRANT_TRANSFER = 0x05
CMD_IMAGE_DATA = 0x02
CMD_DRAW_RECT = 0x06

# -----------------------------------------------------------------------------
# -- IMAGE & FONT CONFIGURATION
# -----------------------------------------------------------------------------
LCD_WIDTH = 160
LCD_HEIGHT = 80
STATE_IMAGE_PATH = "current_display.png"  # Stores the last sent image
OUTPUT_BIN_PATH = "image_160x80.bin"      # For full transfers
UPDATE_BIN_PATH = "update.bin"            # For partial transfers

try:
    FONT_LARGE = ImageFont.truetype("FreeSans.ttf", 16)
    FONT_SMALL = ImageFont.truetype("Ubuntu-L.ttf", 11)
except IOError:
    print("Error: Font file 'DejaVuSans.ttf' not found.")
    sys.exit(1)

# -----------------------------------------------------------------------------
# -- IMAGE GENERATION & DIFFERENCE DETECTION
# -----------------------------------------------------------------------------
def create_image(counter):
    """Generates the display image. The counter makes the content dynamic."""
    bg_color = (10, 20, 40)
    image = Image.new('RGB', (LCD_WIDTH, LCD_HEIGHT), color=bg_color)
    draw = ImageDraw.Draw(image)

    # Display a dynamic temperature value
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

# -----------------------------------------------------------------------------
# -- USB HID COMMUNICATION
# -----------------------------------------------------------------------------
class DeviceManager:
    def __init__(self):
        self.device = None

    def connect(self):
        """Finds and connects to the Custom HID device."""
        if self.device:
            return True
        try:
            for device_dict in hid.enumerate(VID, PID):
                if device_dict['usage_page'] == 0xFF00:
                    print(f"Found device at path: {device_dict['path']}")
                    self.device = hid.device()
                    self.device.open_path(device_dict['path'])
                    print("--- Device Connected ---")
                    return True
            print("Device not found. Retrying...", end='\r')
            return False
        except Exception as e:
            print(f"Connection error: {e}")
            self.device = None
            return False

    def send_data_payload(self, data):
        """Sends a raw data payload in chunks."""
        bytes_sent = 0
        chunk_size = REPORT_LENGTH - 2  # Report ID (1) + Command (1)
        while bytes_sent < len(data):
            chunk = data[bytes_sent : bytes_sent + chunk_size]
            packet = bytearray([REPORT_ID, CMD_IMAGE_DATA])
            packet.extend(chunk)
            if len(packet) < REPORT_LENGTH:
                packet.extend([0] * (REPORT_LENGTH - len(packet)))
            self.device.write(packet)
            bytes_sent += len(chunk)
            # A small delay is crucial for some USB host controllers
            time.sleep(0.001)

    def send_full_image(self, image):
        """Sends the entire image to the device using the quadrant method."""
        print("\n--- Sending Full Image (4 Quadrants) ---")
        image_data = image.tobytes("raw", "RGB")
        
        # Convert RGB888 to RGB565
        rgb565_data = bytearray()
        for i in range(0, len(image_data), 3):
            r, g, b = image_data[i:i+3]
            pixel = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)
            rgb565_data.extend(pixel.to_bytes(2, 'little'))

        quadrant_bytes = LCD_WIDTH * (LCD_HEIGHT // 4) * 2
        for i in range(4):
            print(f"Sending Quadrant {i}...")
            # Command packet: [ID, CMD, quadrant_index]
            cmd_packet = bytearray([REPORT_ID, CMD_START_QUADRANT_TRANSFER, i])
            cmd_packet.extend([0] * (REPORT_LENGTH - len(cmd_packet)))
            self.device.write(cmd_packet)
            time.sleep(0.05)
            
            start = i * quadrant_bytes
            end = start + quadrant_bytes
            self.send_data_payload(rgb565_data[start:end])
        print("--- Full Image Sent ---")

    def send_partial_update(self, image, bbox):
        """Sends only the changed portion of the image."""
        x1, y1, x2, y2 = bbox
        w, h = x2 - x1, y2 - y1
        print(f"\n--- Sending Partial Update at ({x1},{y1}) size {w}x{h} ---")
        
        # Crop the changed area from the new image
        cropped_image = image.crop(bbox)
        
        # Convert cropped image data to RGB565
        image_data = cropped_image.tobytes("raw", "RGB")
        rgb565_data = bytearray()
        for i in range(0, len(image_data), 3):
            r, g, b = image_data[i:i+3]
            pixel = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)
            rgb565_data.extend(pixel.to_bytes(2, 'little'))
            
        # Command packet: [ID, CMD, x, y, w, h]
        cmd_packet = bytearray([REPORT_ID, CMD_DRAW_RECT, x1, y1, w, h])
        cmd_packet.extend([0] * (REPORT_LENGTH - len(cmd_packet)))
        self.device.write(cmd_packet)
        time.sleep(0.05)
        
        self.send_data_payload(rgb565_data)
        print("--- Partial Update Sent ---")

    def close(self):
        if self.device:
            self.device.close()
            print("--- Device Disconnected ---")


# -----------------------------------------------------------------------------
# -- MAIN WORKFLOW
# -----------------------------------------------------------------------------
if __name__ == "__main__":
    manager = DeviceManager()
    update_counter = 0
    try:
        while not manager.connect():
            time.sleep(2)

        # Load the previous state, if it exists
        try:
            previous_image = Image.open(STATE_IMAGE_PATH)
        except FileNotFoundError:
            previous_image = None
            print("No previous state found. A full image transfer will be performed.")

        while True:
            # 1. Generate the new, dynamic image content
            new_image = create_image(update_counter)
            
            # 2. Compare with the previous state to decide on update type
            if not previous_image:
                # No previous state, must do a full transfer
                manager.send_full_image(new_image)
            else:
                bbox = get_image_diff(previous_image, new_image)
                if bbox:
                    # Changes were found, do a partial update
                    manager.send_partial_update(new_image, bbox)
                else:
                    # No changes, do nothing
                    print("No changes detected. Skipping update.", end='\r')

            # 3. Save the new image as the current state for the next iteration
            previous_image = new_image
            previous_image.save(STATE_IMAGE_PATH)
            
            update_counter += 1
            time.sleep(2) # Wait before generating the next update

    except KeyboardInterrupt:
        print("\nExiting.")
    except Exception as e:
        print(f"\nAn unhandled error occurred: {e}")
    finally:
        manager.close()