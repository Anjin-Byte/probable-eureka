use std::u16;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, ImageError, Luma};
use hashbrown::HashMap;

use crate::hex::hex::Hex;
use crate::hex::layout::Layout;
use crate::hex::point::Point;

const IMG_SIZE: usize = 2 << 12;
const ARRAY_LEN: usize = IMG_SIZE.pow(2);

#[derive(Debug)]
pub struct Basket {
    pub total: f32,
    pub count: u32,
}

impl Basket {
    pub fn new(total: f32, count: u32) -> Self {
        Self {
            total,
            count,
        }
    }
}

pub struct Field {
    pub flattened_field: Box<[f32]>,
    pub size: usize,
}

impl Field {
    pub fn new() -> Self {
        Self {
            flattened_field: vec![0_f32; ARRAY_LEN].into_boxed_slice(),
            size: IMG_SIZE,
        }
    }

    pub fn from_r32(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0_u8; ARRAY_LEN * 4];
        file.read_exact(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);
        let mut normalized_f32 = vec![0_f32; ARRAY_LEN].into_boxed_slice();

        normalized_f32.iter_mut().for_each(|val| {
            let pixel_value = cursor
                .read_f32::<LittleEndian>()
                .expect("Failed to read float value");
            *val = pixel_value;
        });

        Ok(Self {
            flattened_field: normalized_f32,
            size: IMG_SIZE,
        })
    }

    pub fn write_png_u16(&self, path: &Path) -> Result<(), ImageError> {
        let img = ImageBuffer::from_fn(IMG_SIZE as u32, IMG_SIZE as u32, |x, y| {
            let value = self.flattened_field[(y as usize * IMG_SIZE) + x as usize];
            let normalized_value = (value * u16::MAX as f32) as u16;
            //dbg!(normalized_value);
            Luma::<u16>([normalized_value])
        });

        img.save(path)
    }

    pub fn hex_kernel(&self, layout: Layout) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bin: HashMap<Hex, Basket> = HashMap::new();
        let mut hex_field: Box<[f32; ARRAY_LEN]> = Box::new([0.0; ARRAY_LEN]);

        const SQRT_3: f32 = 1.73205080756888;
        let size = layout.size.x as f32;

        let height = self.size as f32 / (size * SQRT_3);
        let width = self.size as f32 / (size * 3_f32 / 2_f32);

        let left = -1;
        let right = width.ceil() as i32 + 1;
        let top = 0;
        let bottom = height.ceil() as i32 + 1;

        for q in left..right {
            let q_offset = (q + 1) >> 1;
            for r in (top - q_offset)..(bottom - q_offset) {
                bin.insert(Hex::new(q, r), Basket::new(0_f32, 0));
            }
        }

        self.flattened_field.iter().enumerate().for_each(|(i, value)| {
            let x = i % IMG_SIZE;
            let y = i / IMG_SIZE;

            let hex = Hex::from(Hex::from_point(
                &layout,
                &Point {
                    x: x as f64,
                    y: y as f64,
                },
            ));

            match bin.get_mut(&hex) {
                Some(basket) => {
                    basket.total += value;
                    basket.count += 1;
                }
                None => ()
            }
        });

        for i in 0..hex_field.len() {
            let x = i % IMG_SIZE;
            let y = i / IMG_SIZE;

            let hex = Hex::from(Hex::from_point(
                &layout,
                &Point {
                    x: x as f64,
                    y: y as f64,
                },
            ));

            match bin.get(&hex) {
                Some(value) => {
                    hex_field[i] = value.total / value.count as f32;
                }
                None => (),
            }
        }

        Ok(Self {
            flattened_field: hex_field,
            size: self.size
        })
    }

    pub fn steepness(&self) -> Result<Self, Box<dyn std::error::Error>> {
        let s = IMG_SIZE;

        let mut shifted_right = vec![0.0; ARRAY_LEN];
        let mut shifted_down = vec![0.0; ARRAY_LEN];

        for i in 0..ARRAY_LEN {
            let row = i / IMG_SIZE;
            let col = i % IMG_SIZE;

            if col + 1 < IMG_SIZE {
                let shifted_i = row * IMG_SIZE + (col + 1);
                shifted_right[i] = self.flattened_field[shifted_i];
            } else {
                shifted_right[i] = self.flattened_field[i];
            }
        }

        for j in 0..ARRAY_LEN {
            let row = j / IMG_SIZE;
            let col = j % IMG_SIZE;

            if row + 1 < IMG_SIZE {
                let shifted_j = (row + 1) * IMG_SIZE + col;
                shifted_down[j] = self.flattened_field[shifted_j];
            } else {
                shifted_down[j] = self.flattened_field[j];
            }
        }
        let mut result = vec![0.0; ARRAY_LEN];
        
        for i in 0..ARRAY_LEN {
            let dx = shifted_right[i] - self.flattened_field[i];
            let dy = shifted_down[i] - self.flattened_field[i];
            result[i] = (dx * dx + dy * dy).sqrt();
        }

        let min = result.clone().into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = result.clone().into_iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        for v in result.iter_mut() {
            *v = (*v - min) / (max - min); 
        }

        Ok(Self {
            flattened_field: result.into_boxed_slice(),
            size: s,
        })
    }
}