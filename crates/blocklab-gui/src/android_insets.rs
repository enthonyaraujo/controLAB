//! Recuos do sistema no Android, lidos pela JNI, e a cor dos ícones das
//! barras.
//!
//! - API 30+: `getRootWindowInsets().getInsets(systemBars() | displayCutout())`
//!   e, à parte, `getInsets(ime())` para o teclado virtual.
//! - API 26–29: `getSystemWindowInset*` (que já inclui o teclado, pois a
//!   NativeActivity usa `adjustResize`) mais o `DisplayCutout` (API 28+).
//!
//! O eframe não repassa ao app os eventos do android-activity
//! (`ContentRectChanged`, `ConfigChanged`, `WindowResized`, retomada), então
//! o [`Tracker`] observa os efeitos deles a cada quadro (retângulo de
//! conteúdo do android-activity, tamanho e densidade da janela, foco,
//! teclado pedido pelo egui) e relê os recuos via JNI só quando algo muda,
//! continuando a reler por um instante para acompanhar animações (o
//! teclado sobe em ~300 ms).

// A JNI exige ponteiros crus da JavaVM e da atividade (ver os SAFETY).
#![allow(unsafe_code)]

use eframe::egui;
use jni::errors::Result;
use jni::objects::JObject;
use jni::strings::JNIStr;
use jni::{Env, JValue, JavaVM, jni_sig, jni_str};
use winit::platform::android::activity::AndroidApp;

use crate::safe_area::{PxInsets, SafeArea};

/// O que o sistema cobre, em pixels físicos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SystemInsets {
    /// Barras do sistema e recorte da câmera.
    bars: PxInsets,
    /// Altura do teclado virtual sobre a janela (0 se fechado ou se já
    /// estiver somado em `bars.bottom`, abaixo da API 30).
    ime_bottom: i32,
}

/// Depois de uma mudança, quanto tempo seguir relendo os recuos (s).
const SETTLE: f64 = 1.5;
/// Com o teclado aberto, intervalo entre releituras (s): ele pode fechar
/// sem nenhum outro sinal (gesto de voltar).
const IME_RECHECK: f64 = 1.0;

/// Roda `f` com o `Env` da thread atual e a atividade.
fn with_activity<T>(
    app: &AndroidApp,
    f: impl FnOnce(&mut Env, &JObject) -> Result<T>,
) -> Result<T> {
    // SAFETY: o android-activity entrega o ponteiro da JavaVM do processo,
    // válido enquanto o app roda.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    // Prende a thread à VM (uma vez; depois é barato) com um quadro local
    // próprio, e devolve exceções Java como `Err` em vez de deixá-las
    // pendentes.
    vm.attach_current_thread(|env| {
        // SAFETY: é uma referência global à atividade, mantida pelo
        // android-activity enquanto o app roda; `JObject` só a empresta (não
        // a apaga ao sair de escopo).
        let activity = unsafe { JObject::from_raw(env, app.activity_as_ptr().cast()) };
        f(env, &activity)
    })
}

fn sdk_int(env: &mut Env) -> Result<i32> {
    env.get_static_field(
        jni_str!("android/os/Build$VERSION"),
        jni_str!("SDK_INT"),
        jni_sig!("I"),
    )?
    .i()
}

fn window<'l>(env: &mut Env<'l>, activity: &JObject) -> Result<JObject<'l>> {
    env.call_method(
        activity,
        jni_str!("getWindow"),
        jni_sig!("()Landroid/view/Window;"),
        &[],
    )?
    .l()
}

fn int_method(env: &mut Env, obj: &JObject, name: &JNIStr) -> Result<i32> {
    env.call_method(obj, name, jni_sig!("()I"), &[])?.i()
}

/// `WindowInsets.Type.<name>()` (API 30+).
fn type_mask(env: &mut Env, name: &JNIStr) -> Result<i32> {
    env.call_static_method(
        jni_str!("android/view/WindowInsets$Type"),
        name,
        jni_sig!("()I"),
        &[],
    )?
    .i()
}

