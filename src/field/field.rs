use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use std::u16;

use byteorder::{LittleEndian, ReadBytesExt};
use hashbrown::HashMap;
use image::{DynamicImage, ImageBuffer, ImageError, Luma, RgbaImage};

use crate::hex::hex::Hex;
use crate::hex::layout::Layout;
use crate::hex::point::Point;

const IMG_WIDTH: usize = 8192;
const ARRAY_LEN: usize = IMG_WIDTH * IMG_WIDTH;

#[derive(Debug)]
pub struct Bin {
    pub agr_value: f32,
    pub pixel_count: u32,
}

impl Bin {
    pub fn new(total: f32, count: u32) -> Self {
        Self {
            agr_value: total,
            pixel_count: count,
        }
    }
}

pub struct Field {
    pub flattened_field: Box<[f32]>,
    pub width: usize,
}

impl Field {
    pub fn new() -> Self {
        Self {
            flattened_field: vec![0_f32; ARRAY_LEN].into_boxed_slice(),
            width: IMG_WIDTH,
        }
    }

    pub fn from_r32(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;

        let file_size = file.metadata()?.len();
        if file_size != (ARRAY_LEN * std::mem::size_of::<f32>()) as u64 {
            let extra_bytes = file_size - (ARRAY_LEN * 4) as u64;
            println!("Warning: Ignoring {} extra bytes in the file.", extra_bytes);
            return Err(format!(
                "Unexpected file size: expected {}, got {}",
                ARRAY_LEN * 4,
                file_size
            )
            .into());
        }

        let mut buffer = vec![0_u8; ARRAY_LEN * 4];
        file.read_exact(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);
        let mut normalized_f32 = vec![0_f32; ARRAY_LEN].into_boxed_slice();

        for val in normalized_f32.iter_mut() {
            if cursor.position() >= cursor.get_ref().len() as u64 {
                return Err("Cursor out of bounds while reading file".into());
            }
            *val = cursor.read_f32::<LittleEndian>()?;
        }

        Ok(Self {
            flattened_field: normalized_f32,
            width: IMG_WIDTH,
        })
    }

    pub fn to_resized_rgba_image(&self, max_size: u32) -> Vec<u8> {
        //let original_size = (self.width as u32, self.width as u32);

        // Create an `RgbaImage` from the flattened field data
        let mut image = RgbaImage::new(self.width as u32, self.width as u32);
        self.flattened_field
            .iter()
            .enumerate()
            .for_each(|(i, &value)| {
                let normalized = (value * 255.0) as u8;
                let x = (i % self.width) as u32;
                let y = (i / self.width) as u32;
                image.put_pixel(x, y, image::Rgba([normalized, normalized, normalized, 255]));
            });

        // Resize the image to fit within `max_size`
        let resized_image = DynamicImage::ImageRgba8(image)
            .resize(max_size, max_size, image::imageops::FilterType::Nearest)
            .to_rgba8();

        resized_image.into_raw()
    }

