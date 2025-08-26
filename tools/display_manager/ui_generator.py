# ui_generator.py
"""
Handles all visual and layout aspects of the dynamic display.

This module is responsible for creating the complete UI image that gets sent
to the device. It generates a background, draws weather icons, and arranges
text elements in a clean, structured layout.
"""
from PIL import Image, ImageDraw
from math import sin, cos, radians
import config

def _draw_vibrant_gradient_background() -> Image.Image:
    """
    Creates a visually appealing, high-contrast diagonal gradient background.

    The gradient is calculated based on the (x+y) position of each pixel,
    which produces a smooth diagonal transition between the start and end
    colors defined in the configuration.

    Returns:
        Image.Image: A PIL Image object with the generated gradient.
    """
    image = Image.new('RGB', (config.LCD_WIDTH, config.LCD_HEIGHT))
    r1, g1, b1 = config.COLOR_GRADIENT_START
    r2, g2, b2 = config.COLOR_GRADIENT_END
    
    # Calculate the maximum value for normalization
    max_val = float(config.LCD_WIDTH + config.LCD_HEIGHT)

    for y in range(config.LCD_HEIGHT):
        for x in range(config.LCD_WIDTH):
            # Normalize the position to a 0.0-1.0 scale
            p = (x + y) / max_val
            # Linearly interpolate each color channel
            r = int(r1 + (r2 - r1) * p)
            g = int(g1 + (g2 - g1) * p)
            b = int(b1 + (b2 - b1) * p)
            image.putpixel((x, y), (r, g, b))
    return image

