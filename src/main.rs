mod hex_util;
use hex_util::{Hex, Layout, Point};

use byteorder::{LittleEndian, ReadBytesExt};
use png::{BitDepth, ColorType, Encoder};

use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::path::{Path, PathBuf};

use std::collections::HashMap;

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
            let pixel_value = cursor.read_u16::<LittleEndian>()?;
            let normalized_pixel_value = pixel_value as f64 / u16::MAX as f64;
            normal_image[x][y] = normalized_pixel_value;
        }
    }

    Ok(normal_image)
}

fn write_normal_to_png(
    path: &PathBuf,
    normal: Vec<Vec<f64>>,
    width: usize,
    height: usize,
) -> Result<(), png::EncodingError> {
    let file = File::create(path)?;
    let write = BufWriter::new(file);
    let mut encoder = Encoder::new(write, width as u32, height as u32);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Sixteen);
    let mut writer = encoder.write_header()?;

    let buf_size = (width) * (height) * 2;
    let mut buf = vec![0_u8; buf_size];

    println!("loading into buffer...");
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 2;
            let value = (normal[x][y] * u16::MAX as f64).round() as u16;

            buf[index] = (value >> 8) as u8;
            buf[index + 1] = (value & 0xFF) as u8;
        }
    }

    println!("writing image data...");
    writer.write_image_data(&buf)?;
    Ok(())
}

fn hex_kernel(field: Vec<Vec<f64>>, layout: Layout) -> Vec<Vec<f64>> {
    println!("kernel start...");
    let mut bin: HashMap<Hex, f64> = HashMap::new();
    let mut hex_field: Vec<Vec<f64>> = vec![vec![0.0; field[0].len()]; field.len()];

    const SQRT_3: f64 = 1.73205080756888;
    let size = &layout.size.x;
    let area = (3_f64 * SQRT_3 * (size * size)) / 2_f64;

    let height = (field.len() as f64 / (size * SQRT_3)) as i32;
    let width = (field.len() as f64 / (size * 3_f64 / 2_f64)) as i32;

    let left = 0;
    let right = width;
    let top = 1;
    let bottom = height + 1;

    for q in left..right {
        let q_offset = (q + 1) >> 1;
        for r in (top - q_offset)..(bottom - q_offset) {
            bin.insert(Hex::new(q, r), 0_f64);
        }
    }

    println!("binning...");
    field.iter().enumerate().for_each(|(x, v)| {
        v.iter().enumerate().for_each(|(y, n)| {
            let hex = Hex::from(Hex::from_point(
                &layout,
                &Point {
                    x: x as f64,
                    y: y as f64,
                },
            ));
            match bin.get_mut(&hex) {
                Some(value) => {
                    *value += n;
                }
                None => ()
            }
        });
    });

    println!("painting...");
    field.iter().enumerate().for_each(|(x, v)| {
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
                    hex_field[x][y] = value / area;
                }
                None => (),
            }
        });
    });

    hex_field
}

fn main() {
    let input_path = Path::new("in/FractalTerraces.r16");

    let img_dim: usize = 4096;
    let hex_dim: usize = 140;

    let field = match raw_image_to_normal(input_path, img_dim, img_dim) {
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
    let tes_field = hex_kernel(field, layout);
    println!("kernel time: {:?}", start_time.elapsed());

    let output_path = Path::new("out").join(
        input_path
            .strip_prefix("in")
            .expect("your hacky solution failed lol")
            .with_extension("png"),
    );

    let _ = match write_normal_to_png(&output_path, tes_field, img_dim, img_dim) {
        Ok(()) => {
            println!("image saved at: {:?}", output_path);
        }
        Err(e) => {
            println!("Error writing image file: {}", e);
        }
    };
}
