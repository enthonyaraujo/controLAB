//! Serial background transport. GUI frames never wait for a scientific request.
use eframe::egui;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::Duration;

type Reply = (String, Result<Value, String>);

fn request_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("CONTROLAB_REQUEST_TIMEOUT")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(60)
            .clamp(1, 300),
    )
}

pub struct Backend {
    requests: mpsc::SyncSender<(String, Value)>,
    replies: mpsc::Receiver<Reply>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Backend {
    pub fn new(ctx: egui::Context) -> Self {
        let (requests, incoming) = mpsc::sync_channel::<(String, Value)>(16);
        let (outgoing, replies) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        // The .m interpreter recurses over expression trees; give it a generous stack.
        let worker = thread::Builder::new()
            .name("controlab-backend".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut scripts = crate::scripts::Scripts::default();
                #[cfg(not(target_os = "android"))]
                let mut process: Option<desktop::ScientificProcess> = None;
                while !worker_stop.load(Ordering::Relaxed) {
                    let (tag, payload) = match incoming.recv_timeout(Duration::from_millis(100)) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    let result = if payload["action"] == "script_request" {
                        Ok(scripts.request(&payload))
                    } else {
                        #[cfg(target_os = "android")]
                        let result = Ok(controlab_engine::dispatch::dispatch_with_control(
                            &payload,
                            worker_stop.clone(),
                            request_timeout(),
                        ));
                        #[cfg(not(target_os = "android"))]
                        let result = desktop::request(&mut process, payload, &worker_stop);
                        result
                    };
                    if worker_stop.load(Ordering::Relaxed) || outgoing.send((tag, result)).is_err()
                    {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .expect("backend worker thread");
        Self {
            requests,
            replies,
            stop,
            worker: Some(worker),
        }
    }

    pub fn submit(&self, tag: String, payload: Value) -> Result<(), String> {
        if !payload.is_object() {
            return Err("A requisição deve ser um objeto JSON.".into());
        }
        if serde_json::to_vec(&payload)
            .map_err(|error| error.to_string())?
            .len()
            > 8 * 1024 * 1024 - 1
        {
            return Err("Requisição excede o limite de 8 MiB.".into());
        }
        self.requests
            .try_send((tag, payload))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => {
                    "Fila de cálculos cheia. Aguarde o cálculo atual.".into()
                }
                mpsc::TrySendError::Disconnected(_) => "O serviço científico foi encerrado.".into(),
            })
    }

    pub fn poll(&self) -> Option<Reply> {
        self.replies.try_recv().ok()
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        #[cfg(not(target_os = "android"))]
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        #[cfg(target_os = "android")]
        let _ = self.worker.take();
    }
}

#[cfg(not(target_os = "android"))]
mod desktop {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, Command, Stdio};
    use std::sync::Mutex;
    use std::time::Instant;

    const MAX_REPLY: usize = 32 * 1024 * 1024;
    const SESSION_LOST: &str =
        "A sessão científica foi perdida. Execute novamente para iniciar uma nova sessão.";

    pub(super) struct ScientificProcess {
        child: Child,
        stdin: Arc<Mutex<ChildStdin>>,
        replies: mpsc::Receiver<Result<Value, String>>,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum ScientificBackend {
        Rust,
    }

    impl ScientificBackend {
        fn parse(selection: Option<&str>) -> Result<Self, String> {
            match selection.unwrap_or("rust") {
                "rust" => Ok(Self::Rust),
                "python" => Err("O backend Python foi removido. O serviço científico usa Rust; remova CONTROLAB_SCIENTIFIC_BACKEND ou defina rust.".into()),
                other => Err(format!(
                    "Backend científico desconhecido: {other}. Use rust."
                )),
            }
        }
    }

    fn engine_name() -> &'static str {
        if cfg!(windows) {
            "controlab-engine.exe"
        } else {
            "controlab-engine"
        }
    }

    fn packaged_engine(directory: &Path) -> Option<PathBuf> {
        let path = directory.join("engine").join(engine_name());
        path.is_file().then_some(path)
    }

    fn rust_engine_path(
        explicit: Option<PathBuf>,
        executable_directory: Option<&Path>,
        repository: &Path,
    ) -> Result<PathBuf, String> {
        if let Some(path) = explicit {
            if !path.is_file() {
                return Err(format!(
                    "Engine Rust indisponível em {}. Confira CONTROLAB_ENGINE.",
                    path.display()
                ));
            }
            return path
                .canonicalize()
                .map_err(|error| format!("Engine Rust indisponível: {error}"));
        }
        if let Some(path) = executable_directory.and_then(packaged_engine) {
            return Ok(path);
        }
        let profiles = if cfg!(debug_assertions) {
            ["debug", "release"]
        } else {
            ["release", "debug"]
        };
        for profile in profiles {
            let path = repository.join("target").join(profile).join(engine_name());
            if path.is_file() {
                return Ok(path);
            }
        }
        Err("Engine Rust não encontrada. Compile com cargo build -p controlab-engine ou defina CONTROLAB_ENGINE.".into())
    }

    impl ScientificProcess {
        fn start() -> Result<Self, String> {
            let selection = match std::env::var("CONTROLAB_SCIENTIFIC_BACKEND") {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(_) => {
                    return Err("CONTROLAB_SCIENTIFIC_BACKEND deve conter rust em UTF-8.".into());
                }
            };
            ScientificBackend::parse(selection.as_deref())?;
            let executable = std::env::current_exe().ok();
            let engine = rust_engine_path(
                std::env::var_os("CONTROLAB_ENGINE").map(PathBuf::from),
                executable.as_deref().and_then(Path::parent),
                &Path::new(env!("CARGO_MANIFEST_DIR")).join(".."),
            )?;
            Self::spawn(Command::new(engine)).map_err(|error| format!("Backend Rust: {error}"))
        }

        fn spawn(mut command: Command) -> Result<Self, String> {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit());
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }
            let mut child = command.spawn().map_err(|error| {
                format!("Não foi possível iniciar o serviço científico: {error}.")
            })?;
            let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
            let stdout = child.stdout.take().unwrap();
            let (outgoing, replies) = mpsc::sync_channel(1);
            thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = Vec::new();
                    let result = match reader
                        .by_ref()
                        .take((MAX_REPLY + 2) as u64)
                        .read_until(b'\n', &mut line)
                    {
                        Ok(0) => Err("O serviço científico encerrou a conexão.".into()),
                        Ok(_) if line.len() > MAX_REPLY + 1 || !line.ends_with(b"\n") => {
                            Err("Resposta científica inválida ou maior que 32 MiB.".into())
                        }
                        Ok(_) => serde_json::from_slice::<Value>(&line)
                            .map_err(|error| format!("Resposta JSON inválida: {error}"))
                            .and_then(|value| {
                                if value.is_object() {
                                    Ok(value)
                                } else {
                                    Err("A resposta científica deve ser um objeto JSON.".into())
                                }
                            }),
                        Err(error) => Err(format!("Falha ao ler o serviço científico: {error}")),
                    };
                    let failed = result.is_err();
                    if outgoing.send(result).is_err() || failed {
                        break;
                    }
                }
            });
            Ok(Self {
                child,
                stdin,
                replies,
            })
        }

        fn exchange(&mut self, payload: Value, stop: &AtomicBool) -> Result<Value, String> {
            self.exchange_timeout(payload, stop, request_timeout())
        }

        fn exchange_timeout(
            &mut self,
            payload: Value,
            stop: &AtomicBool,
            timeout: Duration,
        ) -> Result<Value, String> {
            let seconds = timeout.as_secs();
            let deadline = Instant::now() + timeout;
            let mut body = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
            body.push(b'\n');
            let stdin = self.stdin.clone();
            let (written, write_result) = mpsc::channel();
            // Writing large scripts can block too; the same request deadline covers it.
            thread::spawn(move || {
                let result = stdin
                    .lock()
                    .map_err(|_| "Canal científico indisponível.".to_string())
                    .and_then(|mut pipe| {
                        pipe.write_all(&body)
                            .and_then(|_| pipe.flush())
                            .map_err(|error| error.to_string())
                    });
                let _ = written.send(result);
            });
            loop {
                if stop.load(Ordering::Relaxed) {
                    return Err("Aplicativo encerrando.".into());
                }
                if Instant::now() >= deadline {
                    return Err(format!("Tempo de cálculo excedido ({seconds} s)."));
                }
                if let Ok(Err(error)) = write_result.try_recv() {
                    return Err(format!(
                        "Falha ao enviar para o serviço científico: {error}"
                    ));
                }
                match self.replies.recv_timeout(Duration::from_millis(50)) {
                    Ok(response) => return response,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err("O serviço científico encerrou a conexão.".into());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        }
    }

    pub(super) fn request(
        process: &mut Option<ScientificProcess>,
        payload: Value,
        stop: &AtomicBool,
    ) -> Result<Value, String> {
        request_with(process, payload, stop, ScientificProcess::start)
    }

    fn request_with(
        process: &mut Option<ScientificProcess>,
        payload: Value,
        stop: &AtomicBool,
        start: impl FnOnce() -> Result<ScientificProcess, String>,
    ) -> Result<Value, String> {
        if let Some(current) = process.as_mut() {
            match current.child.try_wait() {
                Ok(None) => {}
                _ => {
                    *process = None;
                    return Err(SESSION_LOST.into());
                }
            }
        }
        if process.is_none() {
            *process = Some(start()?);
        }
        let response = process.as_mut().unwrap().exchange(payload, stop);
        if let Err(error) = response {
            *process = None;
            return Err(format!("{error} {SESSION_LOST}"));
        }
        response
    }

    impl Drop for ScientificProcess {
        fn drop(&mut self) {
            #[cfg(unix)]
            {
                unsafe extern "C" {
                    fn kill(pid: i32, sig: i32) -> i32;
                }
                // Signal the whole private group, including any helper the engine spawned.
                unsafe {
                    kill(-(self.child.id() as i32), 15);
                }
                let deadline = Instant::now() + Duration::from_millis(500);
                while Instant::now() < deadline {
                    if matches!(self.child.try_wait(), Ok(Some(_))) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                unsafe {
                    kill(-(self.child.id() as i32), 9);
                }
            }
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
    #[cfg(test)]
    mod packaged_tests {
        #[test]
        fn selection_is_explicit_and_invalid_modes_fail() {
            assert_eq!(
                ScientificBackend::parse(None).unwrap(),
                ScientificBackend::Rust
            );
            assert_eq!(
                ScientificBackend::parse(Some("rust")).unwrap(),
                ScientificBackend::Rust
            );
            assert!(
                ScientificBackend::parse(Some("python"))
                    .unwrap_err()
                    .contains("Rust")
            );
            assert!(ScientificBackend::parse(Some("unknown")).is_err());
        }

        #[test]
        fn rust_discovery_checks_override_package_and_development_in_order() {
            let root = std::env::temp_dir()
                .join(format!("controlab-rust-discovery-{}", std::process::id()));
            let executable_dir = root.join("app");
            let repository = root.join("repo");
            let name = engine_name();
            std::fs::create_dir_all(executable_dir.join("engine")).unwrap();
            std::fs::create_dir_all(repository.join("target/debug")).unwrap();
            let bundled = executable_dir.join("engine").join(name);
            let development = repository.join("target/debug").join(name);
            assert!(rust_engine_path(None, Some(&executable_dir), &repository).is_err());
            std::fs::write(&development, []).unwrap();
            assert_eq!(
                rust_engine_path(None, Some(&executable_dir), &repository).unwrap(),
                development
            );
            std::fs::write(&bundled, []).unwrap();
            assert_eq!(
                rust_engine_path(None, Some(&executable_dir), &repository).unwrap(),
                bundled
            );
            // An invalid explicit override must never select another engine or Python.
            assert!(
                rust_engine_path(
                    Some(root.join("missing")),
                    Some(&executable_dir),
                    &repository
                )
                .is_err()
            );
            assert_eq!(
                rust_engine_path(
                    Some(development.clone()),
                    Some(&executable_dir),
                    &repository
                )
                .unwrap(),
                development.canonicalize().unwrap()
            );
            std::fs::remove_dir_all(root).unwrap();
        }

        use super::*;
        #[cfg(unix)]
        #[test]
        fn json_lines_preserve_recoverable_errors_and_restart_after_transport_failure() {
            let stop = AtomicBool::new(false);
            let mut command = Command::new("sh");
            command.arg("-c").arg(r#"read -r request; echo '{"success":true,"value":1}'; read -r request; echo '{"success":false,"error":"Invalid input"}'; read -r request; echo 'invalid JSON'; sleep 1"#);
            let mut process = Some(ScientificProcess::spawn(command).unwrap());
            let first = request_with(
                &mut process,
                serde_json::json!({"action":"preview"}),
                &stop,
                || panic!("must reuse worker"),
            )
            .unwrap();
            assert_eq!(first["value"], 1);
            let second = request_with(
                &mut process,
                serde_json::json!({"action":"invalid"}),
                &stop,
                || panic!("must reuse worker"),
            )
            .unwrap();
            assert_eq!(second["success"], false);
            assert!(
                process.is_some(),
                "Scientific errors must preserve the process"
            );
            assert!(
                request_with(&mut process, serde_json::json!({}), &stop, || panic!(
                    "must reuse worker"
                ))
                .is_err()
            );
            assert!(process.is_none(), "Malformed JSON must reset the transport");
            let recovered = request_with(&mut process, serde_json::json!({}), &stop, || {
                let mut command = Command::new("sh");
                command
                    .arg("-c")
                    .arg(r#"read -r request; echo '{"success":true,"value":2}'; sleep 1"#);
                ScientificProcess::spawn(command)
            })
            .unwrap();
            assert_eq!(recovered["value"], 2);
        }

        #[cfg(unix)]
        #[test]
        fn shutdown_cancels_a_blocked_scientific_process() {
            let mut command = Command::new("sh");
            command.arg("-c").arg("sleep 120");
            let mut process = Some(ScientificProcess::spawn(command).unwrap());
            let stop = AtomicBool::new(true);
            let started = Instant::now();
            assert!(
                request_with(&mut process, serde_json::json!({}), &stop, || panic!(
                    "must reuse worker"
                ))
                .is_err()
            );
            assert!(process.is_none());
            assert!(started.elapsed() < Duration::from_secs(3));
        }

        #[cfg(unix)]
        #[test]
        fn blocked_io_obeys_the_request_deadline() {
            let mut command = Command::new("sh");
            command.arg("-c").arg("sleep 120");
            let mut process = ScientificProcess::spawn(command).unwrap();
            let started = Instant::now();
            let error = process
                .exchange_timeout(
                    serde_json::json!({}),
                    &AtomicBool::new(false),
                    Duration::from_millis(50),
                )
                .unwrap_err();
            assert!(error.contains("Tempo de cálculo excedido"));
            drop(process);
            assert!(started.elapsed() < Duration::from_secs(3));
        }

        #[cfg(unix)]
        #[test]
        fn shutdown_during_a_request_interrupts_waiting_for_a_reply() {
            let mut command = Command::new("sh");
            command.arg("-c").arg("sleep 120");
            let mut process = Some(ScientificProcess::spawn(command).unwrap());
            let stop = Arc::new(AtomicBool::new(false));
            let thread_stop = stop.clone();
            let cancel = thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                thread_stop.store(true, Ordering::Relaxed);
            });
            let started = Instant::now();
            assert!(
                request_with(&mut process, serde_json::json!({}), &stop, || panic!(
                    "must reuse worker"
                ))
                .is_err()
            );
            cancel.join().unwrap();
            assert!(process.is_none());
            assert!(started.elapsed() < Duration::from_secs(3));
        }

        #[test]
        fn engine_is_selected_only_from_its_dedicated_distribution_directory() {
            let root = std::env::temp_dir()
                .join(format!("controlab-engine-discovery-{}", std::process::id()));
            std::fs::create_dir_all(root.join("engine")).unwrap();
            let name = if cfg!(windows) {
                "controlab-engine.exe"
            } else {
                "controlab-engine"
            };
            assert!(packaged_engine(&root).is_none());
            std::fs::write(root.join("engine").join(name), []).unwrap();
            assert_eq!(packaged_engine(&root), Some(root.join("engine").join(name)));
            std::fs::remove_file(root.join("engine").join(name)).unwrap();
            std::fs::remove_dir(root.join("engine")).unwrap();
            std::fs::remove_dir(root).unwrap();
        }
    }
}
