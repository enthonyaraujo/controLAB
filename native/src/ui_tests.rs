//! Headless rendering checks: run the real forms and studio without starting the scientific service.
use super::*;
use egui::epaint::{Primitive, Shape};

fn fixture(ctx: &egui::Context, light: bool) -> Controlab {
    theme::fonts(ctx);
    theme::apply(ctx, light);
    let schema: Schema = serde_json::from_str(include_str!("../modules.json")).unwrap();
    let prefs = Preferences {
        theme: if light { "light" } else { "dark" }.into(),
        scale: 100,
        folder: "/workspace/ControLAB".into(),
        ..Default::default()
    };
    let mut app = Controlab::with(ctx.clone(), schema, prefs);
    app.workspace = json!([{"name":"A","type":"matrix","dimensions":"2 × 2","value":"[1 2; 3 4]"}]);
    app.files = json!([
        {"name":"exemplos","directory":true,"path":"/workspace/ControLAB/exemplos"},
        {"name":"controle.m","directory":false,"path":"/workspace/ControLAB/controle.m"}
    ]);
    app.history = vec!["/workspace/ControLAB".into()];
    app.script_plots = vec![
        json!({"title":"Resposta ao degrau","xLabel":"t","yLabel":"y","series":[{"name":"y(t)","color":[28,125,118],"points":[[0.0,0.0],[0.5,0.4],[1.0,0.8]]}]}),
    ];
    app
}

fn console_text(app: &Controlab) -> String {
    app.console
        .iter()
        .map(|(text, _)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_labels(shape: &Shape, labels: &mut String) {
    match shape {
        Shape::Text(text) => {
            labels.push_str(text.galley.text());
            labels.push('\n');
        }
        Shape::Vec(shapes) => {
            for shape in shapes {
                collect_labels(shape, labels);
            }
        }
        _ => {}
    }
}

fn assert_shape_geometry(shape: &Shape, page: &str, width: f32) {
    match shape {
        Shape::Noop => return,
        Shape::Vec(shapes) => {
            for shape in shapes {
                assert_shape_geometry(shape, page, width);
            }
            return;
        }
        Shape::Text(text) => {
            assert!(
                text.pos.x.is_finite() && text.pos.y.is_finite() && text.angle.is_finite(),
                "Invalid text placement on {page} at {width}"
            );
            // Empty text has mesh_bounds=Rect::NOTHING. Rotating that sentinel
            // yields 0*infinity=NaN even at angle zero, but emits no geometry.
            if text.galley.num_vertices == 0 {
                return;
            }
        }
        _ => {}
    }
    let bounds = shape.visual_bounding_rect();
    for coordinate in [bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y] {
        assert!(
            !coordinate.is_nan(),
            "NaN painted shape bounds on {page} at {width}"
        );
    }
}

fn render(app: &mut Controlab, ctx: &egui::Context, width: f32) -> String {
    render_sized(app, ctx, width, 1000.0)
}

/// egui paints only what is inside the viewport; tall screens expose long results.
fn render_sized(app: &mut Controlab, ctx: &egui::Context, width: f32, height: f32) -> String {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / ctx.zoom_factor(), height / ctx.zoom_factor()),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| app.frame(ui));
    let shapes = std::mem::take(&mut output.shapes);
    let pixels_per_point = output.pixels_per_point;
    // Clear renderer-owned deltas before any assertion can unwind this frame.
    output.drop_without_applying_deltas();
    assert!(!shapes.is_empty(), "No shapes for {}", app.page);
    let mut labels = String::new();
    for clipped in &shapes {
        assert_shape_geometry(&clipped.shape, &app.page, width);
        for coordinate in [
            clipped.clip_rect.min.x,
            clipped.clip_rect.min.y,
            clipped.clip_rect.max.x,
            clipped.clip_rect.max.y,
        ] {
            assert!(
                !coordinate.is_nan(),
                "NaN clipping rectangle on {} at {width}",
                app.page
            );
        }
        collect_labels(&clipped.shape, &mut labels);
    }
    let primitives = ctx.tessellate(shapes, pixels_per_point);
    assert!(
        !primitives.is_empty(),
        "No painted primitives for {}",
        app.page
    );
    for primitive in primitives {
        if let Primitive::Mesh(mesh) = primitive.primitive {
            for vertex in mesh.vertices {
                assert!(
                    vertex.pos.x.is_finite() && vertex.pos.y.is_finite(),
                    "Non-finite vertex on {} at {width}",
                    app.page
                );
                assert!(
                    vertex.uv.x.is_finite() && vertex.uv.y.is_finite(),
                    "Non-finite texture coordinate on {} at {width}",
                    app.page
                );
            }
        }
    }
    // Desktop shows the INÍCIO and APPS tabs; narrower layouts title the page instead.
    let title = match app.page.as_str() {
        "home" => "Início",
        "apps" => "Apps",
        id => apps::title(id).unwrap(),
    };
    assert!(
        labels.contains("APPS") || labels.contains(title),
        "Missing navigation on {} at {width}",
        app.page
    );
    labels
}

