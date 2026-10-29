//! Área segura: quanto de cada borda da janela fica coberto pelo sistema
//! (barra de status, barra de navegação, recorte da câmera, teclado
//! virtual).
//!
//! No Android ela vem de `android_insets` e é entregue ao egui em
//! `RawInput::safe_area_insets` (ver `App::raw_input_hook`). A partir daí o
//! egui faz o resto: a `Ui` raiz, os painéis, menus, janelas e diálogos
//! ficam dentro de `ctx.content_rect()` (a tela menos esses recuos), e só o
//! fundo (a cor de limpeza, que é a cor das barras) chega às bordas. No
//! computador ela fica em zero e nada muda.

use eframe::egui::{self, epaint::MarginF32};

/// Recuos em pixels físicos, como o Android informa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PxInsets {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

/// Recuos em pontos do egui.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeArea {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl SafeArea {
    /// Converte de pixels físicos para pontos (divide pelo fator de escala).
    pub fn from_pixels(px: PxInsets, pixels_per_point: f32) -> Self {
        let ppp = if pixels_per_point > 0.0 {
            pixels_per_point
        } else {
            1.0
        };
        let pt = |v: i32| v.max(0) as f32 / ppp;
        SafeArea {
            top: pt(px.top),
            bottom: pt(px.bottom),
            left: pt(px.left),
            right: pt(px.right),
        }
    }

    pub fn to_egui(self) -> egui::SafeAreaInsets {
        egui::SafeAreaInsets(MarginF32 {
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Rect, pos2};

    #[test]
    fn pixels_viram_pontos() {
        // Celular com densidade 2,75: status de 132 px, gestos de 66 px.
        let px = PxInsets {
            top: 132,
            bottom: 66,
            left: 0,
            right: -5,
        };
        let s = SafeArea::from_pixels(px, 2.75);
        assert_eq!(
            s,
            SafeArea {
                top: 48.0,
                bottom: 24.0,
                left: 0.0,
                right: 0.0,
            }
        );
        let tela = Rect::from_min_max(pos2(0.0, 0.0), pos2(392.0, 873.0));
        let conteudo = tela - s.to_egui();
        assert_eq!(
            conteudo,
            Rect::from_min_max(pos2(0.0, 48.0), pos2(392.0, 849.0))
        );
    }

    #[test]
    fn computador_fica_em_zero() {
        assert_eq!(
            SafeArea::default().to_egui(),
            egui::SafeAreaInsets::default()
        );
    }
}
