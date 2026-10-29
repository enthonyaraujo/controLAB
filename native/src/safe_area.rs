//! Screen edges covered by the system: status and navigation bars, display cutouts and the
//! soft keyboard. egui removes `RawInput::safe_area_insets` from `Context::content_rect`, which
//! bounds `Controlab::frame`, every `Area`, popup, menu and tooltip; only the backgrounds painted
//! by `frame` reach the screen edges. egui-winit fills the insets on iOS only, so Android reads
//! them over JNI (`android::insets`). On desktop they stay zero and nothing changes.
// Only Android reads and tracks insets; the tests exercise the same logic on desktop.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use eframe::egui::{self, Color32};

/// Insets from each edge: physical pixels as read, egui points once scaled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeArea {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    /// Soft keyboard height from the bottom edge, navigation bar included.
    pub ime: f32,
}

impl SafeArea {
    pub fn scaled(self, factor: f32) -> Self {
        Self {
            top: self.top * factor,
            bottom: self.bottom * factor,
            left: self.left * factor,
            right: self.right * factor,
            ime: self.ime * factor,
        }
    }

    /// The open keyboard raises the bottom edge above it.
    pub fn insets(self) -> egui::SafeAreaInsets {
        egui::SafeAreaInsets(egui::epaint::MarginF32 {
            top: self.top,
            bottom: self.bottom.max(self.ime),
            left: self.left,
            right: self.right,
        })
    }
}

/// Animations of the keyboard and of a rotation end within this time after their trigger.
const SETTLE: f64 = 1.0;
const SETTLE_INTERVAL: f64 = 0.1;
/// The keyboard can close without the field losing focus (back gesture).
const TYPING_INTERVAL: f64 = 0.5;
/// A frame after this pause was caused by something outside the app: a touch, or a redraw
/// such as the 180° rotation that moves the cutout to the other side at the same size.
const WAKE: f64 = 0.5;

/// Window state whose change may move the insets. winit 0.30 drops Android's
/// `ContentRectChanged` and `InsetsChanged`, and the keyboard does not resize the window.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Trigger {
    /// Window size in physical pixels: rotation, split screen, resizing.
    pub size: [i32; 2],
    /// Return from the background, focus coming back after a system dialog.
    pub focused: bool,
    /// A text field has focus, so the soft keyboard may be opening or closing.
    pub typing: bool,
}

/// Decides when to ask the system again, so JNI is never called on every frame.
pub struct Watch {
    trigger: Option<Trigger>,
    settle_until: f64,
    last_read: f64,
    last_frame: f64,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            trigger: None,
            settle_until: f64::NEG_INFINITY,
            last_read: f64::NEG_INFINITY,
            last_frame: f64::NEG_INFINITY,
        }
    }
}

impl Watch {
    /// Whether to read the insets in this frame: at start, then briefly after each trigger
    /// change while the geometry settles, once on waking from idle, and slowly while a text
    /// field has focus.
    pub fn due(&mut self, now: f64, trigger: Trigger) -> bool {
        let woke = now - self.last_frame > WAKE;
        self.last_frame = now;
        if self.trigger != Some(trigger) {
            self.trigger = Some(trigger);
            self.settle_until = now + SETTLE;
            self.last_read = f64::NEG_INFINITY;
        }
        let interval = if now <= self.settle_until || woke {
            SETTLE_INTERVAL
        } else if trigger.typing {
            TYPING_INTERVAL
        } else {
            return false;
        };
        if now - self.last_read < interval {
            return false;
        }
        self.last_read = now;
        true
    }

    /// Delay of the next frame that must look again, if any.
    pub fn next_check(&self, now: f64) -> Option<f64> {
        if now <= self.settle_until {
            Some(SETTLE_INTERVAL)
        } else {
            self.trigger
                .is_some_and(|t| t.typing)
                .then_some(TYPING_INTERVAL)
        }
    }
}

/// Dark system icons over a light background, light icons over a dark one.
pub fn wants_dark_icons(background: Color32) -> bool {
    let c = egui::Rgba::from(background);
    0.2126 * c.r() + 0.7152 * c.g() + 0.0722 * c.b() > 0.18
}

/// Android state behind `Controlab::safe_area`.
#[derive(Default)]
pub struct Tracker {
    watch: Watch,
    /// Last reading, in physical pixels.
    pixels: SafeArea,
    /// `(dark status icons, dark navigation icons)` last applied to the window.
    icons: Option<(bool, bool)>,
}

