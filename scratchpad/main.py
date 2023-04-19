import math
import multiprocessing
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from queue import Queue

import cv2
import numpy as np
import pygame
import tqdm
from PIL import Image
from hex_util import Hex, Layout, Point

def as_surf(np_arr, n):
    bg_surf = pygame.surfarray.make_surface(np_arr)
    bg_surf = pygame.transform.scale(bg_surf, (n, n))
    return bg_surf

def height(field, i, low=0, high=255, s_r=10000, size=50):
    _range = size / 2
    top = int(65535 * ((i - _range) / s_r))
    bottom = int(65535 * ((i + _range) / s_r))

    tmp = np.copy(field)
    for row in tmp:
        row[row < top] = low
        row[row > bottom] = low
        row[row != 0] = high
    return tmp

def steepness(field):
    shifted = np.zeros_like(field)
    shifted[:, 1:] = field[:, :-1]
    rolled = np.roll(field, -1, axis=0)
    dx = shifted - field
    dy = rolled - field

    result = np.sqrt(dx * dx + dy * dy)
    return result.astype(np.uint8)

def display(img, win_name):
    cv2.namedWindow(win_name, cv2.WINDOW_NORMAL)
    cv2.imshow(win_name, img)
    cv2.resizeWindow(win_name, 1024, 1024)
    cv2.waitKey(0)

def process_hex(i, r_offset, layout, heightfield, image, queue):
    j_values = np.arange(0 - r_offset, right - r_offset + 1)

    for j in tqdm.tqdm(j_values):
        tmp = np.ones_like(image)
        corners = Layout.polygon_corners(layout, Hex(j, i, (-1 * j - i)))
        corners = np.asarray([[i.x, i.y] for i in corners], dtype=np.int32)

        cv2.fillPoly(tmp, pts=[corners], color=0)

        mask = np.ma.masked_array(heightfield, mask=tmp)
        if not mask[~mask.mask].data.size == 0:
            mean = cp.asnumpy(cp.sum(cp.array(mask[~mask.mask]))) // hex_area
            queue.put((corners, mean))

def tes_honeycomb(field, layout, filename, n_workers=n_workers):
    image = np.zeros_like(field, dtype=cp.uint16)
    queue = Queue()

    with ThreadPoolExecutor(max_workers=n_workers) as executor:
        i_values = np.arange(0 , bottom + 1)
        for i in i_values:
            r_offset = i//2
            executor.submit(process_hex, i, r_offset, layout, field, image, queue)

    while not queue.empty():
        corners, mean = queue.get()
        cv2.fillPoly(image, pts=[corners], color=[mean])

    image = Image.fromarray(image, mode='I;16')
    image.save(filename)

def main(args=None):
    input_file = ".\\raw_files\\FractalTerraces.r16"
    output_filename = "test_r16_fractal"
    field = np.fromfile(input_file, dtype=np.uint16)
    n = int(math.sqrt(len(field)))
    fieldSize = (n, n)
    field = field.reshape(fieldSize)

    cv2.imwrite(f"{output_filename}.png", np.asarray(field))
