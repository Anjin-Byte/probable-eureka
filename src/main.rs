use image::{GrayImage, Luma};
use std::{path::Path};
use std::fs::File;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

fn read_raw_image(file_path: &Path, width: u32, height: u32) -> Result<GrayImage, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut cursor = Cursor::new(buffer);
    let mut gray_image = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel_value = cursor.read_u16::<LittleEndian>()?;
            let normalized_pixel_value = (pixel_value as f64 / u16::MAX as f64 * u8::MAX as f64).round() as u8;
            gray_image.put_pixel(x, y, Luma([normalized_pixel_value]));
        }
    }

    Ok(gray_image)
}

fn hex_tessellation_kernal(field: GrayImage, hex_dim: u8) -> GrayImage {
    println!("hex_tessellation_kernal({:?}, {})", field, hex_dim);
    field
}

fn main() {
    let input_path = Path::new("/Users/thales/Documents/probable-eureka/Combine.raw");
    let output_path = input_path.with_extension("png");

    let img_dim: u32 = 4096;
    let hex_dim: u8 = 15;
    
    let field = match read_raw_image(input_path, img_dim, img_dim) {
        Ok(image) => image,
        Err(e) => {
            println!("Error opening image file: {}", e);
            return;
        }
    };

    let output_image = hex_tessellation_kernal(field, hex_dim);
    output_image
        .save(&output_path)
        .expect("Failed to save the output image");

    println!("Hexagonal rastered image saved at: {:?}", output_path);
}