    pub fn write_png_u16(&self, path: &Path) -> Result<(), ImageError> {
        assert_eq!(ARRAY_LEN, IMG_WIDTH * IMG_WIDTH, "Image is not square!");

        // Compute min and max to normalize the field
        let (min, max) = self
            .flattened_field
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), &value| {
                (min.min(value), max.max(value))
            });

        let range = if max - min > std::f32::EPSILON {
            max - min
        } else {
            1.0 // Avoid division by zero; treat as uniform field
        };

        // Create the image buffer
        let img = ImageBuffer::from_fn(IMG_WIDTH as u32, IMG_WIDTH as u32, |x, y| {
            let value = self.flattened_field[(y as usize * IMG_WIDTH) + x as usize];
            // Normalize to [0.0, 1.0]
            let normalized_value = (value - min) / range;
            // Scale to u16 range
            let u16_value = (normalized_value * u16::MAX as f32) as u16;
            Luma::<u16>([u16_value])
        });

        img.save(path)
    }

    pub fn hex_aggregate(&self, layout: Layout) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bin: HashMap<Hex, Bin> = HashMap::new();
        let mut hex_field: Box<[f32; ARRAY_LEN]> = Box::new([0.0; ARRAY_LEN]);

        const SQRT_3: f32 = 1.73205080756888;
        let size = layout.size.x as f32;

        let height = self.width as f32 / (size * SQRT_3);
        let width = self.width as f32 / (size * 3_f32 / 2_f32);

        let left = -1;
        let right = width.ceil() as i32 + 1;
        let top = 0;
        let bottom = height.ceil() as i32 + 1;

        for q in left..right {
            let q_offset = (q + 1) >> 1;
            for r in (top - q_offset)..(bottom - q_offset) {
                bin.insert(Hex::new(q, r), Bin::new(0_f32, 0));
            }
        }

        self.flattened_field
            .iter()
            .enumerate()
            .for_each(|(i, value)| {
                let hex = Hex::from(Hex::from_point(
                    &layout,
                    &Point {
                        x: (i % IMG_WIDTH) as f64,
                        y: (i / IMG_WIDTH) as f64,
                    },
                ));

                match bin.get_mut(&hex) {
                    Some(basket) => {
                        basket.agr_value += value;
                        basket.pixel_count += 1;
                    }
                    None => (),
                }
            });

        for i in 0..hex_field.len() {
            let hex = Hex::from(Hex::from_point(
                &layout,
                &Point {
                    x: (i % IMG_WIDTH) as f64,
                    y: (i / IMG_WIDTH) as f64,
                },
            ));

            match bin.get(&hex) {
                Some(value) => {
                    hex_field[i] = value.agr_value / value.pixel_count as f32;
                }
                None => (),
            }
        }

        Ok(Self {
            flattened_field: hex_field,
            width: self.width,
        })
    }

    /// # Arguments
    /// * `dx` - Horizontal shift (positive is right, negative is left).
    /// * `dy` - Vertical shift (positive is down, negative is up).
    fn shift(field: &[f32], width: usize, dx: isize, dy: isize) -> Vec<f32> {
        let height = field.len() / width;
        let mut shifted = vec![0.0; field.len()];

        for i in 0..field.len() {
            let row = i / width;
            let col = i % width;

            // Calculate new coordinates after the shift
            let new_row = row as isize + dy;
            let new_col = col as isize + dx;

            if new_row >= 0 && new_row < height as isize && new_col >= 0 && new_col < width as isize
            {
                let new_idx = (new_row as usize) * width + (new_col as usize);
                shifted[i] = field[new_idx];
            } else {
                // Retain the original value at the boundary
                shifted[i] = field[i];
            }
        }

        shifted
    }

    pub fn sobel(&self) -> Result<Self, Box<dyn std::error::Error>> {
        let top_left        = Self::shift(&self.flattened_field, IMG_WIDTH, -1, -1);
        let top             = Self::shift(&self.flattened_field, IMG_WIDTH, 0, -1);
        let top_right       = Self::shift(&self.flattened_field, IMG_WIDTH, 1, -1);

        let left            = Self::shift(&self.flattened_field, IMG_WIDTH, -1, 0);
        let right           = Self::shift(&self.flattened_field, IMG_WIDTH, 1, 0);

        let bottom_left     = Self::shift(&self.flattened_field, IMG_WIDTH, -1, 1);
        let bottom          = Self::shift(&self.flattened_field, IMG_WIDTH, 0, 1);
        let bottom_right    = Self::shift(&self.flattened_field, IMG_WIDTH, 1, 1);

        let mut gradient_x = vec![0.0; ARRAY_LEN];
        let mut gradient_y = vec![0.0; ARRAY_LEN];

        for i in 0..ARRAY_LEN {
            gradient_x[i] = (top_right[i] + 2.0 * right[i] + bottom_right[i])
                - (top_left[i] + 2.0 * left[i] + bottom_left[i]);

            gradient_y[i] = (top_left[i] + 2.0 * top[i] + top_right[i])
                - (bottom_left[i] + 2.0 * bottom[i] + bottom_right[i]);
        }

        // Combine gradients to compute magnitude
        let mut result = vec![0.0; ARRAY_LEN];
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for i in 0..ARRAY_LEN {
            result[i] = (gradient_x[i].powi(2) + gradient_y[i].powi(2)).sqrt();
            min = min.min(result[i]);
            max = max.max(result[i]);
        }

        for v in result.iter_mut() {
            if max - min > std::f32::EPSILON {
                *v = (*v - min) / (max - min);
            } else {
                *v = 0.0;
            }
        }

        Ok(Self {
            flattened_field: result.into_boxed_slice(),
            width: IMG_WIDTH,
        })
    }

    pub fn prewitt(&self) -> Result<Self, Box<dyn std::error::Error>> {
        let top_left        = Self::shift(&self.flattened_field, IMG_WIDTH, -1, -1);
        let top             = Self::shift(&self.flattened_field, IMG_WIDTH, 0, -1);
        let top_right       = Self::shift(&self.flattened_field, IMG_WIDTH, 1, -1);

        let left            = Self::shift(&self.flattened_field, IMG_WIDTH, -1, 0);
        let right           = Self::shift(&self.flattened_field, IMG_WIDTH, 1, 0);

        let bottom_left     = Self::shift(&self.flattened_field, IMG_WIDTH, -1, 1);
        let bottom          = Self::shift(&self.flattened_field, IMG_WIDTH, 0, 1);
        let bottom_right    = Self::shift(&self.flattened_field, IMG_WIDTH, 1, 1);

        let mut gradient_x = vec![0.0; ARRAY_LEN];
        let mut gradient_y = vec![0.0; ARRAY_LEN];

        for i in 0..ARRAY_LEN {
            gradient_x[i] = (top_right[i] + right[i] + bottom_right[i])
                - (top_left[i] + left[i] + bottom_left[i]);

            gradient_y[i] = (top_left[i] + top[i] + top_right[i])
                - (bottom_left[i] + bottom[i] + bottom_right[i]);
        }

        let mut result = vec![0.0; ARRAY_LEN];
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for i in 0..ARRAY_LEN {
            result[i] = (gradient_x[i].powi(2) + gradient_y[i].powi(2)).sqrt();
            min = min.min(result[i]);
            max = max.max(result[i]);
        }

        for v in result.iter_mut() {
            if max - min > std::f32::EPSILON {
                *v = (*v - min) / (max - min);
            } else {
                *v = 0.0;
            }
        }

        Ok(Self {
            flattened_field: result.into_boxed_slice(),
            width: IMG_WIDTH,
        })
    }

    pub fn steepness(&self) -> Result<Self, Box<dyn std::error::Error>> {
        let shifted_right   = Self::shift(&self.flattened_field, IMG_WIDTH, 1, 0);
        let shifted_down    = Self::shift(&self.flattened_field, IMG_WIDTH, 0, 1);

        let mut result = vec![0.0; ARRAY_LEN];
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for i in 0..ARRAY_LEN {
            let dx = shifted_right[i] - self.flattened_field[i];
            let dy = shifted_down[i] - self.flattened_field[i];
            result[i] = (dx * dx + dy * dy).sqrt();

            min = min.min(result[i]);
            max = max.max(result[i]);
        }

        for v in result.iter_mut() {
            if max - min > std::f32::EPSILON {
                *v = (*v - min) / (max - min);
            } else {
                *v = 0.0;
            }
        }

        Ok(Self {
            flattened_field: result.into_boxed_slice(),
            width: IMG_WIDTH,
        })
    }

    fn compute_eigenvalues(hessian: [[f32; 2]; 2]) -> (f32, f32) {
        let trace = hessian[0][0] + hessian[1][1];
        let determinant = hessian[0][0] * hessian[1][1] - hessian[0][1] * hessian[1][0];
        let discriminant = (trace.powi(2) - 4.0 * determinant).sqrt();

        let lambda1 = (trace + discriminant) / 2.0;
        let lambda2 = (trace - discriminant) / 2.0;

        (lambda1, lambda2)
    }

    /// Normalize the field values to a specified range [new_min, new_max].
    pub fn normalize(&self, new_min: f32, new_max: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        // Calculate the current min and max values
        for &value in self.flattened_field.iter() {
            min = min.min(value);
            max = max.max(value);
        }

        if max - min < std::f32::EPSILON {
            // Handle edge case: all values are identical
            return Err("Normalization failed: All values in the field are identical.".into());
        }

        // Compute the normalization factor
        let range = max - min;
        let new_range = new_max - new_min;

        let normalized_field: Vec<f32> = self
            .flattened_field
            .iter()
            .map(|&value| ((value - min) / range) * new_range + new_min)
            .collect();

        Ok(Self {
            flattened_field: normalized_field.into_boxed_slice(),
            width: self.width,
        })
        }

    pub fn structural_lines(&self) -> Result<(Self, Self, Self, Self), Box<dyn std::error::Error>> {
        // Step 1: Compute the gradient
        let gradient_x = Self::shift(&self.flattened_field, self.width, 1, 0)
            .iter()
            .zip(self.flattened_field.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>();

        let gradient_y = Self::shift(&self.flattened_field, self.width, 0, 1)
            .iter()
            .zip(self.flattened_field.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>();

        // Step 2: Compute second-order derivatives (Hessian components)
        let dxx = Self::shift(&gradient_x, self.width, 1, 0)
            .iter()
            .zip(gradient_x.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>();

        let dyy = Self::shift(&gradient_y, self.width, 0, 1)
            .iter()
            .zip(gradient_y.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>();

        let dxy = Self::shift(&gradient_x, self.width, 0, 1)
            .iter()
            .zip(gradient_y.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>();

        // Step 3: Compute principal curvatures and directions
        let mut crests = vec![0.0; self.flattened_field.len()];
        let mut thalwegs = vec![0.0; self.flattened_field.len()];
        let mut convex_lines = vec![0.0; self.flattened_field.len()];
        let mut concave_lines = vec![0.0; self.flattened_field.len()];

        for i in 0..self.flattened_field.len() {
            let hessian = [[dxx[i], dxy[i]], [dxy[i], dyy[i]]];
            let (lambda1, lambda2) = Self::compute_eigenvalues(hessian);

            // Curvature logic
            if lambda1 > 0.0 {
                crests[i] = lambda1;
            } else if lambda1 < 0.0 {
                thalwegs[i] = lambda1;
            }

            if lambda2 > 0.0 {
                convex_lines[i] = lambda2;
            } else if lambda2 < 0.0 {
                concave_lines[i] = lambda2;
            }
        }

        // Normalize results for visualization
        Ok((
            Self {
                flattened_field: crests.into_boxed_slice(),
                width: self.width,
            }.normalize(0.0, 1.0).expect("Failed to normalize crests field"),
            Self {
                flattened_field: thalwegs.into_boxed_slice(),
                width: self.width,
            }.normalize(0.0, 1.0).expect("Failed to normalize thalwegs field"),
            Self {
                flattened_field: convex_lines.into_boxed_slice(),
                width: self.width,
            }.normalize(0.0, 1.0).expect("Failed to normalize convex_lines field"),
            Self {
                flattened_field: concave_lines.into_boxed_slice(),
                width: self.width,
            }.normalize(0.0, 1.0).expect("Failed to normalize concave_lines field"),
        ))
    }
}