#[test]
fn studio_and_six_modules_render_at_phone_tablet_and_desktop_sizes() {
    for light in [false, true] {
        for width in [390.0, 768.0, 1440.0] {
            let ctx = egui::Context::default();
            let mut app = fixture(&ctx, light);
            render(&mut app, &ctx, width);
            let labels = render(&mut app, &ctx, width);
            assert!(
                labels.contains("Command Window") && labels.contains("Novo por aqui?"),
                "Command Window missing at {width}"
            );
            assert!(
                !labels.lines().any(|l| l == "ControLAB") && !labels.contains("Bem-vindo"),
                "The brand and the welcome page were replaced at {width}"
            );
            assert!(labels.contains("Novo script"));
            if width >= 1024.0 {
                for (_, label) in kit::TABS {
                    assert!(labels.contains(label), "Desktop tab missing: {label}");
                }
            } else {
                assert!(
                    labels.contains("Início") && labels.contains("Apps"),
                    "Mobile navigation missing at {width}"
                );
            }
            app.page = "apps".into();
            render(&mut app, &ctx, width);
            let labels = render(&mut app, &ctx, width);
            for (id, _, title, _) in apps::APPS {
                assert!(labels.contains(title), "App {id} missing at {width}");
            }
            app.page = "home".into();
            app.open_example(0);
            render(&mut app, &ctx, width);
            let labels = render(&mut app, &ctx, width);
            assert!(
                labels.contains("multiplicacao_matrizes.m") && labels.contains("Command Window"),
                "Editor tab or docked Command Window missing at {width}"
            );
            assert!(
                !labels.contains("Novo por aqui?"),
                "Banner over the editor at {width}"
            );
            for id in ORDER {
                app.page = id.into();
                render(&mut app, &ctx, width);
                let labels = render(&mut app, &ctx, width);
                assert!(
                    labels.contains("Calcular"),
                    "Ribbon missing: {id} at {width}"
                );
                if width >= 1024.0 {
                    assert!(
                        labels.contains("Parâmetros") && labels.contains("Resultados"),
                        "Panels missing: {id} at {width}"
                    );
                    assert!(
                        labels.contains(results::figure_title(id)),
                        "Figure title missing: {id}"
                    );
                }
                // Loading, error and placeholder states without I/O.
                let state = app.analyses.get_mut(id).unwrap();
                state.pending = true;
                render(&mut app, &ctx, width);
                let state = app.analyses.get_mut(id).unwrap();
                state.pending = false;
                state.error = "Parâmetro de teste inválido".into();
                render(&mut app, &ctx, width);
                app.analyses.get_mut(id).unwrap().error.clear();
            }
            app.settings = true;
            // Areas spend their first frame sizing themselves invisibly.
            render(&mut app, &ctx, width);
            let labels = render(&mut app, &ctx, width);
            assert!(labels.contains("Configurações & Sobre"));
            app.settings = false;
            assert!(
                app.backend.poll().is_none(),
                "Rendering unexpectedly dispatched work"
            );
        }
    }
}

#[test]
fn controller_payload_preserves_the_specifications_selected_by_the_form() {
    let schema: Schema = serde_json::from_str(include_str!("../modules.json")).unwrap();
    let module = &schema["controllers"];
    let mut values = module.defaults();
    values.insert("mp".into(), "25".into());
    values.insert("wn".into(), "5".into());
    values.insert("primary_damping_spec".into(), "mp".into());
    values.insert("primary_freq_spec".into(), "wn".into());
    let payload = module.payload(&values, false).unwrap();
    assert_eq!(payload["mp"], 25.0);
    assert_eq!(payload["wn"], 5.0);
    assert_eq!(payload["primary_damping_spec"], "mp");
    assert_eq!(payload["primary_freq_spec"], "wn");
    values.insert("controller_type".into(), "lag".into());
    let lag = module.payload(&values, true).unwrap();
    assert!(lag.get("mp").is_none());
    assert!(lag.get("wn").is_none());
    assert_eq!(lag["theme"], "light");
}

