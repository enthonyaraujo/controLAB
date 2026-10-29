//! Fontes embutidas.
//!
//! - Ubuntu (Regular e Medium): o texto da interface. A Medium fica numa
//!   família própria ([`MEDIUM`]) para títulos e botões — o egui não tem
//!   negrito sintético. Licença: `assets/Ubuntu-LICENSE.txt`.
//! - DejaVu Sans: reserva. A Ubuntu não tem →, ₖ e outros símbolos usados
//!   na interface; sem uma reserva eles dependeriam das fontes do sistema
//!   (e virariam "□" em algumas máquinas). A mesma fonte é usada no PNG
//!   exportado, para o texto sair igual em qualquer computador.
//!   Licença livre, ver `assets/DejaVuSans-LICENSE.txt`.

use eframe::egui;

pub const DEJAVU_SANS: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
pub const DEJAVU_FAMILY: &str = "DejaVu Sans";
const UBUNTU_REGULAR: &[u8] = include_bytes!("../assets/Ubuntu-Regular.ttf");
const UBUNTU_MEDIUM: &[u8] = include_bytes!("../assets/Ubuntu-Medium.ttf");

/// Nome da família com a Ubuntu Medium.
pub const MEDIUM: &str = "medium";

pub fn install(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in [
        ("Ubuntu-Regular", UBUNTU_REGULAR),
        ("Ubuntu-Medium", UBUNTU_MEDIUM),
        ("DejaVuSans", DEJAVU_SANS),
    ] {
        fonts
            .font_data
            .insert(name.into(), FontData::from_static(bytes).into());
    }
    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, "Ubuntu-Regular".into());
    proportional.push("DejaVuSans".into());
    // Medium: as mesmas reservas da proporcional, com a Medium na frente.
    let mut medium = proportional.clone();
    medium.insert(0, "Ubuntu-Medium".into());
    fonts
        .families
        .insert(FontFamily::Name(MEDIUM.into()), medium);
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("DejaVuSans".into());
    ctx.set_fonts(fonts);
}
