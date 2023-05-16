mod hex_util;
use hex_util::{Hex, Layout, Point};

extern crate image;

use byteorder::{LittleEndian, ReadBytesExt};
use png::{BitDepth, ColorType, Encoder};

use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::path::Path;

use std::collections::HashMap;

#[allow(dead_code)]
fn raw_image_to_normal(
    file_path: &Path,
    width: usize,
    height: usize,
) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut cursor = Cursor::new(buffer);
    let mut normal_image = vec![vec![0.0_f64; height]; width];

    println!("reading .raw file...");
    for y in 0..height {
        for x in 0..width {
            let pixel_value = cursor.read_u32::<LittleEndian>()?;
            let normalized_pixel_value = pixel_value as f64 / u32::MAX as f64;
            normal_image[x][y] = normalized_pixel_value;
        }
    }

    Ok(normal_image)
}

pub struct Field {
    pub normal: Vec<Vec<f32>>,
    pub size: usize,
}

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

impl Field {
    pub fn new(size: usize) -> Self {
        Self {
            normal: vec![vec![0.0; size]; size],
            size,
        }
    }
    
    pub fn from_raw_f32(    
        path: &Path,
        size: usize
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
    
        let mut cursor = Cursor::new(buffer);
        let mut normal = vec![vec![0.0_f32; size]; size];

        for y in 0..size {
            for x in 0..size {
                let pixel_value = cursor.read_f32::<LittleEndian>()?;
                normal[y][x] = pixel_value;
            }
        }
    
        Ok(
            Self {
                normal,
                size,
            }
        )
    }

    pub fn write_raw_f32(
        &self, 
        path: &Path
    ) -> Result<(), Box<dyn Error>> {
        let mut file = BufWriter::new(File::create(path)?);
    
        println!("Writing f32 values to raw file...");
        for y in 0..self.size {
            for x in 0..self.size {
                byteorder::WriteBytesExt::write_f32::<LittleEndian>(&mut file, self.normal[x][y])?;
            }   
        }
    
        Ok(())
    }

    pub fn write_png_u16(
        &self, 
        path: &Path
    ) -> Result<(), png::EncodingError> {
        let file = File::create(path)?;
        let write = BufWriter::new(file);
        let mut encoder = Encoder::new(write, self.size as u32, self.size as u32);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Sixteen);
        let mut writer = encoder.write_header()?;
        
        let buf_size = self.size * self.size * 2;
        let mut buf = vec![0_u8; buf_size];
    
        println!("loading into buffer...");
        for y in 0..self.size {
            for x in 0..self.size {
                let index = (y * self.size + x) * 2;
                let value = (self.normal[x][y] * u16::MAX as f32).round() as u16;
                buf[index] = (value >> 8) as u8;
                buf[index + 1] = (value & 0xFF) as u8;
            }
        }
    
        println!("writing image data...");
        writer.write_image_data(&buf)?;
        Ok(())
    }

    pub fn hex_kernel(
        &self, 
        layout: Layout
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("kernel start...");
        let mut bin: HashMap<Hex, Basket> = HashMap::new();
        let mut hex_field: Vec<Vec<f32>> = vec![vec![0.0; self.size]; self.size];
    
        const SQRT_3: f32 = 1.73205080756888;
        let size = layout.size.x as f32;
        //let area = (3_f32 * SQRT_3 * (size * size)) / 2_f32;

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
    
        println!("binning...");
        self.normal.iter().enumerate().for_each(|(x, v)| {
            v.iter().enumerate().for_each(|(y, n)| {
                let hex = Hex::from(Hex::from_point(
                    &layout,
                    &Point {
                        x: x as f64,
                        y: y as f64,
                    },
                ));
    
                match bin.get_mut(&hex) {
                    Some(basket) => {
                        basket.total += n;
                        basket.count += 1;
                    }
                    None => ()
                }
            });
        });
    
        println!("painting...");
        self.normal.iter().enumerate().for_each(|(x, v)| {
            v.iter().enumerate().for_each(|(y, _)| {
                let hex = Hex::from(Hex::from_point(
                    &layout,
                    &Point {
                        x: x as f64,
                        y: y as f64,
                    },
                ));
    
                match bin.get(&hex) {
                    Some(value) => {
                        hex_field[x][y] = value.total / value.count as f32;
                    }
                    None => (),
                }
            });
        });
    
        Ok(Self {
            normal: hex_field,
            size: self.size
        })
    }
}

fn main() {
    let input_path = Path::new("in/FractalTerraces.tif");

    let img_dim: usize = 2 << 12;
    let hex_dim: usize = img_dim / (2 << 5);
    println!("img: {} | hex: {}", img_dim, hex_dim);
    let field = match Field::from_raw_f32(input_path, img_dim) {
        Ok(image) => image,
        Err(e) => {
            println!("Error opening image file: {}", e);
            return;
        }
    };
    
    let size = Point {
        x: hex_dim as f64 / 2.0,
        y: hex_dim as f64 / 2.0,
    };

    let origin = Point { x: size.x, y: 0_f64};
    let layout = Layout::new(size, origin);

    let start_time = std::time::Instant::now();
    let hex_field = match field.hex_kernel(layout) {
        Ok(image) => image,
        Err(e) => {
            println!("hex kernel error: {}", e);
            return;
        }
    };
    println!("kernel time: {:?}", start_time.elapsed());

    if let Err(e) = hex_field.write_raw_f32(
        &Path::new("out").join(
        input_path
            .strip_prefix("in")
            .expect("hacky solution failed")
            .with_extension("r32")
        )
    ) {
        println!("Error saving image: {}", e);
    } else {
        println!("r32 saved successfully.");
    }

    if let Err(e) = hex_field.write_png_u16(
        &Path::new("out").join(
        input_path
            .strip_prefix("in")
            .expect("hacky solution failed")
            .with_extension("png")
        )
    ) {
        println!("Error saving image: {}", e);
    } else {
        println!("png saved successfully.");
    }
}
