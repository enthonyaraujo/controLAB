//! Zoom e deslocamento (pan) do canvas: conversão mundo ↔ tela.
//!
//! tela = origem_do_canvas + deslocamento + mundo · zoom

use eframe::egui::{Pos2, Rect, Vec2};

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub offset: Vec2,
    pub zoom: f32,
    /// Canto superior esquerdo do canvas na tela (atualizado a cada quadro).
    pub origin: Pos2,
}

impl Default for View {
    fn default() -> Self {
        View {
            offset: Vec2::ZERO,
            zoom: 1.0,
            origin: Pos2::ZERO,
        }
    }
}

impl View {
    pub fn to_screen(self, p: Pos2) -> Pos2 {
        self.origin + self.offset + p.to_vec2() * self.zoom
    }

    pub fn to_world(self, s: Pos2) -> Pos2 {
        ((s - self.origin - self.offset) / self.zoom).to_pos2()
    }

    pub fn rect_to_screen(self, r: Rect) -> Rect {
        Rect::from_min_max(self.to_screen(r.min), self.to_screen(r.max))
    }

    /// Zoom mantendo fixo o ponto do mundo sob `anchor` (o cursor).
    pub fn zoom_at(&mut self, anchor: Pos2, factor: f32) {
        let world = self.to_world(anchor);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        // Recalcula o deslocamento para `world` continuar sob `anchor`.
        self.offset = anchor - self.origin - world.to_vec2() * self.zoom;
    }

    /// Enquadra `bounds` (mundo) dentro de `canvas` (tela), com margem
    /// (menor em telas pequenas, como a do celular).
    pub fn fit(&mut self, bounds: Rect, canvas: Rect) {
        let margin = (canvas.width().min(canvas.height()) * 0.08).clamp(16.0, 60.0);
        let zx = (canvas.width() - 2.0 * margin) / bounds.width().max(1.0);
        let zy = (canvas.height() - 2.0 * margin) / bounds.height().max(1.0);
        self.zoom = zx.min(zy).clamp(MIN_ZOOM, 1.6);
        self.origin = canvas.min;
        self.offset = canvas.center() - canvas.min - bounds.center().to_vec2() * self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    #[test]
    fn ida_e_volta_e_zoom_no_cursor() {
        let mut v = View {
            origin: pos2(100.0, 50.0),
            ..View::default()
        };
        let w = pos2(30.0, 40.0);
        assert_eq!(v.to_world(v.to_screen(w)), w);
        let cursor = pos2(300.0, 200.0);
        let before = v.to_world(cursor);
        v.zoom_at(cursor, 2.0);
        let after = v.to_world(cursor);
        assert!((before - after).length() < 1e-3);
        assert_eq!(v.zoom, 2.0);
    }
}