#[test]
fn rejected_oversized_requests_release_the_studio_and_analysis_pending_flags() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    let oversized = "x".repeat(8 * 1024 * 1024);
    app.script_pending = true;
    app.send(
        "script",
        json!({"action":"script_request","operation":"execute","source":oversized}),
    );
    assert!(!app.script_pending);
    assert!(app.status.contains("8 MiB"));
    app.analyses.get_mut("time").unwrap().pending = true;
    app.send("time", json!({"action":"time_response","expr":oversized}));
    let analysis = &app.analyses["time"];
    assert!(!analysis.pending);
    assert!(analysis.error.contains("8 MiB"));
    assert!(app.backend.poll().is_none());
}

#[test]
fn backend_transport_recovers_from_engine_errors_and_preserves_the_script_session() {
    use std::time::{Duration, Instant};
    let backend = backend::Backend::new(egui::Context::default());
    let requests = [
        ("preview", json!({"action":"preview","expr":"1/(s+1)"})),
        ("invalid", json!({"action":"not_a_controlab_action"})),
        (
            "assign",
            json!({"action":"script_request","operation":"execute","source":"a=7;"}),
        ),
        (
            "reuse",
            json!({"action":"script_request","operation":"execute","source":"a+2"}),
        ),
        ("recovered", json!({"action":"preview","expr":"2/(s+2)"})),
    ];
    for (tag, payload) in requests {
        backend.submit(tag.into(), payload).unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut replies = BTreeMap::new();
    while replies.len() < 5 {
        if let Some((tag, reply)) = backend.poll() {
            let reply = reply.unwrap_or_else(|error| panic!("Transport failed for {tag}: {error}"));
            assert!(
                replies.insert(tag, reply).is_none(),
                "Duplicate response tag"
            );
        } else {
            assert!(
                Instant::now() < deadline,
                "Timed out waiting for the native scientific transport"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    assert_eq!(replies["preview"]["success"], true);
    assert_eq!(replies["preview"]["den"], json!([1.0, 1.0]));
    assert_eq!(replies["invalid"]["success"], false);
    assert!(
        replies["invalid"]["error"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
    assert_eq!(replies["assign"]["success"], true);
    assert_eq!(replies["reuse"]["success"], true);
    assert!(replies["reuse"]["output"].as_str().unwrap().contains('9'));
    assert_eq!(replies["recovered"]["num"], json!([2.0]));
}

#[test]
fn scientific_png_data_uris_and_stacked_time_plots_load_as_textures() {
    use base64::Engine;
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(2, 2, image::Rgba([20, 30, 40, 255]))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
    let uri = format!("data:image/png;base64,{encoded}");
    let ctx = egui::Context::default();
    let single = results::plots(&ctx, "routh", &json!({"plot_image":encoded}));
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].2.size(), [2, 2]);
    assert_eq!(single[0].1, "Figura 1 — Raízes no plano complexo s");
    // The selected input's figure is also the top-level image; it must not repeat.
    let time = results::plots(
        &ctx,
        "time",
        &json!({"image":uri,"plots":{"step":{"image":uri},"ramp":{"image":uri},"impulse":{"image":encoded}}}),
    );
    let captions: Vec<&str> = time.iter().map(|(_, c, _)| c.as_str()).collect();
    assert_eq!(
        captions,
        [
            "Figura 1 — Resposta ao degrau",
            "Figura 2 — Resposta à rampa",
            "Figura 3 — Resposta ao impulso"
        ]
    );
    assert!(results::texture(&ctx, "invalid", "não é base64").is_none());
}

#[test]
fn console_builtins_are_resolved_by_the_studio_instead_of_the_engine() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    for (command, expected) in [
        ("pwd", "/workspace/ControLAB"),
        ("ls", "exemplos\ncontrole.m"),
        ("ajuda", "run('arquivo.m')"),
    ] {
        app.execute_command(command);
        assert!(
            console_text(&app).contains(expected),
            "{command}: {}",
            console_text(&app)
        );
    }
    app.execute_command("clc");
    assert!(app.console.is_empty());
    assert!(!app.script_pending);
    assert!(app.backend.poll().is_none());
    assert_eq!(cd_target("cd  exemplos "), Some("exemplos"));
    assert_eq!(cd_target("cdx"), None);
    assert_eq!(cd_target("cd"), None);
    assert_eq!(run_target("run('controle.m');"), Some("controle.m"));
    assert_eq!(run_target("run( 'a b.m' ) ;"), Some("a b.m"));
    assert_eq!(run_target("run(controle.m)"), None);
    assert_eq!(run_target("run('')"), None);
}

fn wait(app: &mut Controlab, ctx: &egui::Context, what: &str, done: impl Fn(&Controlab) -> bool) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(120);
    while !done(app) {
        assert!(
            Instant::now() < deadline,
            "Timed out waiting for {what}: {}",
            console_text(app)
        );
        std::thread::sleep(Duration::from_millis(10));
        app.poll(ctx);
    }
}

#[test]
fn saving_a_document_that_owns_its_file_overwrites_without_a_dialog() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    let folder = std::env::temp_dir().join(format!("controlab-save-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("salvo.m");
    std::fs::write(&path, "a=1;\n").unwrap();
    app.documents.push(Document {
        name: "salvo.m".into(),
        path: path.to_string_lossy().into(),
        source: "a=2;\n".into(),
        saved: "a=1;\n".into(),
    });
    app.active = Some(0);
    app.save_document();
    assert!(
        app.pending_save.is_none(),
        "An existing file must not open the save dialog"
    );
    assert!(app.script_pending);
    wait(&mut app, &ctx, "save", |a| !a.script_pending);
    let written = std::fs::read_to_string(&path);
    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(written.unwrap(), "a=2;\n", "status: {}", app.status);
    assert!(!app.documents[0].dirty());
    assert!(app.confirm.is_none());
}

#[test]
fn console_cd_and_run_resolve_paths_against_the_current_folder() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    let folder = std::env::temp_dir().join(format!("controlab-run-{}", std::process::id()));
    std::fs::create_dir_all(folder.join("sub")).unwrap();
    std::fs::write(folder.join("sub/calc.m"), "r=6*7\n").unwrap();
    app.prefs.folder = folder.to_string_lossy().into();
    app.execute_command("cd sub");
    wait(&mut app, &ctx, "cd", |a| a.prefs.folder.ends_with("sub"));
    app.execute_command("run('calc.m');");
    wait(&mut app, &ctx, "run", |a| console_text(a).contains("42"));
    app.execute_command("whos");
    wait(&mut app, &ctx, "whos", |a| {
        console_text(a).contains("r\tdouble")
    });
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(
        !app.console.iter().any(|(_, kind)| *kind == "error"),
        "{}",
        console_text(&app)
    );
}

#[test]
fn structured_results_render_real_engine_replies_and_typeset_their_formulas() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    let expected = [
        ("time", "Razão de amortecimento (ζ)"),
        ("frequency", "Margem de ganho"),
        ("controllers", "PASSO 1"),
        ("state-space", "REALIMENTAÇÃO DE ESTADOS"),
        ("lgr", "Polos em malha aberta"),
        ("routh", "Tabela de Routh:"),
    ];
    for (id, _) in expected {
        app.calculate(id, 1080.0);
    }
    wait(&mut app, &ctx, "the engine", |a| {
        a.analyses.values().all(|s| !s.pending)
    });
    for (id, label) in expected {
        let state = &app.analyses[id];
        assert!(state.error.is_empty(), "{id}: {}", state.error);
        assert!(!state.figures.is_empty(), "{id}: no figure");
        app.page = id.into();
        render_sized(&mut app, &ctx, 1920.0, 8000.0);
        let labels = render_sized(&mut app, &ctx, 1920.0, 8000.0);
        assert!(labels.contains(label), "{id}: missing {label}");
    }
    for (w, h) in [
        (390.0, 844.0),
        (600.0, 960.0),
        (768.0, 1024.0),
        (844.0, 390.0),
    ] {
        for id in ORDER {
            app.page = id.into();
            for tab in [1, 2] {
                app.analyses.get_mut(id).unwrap().mobile_tab = tab;
                for _ in 0..3 {
                    render_sized(&mut app, &ctx, w, h);
                }
            }
        }
    }
    assert_eq!(app.analyses["time"].figures.len(), 3);
    assert!(
        app.prefs.recents.len() >= 6,
        "Calculations are listed as recents"
    );
    app.page = "lgr".into();
    app.analyses.get_mut("lgr").unwrap().view = 1;
    let labels = render_sized(&mut app, &ctx, 1920.0, 8000.0);
    assert!(labels.contains("Passos 1, 2, 3:"));
    assert!(
        app.formulas.busy(),
        "Views did not ask for typeset formulas"
    );
    while app.formulas.busy() {
        if let Some(payload) = app.formulas.next_request() {
            app.send("formulas", payload);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.poll(&ctx);
    }
    assert!(
        app.formulas.typeset() > 20,
        "Only {} formulas typeset",
        app.formulas.typeset()
    );
}

fn touch_context() -> egui::Context {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![
                egui::Event::Touch {
                    device_id: egui::TouchDeviceId(1),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::Start,
                    pos: egui::pos2(-100.0, -100.0),
                    force: None,
                },
                egui::Event::Touch {
                    device_id: egui::TouchDeviceId(1),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::End,
                    pos: egui::pos2(-100.0, -100.0),
                    force: None,
                },
            ],
            ..Default::default()
        },
        |_| {},
    );
    output.textures_delta.clear();
    ctx
}

