//! Ponto de entrada no Android (NativeActivity): o sistema carrega a
//! `libblocklab_gui.so` do APK e chama `android_main`.
//!
//! Sem diálogos nativos de arquivo, os diagramas, as exportações e as
//! preferências ficam na pasta do app (`Android/data/<pacote>/files`,
//! visível pelo cabo USB); "Abrir…" lista os `.json` dessa pasta.
//!
//! A janela vai de borda a borda: a área coberta pelas barras do sistema,
//! pelo recorte da câmera e pelo teclado vem de `android_insets`.

use blocklab_core::format::Mode;
use winit::platform::android::activity::AndroidApp;

use crate::{android_insets, app, files, prefs};

// O Android acha a função pelo nome, por isso `no_mangle` (que o lint de
// código inseguro acusa). O outro ponto liberado é a JNI de
// `android_insets`.
#[allow(unsafe_code)]
#[no_mangle]
fn android_main(android: AndroidApp) {
    if let Some(dir) = android
        .external_data_path()
        .or_else(|| android.internal_data_path())
    {
        files::set_data_dir(dir);
    }
    let tracker = android_insets::Tracker::new(android.clone());
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    let result = eframe::run_native(
        "BlockLab",
        options,
        Box::new(|cc| {
            let mut app = app::App::new(&cc.egui_ctx, app::startup_diagram(), None, Mode::Symbolic);
            if let Some(path) = prefs::default_path() {
                app.load_prefs(&cc.egui_ctx, path);
            }
            app.show_home();
            app.insets = Some(tracker);
            Ok(Box::new(app))
        }),
    );
    if let Err(e) = result {
        eprintln!("BlockLab: {e}");
    }
}
