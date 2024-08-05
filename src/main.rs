pub mod hex;
pub mod field;

extern crate image;

use std::path::Path;

use field::field::Field;
use hex::{layout::Layout, point::Point};


//#[allow(dead_code)]
/* fn raw_image_to_normal(
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
} */

fn write_wrapper(f: &Field, p: &Path) {
/*     if let Err(e) = f.write_raw_f32(
        &Path::new("out").join(p.with_extension("r32"))
    ) {
        println!("Error saving r32 image: {}", e);
    } else {
        println!("r32 saved successfully...");
    }

    if let Err(e) = f.write_png_u16(
        &Path::new("out").join(p.with_extension("png"))
    ) {
        println!("Error saving png image: {}", e);
    } else {
        println!("png saved successfully...");
    } */
}

fn stem_builder<'a>(p: &Path, postfix: &str) -> String {
    let stem = p.file_stem()
            .expect("file has no stem")
            .to_string_lossy();
    let result = format!("{}_{}", stem, postfix);
    result
}

fn main() {
    //let start_time = std::time::Instant::now();
    let input_path = Path::new("repo/Combine.r32");

    let img_dim: usize = 2 << 12;
    let hex_dim: usize = img_dim / (2 << 2);

    let field = Field::from_r32(input_path).expect("failed to read in r32");

    let size = Point {
        x: hex_dim as f64 / 2.0,
        y: hex_dim as f64 / 2.0,
    };

    let origin = Point { x: size.x, y: 0_f64};
    let layout = Layout::new(size, origin);
    
    let hex_field = field.steepness().expect("hex kernel failed");
    //dbg!(&hex_field.flattened_field);
    let binding = stem_builder(input_path, "hex");
    let output_path = Path::new(&binding);

    if let Err(e) = hex_field.write_png_u16(
        &Path::new("out").join(output_path.with_extension("png"))
    ) {
        println!("Error saving r32 image: {}", e);
    } else {
        println!("r32 saved successfully...");
    }

/*     println!("img: {} | hex: {}", img_dim, hex_dim);

    let field = match Field::from_raw_f32(input_path, img_dim) {
        Ok(image) => image,
        Err(e) => {
            println!("Error opening image file: {}", e);
            return;
        }
    };
    println!("read disc time: {:?}", start_time.elapsed());
    
    let size = Point {
        x: hex_dim as f64 / 2.0,
        y: hex_dim as f64 / 2.0,
    };

    let origin = Point { x: size.x, y: 0_f64};
    let layout = Layout::new(size, origin);

    



    let hex_field = match field.hex_kernel(layout) {
        Ok(image) => image,
        Err(e) => {
            println!("hex kernel error: {}", e);
            return;
        }
    };
    println!("hex kernel time: {:?}", start_time.elapsed());
    let output_name = stem_builder(input_path, "hex");
    write_wrapper(&hex_field, &Path::new(&output_name));
    println!("write time: {:?}", start_time.elapsed());





    let steepness_field = match field.steepness() {
        Ok(image) => image,
        Err(e) => {
            println!("steepness kernel error: {}", e);
            return;
        }
    };
    println!("steepness kernel time: {:?}", start_time.elapsed());
    let output_name = stem_builder(input_path, "steepness");
    write_wrapper(&steepness_field, &Path::new(&output_name));
    println!("write time: {:?}", start_time.elapsed()); */
}
