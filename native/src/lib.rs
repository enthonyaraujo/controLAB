//! Native ControLAB UI. Scientific calculations remain in the canonical Python engine;
//! the interface preserves ControLAB’s visual identity using native egui widgets.
#[cfg(target_os = "android")]
pub mod android;
mod backend;
mod icons;
mod kit;
mod model;
mod overlays;
mod page;
mod results;
mod scripts;
mod studio;
mod texfmt;
mod theme;
use eframe::egui;
use model::{Module, Schema};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const ORDER: [&str; 6] = [
    "time",
    "routh",
    "lgr",
    "frequency",
    "controllers",
    "state-space",
];

/// Root font size (px) of the previous renderer at the 1920 px captures the metrics come from.
pub const MEASURED_REM: f32 = 16.68;

/// Breakpoints of `styles.css`: phone below 600 px, tablet below 1024 px.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Desktop,
    Tablet,
    Phone,
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub mode: Mode,
    /// `(pointer: coarse)`: Android, or `CONTROLAB_TOUCH` for captures.
    pub touch: bool,
    pub css_width: f32,
    pub keyboard_open: bool,
    /// `html { font-size: clamp(15px, 0.4vw + 9px, 18px) * scale }` relative to MEASURED_REM.
    pub rem: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            mode: Mode::Desktop,
            touch: false,
            rem: 1.0,
            css_width: 1920.0,
            keyboard_open: false,
        }
    }
}

impl Layout {
    fn new(css_width: f32, scale: f32, touch: bool) -> Self {
        let mode = if css_width < 600.0 {
            Mode::Phone
        } else if css_width < 1024.0 {
            Mode::Tablet
        } else {
            Mode::Desktop
        };
        let rem = (0.004 * css_width + 9.0).clamp(15.0, 18.0) * scale / MEASURED_REM;
        Self {
            mode,
            touch,
            rem,
            css_width,
            keyboard_open: false,
        }
    }
}

