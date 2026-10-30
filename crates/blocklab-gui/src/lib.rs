//! Interface gráfica (eframe/egui) do módulo Diagrama de Blocos do ControLAB.

pub mod app;
mod canvas;
mod doc;
mod draw;
pub mod export;
pub mod file_actions;
pub mod files;
mod fonts;
mod geometry;
mod home;
mod home_mobile;
pub mod icon;
mod icons;
mod layout;
mod mobile;
mod palette;
pub mod prefs;
mod result_panel;
pub mod safe_area;
mod solver;
pub mod theme;
mod view;

#[cfg(test)]
mod ui_tests;

use std::path::Path;

use eframe::egui;

/// Grava uma captura da janela em PNG.
pub fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let [w, h] = image.size;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())
}
