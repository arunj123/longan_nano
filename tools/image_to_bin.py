import sys
from PIL import Image

# --- Configuration ---
# The output path and dimensions remain fixed as they are tied to the device.
OUTPUT_PATH = "image_160x80.bin"
LCD_WIDTH = 160
LCD_HEIGHT = 80

def process_image(input_path):
    """
    Opens an image, resizes it to 160x80, converts it to the RGB565 binary format,
    and saves it to the specified output path.
    """
    try:
        print(f"Opening '{input_path}'...")
        img = Image.open(input_path)
        
        # Resize the image to match the LCD dimensions and convert it to RGB color space.
        # ANTIALIAS filter provides a high-quality downscaling result.
        img = img.resize((LCD_WIDTH, LCD_HEIGHT), Image.Resampling.LANCZOS).convert("RGB")
        print("Image resized to {}x{} and converted to RGB.".format(LCD_WIDTH, LCD_HEIGHT))

        # Open the binary file for writing.
        with open(OUTPUT_PATH, "wb") as f:
            # Iterate through each pixel in the resized image.
            for r, g, b in img.getdata():
                # Convert the 24-bit RGB888 pixel to a 16-bit RGB565 value.
                # R: Top 5 bits -> [15:11]
                # G: Top 6 bits -> [10:5]
                # B: Top 5 bits -> [4:0]
                pixel_value = ((r & 0b11111000) << 8) | ((g & 0b11111100) << 3) | (b >> 3)
                
                # Write the 16-bit value to the file as two bytes in little-endian format.
                f.write(pixel_value.to_bytes(2, 'little'))
                
        output_size = LCD_WIDTH * LCD_HEIGHT * 2
        print(f"\nSuccessfully created '{OUTPUT_PATH}' ({output_size} bytes).")
        print("You can now transfer this file to the Longan Nano using the test script.")

    except FileNotFoundError:
        print(f"\nError: Input file not found at '{input_path}'")
        sys.exit(1) # Exit with an error code
    except Exception as e:
        print(f"\nAn unexpected error occurred: {e}")
        sys.exit(1)

def main():
    """
    Main function to handle command-line arguments.
    """
    # sys.argv is a list containing the script name and its command-line arguments.
    # We check if at least one argument (the image path) was provided.
    if len(sys.argv) < 2:
        print("-----------------------------------------------------------")
        print(" Image to RGB565 Binary Converter for Longan Nano")
        print("-----------------------------------------------------------")
        print("Converts any standard image file into the raw binary format")
        print("required for the Longan Nano LCD screen.")
        print("\nUsage:")
        # sys.argv[0] is the name of the script itself (e.g., prepare_image.py)
        print(f"  python {sys.argv[0]} <path_to_your_image.jpg>")
        print("\nExample:")
        print(f"  python {sys.argv[0]} my_photo.png")
        sys.exit(1) # Exit because the required argument is missing

    # The first command-line argument is the input image path.
    input_image_path = sys.argv[1]
    process_image(input_image_path)

if __name__ == "__main__":
    main()