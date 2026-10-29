// No Windows, o executável de release é um programa de janela: sem isso,
// abrir o BlockLab com duplo clique também abriria um console preto.
// (Em debug o console fica, para ver mensagens e erros.)
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! BlockLab: monte diagramas de blocos e obtenha Y(s)/X(s).
//!
//! ```text
//! blocklab                                   # abre a interface
//! blocklab diagrama.json                     # abre um arquivo
//! blocklab diagrama.json --export fig.png    # exporta PNG/SVG sem abrir a janela
//! blocklab --screenshot tela.png [--example N] [--numeric] [--theme claro|escuro]
//!          [--size 390x844] [--safe-area 48,24,0,0]
//!                                            # captura a janela e sai
//! ```

use std::path::PathBuf;

use blocklab_core::diagram::examples;
use blocklab_core::format::Mode;
use blocklab_gui::safe_area::SafeArea;
use blocklab_gui::{app, export, file_actions, files, icon, prefs, theme};
use eframe::egui;

struct Args {
    file: Option<PathBuf>,
    export: Option<PathBuf>,
    screenshot: Option<PathBuf>,
    example: Option<usize>,
    numeric: bool,
    theme: Option<egui::ThemePreference>,
    /// Tamanho inicial da janela (ex.: 390x844 para ver o arranjo de celular).
    size: Option<[f32; 2]>,
    /// Recuos simulados (topo, base, esquerda, direita, em pontos), para
    /// ver no computador como fica com as barras de um celular.
    safe_area: Option<SafeArea>,
}

/// "48,24,0,0" → recuos de topo, base, esquerda e direita.
fn parse_safe_area(s: &str) -> Option<SafeArea> {
    let v: Vec<f32> = s
        .split(',')
        .map(|p| p.trim().parse().ok().filter(|x: &f32| *x >= 0.0))
        .collect::<Option<_>>()?;
    let [top, bottom, left, right] = v[..] else {
        return None;
    };
    Some(SafeArea {
        top,
        bottom,
        left,
        right,
    })
}

/// "390x844" → [390, 844].
fn parse_size(s: &str) -> Option<[f32; 2]> {
    let (w, h) = s.split_once(['x', 'X'])?;
    let (w, h) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    (w >= 1.0 && h >= 1.0).then_some([w, h])
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        file: None,
        export: None,
        screenshot: None,
        example: None,
        numeric: false,
        theme: None,
        size: None,
        safe_area: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--screenshot" => {
                args.screenshot = Some(
                    it.next()
                        .ok_or("--screenshot precisa de um arquivo")?
                        .into(),
                )
            }
            "--example" => {
                let n = it.next().ok_or("--example precisa de um número")?;
                args.example = Some(n.parse().map_err(|_| format!("número inválido: {n}"))?);
            }
            "--numeric" => args.numeric = true,
            "--theme" => {
                let t = it
                    .next()
                    .ok_or("--theme precisa de claro, escuro ou sistema")?;
                args.theme =
                    Some(theme::parse_preference(&t).ok_or(format!("tema desconhecido: {t}"))?);
            }
            "--safe-area" => {
                let s = it
                    .next()
                    .ok_or("--safe-area precisa de TOPO,BASE,ESQ,DIR")?;
                args.safe_area = Some(parse_safe_area(&s).ok_or(format!("recuos inválidos: {s}"))?);
            }
            "--size" => {
                let s = it.next().ok_or("--size precisa de LARGURAxALTURA")?;
                args.size = Some(parse_size(&s).ok_or(format!("tamanho inválido: {s}"))?);
            }
            "--export" => {
                args.export = Some(
                    it.next()
                        .ok_or("--export precisa de um arquivo .png ou .svg")?
                        .into(),
                )
            }
            "-h" | "--help" => {
                println!(
                    "uso: blocklab [diagrama.json] [--export fig.png|fig.svg] \
                     [--screenshot tela.png] [--example N] [--numeric] \
                     [--theme claro|escuro|sistema] [--size 390x844] \
                     [--safe-area TOPO,BASE,ESQ,DIR]"
                );
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                return Err(format!("argumento desconhecido: {other}"))
            }
            other => args.file = Some(other.into()),
        }
    }
    Ok(args)
}

fn main() -> eframe::Result {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let start_home = args.file.is_none() && args.example.is_none();
    let diagram = match args.example {
        Some(n) => match examples::all().into_iter().nth(n.wrapping_sub(1)) {
            Some((_, d)) => d,
            None => {
                eprintln!("não há exemplo {n}");
                std::process::exit(2);
            }
        },
        None => app::startup_diagram(),
    };
    if let Some(out) = &args.export {
        // Exportação em lote: sem janela.
        let d = match &args.file {
            Some(f) => files::load(f).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1)
            }),
            None => diagram,
        };
        let ext = out
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let bytes = match ext.as_str() {
            "svg" => Ok(export::to_svg(&d).into_bytes()),
            "png" => export::to_png(&d, 2.0),
            _ => Err("a extensão precisa ser .png ou .svg".to_string()),
        };
        match bytes.and_then(|b| files::write_atomic(out, &b)) {
            Ok(()) => {
                println!("exportado: {}", out.display());
                std::process::exit(0)
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1)
            }
        }
    }
    let mode = if args.numeric {
        Mode::Numeric
    } else {
        Mode::Symbolic
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BlockLab")
            .with_inner_size(args.size.unwrap_or([1440.0, 900.0]))
            // Pequena o bastante para o arranjo de celular.
            .with_min_inner_size([320.0, 480.0])
            .with_icon(icon::icon(256)),
        ..Default::default()
    };
    eframe::run_native(
        "BlockLab",
        options,
        Box::new(move |cc| {
            let screenshot = args.screenshot.is_some();
            let mut app = app::App::new(&cc.egui_ctx, diagram, args.screenshot, mode);
            // Capturas automáticas não leem nem gravam as preferências.
            if !screenshot {
                if let Some(path) = prefs::default_path() {
                    app.load_prefs(&cc.egui_ctx, path);
                }
            }
            app.simulated_safe_area = args.safe_area;
            if let Some(t) = args.theme {
                cc.egui_ctx.set_theme(t);
                app.prefs.theme = t;
            }
            if start_home {
                app.show_home();
            }
            if let Some(path) = args.file {
                app.perform(&cc.egui_ctx, file_actions::Pending::Open(Some(path)));
            }
            Ok(Box::new(app))
        }),
    )
}