fn event_frame(
    app: &mut Controlab,
    ctx: &egui::Context,
    w: f32,
    h: f32,
    events: Vec<egui::Event>,
) -> Vec<egui::epaint::ClippedShape> {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / ctx.zoom_factor(), h / ctx.zoom_factor()),
            )),
            events,
            ..Default::default()
        },
        |ui| app.frame(ui),
    );
    let shapes = std::mem::take(&mut output.shapes);
    output.drop_without_applying_deltas();
    shapes
}

fn click_label(app: &mut Controlab, ctx: &egui::Context, w: f32, h: f32, label: &str) {
    for _ in 0..3 {
        event_frame(app, ctx, w, h, vec![]);
    }
    let shapes = event_frame(app, ctx, w, h, vec![]);
    fn locate(shape: &Shape, clip: egui::Rect, label: &str) -> Option<egui::Pos2> {
        match shape {
            Shape::Text(t) if t.galley.text() == label => {
                let p = t.pos + t.galley.size() / 2.0;
                clip.contains(p).then_some(p)
            }
            Shape::Vec(v) => v.iter().find_map(|s| locate(s, clip, label)),
            _ => None,
        }
    }
    let p = shapes
        .iter()
        .rev()
        .find_map(|s| locate(&s.shape, s.clip_rect, label))
        .unwrap_or_else(|| panic!("Visible label not found: {label} at {w}x{h}"));
    for pressed in [true, false] {
        event_frame(
            app,
            ctx,
            w,
            h,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
}

#[test]
fn responsive_breakpoints_match_electron_at_all_supported_scales() {
    for scale in [80, 100, 125] {
        for (w, mode) in [
            (360.0, Mode::Phone),
            (599.0, Mode::Phone),
            (600.0, Mode::Tablet),
            (767.0, Mode::Tablet),
            (768.0, Mode::Tablet),
            (1023.0, Mode::Tablet),
            (1024.0, Mode::Desktop),
            (1280.0, Mode::Desktop),
        ] {
            let ctx = touch_context();
            let mut app = fixture(&ctx, false);
            app.prefs.scale = scale;
            app.page = "time".into();
            for _ in 0..3 {
                render_sized(&mut app, &ctx, w, 800.0);
            }
            assert_eq!(app.layout.mode, mode, "{w} at scale {scale}");
            assert!(app.layout.touch);
            let labels = render_sized(&mut app, &ctx, w, 800.0);
            if mode == Mode::Phone {
                assert!(labels.contains("Gráfico") && labels.contains("Parâmetros"));
                assert!(labels.contains("^"));
            }
            if mode == Mode::Tablet {
                assert!(labels.contains("Figura") && labels.contains("Resultados"));
            }
        }
    }
}

#[test]
fn desktop_apps_tab_opens_the_gallery_and_stays_active_inside_an_app() {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    click_label(&mut app, &ctx, 1440.0, 900.0, "APPS");
    assert_eq!(app.page, "apps");
    click_label(&mut app, &ctx, 1440.0, 900.0, "Controladores");
    assert_eq!(app.page, "controllers");
    assert_eq!(kit::section(&app.page), "apps");
    click_label(&mut app, &ctx, 1440.0, 900.0, "APPS");
    assert_eq!(app.page, "apps");
    click_label(&mut app, &ctx, 1440.0, 900.0, "INÍCIO");
    assert_eq!(app.page, "home");
    // Closing the examples notice keeps the Command Window in the centre.
    let labels = render_sized(&mut app, &ctx, 1440.0, 900.0);
    assert!(labels.contains("Novo por aqui?"));
    app.examples_banner = false;
    let labels = render_sized(&mut app, &ctx, 1440.0, 900.0);
    assert!(labels.contains("Command Window") && !labels.contains("Novo por aqui?"));
}

#[test]
fn command_window_prompt_follows_the_last_output_line() {
    fn top_of(shapes: &[egui::epaint::ClippedShape], label: &str) -> f32 {
        fn find(shape: &Shape, label: &str) -> Option<f32> {
            match shape {
                Shape::Text(t) if t.galley.text() == label => Some(t.pos.y),
                Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
                _ => None,
            }
        }
        shapes
            .iter()
            .find_map(|s| find(&s.shape, label))
            .unwrap_or_else(|| panic!("{label} not painted"))
    }
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx, false);
    app.console = vec![
        ("Pronto.".into(), "output"),
        (">> disp(1)".into(), "cmd"),
        ("1".into(), "output"),
    ];
    for _ in 0..3 {
        event_frame(&mut app, &ctx, 1440.0, 900.0, vec![]);
    }
    let shapes = event_frame(&mut app, &ctx, 1440.0, 900.0, vec![]);
    let (last, prompt) = (top_of(&shapes, "1"), top_of(&shapes, ">>"));
    assert!(
        prompt > last && prompt - last < 40.0,
        "prompt at {prompt}, last output at {last}"
    );
}

#[test]
fn phone_navigation_tabs_and_studio_drawers_are_operable() {
    let ctx = touch_context();
    let mut app = fixture(&ctx, false);
    click_label(&mut app, &ctx, 390.0, 844.0, "Arquivos");
    assert!(app.show_files);
    let labels = render_sized(&mut app, &ctx, 390.0, 844.0);
    assert!(labels.contains("controle.m"));
    click_label(&mut app, &ctx, 390.0, 844.0, "Fechar");
    assert!(!app.show_files);
    click_label(&mut app, &ctx, 390.0, 844.0, "Variáveis");
    assert!(app.show_vars);
    for _ in 0..3 {
        render_sized(&mut app, &ctx, 390.0, 844.0);
    }
    let labels = render_sized(&mut app, &ctx, 390.0, 844.0);
    assert!(labels.contains("Variáveis"));
    click_label(&mut app, &ctx, 390.0, 844.0, "Fechar");
    assert!(!app.show_vars);
    click_label(&mut app, &ctx, 390.0, 844.0, "Apps");
    assert_eq!(app.page, "apps");
    click_label(&mut app, &ctx, 390.0, 844.0, "Resposta no tempo");
    assert_eq!(app.page, "time");
    click_label(&mut app, &ctx, 390.0, 844.0, "Gráfico");
    assert_eq!(app.analyses["time"].mobile_tab, 1);
    click_label(&mut app, &ctx, 390.0, 844.0, "Resultados");
    assert_eq!(app.analyses["time"].mobile_tab, 2);
    click_label(&mut app, &ctx, 390.0, 844.0, "Parâmetros");
    assert_eq!(app.analyses["time"].mobile_tab, 0);
    let before = app.analyses["time"].values["expr"].clone();
    click_label(&mut app, &ctx, 390.0, 844.0, "^");
    assert_eq!(app.analyses["time"].values["expr"], format!("{before}^"));
    let labels = render_sized(&mut app, &ctx, 390.0, 450.0);
    assert!(app.layout.keyboard_open);
    assert!(!labels.contains("Calcular"));
    ctx.memory_mut(|m| m.surrender_focus(egui::Id::new(("expression", "time"))));
    let labels = render_sized(&mut app, &ctx, 390.0, 844.0);
    assert!(labels.contains("Calcular"));
}

#[test]
fn tablet_results_remain_accessible_when_lgr_memorial_is_selected() {
    let ctx = touch_context();
    let mut app = fixture(&ctx, true);
    app.page = "lgr".into();
    app.analyses.get_mut("lgr").unwrap().view = 1;
    for _ in 0..3 {
        render_sized(&mut app, &ctx, 768.0, 1024.0);
    }
    click_label(&mut app, &ctx, 768.0, 1024.0, "Resultados");
    assert_eq!(app.analyses["lgr"].mobile_tab, 2);
    let labels = render_sized(&mut app, &ctx, 768.0, 1024.0);
    assert!(labels.contains("Resultados"));
    assert!(!labels.contains("Passo a passo (7 Passos)"));
}

#[test]
fn virtual_math_keyboard_replaces_unicode_selection_at_the_caret() {
    let ctx = egui::Context::default();
    let id = egui::Id::new("formula-selection");
    let mut state = egui::text_edit::TextEditState::default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(1),
            egui::text::CCursor::new(3),
        )));
    state.store(&ctx, id);
    let mut value = "αβγ+s".to_owned();
    kit::insert_at_cursor(&ctx, id, &mut value, "^");
    assert_eq!(value, "α^+s");
    assert_eq!(
        egui::text_edit::TextEditState::load(&ctx, id)
            .unwrap()
            .cursor
            .char_range()
            .unwrap()
            .primary
            .index
            .0,
        2
    );
}

