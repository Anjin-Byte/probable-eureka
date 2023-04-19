import multiprocessing
import sys
import numpy as np
from concurrent.futures import ThreadPoolExecutor
from PIL import Image
from hex_util import Hex, Layout, Point

def heightmap_value(heightmap, x, y):
    h, w = heightmap.shape
    if 0 <= x < w and 0 <= y < h:
        return heightmap[y, x]
    return 0

def precompute_hexagon_heights(field, layout, hex_rounded):
    height, width = field.shape
    hex_max_heights = {}

    for h in np.unique(hex_rounded):
        corners = Layout.polygon_corners(layout, h)
        corner_coords = [(int(corner.x), int(corner.y)) for corner in corners]

        valid_corner_coords = [(x, y) for x, y in corner_coords if 0 <= x < width and 0 <= y < height]
        max_height = np.max([heightmap_value(field, x, y) for x, y in valid_corner_coords])

        hex_max_heights[h] = max_height

    return hex_max_heights

def process_hexagons(field, layout, hex_d, img_d):
    height, width = field.shape
    processed_image = np.zeros((height, width), dtype=np.uint16)

    x_coords, y_coords = np.meshgrid(np.arange(width), np.arange(height))
    hex_centers = np.column_stack((x_coords.ravel(), y_coords.ravel()))
    hex_centers = [Layout.pixel_to_hex(layout, Point(coord[0], coord[1])) for coord in hex_centers]
    hex_rounded = [Hex.hex_round(center) for center in hex_centers]

    hex_max_heights = precompute_hexagon_heights(field, layout, hex_rounded)

    for h in np.unique(hex_rounded):
        mask = np.array([Hex.equal(hr, h) for hr in hex_rounded]).reshape(height, width)
        processed_image[mask] = hex_max_heights[h]

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
    hex_d = 150
    img_d = 4096

    field = np.fromfile(input_file, dtype=np.uint16)
    field = field.reshape((img_d, img_d))

    tessellate_heightmap(field, hex_d, img_d, output_filename)

if __name__ == "__main__":
    rc = 1
    try:
        main()
        rc = 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
    sys.exit(rc)