#[cfg(target_os = "android")]
impl Tracker {
    /// Reads the insets when they may have changed and passes them to egui for the next pass.
    /// `zoom` is the factor that pass uses: `set_zoom_factor` only applies at its start.
    pub fn hook(
        &mut self,
        ctx: &egui::Context,
        raw_input: &mut egui::RawInput,
        zoom: f32,
    ) -> SafeArea {
        let native = raw_input
            .viewport()
            .native_pixels_per_point
            .or_else(|| ctx.native_pixels_per_point())
            .unwrap_or(1.0);
        // egui-winit converted `screen_rect` with the zoom of the previous pass.
        let size = raw_input.screen_rect.map_or([0, 0], |r| {
            let px = r.size() * native * ctx.zoom_factor();
            [px.x.round() as i32, px.y.round() as i32]
        });
        let now = raw_input.time.unwrap_or(0.0);
        let trigger = Trigger {
            size,
            focused: raw_input.focused,
            typing: ctx.text_edit_focused(),
        };
        if self.watch.due(now, trigger)
            && let Some(pixels) = crate::android::insets::read()
        {
            self.pixels = pixels;
        }
        if let Some(delay) = self.watch.next_check(now) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(delay));
        }
        let area = self.pixels.scaled(1.0 / (native * zoom));
        raw_input.safe_area_insets = Some(area.insets());
        area
    }

    /// Icon contrast of the status bar over `top` and of the navigation bar over `bottom`;
    /// asks the window only when it changes, and does not retry a failure every frame.
    pub fn bar_icons(&mut self, top: Color32, bottom: Color32) {
        let icons = (wants_dark_icons(top), wants_dark_icons(bottom));
        if self.icons != Some(icons) {
            self.icons = Some(icons);
            crate::android::insets::set_bar_icons(icons.0, icons.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger(width: i32, typing: bool) -> Trigger {
        Trigger {
            size: [width, 2400],
            focused: true,
            typing,
        }
    }

    /// Frames every 16 ms over `[from, to)`; how many of them read the insets.
    fn reads(watch: &mut Watch, from: f64, to: f64, trigger: Trigger) -> usize {
        let frames = ((to - from) / 0.016).ceil() as usize;
        (0..frames)
            .filter(|i| watch.due(from + *i as f64 * 0.016, trigger))
            .count()
    }

    #[test]
    fn insets_are_read_only_while_the_geometry_settles() {
        let mut watch = Watch::default();
        let portrait = trigger(1080, false);
        assert!(watch.due(0.0, portrait), "read at start");
        assert!(!watch.due(0.016, portrait), "throttled");
        let settling = reads(&mut watch, 0.032, 1.0, portrait);
        assert!(
            (8..=10).contains(&settling),
            "{settling} reads while settling"
        );
        assert_eq!(
            reads(&mut watch, 1.1, 30.0, portrait),
            0,
            "animating: no JNI"
        );
        assert_eq!(watch.next_check(30.0), None, "idle: no repaint");
        assert!(
            watch.due(60.0, portrait),
            "first frame after idle: 180° rotation"
        );
        assert!(!watch.due(60.016, portrait));
        assert!(watch.due(60.032, trigger(2400, false)), "rotation");
        reads(&mut watch, 60.048, 62.0, trigger(2400, false));
        assert!(watch.due(62.0, trigger(2400, true)), "keyboard opening");
        reads(&mut watch, 62.016, 63.5, trigger(2400, true));
        let typing = reads(&mut watch, 63.5, 65.5, trigger(2400, true));
        assert!((3..=4).contains(&typing), "keyboard may close: every 0.5 s");
        assert_eq!(watch.next_check(65.5), Some(TYPING_INTERVAL));
    }

    #[test]
    fn the_keyboard_raises_the_bottom_edge_and_icons_follow_the_background() {
        let area = SafeArea {
            top: 24.0,
            bottom: 48.0,
            left: 0.0,
            right: 0.0,
            ime: 300.0,
        };
        assert_eq!(area.insets().0.bottom, 300.0);
        assert_eq!(SafeArea { ime: 0.0, ..area }.insets().0.bottom, 48.0);
        assert_eq!(area.scaled(0.5).top, 12.0);
        let p = crate::theme::palette(true);
        assert!(
            !wants_dark_icons(p.shell),
            "white icons over the teal header"
        );
        assert!(wants_dark_icons(Color32::WHITE));
        assert!(!wants_dark_icons(Color32::from_rgb(30, 36, 44)));
    }
}
