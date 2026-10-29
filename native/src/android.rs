//! Android NativeActivity entry point and in-process Rust scientific engine.

use std::{path::PathBuf, sync::OnceLock};

use android_activity::AndroidApp;
use jni::{
    JavaVM, jni_sig, jni_str,
    objects::{JObject, JString},
    refs::Global,
};

struct Bridge {
    files_dir: PathBuf,
}

static BRIDGE: OnceLock<Bridge> = OnceLock::new();

fn initialize(app: &AndroidApp) -> Result<(), String> {
    if BRIDGE.get().is_some() {
        return Ok(());
    }
    // AndroidApp owns these JNI handles for the lifetime of this entry point.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let files_dir = vm
        .attach_current_thread(|env| -> jni::errors::Result<_> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            // Borrow android-activity's global reference; never delete its handle.
            let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&raw_activity)? };
            let application = env
                .call_method(
                    &activity,
                    jni_str!("getApplication"),
                    jni_sig!("()Landroid/app/Application;"),
                    &[],
                )?
                .l()?;
            let directory = env
                .call_method(
                    &application,
                    jni_str!("getFilesDir"),
                    jni_sig!("()Ljava/io/File;"),
                    &[],
                )?
                .l()?;
            let path = env
                .call_method(
                    &directory,
                    jni_str!("getAbsolutePath"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let path = env.cast_local::<JString>(path)?.try_to_string(env)?;
            Ok(PathBuf::from(path))
        })
        .map_err(|error| format!("Inicialização JNI: {error}"))?;
    let _ = BRIDGE.set(Bridge { files_dir });
    Ok(())
}

/// Persistent private storage, scoped to the app rather than the process environment.
pub fn config_dir() -> Option<PathBuf> {
    BRIDGE.get().map(|bridge| bridge.files_dir.join("config"))
}

/// Default script directory survives cache cleanup and requires no broad storage permission.
pub fn workspace_dir() -> Option<PathBuf> {
    BRIDGE
        .get()
        .map(|bridge| bridge.files_dir.join("ControLAB"))
}

/// Entry called by android-activity's NativeActivity glue, independent of a WebView.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    if let Err(error) = initialize(&app) {
        eprintln!("{error}");
        return;
    }
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native("ControLAB", options, Box::new(|cc| Ok(crate::app(cc))))
    {
        eprintln!("Interface Android: {error}");
    }
}