#[test]
fn compact_landscape_and_long_results_keep_finite_geometry() {
    for light in [false, true] {
        for (w, h) in [
            (360.0, 640.0),
            (390.0, 844.0),
            (600.0, 960.0),
            (768.0, 1024.0),
            (844.0, 390.0),
            (1024.0, 600.0),
        ] {
            let ctx = touch_context();
            let mut app = fixture(&ctx, light);
            for id in ORDER {
                app.page = id.into();
                for tab in 0..3 {
                    app.analyses.get_mut(id).unwrap().mobile_tab = tab;
                    for _ in 0..3 {
                        render_sized(&mut app, &ctx, w, h);
                    }
                }
            }
            app.page = "home".into();
            app.open_example(0);
            for _ in 0..3 {
                render_sized(&mut app, &ctx, w, h);
            }
        }
    }
}

#[test]
fn android_safe_areas_are_respected_without_changing_css_breakpoints() {
    let ctx = touch_context();
    let mut app = fixture(&ctx, false);
    app.page = "time".into();
    for _ in 0..4 {
        let z = ctx.zoom_factor();
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0 / z, 960.0 / z),
                )),
                safe_area_insets: Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
                    top: 24.0 / z,
                    bottom: 24.0 / z,
                    left: 30.0 / z,
                    right: 30.0 / z,
                })),
                ..Default::default()
            },
            |ui| app.frame(ui),
        );
        output.drop_without_applying_deltas();
    }
    assert_eq!(app.layout.mode, Mode::Tablet);
    let shell = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("shell"))
        .expect("shell panel");
    let safe = ctx.content_rect();
    assert!(shell.outer_rect.top() >= safe.top() - 0.1);
    assert!(shell.outer_rect.right() <= safe.right() + 0.1);
}

