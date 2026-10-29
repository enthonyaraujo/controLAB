//! BlockLab: a interface (eframe/egui) como biblioteca.
//!
//! No computador quem a usa é o executável (`src/main.rs`, com a linha de
//! comando); no Android, o APK carrega esta biblioteca e chama
//! `android_main` (`src/android.rs`).

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

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
mod android_insets;

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
