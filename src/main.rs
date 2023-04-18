use image::{GrayImage, Luma};
use std::path::Path;
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

fn main() {
    let input_path = Path::new("/Users/thales/Documents/hex_rasterizer/Combine.raw");
    let input_image = match read_raw_image(input_path, 4096, 4096) {
        Ok(image) => image,
        Err(e) => {
            println!("Error opening image file: {}", e);
            return;
        }
    };

    // Apply the hexagonal kernel to the image
    let mask_size = 40;
    let output_image = apply_hexagonal_kernel(&input_image, mask_size);

    // Save the output image
    let output_path = Path::new(input_path).with_extension("hex_rastered.png");
    output_image
        .save(&output_path)
        .expect("Failed to save the output image");

    println!("Hexagonal rastered image saved at: {:?}", output_path);
}

fn apply_hexagonal_kernel(image: &GrayImage, mask_size: usize) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut output_image = image.clone();

    let hex_step = mask_size as f32;
    let vertical_step = hex_step * (3f32.sqrt() / 2f32);
    let mut row_offset = false;

    for y in (0..height as usize).step_by(vertical_step as usize) {
        let y_offset = if row_offset { hex_step / 2f32 } else { 0f32 };

        for x in (0..width as usize).step_by(mask_size) {
            let x_offset = (x as f32 + y_offset) as usize;
            let mut sum = 0;
            let mut count = 0;

            for dy in 0..vertical_step as usize {
                for dx in 0..mask_size {
                    let px = x_offset + dx;
                    let py = y + dy;

                    if px < width as usize && py < height as usize {
                        let pixel_value = image.get_pixel(px as u32, py as u32)[0];
                        sum += pixel_value as u32;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                let average_value = (sum / count) as u8;

                for dy in 0..vertical_step as usize {
                    for dx in 0..mask_size {
                        let px = x_offset + dx;
                        let py = y + dy;

                        if px < width as usize && py < height as usize {
                            output_image.put_pixel(px as u32, py as u32, Luma([average_value]));
                        }
                    }
                }
            }
        }
        row_offset = !row_offset;
    }

    output_image
}
