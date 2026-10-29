//! Ícone da janela (barra de tarefas no Windows, painel no Linux),
//! desenhado como SVG e rasterizado com `resvg`: um somador e um bloco
//! com realimentação.

use eframe::egui;

const ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect x="8" y="8" width="240" height="240" rx="48" fill="#1f5f99"/>
  <g stroke="#ffffff" stroke-width="12" fill="none" stroke-linecap="round" stroke-linejoin="round">
    <path d="M30 116 H70"/>
    <circle cx="104" cy="116" r="26"/>
    <path d="M86 98 L122 134 M122 98 L86 134" stroke-width="8"/>
    <path d="M130 116 H146"/>
    <rect x="160" y="88" width="62" height="56" rx="6"/>
    <path d="M222 116 H234 V200 H104 V156"/>
  </g>
  <g fill="#ffffff">
    <polygon points="80,116 64,104 64,128"/>
    <polygon points="160,116 144,104 144,128"/>
    <polygon points="104,144 92,160 116,160"/>
  </g>
</svg>"##;

/// Ícone `size`×`size` em RGBA não pré-multiplicado (o que o egui espera).
pub fn icon(size: u32) -> egui::IconData {
    use resvg::{tiny_skia, usvg};
    let tree = usvg::Tree::from_str(ICON_SVG, &usvg::Options::default())
        .expect("o SVG do ícone é fixo e válido");
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("tamanho de ícone válido");
    let scale = size as f32 / tree.size().width();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    // tiny-skia guarda as cores pré-multiplicadas pelo alfa; desfaz isso.
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icone_tem_fundo_e_cantos_transparentes() {
        let ic = icon(64);
        assert_eq!(ic.rgba.len(), 64 * 64 * 4);
        let px = |x: usize, y: usize| &ic.rgba[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
        assert_eq!(px(0, 0)[3], 0, "canto arredondado deve ser transparente");
        // Um ponto do fundo azul, longe dos traços brancos.
        assert_eq!(px(20, 50), [0x1f, 0x5f, 0x99, 0xff]);
    }
}
