//! Visual do BlockLab: cores dos modos claro e escuro, tamanhos de texto e
//! o estilo dos widgets do egui. As cores vêm do protótipo da tela inicial.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, Stroke, Style, TextStyle, Theme, ThemePreference,
    Visuals, vec2,
};

/// Cores com nome (uma tabela por tema).
pub struct Tokens {
    /// Fundo da área central (atrás dos cartões).
    pub app_bg: Color32,
    /// Painéis, cartões e campos.
    pub surface: Color32,
    /// Botão secundário, fundo das miniaturas.
    pub subtle: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    /// Linhas que separam painéis.
    pub border: Color32,
    pub card_border: Color32,
    pub control_border: Color32,
    pub control_border_hover: Color32,
    pub canvas: Color32,
    pub grid: Color32,
    pub grid_major: Color32,
    pub preview_bg: Color32,
    pub formula_bg: Color32,
    pub nav_selected: Color32,
    /// Pílula atrás do ícone do item escolhido (tablet e celular).
    pub nav_pill: Color32,
    pub recent_bg: Color32,
    pub dashed: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub on_accent_weak: Color32,
    pub link: Color32,
    /// Contorno do que está selecionado no canvas.
    pub selected: Color32,
    pub selection_bg: Color32,
    pub text_strong: Color32,
    pub text: Color32,
    /// Títulos de painel e o traço da fração.
    pub text_heading: Color32,
    pub text_nav: Color32,
    pub text_muted: Color32,
    pub text_section: Color32,
    pub placeholder: Color32,
    pub icon: Color32,
    pub warn: Color32,
    /// Fundo do item da lista do Mason sob o mouse (combina com o
    /// destaque laranja no canvas).
    pub highlight_bg: Color32,
    pub error: Color32,
    /// Miniaturas dos cartões.
    pub mini_line: Color32,
    pub mini_text: Color32,
    pub mini_fill: Color32,
    /// Pré-visualização (miniatura maior, painel da direita).
    pub preview_line: Color32,
    pub preview_text: Color32,
    pub preview_fill: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const LIGHT: Tokens = Tokens {
    app_bg: hex(0xf3f4f6),
    surface: hex(0xffffff),
    subtle: hex(0xf7f8fa),
    hover: hex(0xeef0f3),
    pressed: hex(0xe4e7eb),
    border: hex(0xdadde2),
    card_border: hex(0xd5d9df),
    control_border: hex(0xcdd2d8),
    control_border_hover: hex(0xb4bbc3),
    canvas: hex(0xf7f8fa),
    grid: hex(0xe6e9ed),
    grid_major: hex(0xd9dde2),
    preview_bg: hex(0xe9ecef),
    formula_bg: hex(0xf7f8fa),
    nav_selected: hex(0xe8f1f7),
    nav_pill: hex(0xd6e7f2),
    recent_bg: hex(0xffffff),
    dashed: hex(0xc3c9d0),
    accent: hex(0x0e6a99),
    accent_hover: hex(0x0c5d87),
    on_accent: hex(0xffffff),
    on_accent_weak: hex(0xd5e5ed),
    link: hex(0x0b5f8a),
    selected: hex(0x0e6a99),
    selection_bg: hex(0xc9dfec),
    text_strong: hex(0x14181d),
    text: hex(0x1e2329),
    text_heading: hex(0x2a3038),
    text_nav: hex(0x3a414a),
    text_muted: hex(0x5f6873),
    text_section: hex(0x525b66),
    placeholder: hex(0x85888b),
    icon: hex(0x6b7480),
    warn: hex(0xb25e00),
    highlight_bg: hex(0xfbeedd),
    error: hex(0xc4312a),
    mini_line: hex(0xafb6bd),
    mini_text: hex(0x565c64),
    mini_fill: hex(0xffffff),
    preview_line: hex(0x8a939e),
    preview_text: hex(0x3a414a),
    preview_fill: hex(0xffffff),
};

pub const DARK: Tokens = Tokens {
    app_bg: hex(0x121212),
    surface: hex(0x1a1a1a),
    subtle: hex(0x232323),
    hover: hex(0x2a2a2a),
    pressed: hex(0x313131),
    border: hex(0x2b2b2b),
    card_border: hex(0x2e2e2e),
    control_border: hex(0x333333),
    control_border_hover: hex(0x454545),
    canvas: hex(0x141414),
    grid: hex(0x1f1f1f),
    grid_major: hex(0x292929),
    preview_bg: hex(0x1c1c1c),
    formula_bg: hex(0x161616),
    nav_selected: hex(0x2a2a2a),
    nav_pill: hex(0x1f3644),
    recent_bg: hex(0x121212),
    dashed: hex(0x333333),
    accent: hex(0x0e6a99),
    accent_hover: hex(0x1478ab),
    on_accent: hex(0xffffff),
    on_accent_weak: hex(0xd5e5ed),
    link: hex(0x5fb4e0),
    selected: hex(0x5fb4e0),
    selection_bg: hex(0x164a66),
    text_strong: hex(0xe6e6e6),
    text: hex(0xdadada),
    text_heading: hex(0xcfcfcf),
    text_nav: hex(0xc4c4c4),
    text_muted: hex(0x9a9a9a),
    text_section: hex(0xa8a8a8),
    placeholder: hex(0x828282),
    icon: hex(0x9a9a9a),
    warn: hex(0xe0a040),
    highlight_bg: hex(0x3a2e1c),
    error: hex(0xff6b61),
    mini_line: hex(0x4e4e4e),
    mini_text: hex(0xafafaf),
    mini_fill: hex(0x1c1c1c),
    preview_line: hex(0x6e6e6e),
    preview_text: hex(0xc8c8c8),
    preview_fill: hex(0x1a1a1a),
};

/// Cores do tema em uso.
pub fn tokens(v: &Visuals) -> &'static Tokens {
    if v.dark_mode { &DARK } else { &LIGHT }
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn medium(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Nome da preferência no menu e no arquivo de preferências.
pub fn preference_name(p: ThemePreference) -> &'static str {
    match p {
        ThemePreference::Light => "claro",
        ThemePreference::Dark => "escuro",
        ThemePreference::System => "sistema",
    }
}

pub fn parse_preference(s: &str) -> Option<ThemePreference> {
    match s.trim() {
        "claro" => Some(ThemePreference::Light),
        "escuro" => Some(ThemePreference::Dark),
        "sistema" => Some(ThemePreference::System),
        _ => None,
    }
}

/// Instala fontes e o estilo dos dois temas.
pub fn install(ctx: &egui::Context) {
    crate::fonts::install(ctx);
    ctx.set_style_of(Theme::Light, style(&LIGHT, false));
    ctx.set_style_of(Theme::Dark, style(&DARK, true));
}

fn widget(fill: Color32, stroke: Stroke, fg: Color32, fg_width: f32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: stroke,
        fg_stroke: Stroke::new(fg_width, fg),
        corner_radius: CornerRadius::same(6),
        expansion: 0.0,
    }
}

