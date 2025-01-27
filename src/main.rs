pub mod field;
pub mod hex;
pub mod frontend;

use field::field::Field;
use hex::{layout::Layout, point::Point};
//use frontend::app::App;

extern crate image;

use std::path::Path;

//#[allow(dead_code)]
/*
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
*/

fn stem_builder<'a>(p: &Path, postfix: &str) -> String {
    let stem = p.file_stem().expect("file has no stem").to_string_lossy();
    let result = format!("{}_{}", stem, postfix);
    result
}

fn main() {
    //let start_time = std::time::Instant::now();
    let input_path = Path::new("repo/arid_demo_8k/FractalTerraces.r32");

    let img_dim: usize = 8192;
    let hex_dim: usize = img_dim / 288;
    let field = Field::from_r32(input_path).expect("failed to read in r32");

    /*
    let _ = eframe::run_native(
        "Image Viewer",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            let mut app = App::new(img_dim);
            app.update_image(&cc.egui_ctx, &field);
            Ok(Box::new(app))
        }),
    );
    */

    let size = Point {
        x: hex_dim as f64 / 2.0,
        y: hex_dim as f64 / 2.0,
    };

    let origin = Point {
        x: size.x,
        y: 0_f64,
    };

    let layout = Layout::new(size, origin);

    let binding = stem_builder(input_path, "original");
    let output_path = Path::new(&binding);

    if let Err(e) =
        field.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving original image: {}", e);
    } else {
        println!(" saved original successfully...");
    }

        
    let hex_field = field.hex_aggregate(layout).expect("hex kernel failed");
    let binding = stem_builder(input_path, "hex");
    let output_path = Path::new(&binding);
    
    if let Err(e) =
        hex_field.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving  image: {}", e);
    } else {
        println!(" saved hex successfully...");
    }


    let steepness_field = field.steepness().expect("steepness kernel failed");
    let binding = stem_builder(input_path, "steepness");
    let output_path = Path::new(&binding);
    
    if let Err(e) =
        steepness_field.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving  image: {}", e);
    } else {
        println!(" saved steepness successfully...");
    }


    let prewitt = field.prewitt().expect("steepness kernel failed");
    let binding = stem_builder(input_path, "prewitt");
    let output_path = Path::new(&binding);

    if let Err(e) =
        prewitt.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving steepness image: {}", e);
    } else {
        println!(" saved prewitt successfully...");
    }


    let sobel = hex_field.sobel().expect("sobel kernel failed");
    let binding = stem_builder(input_path, "sobel");
    let output_path = Path::new(&binding);

    if let Err(e) =
        sobel.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved sobel successfully...");
    }

    let sobel = field.sobel().expect("sobel kernel failed");

    let binding = stem_builder(input_path, "sobel");
    let output_path = Path::new(&binding);
    if let Err(e) =
        sobel.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved sobel successfully...");
    }

    let structural = field.structural_lines().expect("sobel kernel failed");

    let binding = stem_builder(input_path, "crests");
    let output_path = Path::new(&binding);
    if let Err(e) =
        structural.0.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved crests successfully...");
    }

    let binding = stem_builder(input_path, "thalwegs");
    let output_path = Path::new(&binding);
    if let Err(e) =
        structural.1.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved thalwegs successfully...");
    }

    let binding = stem_builder(input_path, "convex_lines");
    let output_path = Path::new(&binding);
    if let Err(e) =
        structural.2.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved convex_lines successfully...");
    }

    let binding = stem_builder(input_path, "concave_lines");
    let output_path = Path::new(&binding);
    if let Err(e) =
        structural.3.write_png_u16(&Path::new("out").join(output_path.with_extension("png")))
    {
        println!("Error saving prewitt image: {}", e);
    } else {
        println!(" saved concave_lines successfully...");
    }
    /*
    println!("img: {} | hex: {}", img_dim, hex_dim);

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
    println!("write time: {:?}", start_time.elapsed());
    */
}
