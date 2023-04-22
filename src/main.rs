mod hex_util;
//use hex_util::{Hex, Layout, Point};
use hex_util::{Hex, Layout, Point};

use byteorder::{LittleEndian, ReadBytesExt};
use png::{BitDepth, ColorType, Encoder};

use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::path::{Path, PathBuf};

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

fn hex_tessellation_kernal(field: Vec<Vec<f64>>, layout: Layout, hex_dim: usize) -> Vec<Vec<f64>> {
    //println!("layout: {:?} | hex_dim: {}", layout, hex_dim);
    println!("starting");
    let mut new_field = field.clone(); // Create a mutable copy of field
    for (x, v) in field.iter().enumerate() {
        for (y, n) in v.iter().enumerate() {
            let hex =  Hex::from_point(&layout, &Point { x: x as f64, y: y as f64});
            if hex.q > 10.0 && hex.r > 10.0 {
                new_field[x][y] = 1.0; // Modify new_field instead of n
            }
        }
    }
    println!("stopping");
    new_field // Return the modified field
}


fn main() {
    let input_path = Path::new("in/Thermal.r16");

    let img_dim: usize = 4096;
    let hex_dim: usize = 200;

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

    let origin = Point { x: 0.0, y: 0.0 };

    let layout = Layout::new(size, origin);
    let tes_field = hex_tessellation_kernal(field, layout, hex_dim);

    let output_path = Path::new("out").join(
        input_path
            .strip_prefix("in")
            .expect("your hacky solution failed lol")
            .with_extension("png"),
    );

    let _ = match write_normal_to_png(&output_path, tes_field, img_dim, img_dim) {
        Ok(()) => {
            println!("Hexagonal rastered image saved at: {:?}", output_path);
        }
        Err(e) => {
            println!("Error writing image file: {}", e);
        }
    };
}
