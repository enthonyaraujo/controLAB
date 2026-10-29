//! System bar, display cutout and soft keyboard insets of the NativeActivity window, and the
//! contrast of the system bar icons. The window stays edge-to-edge (targetSdk 36): the native
//! surface covers the whole window, and `safe_area` keeps the interface out of these insets.

use std::sync::Mutex;

use android_activity::AndroidApp;
use jni::{Env, JValue, jni_sig, jni_str, objects::JObject, refs::Global};

use crate::safe_area::SafeArea;

/// AndroidApp of the running `android_main`, replaced if Android recreates the Activity.
static APP: Mutex<Option<AndroidApp>> = Mutex::new(None);

pub(super) fn attach(app: &AndroidApp) {
    *APP.lock().unwrap_or_else(|e| e.into_inner()) = Some(app.clone());
}

/// Runs `f` with the current Activity, borrowing android-activity's global reference.
fn with_activity<T>(
    what: &str,
    f: impl FnOnce(&mut Env, &JObject, i32) -> jni::errors::Result<T>,
) -> Option<T> {
    let bridge = super::BRIDGE.get()?;
    let app = APP.lock().ok()?.clone()?;
    let sdk = app.config().sdk_version();
    let raw = app.activity_as_ptr() as jni::sys::jobject;
    bridge
        .vm
        .attach_current_thread(|env| -> jni::errors::Result<T> {
            // Never delete this handle: android-activity owns it.
            let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&raw)? };
            f(env, &activity, sdk)
        })
        .map_err(|error| eprintln!("{what}: {error}"))
        .ok()
}

/// Reads `int top, bottom, left, right` of an `android.graphics.Insets`.
fn edges(env: &mut Env, insets: &JObject) -> jni::errors::Result<[i32; 4]> {
    let mut edges = [0; 4];
    for (value, name) in edges.iter_mut().zip([
        jni_str!("top"),
        jni_str!("bottom"),
        jni_str!("left"),
        jni_str!("right"),
    ]) {
        *value = env.get_field(insets, name, jni_sig!("I"))?.i()?;
    }
    Ok(edges)
}

/// `WindowInsets.Type.<name>()`.
fn inset_type(env: &mut Env, name: &jni::strings::JNIStr) -> jni::errors::Result<i32> {
    env.call_static_method(
        jni_str!("android/view/WindowInsets$Type"),
        name,
        jni_sig!("()I"),
        &[],
    )?
    .i()
}

/// Insets of the window in physical pixels; `None` before the decor view is attached.
pub fn read() -> Option<SafeArea> {
    with_activity("Insets do sistema", |env, activity, sdk| {
        let window = env
            .call_method(
                activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
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
        let ([top, bottom, left, right], ime) = if sdk >= 30 {
            let bars = inset_type(env, jni_str!("systemBars"))?
                | inset_type(env, jni_str!("displayCutout"))?;
            let ime = inset_type(env, jni_str!("ime"))?;
            let get = |env: &mut Env, mask: i32| -> jni::errors::Result<[i32; 4]> {
                let value = env
                    .call_method(
                        &insets,
                        jni_str!("getInsets"),
                        jni_sig!("(I)Landroid/graphics/Insets;"),
                        &[JValue::Int(mask)],
                    )?
                    .l()?;
                edges(env, &value)
            };
            (get(env, bars)?, get(env, ime)?[1])
        } else {
            // Before API 30 the system window insets also include the open keyboard.
            let mut edges = [0; 4];
            for (value, name) in edges.iter_mut().zip([
                jni_str!("getSystemWindowInsetTop"),
                jni_str!("getSystemWindowInsetBottom"),
                jni_str!("getSystemWindowInsetLeft"),
                jni_str!("getSystemWindowInsetRight"),
            ]) {
                *value = env.call_method(&insets, name, jni_sig!("()I"), &[])?.i()?;
            }
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
                    for (value, name) in edges.iter_mut().zip([
                        jni_str!("getSafeInsetTop"),
                        jni_str!("getSafeInsetBottom"),
                        jni_str!("getSafeInsetLeft"),
                        jni_str!("getSafeInsetRight"),
                    ]) {
                        *value = (*value)
                            .max(env.call_method(&cutout, name, jni_sig!("()I"), &[])?.i()?);
                    }
                }
            }
            (edges, 0)
        };
        Ok(Some(SafeArea {
            top: top as f32,
            bottom: bottom as f32,
            left: left as f32,
            right: right as f32,
            ime: ime as f32,
        }))
    })
    .flatten()
}

/// Dark (`true`) or light icons on the status and navigation bars. Window appearance must
/// change on the UI thread, so `NativeApplication.setSystemBarIcons` posts it there.
pub fn set_bar_icons(dark_status: bool, dark_navigation: bool) {
    let Some(bridge) = super::BRIDGE.get() else {
        return;
    };
    with_activity("Ícones das barras do sistema", |env, activity, _| {
        env.call_method(
            &bridge.application,
            jni_str!("setSystemBarIcons"),
            jni_sig!("(Landroid/app/Activity;ZZ)V"),
            &[
                JValue::Object(activity),
                JValue::Bool(dark_status),
                JValue::Bool(dark_navigation),
            ],
        )?;
        Ok(())
    });
}