/// `insets.getInsets(mask)` lido como [`PxInsets`] (API 30+).
fn insets_of(env: &mut Env, insets: &JObject, mask: i32) -> Result<PxInsets> {
    let r = env
        .call_method(
            insets,
            jni_str!("getInsets"),
            jni_sig!("(I)Landroid/graphics/Insets;"),
            &[JValue::Int(mask)],
        )?
        .l()?;
    let mut field = |name: &JNIStr| env.get_field(&r, name, jni_sig!("I"))?.i();
    Ok(PxInsets {
        top: field(jni_str!("top"))?,
        bottom: field(jni_str!("bottom"))?,
        left: field(jni_str!("left"))?,
        right: field(jni_str!("right"))?,
    })
}

/// Lê os recuos da janela. `None` enquanto a janela ainda não está anexada.
fn query(app: &AndroidApp) -> Result<Option<SystemInsets>> {
    with_activity(app, |env, activity| {
        let sdk = sdk_int(env)?;
        let window = window(env, activity)?;
        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let insets = env
            .call_method(
                &decor,
                jni_str!("getRootWindowInsets"),
                jni_sig!("()Landroid/view/WindowInsets;"),
                &[],
            )?
            .l()?;
        if insets.is_null() {
            return Ok(None);
        }
        if sdk >= 30 {
            let bars = type_mask(env, jni_str!("systemBars"))?
                | type_mask(env, jni_str!("displayCutout"))?;
            let ime = type_mask(env, jni_str!("ime"))?;
            return Ok(Some(SystemInsets {
                bars: insets_of(env, &insets, bars)?,
                ime_bottom: insets_of(env, &insets, ime)?.bottom,
            }));
        }
        // Até a API 29: os recuos "da janela do sistema" (barras e, com
        // adjustResize, o teclado).
        let mut bars = PxInsets {
            top: int_method(env, &insets, jni_str!("getSystemWindowInsetTop"))?,
            bottom: int_method(env, &insets, jni_str!("getSystemWindowInsetBottom"))?,
            left: int_method(env, &insets, jni_str!("getSystemWindowInsetLeft"))?,
            right: int_method(env, &insets, jni_str!("getSystemWindowInsetRight"))?,
        };
        if sdk >= 28 {
            let cutout = env
                .call_method(
                    &insets,
                    jni_str!("getDisplayCutout"),
                    jni_sig!("()Landroid/view/DisplayCutout;"),
                    &[],
                )?
                .l()?;
            if !cutout.is_null() {
                bars.top = bars
                    .top
                    .max(int_method(env, &cutout, jni_str!("getSafeInsetTop"))?);
                bars.bottom =
                    bars.bottom
                        .max(int_method(env, &cutout, jni_str!("getSafeInsetBottom"))?);
                bars.left = bars
                    .left
                    .max(int_method(env, &cutout, jni_str!("getSafeInsetLeft"))?);
                bars.right =
                    bars.right
                        .max(int_method(env, &cutout, jni_str!("getSafeInsetRight"))?);
            }
        }
        Ok(Some(SystemInsets {
            bars,
            ime_bottom: 0,
        }))
    })
}

/// Ícones escuros nas barras (`light = true`, para o tema claro) ou claros.
/// Devolve `false` abaixo da API 30, onde isso exigiria a thread de
/// interface do Java (`setSystemUiVisibility` recusa outras threads).
fn set_light_bars(app: &AndroidApp, light: bool) -> Result<bool> {
    // WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS | _NAVIGATION_BARS
    const LIGHT_BARS: i32 = 8 | 16;
    with_activity(app, |env, activity| {
        if sdk_int(env)? < 30 {
            return Ok(false);
        }
        let window = window(env, activity)?;
        let controller = env
            .call_method(
                &window,
                jni_str!("getInsetsController"),
                jni_sig!("()Landroid/view/WindowInsetsController;"),
                &[],
            )?
            .l()?;
        if controller.is_null() {
            return Ok(false);
        }
        let appearance = if light { LIGHT_BARS } else { 0 };
        env.call_method(
            &controller,
            jni_str!("setSystemBarsAppearance"),
            jni_sig!("(II)V"),
            &[JValue::Int(appearance), JValue::Int(LIGHT_BARS)],
        )?;
        Ok(true)
    })
}

