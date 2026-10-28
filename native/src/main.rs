#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() -> eframe::Result {
    // Same default window as the previous Electron shell; screenshots use a fixed size.
    let screenshot = std::env::var_os("CONTROLAB_SCREENSHOT").is_some();
    let size = match std::env::var("CONTROLAB_SCREENSHOT_SIZE") {
        Ok(size) => size
            .split_once('x')
            .and_then(|(w, h)| Some([w.parse().ok()?, h.parse().ok()?]))
            .unwrap_or([1920.0, 1048.0]),
        Err(_) if screenshot => [1920.0, 1048.0],
        Err(_) => [1360.0, 880.0],
    };
    eframe::run_native(
        "ControLAB - Sistemas de Controle",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size(size)
                .with_min_inner_size(if screenshot {
                    [100.0, 100.0]
                } else {
                    [360.0, 640.0]
                }),
            ..Default::default()
        },
        Box::new(|cc| Ok(controlab_native::app(cc))),
    )
}