#[test]
fn only_backgrounds_extend_under_the_system_bars_and_the_keyboard() {
    let ctx = touch_context();
    let mut app = fixture(&ctx, true);
    let p = theme::palette(true);
    let (nav, _) = kit::bottom_nav_colors(&p);
    // Phone portrait, then with the keyboard raising the bottom edge.
    for (page, bottom) in [("home", 48.0), ("time", 48.0), ("time", 320.0)] {
        app.page = page.into();
        let mut shapes = vec![];
        for _ in 0..4 {
            let z = ctx.zoom_factor();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(390.0 / z, 844.0 / z),
                    )),
                    safe_area_insets: Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
                        top: 32.0 / z,
                        bottom: bottom / z,
                        left: 0.0,
                        right: 0.0,
                    })),
                    ..Default::default()
                },
                |ui| app.frame(ui),
            );
            shapes = std::mem::take(&mut output.shapes);
            output.drop_without_applying_deltas();
        }
        let safe = ctx.content_rect();
        let viewport = ctx.viewport_rect();
        let filled = |color: egui::Color32, rect: egui::Rect| {
            shapes.iter().any(|s| match &s.shape {
                Shape::Rect(r) => r.fill == color && r.rect.contains_rect(rect.shrink(0.5)),
                _ => false,
            })
        };
        let top = egui::Rect::from_min_max(viewport.min, egui::pos2(viewport.right(), safe.top()));
        let under =
            egui::Rect::from_min_max(egui::pos2(viewport.left(), safe.bottom()), viewport.max);
        assert!(filled(p.shell, top), "status bar background on {page}");
        assert!(filled(nav, under), "navigation bar background on {page}");
        for s in &shapes {
            let outside =
                s.clip_rect.top() < safe.top() - 0.5 || s.clip_rect.bottom() > safe.bottom() + 0.5;
            if let Shape::Text(t) = &s.shape {
                assert!(
                    !outside,
                    "text {:?} clipped under a system bar",
                    t.galley.text()
                );
            }
            if let Shape::Rect(r) = &s.shape
                && outside
            {
                assert!(
                    r.fill == p.shell || r.fill == nav,
                    "{page}: {r:?} under a system bar"
                );
            }
        }
    }
}

