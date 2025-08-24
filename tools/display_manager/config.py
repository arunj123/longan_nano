# config.py
"""
Configuration file for the LCD Display Manager.
This version uses a vibrant gradient and the final, polished font layout.
"""

# -- Device Configuration --
VID = 0x28E9
PID = 0xABCD
REPORT_LENGTH = 64
REPORT_ID = 0x00
CMD_IMAGE_DATA = 0x02
CMD_DRAW_RECT = 0x06
DEVICE_BUFFER_SIZE = 4096
MAX_PIXELS_PER_CHUNK = DEVICE_BUFFER_SIZE // 2
LCD_WIDTH = 160
LCD_HEIGHT = 80

# -- Location & Weather --
LOCATION_LAT = 49.4622  # Hasenbühl, Nürnberg
LOCATION_LON = 11.0118
WEATHER_UPDATE_INTERVAL_SECONDS = 15 * 60

# -- UI Layout and Fonts (FINAL POLISHED LAYOUT) --
try:
    from PIL import ImageFont
    FONT_PATH_BOLD = "FreeSansBold.ttf"
    FONT_PATH_REGULAR = "Ubuntu-L.ttf"
    
    # Fonts are sized for their specific panes in the layout
    FONT_TIME = ImageFont.truetype(FONT_PATH_BOLD, 32)
    FONT_TEMP = ImageFont.truetype(FONT_PATH_BOLD, 22) # <--- MADE LARGER
    FONT_DATE = ImageFont.truetype(FONT_PATH_REGULAR, 16)
except IOError:
    raise IOError("Error: Font files not found. Make sure 'FreeSansBold.ttf' and 'Ubuntu-L.ttf' are in the same directory.")

# -- UI Colors (FINAL) --
COLOR_GRADIENT_START = (20, 0, 80)      # Deep Violet
COLOR_GRADIENT_END = (100, 20, 120)     # Vibrant Magenta
COLOR_TEXT_PRIMARY = (255, 255, 255)
COLOR_TEXT_SECONDARY = (200, 200, 220)
COLOR_SEPARATOR = (255, 255, 255, 35) # White with low alpha for subtlety

# -- State File --
STATE_IMAGE_PATH = "current_display.png"