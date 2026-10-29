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
                let mut process: Option<desktop::PythonProcess> = None;
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
                        let result = crate::android::request(&payload);
                        #[cfg(not(target_os = "android"))]
                        let result = desktop::request(&mut process, payload, &worker_stop);
                        result
                    };
                    if outgoing.send((tag, result)).is_err() {
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

    pub(super) struct PythonProcess {
        child: Child,
        stdin: Arc<Mutex<ChildStdin>>,
        replies: mpsc::Receiver<Result<Value, String>>,
    }

    fn bridge_path() -> Result<PathBuf, String> {
        if let Some(path) = std::env::var_os("CONTROLAB_NATIVE_BRIDGE") {
            let path = PathBuf::from(path);
            return path
                .canonicalize()
                .map_err(|error| format!("Ponte Python indisponível: {error}"));
        }
        let mut candidates = Vec::new();
        if let Ok(executable) = std::env::current_exe()
            && let Some(directory) = executable.parent()
        {
            // Shipped layouts: bridge alongside executable or in python/ resources.
            for relative in [
                "native_bridge.py",
                "python/native_bridge.py",
                "resources/python/native_bridge.py",
                "../Resources/python/native_bridge.py",
            ] {
                candidates.push(directory.join(relative));
            }
        }
        candidates
            .push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/python/native_bridge.py"));
        candidates
            .into_iter()
            .find(|path| path.is_file())
            .ok_or_else(|| {
                "Ponte Python não encontrada. Defina CONTROLAB_NATIVE_BRIDGE.".to_string()
            })?
            .canonicalize()
            .map_err(|error| error.to_string())
    }

    fn python_path(bridge: &Path) -> PathBuf {
        if let Some(path) = std::env::var_os("CONTROLAB_PYTHON") {
            return path.into();
        }
        let name = if cfg!(windows) {
            ".venv/Scripts/python.exe"
        } else {
            ".venv/bin/python"
        };
        // Prefer a Python environment bundled near the bridge, then the repo venv.
        for root in bridge.ancestors().skip(1).take(4) {
            let candidate = root.join(name);
            if candidate.is_file() {
                return candidate;
            }
        }
        PathBuf::from(if cfg!(windows) { "python" } else { "python3" })
    }

    fn packaged_engine(directory: &Path) -> Option<PathBuf> {
        let name = if cfg!(windows) {
            "controlab-engine.exe"
        } else {
            "controlab-engine"
        };
        let path = directory.join("engine").join(name);
        path.is_file().then_some(path)
    }

    impl PythonProcess {
        fn start() -> Result<Self, String> {
            let engine = if let Some(path) = std::env::var_os("CONTROLAB_ENGINE") {
                Some(
                    PathBuf::from(path)
                        .canonicalize()
                        .map_err(|error| format!("Engine empacotada indisponível: {error}"))?,
                )
            } else {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().and_then(packaged_engine))
            };
            let mut command = if let Some(engine) = engine {
                Command::new(engine)
            } else {
                let bridge = bridge_path()?;
                let mut command = Command::new(python_path(&bridge));
                command.arg("-u").arg(&bridge);
                command
            };
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .env("PYTHONIOENCODING", "utf-8")
                .env("MPLBACKEND", "Agg");
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
                format!("Não foi possível iniciar Python: {error}. Configure CONTROLAB_PYTHON.")
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
                        Ok(0) => Err("Python encerrou a conexão.".into()),
                        Ok(_) if line.len() > MAX_REPLY + 1 || !line.ends_with(b"\n") => {
                            Err("Resposta Python inválida ou maior que 32 MiB.".into())
                        }
                        Ok(_) => serde_json::from_slice::<Value>(&line)
                            .map_err(|error| format!("Resposta JSON inválida: {error}"))
                            .and_then(|value| {
                                if value.is_object() {
                                    Ok(value)
                                } else {
                                    Err("A resposta Python deve ser um objeto JSON.".into())
                                }
                            }),
                        Err(error) => Err(format!("Falha ao ler Python: {error}")),
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
            let seconds = std::env::var("CONTROLAB_REQUEST_TIMEOUT")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(60)
                .clamp(1, 300);
            let deadline = Instant::now() + Duration::from_secs(seconds);
            let mut body = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
            body.push(b'\n');
            let stdin = self.stdin.clone();
            let (written, write_result) = mpsc::channel();
            // Writing large scripts can block too; the same request deadline covers it.
            thread::spawn(move || {
                let result = stdin
                    .lock()
                    .map_err(|_| "Canal Python indisponível.".to_string())
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
                    return Err(format!("Falha ao enviar para Python: {error}"));
                }
                match self.replies.recv_timeout(Duration::from_millis(50)) {
                    Ok(response) => return response,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err("Python encerrou a conexão.".into());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        }
    }

    pub(super) fn request(
        process: &mut Option<PythonProcess>,
        payload: Value,
        stop: &AtomicBool,
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
            *process = Some(PythonProcess::start()?);
        }
        let response = process.as_mut().unwrap().exchange(payload, stop);
        if let Err(error) = response {
            *process = None;
            return Err(format!("{error} {SESSION_LOST}"));
        }
        response
    }

    impl Drop for PythonProcess {
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
        use super::*;
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