#[test]
fn landscape_top_navigation_leaves_room_for_settings_at_1024_pixels() {
    for scale in [80, 100, 125] {
        let ctx = touch_context();
        let mut app = fixture(&ctx, false);
        app.prefs.scale = scale;
        app.page = "time".into();
        for _ in 0..4 {
            event_frame(&mut app, &ctx, 1024.0, 768.0, vec![]);
        }
        let shapes = event_frame(&mut app, &ctx, 1024.0, 768.0, vec![]);
        let label = shapes
            .iter()
            .find_map(|s| {
                if let Shape::Text(t) = &s.shape {
                    (t.galley.text() == "APPS").then_some(t)
                } else {
                    None
                }
            })
            .expect("all navigation labels must remain visible");
        assert!(
            label.pos.x + label.galley.size().x <= ctx.content_rect().right() - 42.0,
            "last tab overlaps settings at scale {scale}"
        );
    }
}

#[test]
fn a_two_finger_pinch_zooms_the_phone_figure() {
    let ctx = touch_context();
    let mut app = fixture(&ctx, false);
    app.page = "lgr".into();
    let tex = ctx.load_texture(
        "test-plot",
        egui::ColorImage::filled([10, 10], egui::Color32::WHITE),
        Default::default(),
    );
    let a = app.analyses.get_mut("lgr").unwrap();
    a.mobile_tab = 1;
    a.figures = vec![("plot".into(), "Figure".into(), tex)];
    for _ in 0..4 {
        event_frame(&mut app, &ctx, 390.0, 844.0, vec![]);
    }
    event_frame(
        &mut app,
        &ctx,
        390.0,
        844.0,
        vec![egui::Event::PointerMoved(egui::pos2(190.0, 350.0))],
    );
    for (phase, left, right) in [
        (egui::TouchPhase::Start, 160.0, 220.0),
        (egui::TouchPhase::Move, 140.0, 240.0),
    ] {
        let events = [(10, left), (11, right)]
            .into_iter()
            .map(|(id, x)| egui::Event::Touch {
                device_id: egui::TouchDeviceId(2),
                id: egui::TouchId(id),
                phase,
                pos: egui::pos2(x, 350.0),
                force: None,
            })
            .collect();
        event_frame(&mut app, &ctx, 390.0, 844.0, events);
    }
    assert!(
        app.analyses["lgr"].zoom > 1.1,
        "pinch must enlarge the plot"
    );
}

#[test]
fn settings_and_confirmations_fit_phone_and_short_landscape_viewports() {
    for (w, h) in [(360.0, 640.0), (390.0, 844.0), (844.0, 390.0)] {
        let ctx = touch_context();
        let mut app = fixture(&ctx, false);
        app.settings = true;
        for _ in 0..5 {
            render_sized(&mut app, &ctx, w, h);
        }
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("settings")))
            .unwrap();
        let safe = ctx.content_rect();
        assert!(
            rect.width() <= safe.width() + 1.0,
            "settings overflow horizontally at {w}: {rect:?}"
        );
        assert!(
            rect.height() <= safe.height() + 1.0,
            "settings overflow vertically at {w}: {rect:?}"
        );
        app.settings = false;
        app.confirm = Some((
            "clear-workspace".into(),
            "Confirmar limpeza das variáveis".into(),
        ));
        for _ in 0..4 {
            render_sized(&mut app, &ctx, w, h);
        }
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("confirm")))
            .expect("confirmation must be rendered");
        assert!(rect.width() <= safe.width() + 1.0);
    }
}
