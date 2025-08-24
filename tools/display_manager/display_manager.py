# display_manager.py
"""
Main application file.
Connects to the hardware, manages the main loop, fetches data,
and uses the ui_generator to create and display the image.
"""
import time
import os
from datetime import datetime
from PIL import ImageChops
import hid

import config
import weather
import ui_generator

class DeviceManager:
    # ... (This class is correct, no changes needed)
    def __init__(self): self.device, self.sequence_number = None, 0
    def connect(self):
        if self.device: return True
        try:
            for d in hid.enumerate(config.VID, config.PID):
                if d['usage_page'] == 0xFF00: self.device = hid.device(); self.device.open_path(d['path']); self.device.set_nonblocking(1); self.sequence_number = 0; print("--- Device Connected ---"); return True
            print("Device not found. Retrying...", end='\r'); return False
        except Exception as e: print(f"Connection error: {e}"); self.device = None; return False
    def send_data_payload(self, data):
        sent, size = 0, config.REPORT_LENGTH - 1
        while sent < len(data): chunk = data[sent:sent + size]; pkt = bytearray([config.REPORT_ID, config.CMD_IMAGE_DATA]); pkt.extend(chunk); pkt.extend([0] * (config.REPORT_LENGTH - len(pkt))); self.device.write(pkt); sent += len(chunk); time.sleep(0.001)
    def send_rect_update(self, image, bbox):
        if not bbox: return
        x1, y1, x2, y2 = bbox; w, h = x2 - x1, y2 - y1
        if w <= 0 or h <= 0: return
        if w * h > config.MAX_PIXELS_PER_CHUNK:
            rows = config.MAX_PIXELS_PER_CHUNK // w or 1
            for y_off in range(0, h, rows): ch = min(rows, h - y_off); self.send_rect_update(image, (x1, y1 + y_off, x2, y1 + y_off + ch))
            return
        print(f"--- Sending Tile #{self.sequence_number} at ({x1},{y1}) size {w}x{h} ---"); img_data = image.crop(bbox).tobytes("raw", "RGB"); d = bytearray()
        for i in range(0, len(img_data), 3): r, g, b = img_data[i:i+3]; p = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3); d.extend(p.to_bytes(2, 'little'))
        s_lsb, s_msb = self.sequence_number & 0xFF, (self.sequence_number >> 8) & 0xFF; payload = bytearray([x1, y1, w, h, s_lsb, s_msb]); cmd = bytearray([config.REPORT_ID, config.CMD_DRAW_RECT]); cmd.extend(payload); cmd.extend([0] * (config.REPORT_LENGTH - len(cmd))); self.device.write(cmd); time.sleep(0.005)
        self.send_data_payload(d); self.sequence_number = (self.sequence_number + 1) & 0xFFFF
    def close(self):
        if self.device: self.device.close(); print("--- Device Disconnected ---")


if __name__ == "__main__":
    if os.path.exists(config.STATE_IMAGE_PATH):
        os.remove(config.STATE_IMAGE_PATH)

    manager = DeviceManager()
    try:
        while not manager.connect(): time.sleep(2)
        
        previous_image, current_weather, last_weather_check = None, None, 0

        while True:
            if (time.time() - last_weather_check) > config.WEATHER_UPDATE_INTERVAL_SECONDS:
                current_weather = weather.get_weather(config.LOCATION_LAT, config.LOCATION_LON)
                last_weather_check = time.time()
            
            now = datetime.now()
            time_string = now.strftime("%H:%M")
            # --- MODIFIED: Cleaner date format ---
            date_string = now.strftime("%a %b %d")

            new_image = ui_generator.create_image(time_string, date_string, current_weather)

            if not previous_image:
                print("\n--- Sending Full Initial Image ---")
                manager.send_rect_update(new_image, (0, 0, config.LCD_WIDTH, config.LCD_HEIGHT))
            else:
                bbox = ImageChops.difference(previous_image, new_image).getbbox()
                if bbox: manager.send_rect_update(new_image, bbox)

            new_image.save(config.STATE_IMAGE_PATH)
            previous_image = new_image
            time.sleep(5)
            
    except KeyboardInterrupt:
        print("\nExiting.")
    except Exception as e:
        print(f"\nAn unhandled error occurred: {e}")
    finally:
        manager.close()