fn visuals(t: &Tokens, dark: bool) -> Visuals {
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.panel_fill = t.surface;
    v.window_fill = t.surface;
    v.extreme_bg_color = t.surface;
    v.text_edit_bg_color = Some(t.surface);
    v.faint_bg_color = t.subtle;
    v.code_bg_color = t.subtle;
    v.hyperlink_color = t.link;
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.error;
    v.weak_text_color = Some(t.text_muted);
    v.window_stroke = Stroke::new(1.0, t.border);
    v.window_corner_radius = CornerRadius::same(8);
    v.menu_corner_radius = CornerRadius::same(6);
    v.selection.bg_fill = t.selection_bg;
    v.selection.stroke = Stroke::new(1.0, t.selected);
    v.text_cursor.stroke = Stroke::new(2.0, t.accent);
    v.widgets.noninteractive = widget(t.surface, Stroke::new(1.0, t.border), t.text, 1.0);
    v.widgets.inactive = widget(t.subtle, Stroke::new(1.0, t.control_border), t.text, 1.0);
    v.widgets.hovered = widget(
        t.hover,
        Stroke::new(1.0, t.control_border_hover),
        t.text_strong,
        1.5,
    );
    v.widgets.active = widget(
        t.pressed,
        Stroke::new(1.0, t.control_border_hover),
        t.text_strong,
        2.0,
    );
    v.widgets.open = widget(
        t.hover,
        Stroke::new(1.0, t.control_border),
        t.text_strong,
        1.0,
    );
    v
}

fn style(t: &Tokens, dark: bool) -> Style {
    let mut s = Style {
        visuals: visuals(t, dark),
        ..Style::default()
    };
    s.text_styles = [
        (TextStyle::Small, regular(11.0)),
        (TextStyle::Body, regular(13.0)),
        (TextStyle::Button, regular(13.0)),
        (TextStyle::Heading, medium(15.0)),
        (TextStyle::Monospace, FontId::monospace(12.0)),
    ]
    .into();
    s.spacing.item_spacing = vec2(8.0, 6.0);
    s.spacing.button_padding = vec2(8.0, 3.0);
    s.spacing.interact_size.y = 22.0;
    s.spacing.menu_margin = Margin::same(6);
    s
}
