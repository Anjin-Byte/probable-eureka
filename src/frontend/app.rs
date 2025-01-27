use eframe::egui::{self, TextureOptions};

use crate::field::field::Field;

pub(crate) struct App {
    texture: Option<egui::TextureHandle>,
    image_dimensions: [usize; 2],
}

impl App {
    pub fn new(image_dimensions: [usize; 2]) -> Self {
        Self {
            texture: None,
            image_dimensions,
        }
    }

    pub fn update_image(&mut self, ctx: &eframe::egui::Context, field: &Field) {
        let max_texture_size = 2048;
        let resized_image_data = field.to_resized_rgba_image(max_texture_size);
    
        // Assume square image dimensions after resizing
        let new_dimensions = [max_texture_size as usize, max_texture_size as usize];
    
        let image = egui::ColorImage::from_rgba_unmultiplied(new_dimensions, &resized_image_data);
        self.texture = Some(ctx.load_texture("intermediate_image", image, TextureOptions::default()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                ui.image(texture);
            }
        });
    }
}