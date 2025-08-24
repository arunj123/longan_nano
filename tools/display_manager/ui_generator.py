# ui_generator.py
"""
Handles all visual aspects. This version implements the final, polished UI
with a structured two-pane layout, a separator, and a vertically stacked
right column to guarantee readability.
"""
from PIL import Image, ImageDraw
from math import sin, cos, radians
import config

def _draw_vibrant_gradient_background():
    """Creates a high-contrast diagonal gradient background."""
    image = Image.new('RGB', (config.LCD_WIDTH, config.LCD_HEIGHT))
    r1, g1, b1 = config.COLOR_GRADIENT_START; r2, g2, b2 = config.COLOR_GRADIENT_END
    for y in range(config.LCD_HEIGHT):
        for x in range(config.LCD_WIDTH):
            p = (x + y) / (config.LCD_WIDTH + config.LCD_HEIGHT)
            r=int(r1+(r2-r1)*p); g=int(g1+(g2-g1)*p); b=int(b1+(b2-b1)*p)
            image.putpixel((x, y), (r, g, b))
    return image

def _create_weather_icon(icon_name, size):
    """Generates high-quality icons with a layered, 3D effect."""
    icon = Image.new('RGBA', size, (0,0,0,0)); draw = ImageDraw.Draw(icon); w,h=size
    C_SUN,C_CLOUD=(255,200,0),(220,220,220);C_RAIN,C_SNOW=(100,140,255),(240,240,240);C_CLOUD_DARK,SHADOW=(160,160,170),(0,0,0,50)
    def draw_sun(o=(0,0),c=C_SUN):
        cp=(w/2+o[0],h/2+o[1]);r=8;draw.ellipse((cp[0]-r,cp[1]-r,cp[0]+r,cp[1]+r),fill=c)
        for i in range(8):a=i*45;x1=cp[0]+cos(radians(a))*r;y1=cp[1]+sin(radians(a))*r;x2=cp[0]+cos(radians(a))*(r+3);y2=cp[1]+sin(radians(a))*(r+3);draw.line([(x1,y1),(x2,y2)],fill=c,width=2)
    def draw_cloud(o=(0,0),c=C_CLOUD):
        x,y=o[0]+w/2,o[1]+h/2;draw.ellipse((x-18,y-5,x+8,y+15),fill=c);draw.ellipse((x-8,y-12,x+18,y+12),fill=c)
    sh=(2,2)
    if icon_name=="sun":draw_sun(sh,SHADOW);draw_sun()
    elif icon_name=="cloud":draw_cloud(sh,SHADOW);draw_cloud()
    elif icon_name=="sun_cloud":draw_cloud(sh,SHADOW);draw_sun(o=(-4,-4));draw_cloud(o=(0,4))
    elif icon_name in ["rain","snow"]:
        draw_cloud(sh,SHADOW);draw_cloud(c=C_CLOUD_DARK)
        if icon_name=="rain":
            for pos in [(w/2-6,h/2+4),(w/2+4,h/2+4)]:draw.line((pos[0]+sh[0],pos[1]+sh[1],pos[0]+sh[0],pos[1]+sh[1]+6),fill=SHADOW,width=3);draw.line((pos[0],pos[1],pos[0],pos[1]+6),fill=C_RAIN,width=3)
        else:
            for pos in [(w/2-6,h/2+8),(w/2+4,h/2+8)]:draw.ellipse((pos[0]-1,pos[1]-1,pos[0]+3,pos[1]+3),fill=C_SNOW)
    elif icon_name=="storm":
        draw_cloud(sh,SHADOW);draw_cloud(c=(80,80,90));b=[(w/2+2,h/2),(w/2-4,h/2+8),(w/2,h/2+8),(w/2-6,h/2+18)];draw.line(b,fill=SHADOW,width=4);draw.line(b,fill=C_SUN,width=2)
    return icon

# --- Public Function with FINAL ROBUST LAYOUT ---
def create_image(time_str, date_str, weather_info):
    """Composes the final UI with a structured, non-overlapping layout."""
    image = _draw_vibrant_gradient_background()
    overlay = Image.new('RGBA', image.size, (255, 255, 255, 0))
    draw = ImageDraw.Draw(overlay)
    
    # --- Define Layout Panes ---
    separator_x = 90  # A more balanced position
    pane_left_width = separator_x
    pane_right_x_start = separator_x + 1
    pane_right_width = config.LCD_WIDTH - pane_right_x_start

    # --- LEFT PANE: Time ---
    time_bbox = draw.textbbox((0, 0), time_str, font=config.FONT_TIME)
    time_width = time_bbox[2] - time_bbox[0]
    time_height = time_bbox[3] - time_bbox[1]
    time_x = (pane_left_width - time_width) // 2
    time_y = (config.LCD_HEIGHT - time_height) // 2  - 10
    draw.text((time_x, time_y), time_str, font=config.FONT_TIME, fill=config.COLOR_TEXT_PRIMARY)
    
    # --- Visual Separator ---
    draw.line([(separator_x, 10), (separator_x, config.LCD_HEIGHT - 10)], fill=config.COLOR_SEPARATOR, width=1)
              
    # --- RIGHT PANE: Weather and Date (VERTICALLY STACKED) ---
    if weather_info:
        # 1. Weather Icon (Centered at the top of the pane)
        icon_size = (36, 36)
        icon = _create_weather_icon(weather_info['icon'], icon_size)
        icon_x = pane_right_x_start + (pane_right_width - icon_size[0]) // 2
        overlay.paste(icon, (icon_x, 5), icon)
        
        # 2. Temperature (Centered below icon)
        temp_text = f"{weather_info['temperature']}°"
        temp_bbox = draw.textbbox((0, 0), temp_text, font=config.FONT_TEMP)
        temp_width = temp_bbox[2] - temp_bbox[0]
        temp_x = pane_right_x_start + (pane_right_width - temp_width) // 2
        draw.text((temp_x, 38), temp_text, font=config.FONT_TEMP, fill=config.COLOR_TEXT_PRIMARY)
        
    # 3. Date (Centered at the bottom of the pane)
    date_bbox = draw.textbbox((0, 0), date_str, font=config.FONT_DATE)
    date_width = date_bbox[2] - date_bbox[0]
    date_x = pane_right_x_start // 2 + (pane_right_width - date_width) // 2
    draw.text((date_x, 60), date_str, font=config.FONT_DATE, fill=config.COLOR_TEXT_SECONDARY)
    
    image.paste(overlay, (0, 0), overlay)
    return image.convert('RGB')