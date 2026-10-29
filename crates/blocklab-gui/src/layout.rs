//! Três arranjos da interface, escolhidos pela largura da janela:
//! computador (menus e barra de status), tablet (barra superior e trilho
//! lateral, protótipo de 1194 × 834) e celular (barra superior e barra
//! inferior, protótipo de 390 × 844).
//!
//! Tablet e celular são feitos para o dedo: alvos de 44 px ou mais,
//! nenhum comando que dependa de teclado, de passar o mouse ou de clique
//! direito.

/// Abaixo disto, celular.
pub const PHONE_MAX: f32 = 600.0;
/// Abaixo disto (e acima de [`PHONE_MAX`]), tablet.
pub const TABLET_MAX: f32 = 1200.0;
/// Tamanho mínimo de um alvo de toque.
pub const TOUCH_TARGET: f32 = 44.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    Phone,
    Tablet,
    #[default]
    Desktop,
}

impl Layout {
    pub fn for_width(width: f32) -> Self {
        if width < PHONE_MAX {
            Layout::Phone
        } else if width < TABLET_MAX {
            Layout::Tablet
        } else {
            Layout::Desktop
        }
    }

    /// Arranjo pensado para o dedo (sem menus nem barra de status).
    pub fn is_touch(self) -> bool {
        self != Layout::Desktop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pontos_de_quebra() {
        assert_eq!(Layout::for_width(390.0), Layout::Phone);
        assert_eq!(Layout::for_width(834.0), Layout::Tablet);
        assert_eq!(Layout::for_width(1194.0), Layout::Tablet);
        assert_eq!(Layout::for_width(1280.0), Layout::Desktop);
        assert!(!Layout::Desktop.is_touch());
    }
}