const TOAST_SECONDS: f64 = 2.8;
const MAX_SCRIPT_BYTES: usize = 1024 * 1024;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Recent {
    name: String,
    #[serde(default)]
    expr: String,
    #[serde(default)]
    nav: String,
    #[serde(default)]
    file: String,
    #[serde(default)]
    kind: String,
    timestamp: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
struct Preferences {
    /// Legacy flag from the first native preferences file.
    light: bool,
    /// `light`, `dark` or `system`, as `appearance.js`.
    theme: String,
    scale: u32,
    folder: String,
    recents: Vec<Recent>,
}

#[derive(Default)]
struct Document {
    name: String,
    path: String,
    source: String,
    saved: String,
}
impl Document {
    fn dirty(&self) -> bool {
        self.source != self.saved
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewState {
    Waiting,
    Validating,
    Valid,
    Invalid,
}

pub struct Preview {
    tex: String,
    state: PreviewState,
    error: String,
    /// Expression the last successful preview validated.
    valid_for: String,
    due: Option<f64>,
    in_flight: Option<String>,
}

struct Analysis {
    values: BTreeMap<String, String>,
    result: Value,
    figures: Vec<(String, String, egui::TextureHandle)>,
    pending: bool,
    error: String,
    preview: Preview,
    zoom: f32,
    pan: egui::Vec2,
    response: String,
    /// LGR figure tabs: 0 gráfico, 1 passo a passo, 2 regras.
    view: usize,
    deductions: bool,
    deduction_filter: usize,
    mobile_tab: usize,
}
impl Analysis {
    fn new(module: &Module) -> Self {
        Self {
            values: module.defaults(),
            result: Value::Null,
            figures: vec![],
            pending: false,
            error: String::new(),
            preview: Preview {
                tex: String::new(),
                state: PreviewState::Waiting,
                error: String::new(),
                valid_for: String::new(),
                due: Some(0.0),
                in_flight: None,
            },
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            response: "step".into(),
            view: 0,
            deductions: false,
            deduction_filter: 0,
            mobile_tab: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ToastKind {
    Success,
    Error,
    Info,
}

/// Result of an OS file dialog running on a helper thread.
enum Picked {
    Open(std::path::PathBuf),
    Save(std::path::PathBuf),
    Folder(std::path::PathBuf),
    Export(std::path::PathBuf, Vec<u8>, &'static str),
}

pub struct Controlab {
    backend: backend::Backend,
    formulas: results::Formulas,
    schema: Schema,
    analyses: BTreeMap<String, Analysis>,
    prefs: Preferences,
    light: bool,
    page: String,
    documents: Vec<Document>,
    active: Option<usize>,
    console: Vec<(String, &'static str)>,
    command: String,
    command_history: Vec<String>,
    history_index: usize,
    workspace: Value,
    files: Value,
    file_search: String,
    folder_input: String,
    history: Vec<String>,
    history_cursor: usize,
    parent: Option<String>,
    script_pending: bool,
    status: String,
    script_plots: Vec<Value>,
    show_figures: bool,
    hide_vars: bool,
    focus_layout: bool,
    settings: bool,
    toast: Option<(String, ToastKind, f64)>,
    picked: Arc<Mutex<Option<Picked>>>,
    pending_save: Option<usize>,
    pending_close: Option<usize>,
    plot_width: f32,
    layout: Layout,
    /// Phone Studio drawers (`studio-show-files` / `studio-show-vars`).
    show_files: bool,
    show_vars: bool,
    started: f64,
    update: Option<overlays::UpdateState>,
    confirm: Option<(String, String)>,
    screenshot_requested: bool,
    quit_confirm: bool,
    allow_close: bool,
    now: f64,
}

pub fn app(cc: &eframe::CreationContext<'_>) -> Box<dyn eframe::App> {
    Box::new(Controlab::new(cc))
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

impl Controlab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::fonts(&cc.egui_ctx);
        #[allow(unused_mut)]
        let mut prefs: Preferences = preferences_path()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        if prefs.theme.is_empty() {
            prefs.theme = if prefs.light {
                "light".into()
            } else {
                "system".into()
            };
        }
        if prefs.scale == 0 {
            prefs.scale = 100;
        }
        #[cfg(target_os = "android")]
        if prefs.folder.is_empty()
            && let Some(p) = android::workspace_dir()
        {
            let _ = std::fs::create_dir_all(&p);
            prefs.folder = p.to_string_lossy().into();
        }
        let schema: Schema = serde_json::from_str(include_str!("../modules.json"))
            .expect("validated module definitions");
        let mut app = Self::with(cc.egui_ctx.clone(), schema, prefs);
        app.apply_theme(&cc.egui_ctx);
        app.send(
            "folder",
            json!({"action":"script_request","operation":"list","path":app.prefs.folder}),
        );
        app.send(
            "state",
            json!({"action":"script_request","operation":"state"}),
        );
        #[cfg(not(target_os = "android"))]
        if std::env::var_os("CONTROLAB_SCREENSHOT").is_some() {
            if let Ok(theme) = std::env::var("CONTROLAB_SCREENSHOT_THEME") {
                app.prefs.theme = theme;
                app.apply_theme(&cc.egui_ctx);
            }
            if let Ok(view) = std::env::var("CONTROLAB_SCREENSHOT_LGR_VIEW") {
                let lgr = app.analyses.get_mut("lgr").unwrap();
                lgr.view = view.parse().unwrap_or(0);
                lgr.deductions = std::env::var_os("CONTROLAB_SCREENSHOT_DEDUCTIONS").is_some();
            }
            if let Ok(drawer) = std::env::var("CONTROLAB_SCREENSHOT_DRAWER") {
                app.show_files = drawer == "files";
                app.show_vars = drawer == "vars";
            }
            app.settings = std::env::var_os("CONTROLAB_SCREENSHOT_SETTINGS").is_some();
            if let Ok(source) = std::env::var("CONTROLAB_SCREENSHOT_SCRIPT") {
                app.new_document();
                let i = app.active.unwrap();
                app.documents[i].source = source;
                app.run_current();
            }
            if let Ok(page) = std::env::var("CONTROLAB_SCREENSHOT_PAGE")
                && app.schema.contains_key(&page)
            {
                app.page = page.clone();
                if std::env::var_os("CONTROLAB_SCREENSHOT_EMPTY").is_none() {
                    app.calculate(&page, 1080.0);
                }
            }
            if let Ok(tab) = std::env::var("CONTROLAB_SCREENSHOT_TAB") {
                for a in app.analyses.values_mut() {
                    a.mobile_tab = tab.parse::<usize>().unwrap_or(0).min(2);
                }
            }
        }
        app
    }

    /// State with no I/O started; `new` and the tests build on it.
    fn with(ctx: egui::Context, schema: Schema, prefs: Preferences) -> Self {
        let analyses = schema
            .iter()
            .map(|(id, m)| (id.clone(), Analysis::new(m)))
            .collect();
        Self {
            backend: backend::Backend::new(ctx.clone()),
            formulas: results::Formulas::new(ctx),
            schema,
            analyses,
            light: prefs.theme == "light",
            prefs,
            page: "home".into(),
            documents: vec![],
            active: None,
            console: vec![(
                "Pronto. Crie ou abra um script, ou digite um comando no console.".into(),
                "info",
            )],
            command: String::new(),
            command_history: vec![],
            history_index: 0,
            workspace: json!([]),
            files: json!([]),
            file_search: String::new(),
            folder_input: String::new(),
            history: vec![],
            history_cursor: 0,
            parent: None,
            script_pending: false,
            status: "Pronto".into(),
            script_plots: vec![],
            show_figures: false,
            hide_vars: false,
            focus_layout: false,
            settings: false,
            toast: None,
            picked: Arc::new(Mutex::new(None)),
            pending_save: None,
            pending_close: None,
            plot_width: 1080.0,
            layout: Layout::default(),
            show_files: false,
            show_vars: false,
            started: 0.0,
            update: None,
            confirm: None,
            screenshot_requested: false,
            quit_confirm: false,
            allow_close: false,
            now: 0.0,
        }
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        self.light = match self.prefs.theme.as_str() {
            "light" => true,
            "dark" => false,
            _ => ctx.system_theme().is_none_or(|t| t == egui::Theme::Light),
        };
        theme::apply(ctx, self.light);
    }

    fn palette(&self) -> theme::Palette {
        theme::palette(self.light)
    }

    fn toast(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.toast = Some((text.into(), kind, self.now + TOAST_SECONDS));
    }

    fn send(&mut self, tag: &str, payload: Value) {
        if let Err(error) = self.backend.submit(tag.into(), payload) {
            if tag == "formulas" {
                // Formulas keep their readable text; this is not a calculation failure.
                self.formulas.receive(&Value::Null);
                return;
            }
            if let Some(a) = self.analyses.get_mut(tag) {
                a.pending = false;
                a.error = error.clone();
            }
            if tag == "script" || tag == "clear" || tag.starts_with("save:") {
                self.script_pending = false;
            }
            self.status = error;
        }
    }

    fn add_recent(&mut self, recent: Recent) {
        self.prefs
            .recents
            .retain(|r| !(r.name == recent.name && r.nav == recent.nav && r.file == recent.file));
        self.prefs.recents.insert(0, recent);
        self.prefs.recents.truncate(30);
    }

    fn expression_field(&self, id: &str) -> &'static str {
        if self.schema[id]
            .fields
            .iter()
            .any(|f| f.name == "plant_expr")
        {
            "plant_expr"
        } else {
            "expr"
        }
    }

    /// Sends the calculation of module `id` with the current form, as `run()` did.
    fn calculate(&mut self, id: &str, plot_width: f32) {
        let module = &self.schema[id];
        let a = &self.analyses[id];
        if a.pending {
            return;
        }
        match module.payload(&a.values, self.light) {
            Ok(mut payload) => {
                payload["response_type"] = json!(a.response);
                payload["language"] = json!("pt-BR");
                if plot_width > 100.0 {
                    payload["width"] = json!(plot_width.round());
                }
                payload["dpi"] = json!(100);
                let a = self.analyses.get_mut(id).unwrap();
                a.pending = true;
                a.error.clear();
                a.mobile_tab = 1;
                self.send(id, payload);
            }
            Err(e) => {
                self.analyses.get_mut(id).unwrap().error = e.clone();
                self.toast(format!("Erro: {e}"), ToastKind::Error);
            }
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Some((tag, reply)) = self.backend.poll() {
            let reply = reply.unwrap_or_else(|error| json!({"success":false,"error":error}));
            if tag == "formulas" {
                self.formulas.receive(&reply);
                continue;
            }
            let success = reply["success"].as_bool().unwrap_or(false);
            let error = reply["error"]
                .as_str()
                .or(reply["diagnostic"]["message"].as_str())
                .unwrap_or("Falha na operação")
                .to_owned();
            if let Some(id) = tag.strip_prefix("preview:") {
                let field = self.expression_field(id);
                let Some(a) = self.analyses.get_mut(id) else {
                    continue;
                };
                let sent = a.preview.in_flight.take().unwrap_or_default();
                if a.values.get(field).map(|v| v.trim()) != Some(sent.trim()) {
                    continue; // Superseded by a newer edit.
                }
                if success {
                    a.preview.tex = reply["latex_fac"]
                        .as_str()
                        .or(reply["latex_exp"].as_str())
                        .unwrap_or_default()
                        .to_owned();
                    a.preview.state = PreviewState::Valid;
                    a.preview.valid_for = sent;
                    a.preview.error.clear();
                } else {
                    a.preview.state = PreviewState::Invalid;
                    a.preview.error = error
                        .split_once(": ")
                        .map_or(error.clone(), |(_, m)| m.to_owned());
                }
                continue;
            }
            if self.analyses.contains_key(&tag) {
                self.finish_calculation(ctx, &tag, reply, success, error);
                continue;
            }
            self.finish_script(&tag, reply, success, error);
        }
    }

    fn finish_calculation(
        &mut self,
        ctx: &egui::Context,
        tag: &str,
        reply: Value,
        success: bool,
        error: String,
    ) {
        let field = self.expression_field(tag);
        let title = self.schema[tag].title.clone();
        let a = self.analyses.get_mut(tag).unwrap();
        a.pending = false;
        if !success {
            a.error = error.clone();
            let message = if tag == "routh" {
                format!("Erro na análise: {error}")
            } else {
                format!("Erro: {error}")
            };
            self.toast(message, ToastKind::Error);
            return;
        }
        a.error.clear();
        a.figures = results::plots(ctx, tag, &reply);
        a.zoom = 1.0;
        a.pan = egui::Vec2::ZERO;
        a.result = reply;
        let expr = a.values.get(field).cloned().unwrap_or_default();
        let message = match tag {
            "routh" => "Estabilidade calculada com sucesso!".to_owned(),
            "lgr" => "LGR calculado e traçado com sucesso!".to_owned(),
            _ => format!("{title} concluída."),
        };
        let name = if expr.is_empty() { title } else { expr.clone() };
        self.add_recent(Recent {
            name,
            expr,
            nav: tag.into(),
            file: String::new(),
            kind: "math".into(),
            timestamp: unix_millis(),
        });
        self.toast(message, ToastKind::Success);
    }

    fn finish_script(&mut self, tag: &str, reply: Value, success: bool, error: String) {
        match tag {
            "updates" => self.update = Some(overlays::UpdateState::from_reply(&reply)),
            "state" => {
                if reply["workspace"].is_array() {
                    self.workspace = reply["workspace"].clone();
                }
            }
            "whos" => {
                let rows: Vec<String> = texfmt::list(&reply["workspace"])
                    .iter()
                    .map(|v| {
                        ["name", "type", "dimensions", "value"]
                            .map(|k| v[k].as_str().unwrap_or(""))
                            .join("\t")
                    })
                    .collect();
                self.log(
                    if rows.is_empty() {
                        "Workspace vazio.".into()
                    } else {
                        rows.join("\n")
                    },
                    "output",
                );
            }
            "run" => {
                if success {
                    let name = reply["name"].as_str().unwrap_or("script.m").to_owned();
                    self.execute(
                        reply["source"].as_str().unwrap_or_default().to_owned(),
                        name,
                    );
                } else {
                    self.log(error, "error");
                }
            }
            "folder" | "cd" | "history" => {
                if !success {
                    if tag == "cd" {
                        self.log(error, "error");
                    }
                    self.status = "Pasta indisponível".into();
                    return;
                }
                self.files = reply["files"].clone();
                self.prefs.folder = reply["folder"].as_str().unwrap_or_default().into();
                self.parent = reply["parent"].as_str().map(str::to_owned);
                self.folder_input = self.prefs.folder.clone();
                if tag != "history"
                    && self.history.get(self.history_cursor) != Some(&self.prefs.folder)
                {
                    if !self.history.is_empty() {
                        self.history.truncate(self.history_cursor + 1);
                    }
                    self.history.push(self.prefs.folder.clone());
                    if self.history.len() > 50 {
                        self.history.remove(0);
                    }
                    self.history_cursor = self.history.len() - 1;
                }
            }
            "read" => {
                if !success {
                    self.log(error.clone(), "error");
                    self.toast(error, ToastKind::Error);
                    return;
                }
                let source = reply["source"].as_str().unwrap_or_default().to_owned();
                let path = reply["path"].as_str().unwrap_or_default().to_owned();
                let name = reply["name"].as_str().unwrap_or("script.m").to_owned();
                if let Some(index) = self.documents.iter().position(|d| d.path == path) {
                    self.active = Some(index);
                } else {
                    self.documents.push(Document {
                        name: name.clone(),
                        path: path.clone(),
                        source: source.clone(),
                        saved: source,
                    });
                    self.active = Some(self.documents.len() - 1);
                }
                self.add_recent(Recent {
                    name: name.clone(),
                    expr: String::new(),
                    nav: String::new(),
                    file: path,
                    kind: "script".into(),
                    timestamp: unix_millis(),
                });
                self.show_figures = false;
                self.page = "home".into();
                self.show_files = false;
                self.status = format!("Aberto: {name}");
            }
            _ if tag.starts_with("save:") => {
                let index = tag["save:".len()..].parse::<usize>().ok();
                self.script_pending = false;
                if success {
                    if let Some(d) = index.and_then(|i| self.documents.get_mut(i)) {
                        // Editing is disabled while saving, so the acknowledged buffer is exact.
                        d.path = reply["path"].as_str().unwrap_or_default().into();
                        d.name = reply["name"].as_str().unwrap_or("script.m").into();
                        d.saved = d.source.clone();
                        let (name, path) = (d.name.clone(), d.path.clone());
                        self.status = format!("Salvo: {name}");
                        self.add_recent(Recent {
                            name,
                            expr: String::new(),
                            nav: String::new(),
                            file: path,
                            kind: "script".into(),
                            timestamp: unix_millis(),
                        });
                    }
                    self.send("folder", json!({"action":"script_request","operation":"list","path":self.prefs.folder}));
                } else if reply["exists"].as_bool() == Some(true) {
                    self.pending_save = index;
                    self.confirm = Some((
                        "Substituir arquivo".into(),
                        format!(
                            "O arquivo {} já existe. Substituir?",
                            reply["path"].as_str().unwrap_or("")
                        ),
                    ));
                } else {
                    self.log(error.clone(), "error");
                    self.toast(error, ToastKind::Error);
                }
            }
            "script" | "clear" => {
                self.script_pending = false;
                if let Some(output) = reply["output"].as_str()
                    && !output.trim().is_empty()
                {
                    self.log(output.trim_end().to_owned(), "output");
                }
                if reply["workspace"].is_array() {
                    self.workspace = reply["workspace"].clone();
                }
                if let Some(plots) = reply["plots"].as_array()
                    && !plots.is_empty()
                {
                    self.script_plots.extend(plots.iter().cloned());
                    let excess = self.script_plots.len().saturating_sub(16);
                    self.script_plots.drain(..excess);
                    self.show_figures = true;
                }
                if let Some(d) = reply["diagnostic"].as_object() {
                    self.log(
                        format!(
                            "Linha {}, coluna {}: {}",
                            d["line"],
                            d["column"],
                            d["message"].as_str().unwrap_or("")
                        ),
                        "error",
                    );
                    self.status = "Erro de execução".into();
                } else if !success {
                    self.log(error, "error");
                    self.status = "Execução indisponível".into();
                } else if tag == "clear" {
                    self.status = "Workspace limpo".into();
                } else {
                    self.status =
                        format!("Concluído em {:.2} s", (self.now - self.started).max(0.0));
                }
            }
            _ => {}
        }
    }

    fn log(&mut self, text: String, kind: &'static str) {
        self.console.push((text, kind));
        let excess = self.console.len().saturating_sub(2000);
        self.console.drain(..excess);
    }

    fn new_document(&mut self) {
        let next = self.documents.len() + 1;
        self.documents.push(Document {
            name: format!("Sem título {next}.m"),
            ..Default::default()
        });
        self.active = Some(self.documents.len() - 1);
        self.show_figures = false;
        self.page = "home".into();
    }

    fn execute(&mut self, source: String, label: String) {
        if self.script_pending {
            self.log("Aguarde a execução atual terminar.".into(), "error");
            return;
        }
        if source.len() > MAX_SCRIPT_BYTES {
            self.log("Script excede 1 MiB.".into(), "error");
            return;
        }
        self.log(format!(">> {label}"), "cmd");
        self.started = self.now;
        self.script_pending = true;
        self.status = "Executando…".into();
        self.send(
            "script",
            json!({"action":"script_request","operation":"execute","source":source}),
        );
    }

    fn run_current(&mut self) {
        match self.active {
            Some(i) => {
                let (source, name) = (
                    self.documents[i].source.clone(),
                    self.documents[i].name.clone(),
                );
                self.execute(source, name);
            }
            None => self.status = "Abra ou crie um script para executar".into(),
        }
    }

    fn clear_workspace(&mut self) {
        if self.script_pending {
            return;
        }
        self.script_pending = true;
        self.send(
            "clear",
            json!({"action":"script_request","operation":"clear"}),
        );
    }

    /// Console commands the previous Studio handled itself; anything else goes to the engine.
    fn execute_command(&mut self, raw: &str) {
        let command = raw.trim();
        if command.is_empty() {
            return;
        }
        if command == "clc" {
            self.console.clear();
            return;
        }
        let folder = std::path::Path::new(&self.prefs.folder).to_path_buf();
        match command {
            "clear" => self.clear_workspace(),
            "help" | "ajuda" => self.log(HELP.trim_end().into(), "output"),
            "pwd" => self.log(self.prefs.folder.clone(), "output"),
            "ls" | "dir" => {
                let names: Vec<&str> = texfmt::list(&self.files)
                    .iter()
                    .filter_map(|f| f["name"].as_str())
                    .collect();
                let text = if names.is_empty() {
                    "(pasta vazia)".to_owned()
                } else {
                    names.join("\n")
                };
                self.log(text, "output");
            }
            "whos" => self.send(
                "whos",
                json!({"action":"script_request","operation":"state"}),
            ),
            _ => {
                if let Some(target) = cd_target(command) {
                    let path = folder.join(target).to_string_lossy().into_owned();
                    self.send(
                        "cd",
                        json!({"action":"script_request","operation":"list","path":path}),
                    );
                } else if let Some(file) = run_target(command) {
                    let path = folder.join(file).to_string_lossy().into_owned();
                    self.send(
                        "run",
                        json!({"action":"script_request","operation":"read","path":path}),
                    );
                } else {
                    self.execute(command.to_owned(), command.to_owned());
                }
            }
        }
    }

    fn save_document(&mut self) {
        if self.script_pending {
            return;
        }
        let Some(i) = self.active else { return };
        if self.documents[i].source.len() > MAX_SCRIPT_BYTES {
            self.log("Script excede 1 MiB.".into(), "error");
            return;
        }
        let path = self.documents[i].path.clone();
        if path.is_empty() {
            let name = if self.documents[i].name.starts_with("Sem título") {
                "script.m".to_owned()
            } else {
                self.documents[i].name.clone()
            };
            self.pending_save = Some(i);
            self.pick(Some(name), "save");
        } else {
            // The document already owns this file: save in place, as the previous Studio did.
            self.write_document(i, path, true);
        }
    }

    fn write_document(&mut self, index: usize, path: String, overwrite: bool) {
        self.script_pending = true;
        self.send(&format!("save:{index}"), json!({"action":"script_request","operation":"save","path":path,"source":self.documents[index].source,"overwrite":overwrite}));
    }

    fn pick(&mut self, name: Option<String>, kind: &'static str) {
        self.pick_export(name, kind, Vec::new());
    }

    /// Opens an OS dialog on a helper thread; the choice is applied on a later frame.
    #[allow(unused_variables)]
    fn pick_export(&mut self, name: Option<String>, kind: &'static str, bytes: Vec<u8>) {
        #[cfg(not(target_os = "android"))]
        {
            let slot = self.picked.clone();
            let folder = self.prefs.folder.clone();
            std::thread::spawn(move || {
                let dialog = rfd::FileDialog::new().set_directory(&folder);
                let choice = match kind {
                    "open" => dialog
                        .add_filter("Script MATLAB", &["m"])
                        .pick_file()
                        .map(Picked::Open),
                    "save" => dialog
                        .add_filter("Script MATLAB", &["m"])
                        .set_file_name(name.unwrap_or_default())
                        .save_file()
                        .map(|p| {
                            let m = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("m"));
                            Picked::Save(if m { p } else { p.with_extension("m") })
                        }),
                    "folder" => dialog.pick_folder().map(Picked::Folder),
                    format => {
                        let svg = format == "svg";
                        let (label, ext) = if svg {
                            ("Gráfico Vetorial SVG", "svg")
                        } else {
                            ("Imagem PNG", "png")
                        };
                        dialog
                            .add_filter(label, &[ext])
                            .set_file_name(name.unwrap_or_default())
                            .save_file()
                            .map(|p| Picked::Export(p, bytes, if svg { "svg" } else { "png" }))
                    }
                };
                if let Some(choice) = choice {
                    *slot.lock().unwrap() = Some(choice);
                }
            });
        }
    }

    fn apply_picked(&mut self) {
        let Some(choice) = self.picked.lock().unwrap().take() else {
            return;
        };
        match choice {
            Picked::Open(path) => self.send(
                "read",
                json!({"action":"script_request","operation":"read","path":path}),
            ),
            Picked::Folder(path) => self.send(
                "folder",
                json!({"action":"script_request","operation":"list","path":path}),
            ),
            Picked::Save(path) => {
                if let Some(i) = self.pending_save.take()
                    && i < self.documents.len()
                {
                    // The OS dialog already confirmed any replacement.
                    self.write_document(i, path.to_string_lossy().into_owned(), true);
                }
            }
            Picked::Export(path, bytes, format) => match std::fs::write(&path, bytes) {
                Ok(()) => self.toast(
                    if format == "svg" {
                        "Gráfico Vetorial SVG salvo com sucesso!"
                    } else {
                        "Imagem PNG salva com sucesso!"
                    },
                    ToastKind::Success,
                ),
                Err(e) => self.toast(format!("Erro ao salvar: {e}"), ToastKind::Error),
            },
        }
    }

    fn figure_bytes(&self, id: &str, svg: bool) -> Option<Vec<u8>> {
        let a = &self.analyses[id];
        let r = &a.result;
        let plot = &r["plots"][&a.response];
        if svg {
            return plot["svg"]
                .as_str()
                .or(r["svg"].as_str())
                .or(r["plot_svg"].as_str())
                .map(|s| s.as_bytes().to_vec());
        }
        let png = plot["image"]
            .as_str()
            .or(r["image"].as_str())
            .or(r["plot_image"].as_str())?;
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(png.strip_prefix("data:image/png;base64,").unwrap_or(png))
            .ok()
    }

    /// Exports the active figure of module `id` through the OS save dialog.
    fn export_figure(&mut self, id: &str, format: &'static str) {
        if let Some(bytes) = self.figure_bytes(id, format == "svg") {
            let name = format!(
                "{}.{format}",
                self.schema[id].filename.as_deref().unwrap_or(id)
            );
            self.pick_export(Some(name), format, bytes);
        }
    }

    fn copy_figure(&mut self, ctx: &egui::Context, id: &str) {
        let image = self.figure_bytes(id, false).and_then(|bytes| {
            let rgba = image::load_from_memory(&bytes).ok()?.to_rgba8();
            Some(egui::ColorImage::from_rgba_unmultiplied(
                [rgba.width() as usize, rgba.height() as usize],
                rgba.as_raw(),
            ))
        });
        if let Some(image) = image {
            ctx.copy_image(image);
            self.toast(
                "Gráfico copiado para a Área de Transferência!",
                ToastKind::Success,
            );
        }
    }

    /// Debounced previews: instant formatting while typing, Python validation after a pause.
    fn schedule_previews(&mut self) {
        let ids: Vec<String> = self.analyses.keys().cloned().collect();
        for id in ids {
            if id == "routh" {
                // The Routh preview is typeset locally, without engine validation.
                self.analyses.get_mut(&id).unwrap().preview.due = None;
                continue;
            }
            let field = self.expression_field(&id);
            let a = self.analyses.get_mut(&id).unwrap();
            let Some(due) = a.preview.due else { continue };
            if due > self.now || a.preview.in_flight.is_some() {
                continue;
            }
            a.preview.due = None;
            let raw = a
                .values
                .get(field)
                .map(|v| v.trim().to_owned())
                .unwrap_or_default();
            if raw.is_empty() || raw == a.preview.valid_for {
                continue;
            }
            a.preview.in_flight = Some(raw.clone());
            self.send(
                &format!("preview:{id}"),
                json!({"action":"preview","mode":"expr","expr":raw}),
            );
        }
    }

    fn save_preferences(&self) {
        if std::env::var_os("CONTROLAB_SCREENSHOT").is_some() {
            return;
        }
        if let Some(p) = preferences_path() {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(bytes) = serde_json::to_vec(&self.prefs) {
                let _ = std::fs::write(p, bytes);
            }
        }
    }

    /// One frame of the whole interface; tests drive it with synthetic input.
    fn frame(&mut self, ui: &mut egui::Ui) {
        let safe = ui.ctx().content_rect();
        let outer = ui.max_rect();
        if safe.min.distance(outer.min) > 0.1 || safe.max.distance(outer.max) > 0.1 {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(safe));
            child.set_clip_rect(safe);
            self.frame_contents(&mut child);
            ui.advance_cursor_after_rect(safe);
        } else {
            self.frame_contents(ui);
        }
    }

    fn frame_contents(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.now = ctx.input(|i| i.time);
        // Media queries use CSS px, which do not change with the egui zoom.
        let zoom = ctx.zoom_factor();
        // Context::viewport_rect rounds to GUI units; use the unrounded input dimensions
        // so a 600/1024 px window does not slip below its CSS breakpoint at non-unit zoom.
        let css_width = (ctx.input(|i| i.viewport_rect().width()) * zoom * 100.0).round() / 100.0;
        let touch = cfg!(target_os = "android")
            || ctx.input(|i| i.has_touch_screen())
            || std::env::var_os("CONTROLAB_TOUCH").is_some();
        self.layout = Layout::new(css_width, self.prefs.scale as f32 / 100.0, touch);
        self.layout.keyboard_open = touch && ctx.text_edit_focused();
        kit::set_layout(&ctx, self.layout);
        // Module pages are sized in rem, the Studio (`studio.css`) in CSS px.
        let target = if self.page == "home" {
            1.0
        } else {
            self.layout.rem
        };
        if (zoom - target).abs() > 0.001 {
            ctx.set_zoom_factor(target);
            ctx.request_repaint();
        }
        let p = self.palette();
        let k = if self.page == "home" {
            self.layout.rem
        } else {
            1.0
        };
        egui::Panel::top("shell")
            .exact_size(46.0 * k)
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                match kit::top_bar(ui, &p, &self.page, k, self.layout.mode == Mode::Desktop) {
                    Some(kit::ShellAction::Navigate(id)) => self.page = id.into(),
                    Some(kit::ShellAction::Settings) => self.settings = true,
                    None => {}
                }
            });
        if self.layout.mode == Mode::Phone && !self.layout.keyboard_open {
            // `.sci-bottom-nav`: module navigation of the phone layout.
            let page = self.page.clone();
            egui::Panel::bottom("bottom-nav")
                .exact_size(3.5 * MEASURED_REM * k)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    if let Some(id) = kit::bottom_nav(ui, &p, &page, k) {
                        self.page = id.into();
                    }
                });
        }
        if self.page == "home" {
            self.studio(ui);
        } else {
            self.module_page(ui);
        }
        self.overlays(&ctx);
    }
}

impl eframe::App for Controlab {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.prefs.theme == "system" {
            let light = ctx.system_theme().is_none_or(|t| t == egui::Theme::Light);
            if light != self.light {
                self.apply_theme(&ctx);
            }
        }
        self.poll(&ctx);
        self.apply_picked();
        self.now = ctx.input(|i| i.time);
        self.schedule_previews();
        if self.page != "home"
            && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter))
        {
            let page = self.page.clone();
            self.calculate(&page, 1080.0);
        }
        if self.page == "home" && !self.settings && self.confirm.is_none() {
            if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
                self.run_current();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                self.save_document();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) {
                self.new_document();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
                self.pick(None, "open");
            }
        }
        self.frame(ui);
        if let Some(payload) = self.formulas.next_request() {
            self.send("formulas", payload);
        }
        if self.toast.is_some() || self.analyses.values().any(|a| a.preview.due.is_some()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && self.documents.iter().any(Document::dirty)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.quit_confirm = true;
        }
        #[cfg(not(target_os = "android"))]
        if let Ok(path) = std::env::var("CONTROLAB_SCREENSHOT") {
            let settled =
                self.analyses.values().all(|a| {
                    !a.pending && a.preview.in_flight.is_none() && a.preview.due.is_none()
                }) && !self.formulas.busy();
            if !self.screenshot_requested {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            if !self.screenshot_requested && settled && self.now > 1.5 {
                self.toast = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.screenshot_requested = true;
            }
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    if let Err(e) = image::save_buffer(
                        &path,
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    ) {
                        eprintln!("{e}");
                    }
                    // A captured script is never saved; do not ask to discard it.
                    self.allow_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_preferences();
    }
}

