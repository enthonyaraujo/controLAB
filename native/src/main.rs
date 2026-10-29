#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if let Some(index) = args.iter().position(|a| a == "--export-diagram") {
        let result = (|| -> Result<(), String> {
            let input = args
                .get(index + 1)
                .ok_or("Informe o JSON de entrada e o SVG/PNG de saída")?;
            let output = args.get(index + 2).ok_or("Informe o SVG/PNG de saída")?;
            let diagram = blocklab_gui::files::load(std::path::Path::new(input))?;
            let path = std::path::Path::new(output);
            let bytes = match path.extension().and_then(|e| e.to_str()) {
                Some("svg") => blocklab_gui::export::to_svg(&diagram).into_bytes(),
                Some("png") => blocklab_gui::export::to_png(&diagram, 2.0)?,
                _ => return Err("Use uma saída .svg ou .png".into()),
            };
            blocklab_gui::files::write_atomic(path, &bytes)
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
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
    // The app id matches controlab.desktop (StartupWMClass) so launchers group the window.
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icons/app-icon.png"))
        .expect("embedded application icon");
    eframe::run_native(
        "ControLAB - Sistemas de Controle",
        eframe::NativeOptions {
            // Immediate diagram viewports share this event loop. On Wayland,
            // a synchronized swap can wait for a frame callback of an occluded
            // child and block events for both windows. Let egui schedule frames.
            glow_options: eframe::egui_glow::GlowConfiguration {
                vsync: false,
                ..Default::default()
            },
            viewport: eframe::egui::ViewportBuilder::default()
                .with_app_id("controlab")
                .with_icon(icon)
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