/// Acompanha a área segura e a cor dos ícones das barras.
pub struct Tracker {
    app: AndroidApp,
    content_rect: Option<(i32, i32, i32, i32)>,
    size: Option<egui::Vec2>,
    pixels_per_point: f32,
    focused: bool,
    keyboard: bool,
    /// Até quando seguir relendo a cada quadro.
    settle_until: f64,
    next_ime_check: f64,
    insets: Option<SystemInsets>,
    /// Tema aplicado aos ícones das barras (`None`: ainda não aplicado).
    bars_dark: Option<bool>,
    /// Abaixo da API 30 não dá para escurecer os ícones: o app desenha um
    /// véu escuro atrás deles no tema claro.
    pub legacy_bars: bool,
}

impl Tracker {
    pub fn new(app: AndroidApp) -> Self {
        Tracker {
            app,
            content_rect: None,
            size: None,
            pixels_per_point: 0.0,
            focused: false,
            keyboard: false,
            settle_until: 0.0,
            next_ime_check: 0.0,
            insets: None,
            bars_dark: None,
            legacy_bars: false,
        }
    }

    /// Chamado a cada quadro, antes de o egui ler a entrada. Devolve a área
    /// segura em pontos.
    pub fn update(&mut self, ctx: &egui::Context, raw: &egui::RawInput) -> SafeArea {
        let now = raw.time.unwrap_or_else(|| ctx.input(|i| i.time));
        let viewport = raw.viewport();
        let pixels_per_point = viewport.native_pixels_per_point.unwrap_or(1.0) * ctx.zoom_factor();
        let size = raw
            .screen_rect
            .or(viewport.inner_rect)
            .map(|r| r.size())
            .or(self.size);
        let focused = viewport.focused.unwrap_or(self.focused);
        let keyboard = ctx.egui_wants_keyboard_input();
        let r = self.app.content_rect();
        let content_rect = (r.left, r.top, r.right, r.bottom);

        // Sinais de que os recuos podem ter mudado: ContentRectChanged,
        // rotação/redimensionamento, densidade (ConfigChanged), retomada
        // (volta o foco) e o teclado pedido ou dispensado pelo egui.
        let changed = self.content_rect != Some(content_rect)
            || self.size != size
            || self.pixels_per_point != pixels_per_point
            || (focused && !self.focused)
            || keyboard != self.keyboard;
        self.content_rect = Some(content_rect);
        self.size = size;
        self.pixels_per_point = pixels_per_point;
        self.focused = focused;
        self.keyboard = keyboard;
        if changed {
            self.settle_until = now + SETTLE;
        }

        let dark = ctx.theme() == egui::Theme::Dark;
        if self.bars_dark != Some(dark) || (changed && focused) {
            match set_light_bars(&self.app, !dark) {
                Ok(applied) => self.legacy_bars = !applied,
                Err(e) => eprintln!("BlockLab: cor das barras do sistema: {e}"),
            }
            self.bars_dark = Some(dark);
        }

        let ime_open = self.insets.is_some_and(|i| i.ime_bottom > 0);
        let settling = now <= self.settle_until;
        let recheck_ime = ime_open && now >= self.next_ime_check;
        if self.insets.is_none() || settling || recheck_ime {
            match query(&self.app) {
                Ok(Some(i)) => self.insets = Some(i),
                Ok(None) => {}
                Err(e) => eprintln!("BlockLab: recuos do sistema: {e}"),
            }
            self.next_ime_check = now + IME_RECHECK;
        }
        if settling {
            ctx.request_repaint();
        } else if self.insets.is_some_and(|i| i.ime_bottom > 0) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(IME_RECHECK));
        }

        let Some(i) = self.insets else {
            return SafeArea::default();
        };
        let px = PxInsets {
            // O teclado cobre a barra de navegação: vale o maior.
            bottom: i.bars.bottom.max(i.ime_bottom),
            ..i.bars
        };
        SafeArea::from_pixels(px, pixels_per_point)
    }
}
