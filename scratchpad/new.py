import multiprocessing
import sys
from concurrent.futures import ThreadPoolExecutor

import cv2
import numpy as np
from PIL import Image
from hex_util import Hex, Layout, Point
from tqdm import tqdm


def heightmap_value(heightmap, x, y):
    h, w = heightmap.shape
    if 0 <= x < w and 0 <= y < h:
        return heightmap[y, x]
    return 0

def process_hexagons(field, layout, hex_d, img_d):
    height, width = field.shape
    processed_image = np.zeros((height, width), dtype=np.uint16)

    for y in tqdm(range(height), desc="Processing", unit="row"):
        for x in range(width):
            hex_center = Layout.pixel_to_hex(layout, Point(x, y))
            hex_rounded = Hex.hex_round(hex_center)

            corners = Layout.polygon_corners(layout, hex_rounded)
            center = Layout.hex_to_pixel(layout, hex_rounded)

            max_height = 0
            for corner in corners:
                corner_x, corner_y = int(corner.x), int(corner.y)
                corner_height = heightmap_value(field, corner_x, corner_y)
                max_height = max(max_height, corner_height)

            processed_image[y, x] = max_height

    return processed_image

def tessellate_heightmap(heightmap, hex_d, img_d, output_filename):
    layout = Layout("pointy", Point(hex_d, hex_d), Point(0, 0))

    with ThreadPoolExecutor(max_workers=multiprocessing.cpu_count()) as executor:
        image_future = executor.submit(process_hexagons, heightmap, layout, hex_d, img_d)

        image = image_future.result()
        Image.fromarray(image, mode='I;16').save(f"{output_filename}.png")

def main(args=None):
    input_file = "/Users/thales/Desktop/scratchpad/raw_files/Thermal.raw"
    output_filename = "thermal"
    hex_d = 15
    img_d = 4096

    field = np.fromfile(input_file, dtype=np.uint16)
    field = field.reshape((img_d, img_d))

    cv2.imwrite(f"{output_filename}.png", field)

    layout = Layout("pointy", Point(hex_d / 2, hex_d / 2), Point(0, 0))
    
    with ThreadPoolExecutor(max_workers=multiprocessing.cpu_count()) as executor:
        image_futures = [executor.submit(process_hexagons, field, layout, hex_d, img_d)]
    
    for i, future in enumerate(image_futures):
        image = future.result()
        Image.fromarray(image, mode='I;16').save(f"{output_filename}_{i}.png")

if __name__ == "__main__":
    rc = 1
    try:
        main()
        rc = 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
    sys.exit(rc)