const HELP: &str = "Scripts .m · ControLAB\n\
F5: executar · Ctrl+S: salvar · Ctrl+O: abrir · Ctrl+N: novo script\n\
Exemplo: A=[1 2;3 4]; disp(A*A)\n\
Gráficos: x=0:0.1:6; plot(x,sin(x))\n\
Controle: G=tf([1],[1 2 1]); step(G)\n\
Console: clear, clc, whos, pwd, ls, cd <pasta>, run('arquivo.m')\n\
O interpretador suporta um subconjunto de MATLAB; funções de usuário e controle de fluxo ainda não estão disponíveis.\n";

/// `cd <pasta>`: the folder may be absolute or relative to the current one.
fn cd_target(command: &str) -> Option<&str> {
    let rest = command.strip_prefix("cd")?;
    rest.starts_with(char::is_whitespace)
        .then(|| rest.trim())
        .filter(|target| !target.is_empty())
}

/// `run('arquivo.m')`, optionally followed by `;`.
fn run_target(command: &str) -> Option<&str> {
    let call = command.strip_suffix(';').unwrap_or(command).trim_end();
    let name = call
        .strip_prefix("run(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('\'')?
        .strip_suffix('\'')?;
    (!name.is_empty() && !name.contains('\'')).then_some(name)
}

fn preferences_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "android")]
    {
        android::config_dir().map(|p| p.join("preferences.json"))
    }
    #[cfg(not(target_os = "android"))]
    {
        dirs::config_dir().map(|p| p.join("controlab-native/preferences.json"))
    }
}

#[cfg(test)]
mod ui_tests;
