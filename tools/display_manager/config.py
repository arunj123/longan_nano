# config.py
"""
Configuration file for the LCD Display Manager.
This version introduces multiple color themes and randomly selects one at startup.
"""
import random # Import the random module
import os

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
LOCATION_LAT = 49.4247  # Hasenbuk, Nürnberg
LOCATION_LON = 11.0896
WEATHER_UPDATE_INTERVAL_SECONDS = 15 * 60

# -- UI Layout and Fonts --
try:
    from PIL import ImageFont

    # Get the absolute path of the directory containing this config file.
    _CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
    # Construct paths to the font files relative to this script's location.
    FONT_PATH_BOLD = os.path.join(_CURRENT_DIR, "fonts", "FreeSansBold.ttf")
    FONT_PATH_REGULAR = os.path.join(_CURRENT_DIR, "fonts", "Ubuntu-L.ttf")
    
    FONT_TIME = ImageFont.truetype(FONT_PATH_BOLD, 32)
    FONT_TEMP = ImageFont.truetype(FONT_PATH_BOLD, 22)
    FONT_DATE = ImageFont.truetype(FONT_PATH_REGULAR, 16)
except IOError:
    raise IOError("Error: Font files not found. Make sure 'FreeSansBold.ttf' and 'Ubuntu-L.ttf' are in the 'fonts' subdirectory.")

# -----------------------------------------------------------------------------
# -- NEW: COLOR THEME PALETTE
# -----------------------------------------------------------------------------
# A list of predefined color themes. You can add your own here.
COLOR_THEMES = [
    {
        "name": "Twilight",
        "gradient_start": (20, 0, 80),      # Deep Violet
        "gradient_end": (100, 20, 120),     # Vibrant Magenta
        "text_primary": (255, 255, 255),
        "text_secondary": (200, 200, 220)
    },
    {
        "name": "Oceanic",
        "gradient_start": (0, 10, 60),      # Deep Ocean Blue
        "gradient_end": (10, 80, 140),      # Bright Cyan
        "text_primary": (255, 255, 255),
        "text_secondary": (190, 210, 220)
    },
    {
        "name": "Sunset",
        "gradient_start": (80, 20, 0),       # Dark Orange
        "gradient_end": (220, 100, 0),      # Bright Yellow-Orange
        "text_primary": (255, 255, 255),
        "text_secondary": (220, 220, 200)
    },
    {
        "name": "Forest",
        "gradient_start": (5, 40, 5),        # Deep Forest Green
        "gradient_end": (50, 120, 50),       # Leafy Green
        "text_primary": (255, 255, 255),
        "text_secondary": (200, 220, 200)
    },
    {
        "name": "Graphite",
        "gradient_start": (20, 20, 25),       # Very Dark Grey
        "gradient_end": (80, 80, 90),       # Lighter Grey/Blue
        "text_primary": (0, 200, 255),      # Bright Cyan Accent
        "text_secondary": (0, 150, 200)
    }
]

# --- RANDOMLY SELECT AND APPLY A THEME ON STARTUP ---
selected_theme = random.choice(COLOR_THEMES)
print(f"--- Theme Selected: {selected_theme['name']} ---")

# -- UI Colors (Now dynamically set from the selected theme) --
COLOR_GRADIENT_START = selected_theme["gradient_start"]
COLOR_GRADIENT_END = selected_theme["gradient_end"]
COLOR_TEXT_PRIMARY = selected_theme["text_primary"]
COLOR_TEXT_SECONDARY = selected_theme["text_secondary"]
COLOR_SEPARATOR = (255, 255, 255, 35) # This remains consistent for a subtle look

# -- State File --
STATE_IMAGE_PATH = "current_display.png"