def _create_weather_icon(icon_name: str, size: tuple[int, int]) -> Image.Image:
    """
    Generates high-quality, anti-aliased weather icons with a layered effect.

    Icons are drawn programmatically using PIL's Draw module. A subtle
    drop-shadow effect is achieved by drawing a semi-transparent, offset
    version of the shape before drawing the main, opaque shape.

    Args:
        icon_name (str): The name of the icon to generate (e.g., "sun", "cloud").
        size (tuple): The (width, height) of the icon canvas.

    Returns:
        Image.Image: A PIL Image object (in RGBA mode) of the generated icon.
    """
    icon = Image.new('RGBA', size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(icon)
    w, h = size
    
    # Define color palette for icons
    C_SUN = (255, 200, 0)
    C_MOON = (240, 240, 230) # A soft off-white for the moon
    C_CLOUD = (220, 220, 220)
    C_RAIN = (100, 140, 255)
    C_SNOW = (240, 240, 240)
    C_CLOUD_DARK = (160, 160, 170)
    C_FOG = (180, 180, 180, 150) # Semi-transparent grey for fog
    SHADOW = (0, 0, 0, 50) # Semi-transparent black for shadows
    shadow_offset = (2, 2)

    def draw_sun(offset=(0, 0), color=C_SUN):
        center = (w / 2 + offset[0], h / 2 + offset[1])
        radius = 8
        draw.ellipse((center[0] - radius, center[1] - radius, center[0] + radius, center[1] + radius), fill=color)
        # Draw sun rays
        for i in range(8):
            angle = radians(i * 45)
            x1 = center[0] + cos(angle) * radius
            y1 = center[1] + sin(angle) * radius
            x2 = center[0] + cos(angle) * (radius + 3)
            y2 = center[1] + sin(angle) * (radius + 3)
            draw.line([(x1, y1), (x2, y2)], fill=color, width=2)

    def draw_crescent(offset=(0, 0), color=C_MOON):

        """Draws a crescent moon shape."""
        cx, cy = w / 2 + offset[0], h / 2 + offset[1]
        r = 10
        # Draw the main lit body of the moon
        draw.ellipse((cx - r, cy - r, cx + r, cy + r), fill=color)
        # Draw a semi-transparent overlapping circle to create the "dark side".
        # This gives the crescent effect against the gradient background.
        # The shadow color is semi-transparent black, which darkens what's behind it.
        draw.ellipse((cx - r + 5, cy - r - 2, cx + r + 5, cy + r - 2), fill=SHADOW)

    def draw_cloud(offset=(0, 0), color=C_CLOUD):
        x, y = offset[0] + w / 2, offset[1] + h / 2
        # Compose cloud from two overlapping ellipses
        draw.ellipse((x - 18, y - 5, x + 8, y + 15), fill=color)
        draw.ellipse((x - 8, y - 12, x + 18, y + 12), fill=color)

    def draw_fog(offset=(0,0), color=C_FOG):
        """Draws horizontal lines to represent fog."""
        for y_pos in [h/2 + 2, h/2 + 7, h/2 + 12]:
            draw.line((w/2 - 15 + offset[0], y_pos + offset[1], w/2 + 15 + offset[0], y_pos + offset[1]), fill=color, width=3)

    # Draw icons based on name, always drawing shadow first for a 3D effect
    # --- Clear Weather Icons ---
    if icon_name == "sun":
        draw_sun(shadow_offset, SHADOW)
        draw_sun()
    elif icon_name == "moon":
        # The crescent itself has a shadow, but we add a drop shadow for the whole shape
        draw_crescent(shadow_offset, SHADOW)
        draw_crescent()
    # --- Cloudy Icons ---
    elif icon_name == "cloud":
        draw_cloud(shadow_offset, SHADOW)
        draw_cloud()
    elif icon_name == "sun_cloud":
        draw_cloud(shadow_offset, SHADOW)
        draw_sun(offset=(-4, -4)) # Sun peeking from behind
        draw_cloud(offset=(0, 4))
    elif icon_name == "moon_cloud":
        draw_cloud(shadow_offset, SHADOW)
        draw_crescent(offset=(-4, -4)) # Moon peeking from behind
        draw_cloud(offset=(0, 4))
    elif icon_name == "fog":
        draw_fog(shadow_offset, SHADOW)
        draw_fog()
    # --- Precipitation Icons (Rain/Snow) ---
    elif icon_name in ["drizzle", "rain", "heavy_rain", "light_rain", "snow", "heavy_snow", "light_snow", "freezing_rain"]:
        draw_cloud(shadow_offset, SHADOW)
        draw_cloud(color=C_CLOUD_DARK)
        # --- Rain Variants ---
        if icon_name == "drizzle":
            for pos in [(w/2 - 8, h/2 + 6), (w/2, h/2 + 6), (w/2 + 8, h/2 + 6)]:
                draw.line((pos[0], pos[1], pos[0] + 1, pos[1] + 4), fill=C_RAIN, width=1)
        elif icon_name == "light_rain":
            for pos in [(w/2 - 7, h/2 + 5), (w/2 + 3, h/2 + 5)]:
                draw.line((pos[0] + shadow_offset[0], pos[1] + shadow_offset[1], pos[0] + shadow_offset[0], pos[1] + shadow_offset[1] + 5), fill=SHADOW, width=2)
                draw.line((pos[0], pos[1], pos[0], pos[1] + 5), fill=C_RAIN, width=2)
        elif icon_name == "rain": # Moderate
            for pos in [(w/2 - 8, h/2 + 5), (w/2, h/2 + 5), (w/2 + 8, h/2 + 5)]:
                draw.line((pos[0] + shadow_offset[0], pos[1] + shadow_offset[1], pos[0] + shadow_offset[0], pos[1] + shadow_offset[1] + 6), fill=SHADOW, width=3)
                draw.line((pos[0], pos[1], pos[0], pos[1] + 6), fill=C_RAIN, width=3)
        elif icon_name == "heavy_rain":
            for pos in [(w/2 - 10, h/2 + 5), (w/2 - 2, h/2 + 5), (w/2 + 6, h/2 + 5)]:
                draw.line((pos[0] + shadow_offset[0], pos[1] + shadow_offset[1], pos[0] + shadow_offset[0], pos[1] + shadow_offset[1] + 8), fill=SHADOW, width=4)
                draw.line((pos[0], pos[1], pos[0], pos[1] + 8), fill=C_RAIN, width=4)
        elif icon_name == "freezing_rain":
            draw.line((w/2 - 7, h/2 + 5, w/2 - 7, h/2 + 11), fill=C_RAIN, width=3)
            draw.ellipse((w/2 + 2, h/2 + 8, w/2 + 6, h/2 + 12), fill=C_SNOW)
        # --- Snow Variants ---
        elif icon_name == "light_snow":
            for pos in [(w/2 - 6, h/2 + 8), (w/2 + 4, h/2 + 8)]:
                draw.ellipse((pos[0] - 1, pos[1] - 1, pos[0] + 2, pos[1] + 2), fill=C_SNOW)
        elif icon_name == "snow": # Moderate
            for pos in [(w/2 - 8, h/2 + 8), (w/2, h/2 + 10), (w/2 + 6, h/2 + 8)]:
                draw.ellipse((pos[0] - 2, pos[1] - 2, pos[0] + 3, pos[1] + 3), fill=C_SNOW)
        elif icon_name == "heavy_snow":
            for pos in [(w/2 - 10, h/2 + 8), (w/2 - 2, h/2 + 10), (w/2 + 6, h/2 + 8), (w/2, h/2 + 5)]:
                draw.ellipse((pos[0] - 2, pos[1] - 2, pos[0] + 4, pos[1] + 4), fill=C_SNOW)
    # --- Storm Icons ---
    elif icon_name == "storm" or icon_name == "storm_hail":
        draw_cloud(shadow_offset, SHADOW)
        draw_cloud(color=(80, 80, 90)) # Dark storm cloud
        # Draw lightning bolt with shadow
        bolt_path = [(w/2 + 2, h/2), (w/2 - 4, h/2 + 8), (w/2, h/2 + 8), (w/2 - 6, h/2 + 18)]
        draw.line(bolt_path, fill=SHADOW, width=4)
        draw.line(bolt_path, fill=C_SUN, width=2)
        if icon_name == "storm_hail":
            # Add hail stones
            for pos in [(w/2 - 10, h/2 + 5), (w/2 + 8, h/2 + 12)]:
                draw.ellipse((pos[0] - 2, pos[1] - 2, pos[0] + 3, pos[1] + 3), fill=C_SNOW)
        
    return icon

def create_image(time_str: str, date_str: str, weather_info: dict | None) -> Image.Image:
    """
    Composes the final UI by layering text and icons over a gradient background.

    The layout is divided into two vertical panes:
    - Left Pane: Displays the current time, horizontally and vertically centered.
    - Right Pane: Displays the weather icon, temperature, and date, stacked
      vertically and centered within the pane.

    Args:
        time_str (str): The formatted time string (e.g., "14:30").
        date_str (str): The formatted date string (e.g., "Mon Jan 01").
        weather_info (dict | None): A dictionary with weather data or None.

    Returns:
        Image.Image: The final, composed UI as an RGB PIL Image.
    """
    image = _draw_vibrant_gradient_background()
    # Create a transparent overlay to draw text and icons on
    overlay = Image.new('RGBA', image.size, (255, 255, 255, 0))
    draw = ImageDraw.Draw(overlay)
    
    # --- Define Layout Panes ---
    separator_x = 100
    pane_left_width = separator_x
    pane_right_x_start = separator_x + 1
    pane_right_width = config.LCD_WIDTH - pane_right_x_start

    # --- LEFT PANE: Time ---
    # Center the time text within the left pane
    time_bbox = draw.textbbox((0, 0), time_str, font=config.FONT_TIME)
    time_width = time_bbox[2] - time_bbox[0]
    time_height = time_bbox[3] - time_bbox[1]
    time_x = (pane_left_width - time_width) // 2 # Center horizontally in the left pane
    time_y = (config.LCD_HEIGHT - time_height) // 2 - 10 # Minor vertical adjustment
    draw.text((time_x, time_y), time_str, font=config.FONT_TIME, fill=config.COLOR_TEXT_PRIMARY)
    
    # --- Visual Separator ---
    # A subtle line between the two panes
    draw.line([(separator_x, 10), (separator_x, config.LCD_HEIGHT - 10)], fill=config.COLOR_SEPARATOR, width=1)
              
    # --- RIGHT PANE: Weather and Date (Vertically Stacked) ---
    if weather_info:
        # 1. Weather Icon (Centered horizontally at the top of the pane)
        icon_size = (36, 36)
        icon = _create_weather_icon(weather_info['icon'], icon_size)
        icon_x = pane_right_x_start + (pane_right_width - icon_size[0]) // 2 # Center in right pane
        overlay.paste(icon, (icon_x, 5), icon)
        
        # 2. Temperature (Centered horizontally below the icon)
        temp_text = f"{weather_info['temperature']}°"
        temp_bbox = draw.textbbox((0, 0), temp_text, font=config.FONT_TEMP)
        temp_width = temp_bbox[2] - temp_bbox[0]
        temp_x = pane_right_x_start + (pane_right_width - temp_width) // 2 # Center in right pane
        draw.text((temp_x, 38), temp_text, font=config.FONT_TEMP, fill=config.COLOR_TEXT_PRIMARY)
        
    # 3. Date (Centered horizontally at the bottom of the pane)
    date_bbox = draw.textbbox((0, 0), date_str, font=config.FONT_DATE)
    date_width = date_bbox[2] - date_bbox[0]
    # Center the date within the right pane.
    date_x = pane_right_x_start // 2 + (pane_right_width - date_width) // 2
    draw.text((date_x, 60), date_str, font=config.FONT_DATE, fill=config.COLOR_TEXT_SECONDARY)
    
    # Composite the overlay onto the background and return the final image
    image.paste(overlay, (0, 0), overlay)
    return image.convert('RGB')