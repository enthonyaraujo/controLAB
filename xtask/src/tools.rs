//! Native distribution tools. No command installs build tools or Rust targets.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};
use toml_edit::{DocumentMut, value};

type Result<T> = std::result::Result<T, String>;
const FORBIDDEN: &[&str] = &[
    "js",
    "jsx",
    "ts",
    "tsx",
    "mjs",
    "cjs",
    "css",
    "html",
    "htm",
    "vue",
    "svelte",
    "webmanifest",
];
const GENERATED: &[&str] = &[
    ".git",
    ".venv",
    ".pytest_cache",
    "__pycache__",
    "target",
    "build",
    "dist",
    "graphify-out",
];
const CONFIGS: &[&str] = &[
    "package.json",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "capacitor.config.json",
];
const SUMMARY: &str = "Interactive suite for control systems engineering";
const DESCRIPTION: &str = "ControLAB: suíte interativa e didática de Engenharia e Sistemas de Controle. Lugar das raízes, Routh-Hurwitz, respostas no tempo e em frequência, projeto de controladores, espaço de estados e scripts .m.";
const HOMEPAGE: &str = "https://github.com/enthonyaraujo/controlab";
const MAINTAINER: &str = "Enthony Araujo <contato@enthony.com.br>";

pub fn run(args: &[String]) -> Result<()> {
    run_at(
        args,
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("Raiz do workspace ausente")?,
    )
}

/// Explicit root keeps filesystem tools independently testable without changing process globals.
pub fn run_at(args: &[String], root: &Path) -> Result<()> {
    let root = fs::canonicalize(root).map_err(err)?;
    let (command, rest) = args.split_first().ok_or("Comando ausente")?;
    if rest.iter().any(|arg| arg == "--help" || arg == "-h") {
        let usage = match command.as_str() {
            "package" => "package [--release] [--offline] [--skip-build] [--output DIR]",
            "installers" => {
                "installers [deb rpm appimage | nsis] [--bundle DIR] [--output DIR] [--version X.Y.Z] [--dry-run]"
            }
            "version" => "version [X.Y.Z]",
            "validate-bundle" => "validate-bundle BUNDLE [--ui]",
            "check-sources" => "check-sources [--root DIR]",
            "android" => {
                "android --output DIR [--abis arm64-v8a,x86_64] [--profile debug|release] [--offline]"
            }
            _ => return Err(format!("Comando desconhecido: {command}")),
        };
        println!("Uso: cargo xtask {usage}");
        return Ok(());
    }
    match command.as_str() {
        "package" => package(&root, rest),
        "installers" => installers(&root, rest),
        "version" => version(&root, rest),
        "validate-bundle" => validate_bundle(rest),
        "check-sources" => check_sources(&root, rest),
        "android" => android(&root, rest),
        _ => Err(format!("Comando desconhecido: {command}")),
    }
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}
fn write(path: &Path, data: impl AsRef<[u8]>) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(err)?;
    }
    fs::write(path, data).map_err(err)
}
fn parse_toml(path: &Path) -> Result<DocumentMut> {
    read(path)?.parse::<DocumentMut>().map_err(err)
}
fn json_file(path: &Path) -> Result<Value> {
    serde_json::from_str(&read(path)?).map_err(err)
}
fn current_version(root: &Path) -> Result<String> {
    parse_toml(&root.join("Cargo.toml"))?["workspace"]["package"]["version"]
        .as_str()
        .map(str::to_owned)
        .ok_or("Versão workspace.package.version ausente".into())
}

#[derive(Default)]
struct Options {
    values: BTreeMap<String, String>,
    flags: BTreeSet<String>,
    positional: Vec<String>,
}
impl Options {
    fn parse(args: &[String], values: &[&str], flags: &[&str]) -> Result<Self> {
        let mut out = Self::default();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if let Some((name, v)) = arg.split_once('=') {
                if !values.contains(&name) {
                    return Err(format!("Opção desconhecida: {name}"));
                }
                if out.values.insert(name.to_owned(), v.to_owned()).is_some() {
                    return Err(format!("Opção repetida: {name}"));
                }
            } else if values.contains(&arg.as_str()) {
                let v = iter
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| format!("Valor ausente para {arg}"))?;
                if out.values.insert(arg.clone(), v.clone()).is_some() {
                    return Err(format!("Opção repetida: {arg}"));
                }
            } else if flags.contains(&arg.as_str()) {
                out.flags.insert(arg.clone());
            } else if arg.starts_with('-') {
                return Err(format!("Opção desconhecida: {arg}"));
            } else {
                out.positional.push(arg.clone());
            }
        }
        Ok(out)
    }
    fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }
    fn get<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.values.get(name).map(String::as_str).unwrap_or(default)
    }
    fn no_positionals(&self) -> Result<()> {
        if self.positional.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Argumentos inesperados: {}",
                self.positional.join(" ")
            ))
        }
    }
}

// Resolve existing ancestors too, so a symlink cannot disguise the repository as output.
fn absolute(path: &Path) -> Result<PathBuf> {
    let input = if path.is_absolute() {
        path.to_owned()
    } else {
        env::current_dir().map_err(err)?.join(path)
    };
    let mut clean = PathBuf::new();
    for c in input.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                clean.pop();
            }
            _ => clean.push(c.as_os_str()),
        }
    }
    let mut ancestor = clean.clone();
    let mut tail = vec![];
    while !ancestor.exists() {
        tail.push(ancestor.file_name().ok_or("Caminho inválido")?.to_owned());
        ancestor.pop();
    }
    let mut resolved = fs::canonicalize(ancestor).map_err(err)?;
    for name in tail.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}
fn empty_output(root: &Path, output: &Path) -> Result<PathBuf> {
    let output = absolute(output)?;
    if root.starts_with(&output) {
        return Err("A saída não pode conter o repositório de origem.".into());
    }
    if output.exists() && (!output.is_dir() || fs::read_dir(&output).map_err(err)?.next().is_some())
    {
        return Err(format!(
            "A pasta de saída deve estar vazia para preservar arquivos existentes: {}",
            output.display()
        ));
    }
    Ok(output)
}
fn checked(command: &mut Command) -> Result<()> {
    let status = command
        .status()
        .map_err(|e| format!("Falha ao iniciar {:?}: {e}", command.get_program()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Comando {:?} falhou: {status}",
            command.get_program()
        ))
    }
}
fn copy_file(source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(err)?;
    }
    fs::copy(source, dest)
        .map_err(|e| format!("{} → {}: {e}", source.display(), dest.display()))?;
    Ok(())
}
fn copy_tree(source: &Path, dest: &Path, exclude_screenshot: bool) -> Result<()> {
    fs::create_dir_all(dest).map_err(err)?;
    for entry in fs::read_dir(source).map_err(err)? {
        let entry = entry.map_err(err)?;
        if exclude_screenshot && entry.file_name() == "validation-ui.png" {
            continue;
        }
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let ty = entry.file_type().map_err(err)?;
        if ty.is_symlink() {
            let target = fs::read_link(&from).map_err(err)?;
            symlink(&target, &to, from.is_dir())?;
        } else if ty.is_dir() {
            copy_tree(&from, &to, false)?;
        } else if ty.is_file() {
            copy_file(&from, &to)?;
        }
    }
    Ok(())
}
#[cfg(unix)]
fn symlink(from: &Path, to: &Path, _directory: bool) -> Result<()> {
    std::os::unix::fs::symlink(from, to).map_err(err)
}
#[cfg(windows)]
fn symlink(from: &Path, to: &Path, directory: bool) -> Result<()> {
    if directory {
        std::os::windows::fs::symlink_dir(from, to)
    } else {
        std::os::windows::fs::symlink_file(from, to)
    }
    .map_err(err)
}
#[cfg(not(any(unix, windows)))]
fn symlink(_from: &Path, _to: &Path, _directory: bool) -> Result<()> {
    Err("Links simbólicos não suportados".into())
}
fn executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(err)?;
    }
    let _ = path;
    Ok(())
}
fn platform() -> &'static str {
    if cfg!(windows) {
        "win32"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "darwin"
    }
}
fn suffix() -> &'static str {
    if cfg!(windows) { ".exe" } else { "" }
}

fn package(root: &Path, args: &[String]) -> Result<()> {
    let opts = Options::parse(
        args,
        &["--output"],
        &["--release", "--offline", "--skip-build"],
    )?;
    opts.no_positionals()?;
    let output = empty_output(
        root,
        &PathBuf::from(
            opts.get(
                "--output",
                root.join("dist/native/controlab")
                    .to_str()
                    .ok_or("Caminho inválido")?,
            ),
        ),
    )?;
    if !opts.flag("--skip-build") {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root)
            .args(["build", "--locked", "--workspace"]);
        if opts.flag("--release") {
            cmd.arg("--release");
        }
        if opts.flag("--offline") {
            cmd.arg("--offline");
        }
        checked(&mut cmd)?;
    }
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let target = if target.is_absolute() {
        target
    } else {
        root.join(target)
    };
    let profile = if opts.flag("--release") {
        "release"
    } else {
        "debug"
    };
    let built = target.join(profile);
    let ui_name = format!("controlab{}", suffix());
    let engine_name = format!("controlab-engine{}", suffix());
    for name in [&ui_name, &engine_name] {
        if !built.join(name).is_file() {
            return Err(format!(
                "Executável ausente: {}. Compile o workspace Cargo primeiro.",
                built.join(name).display()
            ));
        }
    }
    let version = current_version(root)?;
    if !root.join("assets/manifest.json").is_file() {
        return Err("Manifesto de assets ausente".into());
    }
    // Check again after a potentially long build before creating any output.
    empty_output(root, &output)?;
    fs::create_dir_all(&output).map_err(err)?;
    copy_file(&built.join(&ui_name), &output.join(&ui_name))?;
    copy_file(
        &built.join(&engine_name),
        &output.join("engine").join(&engine_name),
    )?;
    copy_tree(
        &root.join("assets"),
        &output.join("resources/assets"),
        false,
    )?;
    copy_file(&root.join("LICENSE"), &output.join("LICENSE"))?;
    let manifest = json!({"version":version,"platform":platform(),"profile":profile,"ui":ui_name,"engine":format!("engine/{engine_name}"),"resources":"resources/assets"});
    write(
        &output.join("bundle.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).map_err(err)?
        ),
    )?;
    println!("{}", output.display());
    Ok(())
}

fn valid_version(input: &str) -> Result<&str> {
    let v = input.strip_prefix('v').unwrap_or(input);
    let parts: Vec<_> = v.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit()))
    {
        Err(format!("Versão inválida (esperado X.Y.Z): {input}"))
    } else {
        Ok(v)
    }
}
fn member_paths(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut paths = vec![root.to_owned()];
    for part in Path::new(pattern).components() {
        let part = part.as_os_str();
        let mut next = vec![];
        for base in paths {
            if part == "*" {
                for entry in fs::read_dir(base).map_err(err)? {
                    let entry = entry.map_err(err)?;
                    if entry.path().is_dir() {
                        next.push(entry.path());
                    }
                }
            } else {
                next.push(base.join(part));
            }
        }
        paths = next;
    }
    Ok(paths)
}
fn version(root: &Path, args: &[String]) -> Result<()> {
    let opts = Options::parse(args, &[], &[])?;
    if opts.positional.is_empty() {
        println!("{}", current_version(root)?);
        return Ok(());
    }
    if opts.positional.len() != 1 {
        return Err("Uso: cargo xtask version [X.Y.Z]".into());
    }
    let version = valid_version(&opts.positional[0])?;
    let manifest_path = root.join("Cargo.toml");
    let mut manifest = parse_toml(&manifest_path)?;
    let members = manifest["workspace"]["members"]
        .as_array()
        .ok_or("workspace.members ausente")?;
    let mut names = BTreeSet::new();
    for member in members {
        for path in member_paths(root, member.as_str().ok_or("Membro workspace inválido")?)? {
            let crate_manifest = parse_toml(&path.join("Cargo.toml"))?;
            if crate_manifest["package"]["version"]["workspace"].as_bool() == Some(true) {
                names.insert(
                    crate_manifest["package"]["name"]
                        .as_str()
                        .ok_or("Crate sem nome")?
                        .to_owned(),
                );
            }
        }
    }
    manifest["workspace"]["package"]["version"] = value(version);
    let lock_path = root.join("Cargo.lock");
    let mut lock = parse_toml(&lock_path)?;
    let mut found = BTreeSet::new();
    for entry in lock["package"]
        .as_array_of_tables_mut()
        .ok_or("Cargo.lock sem pacotes")?
        .iter_mut()
    {
        if entry.get("source").is_none()
            && let Some(name) = entry["name"].as_str()
            && names.contains(name)
        {
            found.insert(name.to_owned());
            entry["version"] = value(version);
        }
    }
    if found != names {
        return Err(format!(
            "Pacotes workspace ausentes em Cargo.lock: {:?}",
            names.difference(&found).collect::<Vec<_>>()
        ));
    }
    // Parse and validate all files before the first write.
    write(&manifest_path, manifest.to_string())?;
    write(&lock_path, lock.to_string())?;
    println!("{version}");
    Ok(())
}

fn forbidden(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| FORBIDDEN.contains(&e.to_ascii_lowercase().as_str()))
}
fn source_walk(root: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(current).map_err(err)? {
        let e = entry.map_err(err)?;
        let name = e.file_name().to_string_lossy().to_string();
        let ty = e.file_type().map_err(err)?;
        if name == "node_modules" && (ty.is_dir() || ty.is_symlink()) {
            out.push(e.path().strip_prefix(root).map_err(err)?.to_owned());
        } else if ty.is_dir() {
            if !GENERATED.contains(&name.as_str()) {
                source_walk(root, &e.path(), out)?;
            }
        } else if forbidden(&e.path()) || CONFIGS.contains(&name.as_str()) {
            out.push(e.path().strip_prefix(root).map_err(err)?.to_owned());
        }
    }
    Ok(())
}
fn check_sources(root: &Path, args: &[String]) -> Result<()> {
    let opts = Options::parse(args, &["--root"], &[])?;
    opts.no_positionals()?;
    let root = absolute(&PathBuf::from(
        opts.get("--root", root.to_str().ok_or("Caminho inválido")?),
    ))?;
    let mut files = vec![];
    source_walk(&root, &root, &mut files)?;
    files.sort();
    if !files.is_empty() {
        return Err(format!(
            "Arquivos de frontend/Node encontrados:\n{}",
            files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    println!(
        "Native source tree validated: no JavaScript, TypeScript, CSS, HTML or Node dependencies."
    );
    Ok(())
}

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new(prefix: &str) -> Result<Self> {
        for _ in 0..100 {
            let path = env::temp_dir().join(format!(
                "{prefix}-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(err(e)),
            }
        }
        Err("Não foi possível criar diretório temporário".into())
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn walk_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in fs::read_dir(root).map_err(err)? {
        let e = e.map_err(err)?;
        let ty = e.file_type().map_err(err)?;
        if ty.is_dir() {
            walk_files(&e.path(), out)?;
        } else {
            out.push(e.path());
        }
    }
    Ok(())
}
fn tree_size(root: &Path) -> Result<u64> {
    let mut files = vec![];
    walk_files(root, &mut files)?;
    let mut size = 0;
    for path in files {
        let m = fs::symlink_metadata(&path).map_err(err)?;
        if m.is_file() {
            size += m.len();
        }
    }
    Ok(size)
}
fn required_glibc(root: &Path) -> Result<String> {
    let mut files = vec![];
    walk_files(root, &mut files)?;
    let mut minor = 17u32;
    for path in files {
        if fs::symlink_metadata(&path)
            .map_err(err)?
            .file_type()
            .is_symlink()
        {
            continue;
        }
        let data = fs::read(path).map_err(err)?;
        if !data.starts_with(b"\x7fELF") {
            continue;
        }
        for (i, _) in data
            .windows(8)
            .enumerate()
            .filter(|(_, p)| *p == b"GLIBC_2.")
        {
            let digits = data[i + 8..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .copied()
                .collect::<Vec<_>>();
            if let Ok(s) = std::str::from_utf8(&digits) {
                minor = minor.max(s.parse().unwrap_or(0));
            }
        }
    }
    Ok(format!("2.{minor}"))
}
pub(crate) fn stage_linux(root: &Path, bundle: &Path, stage: &Path) -> Result<()> {
    copy_tree(bundle, &stage.join("opt/controlab"), true)?;
    fs::create_dir_all(stage.join("usr/bin")).map_err(err)?;
    symlink(
        Path::new("/opt/controlab/controlab"),
        &stage.join("usr/bin/controlab"),
        false,
    )?;
    copy_file(
        &root.join("packaging/linux/controlab.desktop"),
        &stage.join("usr/share/applications/controlab.desktop"),
    )?;
    for entry in fs::read_dir(root.join("packaging/icons/linux")).map_err(err)? {
        let path = entry.map_err(err)?.path();
        if path.extension().is_some_and(|e| e == "png") {
            let stem = path.file_stem().ok_or("Ícone inválido")?;
            copy_file(
                &path,
                &stage
                    .join("usr/share/icons/hicolor")
                    .join(stem)
                    .join("apps/controlab.png"),
            )?;
        }
    }
    Ok(())
}
fn tool_exists(tool: &str) -> bool {
    let path = Path::new(tool);
    if path.components().count() > 1 {
        return path.is_file();
    }
    env::split_paths(&env::var_os("PATH").unwrap_or_default()).any(|dir| {
        dir.join(tool).is_file()
            || (cfg!(windows)
                && ["exe", "cmd", "bat"]
                    .iter()
                    .any(|ext| dir.join(format!("{tool}.{ext}")).is_file()))
    })
}
fn invoke_installer(command: &mut Command, dry: bool, output: &Path) -> Result<()> {
    if dry {
        println!("dry-run: {:?}", command);
        Ok(())
    } else {
        checked(command)?;
        if !output.is_file() {
            Err(format!("Instalador não foi gerado: {}", output.display()))
        } else {
            Ok(())
        }
    }
}
fn installers(root: &Path, args: &[String]) -> Result<()> {
    let opts = Options::parse(args, &["--bundle", "--output", "--version"], &["--dry-run"])?;
    let dry = opts.flag("--dry-run");
    let native = if cfg!(windows) {
        vec!["nsis"]
    } else if cfg!(target_os = "linux") {
        vec!["deb", "rpm", "appimage"]
    } else {
        vec![]
    };
    let formats = if opts.positional.is_empty() {
        native.clone()
    } else {
        opts.positional.iter().map(String::as_str).collect()
    };
    if formats.is_empty() {
        return Err("Host de instaladores não suportado".into());
    }
    for format in &formats {
        if !["deb", "rpm", "appimage", "nsis"].contains(format) {
            return Err(format!("Formato inválido: {format}"));
        }
        if !native.contains(format) {
            return Err(
                "Pacotes Linux são gerados no Linux e o instalador NSIS no Windows.".into(),
            );
        }
    }
    let bundle = absolute(&PathBuf::from(
        opts.get(
            "--bundle",
            root.join("dist/native/controlab")
                .to_str()
                .ok_or("Caminho inválido")?,
        ),
    ))?;
    if !bundle.join("bundle.json").is_file() {
        return Err(format!(
            "Pacote nativo ausente: {}. Execute cargo xtask package primeiro.",
            bundle.display()
        ));
    }
    let output = empty_output(
        root,
        &PathBuf::from(opts.get(
            "--output",
            root.join("release").to_str().ok_or("Caminho inválido")?,
        )),
    )?;
    if bundle.starts_with(&output) || output.starts_with(&bundle) {
        return Err("A saída dos instaladores não pode sobrepor o bundle.".into());
    }
    let tools: BTreeMap<_, _> = formats
        .iter()
        .map(|f| {
            (
                *f,
                match *f {
                    "deb" => "dpkg-deb".to_owned(),
                    "rpm" => "rpmbuild".to_owned(),
                    "nsis" => "makensis".to_owned(),
                    _ => env::var("APPIMAGETOOL").unwrap_or_else(|_| "appimagetool".into()),
                },
            )
        })
        .collect();
    if !dry {
        for tool in tools.values() {
            if !tool_exists(tool) {
                return Err(format!("Ferramenta ausente: {tool}"));
            }
        }
    }
    let version = current_version(root)?;
    let version = valid_version(opts.get("--version", &version))?;
    let machine = env::consts::ARCH;
    let arch = if cfg!(windows) && machine == "x86_64" {
        "x64"
    } else {
        machine
    };
    if !["x86_64", "aarch64", "x64", "arm64"].contains(&arch) {
        return Err(format!("Arquitetura não suportada: {arch}"));
    }
    fs::create_dir_all(&output).map_err(err)?;
    for format in formats {
        let temp = Temp::new(&format!("controlab-{format}"))?;
        let temp = &temp.0;
        let package = match format {
            "deb" => {
                let stage = temp.join("deb");
                stage_linux(root, &bundle, &stage)?;
                let deb_arch = if arch == "x86_64" { "amd64" } else { "arm64" };
                let glibc = required_glibc(&stage)?;
                let size = tree_size(&stage)? / 1024;
                write(
                    &stage.join("DEBIAN/control"),
                    format!(
                        "Package: controlab\nVersion: {version}\nArchitecture: {deb_arch}\nMaintainer: {MAINTAINER}\nInstalled-Size: {size}\nDepends: libc6 (>= {glibc}), libgcc-s1, libgl1, libegl1, libxkbcommon0, libxkbcommon-x11-0\nRecommends: libwayland-client0, libwayland-egl1, xdg-desktop-portal\nSection: science\nPriority: optional\nHomepage: {HOMEPAGE}\nDescription: {SUMMARY}\n {DESCRIPTION}\n"
                    ),
                )?;
                let package = output.join(format!("ControLAB-v{version}-linux-{deb_arch}.deb"));
                invoke_installer(
                    Command::new(&tools[format])
                        .args(["--root-owner-group", "-Zxz", "--build"])
                        .arg(&stage)
                        .arg(&package),
                    dry,
                    &package,
                )?;
                package
            }
            "rpm" => {
                let stage = temp.join("rpm-root");
                stage_linux(root, &bundle, &stage)?;
                let glibc = required_glibc(&stage)?;
                let name = format!("ControLAB-v{version}-linux-{arch}.rpm");
                let spec = temp.join("controlab.spec");
                write(
                    &spec,
                    format!(
                        "%global debug_package %{{nil}}\n%global __os_install_post %{{nil}}\n%global __arch_install_post %{{nil}}\n%global _build_id_links none\nName: controlab\nVersion: {version}\nRelease: 1\nSummary: {SUMMARY}\nLicense: MIT\nURL: {HOMEPAGE}\nAutoReqProv: no\nRequires: libc.so.6(GLIBC_{glibc})(64bit), libgcc_s.so.1()(64bit)\nRequires: libGL.so.1()(64bit), libEGL.so.1()(64bit), libxkbcommon.so.0()(64bit), libxkbcommon-x11.so.0()(64bit)\nRecommends: xdg-desktop-portal\n\n%description\n{DESCRIPTION}\n\n%install\ncp -a '{}'/. %{{buildroot}}/\n\n%files\n/opt/controlab\n/usr/bin/controlab\n/usr/share/applications/controlab.desktop\n/usr/share/icons/hicolor/*/apps/controlab.png\n",
                        stage.display().to_string().replace('\'', "'\\''")
                    ),
                )?;
                let package = output.join(&name);
                invoke_installer(
                    Command::new(&tools[format])
                        .args(["-bb", "--target", arch, "--define"])
                        .arg(format!("_topdir {}", temp.join("rpmbuild").display()))
                        .arg("--define")
                        .arg(format!("_rpmdir {}", output.display()))
                        .arg("--define")
                        .arg(format!("_rpmfilename {name}"))
                        .arg(spec),
                    dry,
                    &package,
                )?;
                package
            }
            "appimage" => {
                let appdir = temp.join("ControLAB.AppDir");
                copy_tree(&bundle, &appdir.join("opt/controlab"), true)?;
                copy_file(
                    &root.join("packaging/linux/controlab.desktop"),
                    &appdir.join("controlab.desktop"),
                )?;
                copy_file(
                    &root.join("packaging/icons/linux/256x256.png"),
                    &appdir.join("controlab.png"),
                )?;
                symlink(Path::new("controlab.png"), &appdir.join(".DirIcon"), false)?;
                let apprun = appdir.join("AppRun");
                write(
                    &apprun,
                    "#!/bin/sh\nHERE=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$HERE/opt/controlab/controlab\" \"$@\"\n",
                )?;
                executable(&apprun)?;
                let package = output.join(format!("ControLAB-v{version}-linux-{arch}.AppImage"));
                invoke_installer(
                    Command::new(&tools[format])
                        .arg("--no-appstream")
                        .arg(appdir)
                        .arg(&package)
                        .env("ARCH", arch),
                    dry,
                    &package,
                )?;
                package
            }
            "nsis" => {
                let source = temp.join("ControLAB");
                copy_tree(&bundle, &source, true)?;
                let package = output.join(format!("ControLAB-v{version}-win-{arch}.exe"));
                invoke_installer(
                    Command::new(&tools[format])
                        .args(["/V3", "/INPUTCHARSET", "UTF8"])
                        .arg(format!("/DVERSION={version}"))
                        .arg(format!("/DSOURCE_DIR={}", source.display()))
                        .arg(format!(
                            "/DICON={}",
                            root.join("packaging/icons/icon.ico").display()
                        ))
                        .arg(format!("/DLICENSE={}", root.join("LICENSE").display()))
                        .arg(format!("/DOUTFILE={}", package.display()))
                        .arg(root.join("packaging/windows/controlab.nsi")),
                    dry,
                    &package,
                )?;
                package
            }
            _ => unreachable!(),
        };
        println!("{}", package.display());
    }
    Ok(())
}

fn bundle_path(bundle: &Path, metadata: &Value, key: &str) -> Result<PathBuf> {
    let relative = Path::new(
        metadata[key]
            .as_str()
            .ok_or_else(|| format!("Campo {key} ausente em bundle.json"))?,
    );
    if relative.is_absolute()
        || relative.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("Caminho {key} deve permanecer no bundle"));
    }
    let path = fs::canonicalize(bundle.join(relative)).map_err(err)?;
    if !path.starts_with(bundle) {
        return Err(format!("Caminho {key} sai do bundle"));
    }
    Ok(path)
}
fn isolated_command(program: &Path, temp: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(temp)
        .env_remove("PYTHONPATH")
        .env_remove("CONTROLAB_ASSETS")
        .env_remove("CONTROLAB_ENGINE")
        .env_remove("CONTROLAB_SCREENSHOT_SCRIPT")
        .env_remove("CONTROLAB_SCREENSHOT_PAGE")
        .env("CONTROLAB_PYTHON", temp.join("no-python"))
        .env("CONTROLAB_NATIVE_BRIDGE", temp.join("no-source-bridge"))
        .env("PYTHONIOENCODING", "utf-8");
    cmd
}
fn captured_timeout(command: &mut Command, input: Vec<u8>, timeout: Duration) -> Result<Vec<u8>> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(err)?;
    let mut stdin = child.stdin.take().ok_or("stdin ausente")?;
    let writer = thread::spawn(move || {
        let result = stdin.write_all(&input);
        drop(stdin);
        result
    });
    let mut stdout = child.stdout.take().ok_or("stdout ausente")?;
    let mut stderr = child.stderr.take().ok_or("stderr ausente")?;
    let reader = thread::spawn(move || {
        let mut data = Vec::new();
        stdout.read_to_end(&mut data).map(|_| data)
    });
    let errors = thread::spawn(move || {
        let mut data = Vec::new();
        stderr.read_to_end(&mut data).map(|_| data)
    });
    let start = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(err)? {
            Some(status) => break status,
            None if start.elapsed() >= timeout => {
                timed_out = true;
                let _ = child.kill();
                break child.wait().map_err(err)?;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    };
    let written = writer.join().map_err(|_| "Falha na thread stdin")?;
    let output = reader
        .join()
        .map_err(|_| "Falha na thread stdout")?
        .map_err(err)?;
    let errors = errors
        .join()
        .map_err(|_| "Falha na thread stderr")?
        .map_err(err)?;
    if timed_out {
        return Err(format!("Tempo limite excedido ({timeout:?})"));
    }
    if !status.success() {
        return Err(format!(
            "Executável falhou: {status}\n{}",
            String::from_utf8_lossy(&errors)
        ));
    }
    written.map_err(err)?;
    Ok(output)
}
fn validate_bundle(args: &[String]) -> Result<()> {
    let opts = Options::parse(args, &[], &["--ui"])?;
    if opts.positional.len() != 1 {
        return Err("Uso: cargo xtask validate-bundle BUNDLE [--ui]".into());
    }
    let bundle = fs::canonicalize(&opts.positional[0]).map_err(err)?;
    let metadata = json_file(&bundle.join("bundle.json"))?;
    let resources = bundle_path(&bundle, &metadata, "resources")?;
    let manifest = json_file(&resources.join("manifest.json"))?;
    for (name, digest) in manifest["files"]
        .as_object()
        .ok_or("Manifesto de recursos sem files")?
    {
        let resource = Path::new(name);
        if resource.is_absolute()
            || resource.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(format!("Recurso fora do bundle: {name}"));
        }
        let path = fs::canonicalize(resources.join(resource)).map_err(err)?;
        if !path.starts_with(&resources) {
            return Err(format!("Recurso fora do bundle: {name}"));
        }
        let actual = format!("{:x}", Sha256::digest(fs::read(path).map_err(err)?));
        if digest.as_str() != Some(&actual) {
            return Err(format!("SHA256 divergente: {name}"));
        }
    }
    let mut files = vec![];
    walk_files(&bundle, &mut files)?;
    for file in files {
        if forbidden(&file) {
            return Err(format!("Web resource in native bundle: {}", file.display()));
        }
    }
    let requests = [
        json!({"action":"preview","expr":"1/(s+1)"}),
        json!({"action":"time_response","expr":"1/(s+1)","language":"en","points":60}),
        json!({"action":"render_formulas","sources":["\\frac{1}{s+1}"]}),
        json!({"action":"calculate","expr":"1/(s*(s+1))"}),
        json!({"action":"routh_hurwitz","expr":"s^3+3*s^2+2*s+1","has_k":false}),
        json!({"action":"frequency_response","expr":"1/(s+1)","points":150}),
        json!({"action":"controller_design","plant_expr":"5/(s*(s+1))","controller_type":"lead","ts":2.0,"xi":0.5,"points":150}),
        json!({"action":"state_space","mode":"matrices","A":"[[0,1],[-2,-3]]","B":"[[0],[1]]","C":"[[1,0]]","D":"[[0]]"}),
    ];
    let input = requests
        .iter()
        .map(|r| format!("{r}\n"))
        .collect::<String>()
        .into_bytes();
    let temp = Temp::new("controlab-bundle-check")?;
    let engine = bundle_path(&bundle, &metadata, "engine")?;
    let stdout = captured_timeout(
        &mut isolated_command(&engine, &temp.0),
        input,
        Duration::from_secs(90),
    )?;
    let replies = String::from_utf8(stdout)
        .map_err(err)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    if replies.len() != requests.len() {
        return Err(format!(
            "Esperadas {} respostas, recebidas {}",
            requests.len(),
            replies.len()
        ));
    }
    for (request, reply) in requests.iter().zip(&replies) {
        if reply["success"] != true {
            return Err(format!("Falha no motor {}: {reply}", request["action"]));
        }
    }
    if !replies[1]["title"]
        .as_str()
        .is_some_and(|s| s.contains("Time"))
    {
        return Err("Resposta de tempo sem título em inglês".into());
    }
    if !replies[2]["images"][0]
        .as_str()
        .is_some_and(|s| !s.is_empty())
    {
        return Err("Renderização de fórmulas sem imagem".into());
    }
    if opts.flag("--ui") {
        let ui = bundle_path(&bundle, &metadata, "ui")?;
        let screenshot = bundle.join("validation-ui.png");
        let mut cmd = isolated_command(&ui, &temp.0);
        // A stale screenshot must never satisfy a failed capture.
        if screenshot.exists() {
            fs::remove_file(&screenshot).map_err(err)?;
        }
        cmd.env("CONTROLAB_SCREENSHOT", &screenshot)
            .env("CONTROLAB_SCREENSHOT_PAGE", "time")
            .env("CONTROLAB_SCREENSHOT_SIZE", "1024x768")
            .env("CONTROLAB_SCREENSHOT_THEME", "dark");
        captured_timeout(&mut cmd, vec![], Duration::from_secs(60))?;
        check_png(&screenshot)?;
        let script = temp.0.join("validation-script.png");
        let mut cmd = isolated_command(&ui, &temp.0);
        cmd.env("CONTROLAB_SCREENSHOT", &script)
            .env("CONTROLAB_SCREENSHOT_SIZE", "1024x768")
            .env("CONTROLAB_SCREENSHOT_THEME", "dark")
            .env_remove("CONTROLAB_SCREENSHOT_PAGE")
            .env("CONTROLAB_ENGINE", temp.0.join("no-engine"))
            .env(
                "CONTROLAB_SCREENSHOT_SCRIPT",
                "A=[1 2;3 4];\ndisp(A*A)\nstep(tf([1],[1 2 1]))",
            );
        captured_timeout(&mut cmd, vec![], Duration::from_secs(60))?;
        check_png(&script)?;
    }
    println!(
        "Bundle validated: independent resources, all six scientific engines and formulas{}",
        if opts.flag("--ui") {
            ", native UI and in-process .m scripts"
        } else {
            ""
        }
    );
    Ok(())
}
fn check_png(path: &Path) -> Result<()> {
    if fs::read(path)
        .map_err(err)?
        .starts_with(b"\x89PNG\r\n\x1a\n")
    {
        Ok(())
    } else {
        Err(format!("Captura PNG inválida: {}", path.display()))
    }
}

fn find_ndk(root: &Path) -> Result<PathBuf> {
    for name in [
        "ANDROID_NDK_HOME",
        "ANDROID_NDK_ROOT",
        "ANDROID_NDK_LATEST_HOME",
        "ANDROID_NDK",
    ] {
        if let Some(path) = env::var_os(name).filter(|p| !p.is_empty()) {
            return Ok(PathBuf::from(path));
        }
    }
    let mut sdks = vec![];
    for name in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Some(path) = env::var_os(name).filter(|p| !p.is_empty()) {
            sdks.push(PathBuf::from(path));
        }
    }
    let properties = root.join("native/android/local.properties");
    if properties.exists() {
        for line in read(&properties)?.lines() {
            if let Some(path) = line.strip_prefix("sdk.dir=") {
                sdks.push(PathBuf::from(
                    path.replace("\\:", ":").replace("\\\\", "\\"),
                ));
            }
        }
    }
    if let Some(home) = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) {
        sdks.push(PathBuf::from(home).join("Android/Sdk"));
    }
    for sdk in sdks {
        let folder = sdk.join("ndk");
        if folder.is_dir() {
            let mut versions = fs::read_dir(folder)
                .map_err(err)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect::<Vec<_>>();
            versions.sort_by_key(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .split('.')
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect::<Vec<_>>()
            });
            if let Some(ndk) = versions.pop() {
                return Ok(ndk);
            }
        }
    }
    Err("NDK não encontrado. Configure ANDROID_NDK_HOME ou ANDROID_HOME.".into())
}
/// Android/Gradle outputs are incremental: unrelated files and other ABIs survive.
pub(crate) fn android_output(root: &Path, output: &Path) -> Result<PathBuf> {
    let output = absolute(output)?;
    if root.starts_with(&output) {
        return Err("A saída não pode conter o repositório de origem.".into());
    }
    if output.exists() && !output.is_dir() {
        return Err(format!(
            "A saída Android deve ser uma pasta: {}",
            output.display()
        ));
    }
    Ok(output)
}

fn android_library_path(output: &Path, abi: &str, name: &str) -> Result<PathBuf> {
    if !["arm64-v8a", "x86_64"].contains(&abi)
        || !["libcontrolab_native.so", "libc++_shared.so"].contains(&name)
    {
        return Err("Destino Android não é uma biblioteca compilada de ABI conhecida".into());
    }
    let output = absolute(output)?;
    let directory = output.join(abi);
    if absolute(&directory)? != directory {
        return Err(
            "A pasta de ABI Android não pode redirecionar a saída por link simbólico".into(),
        );
    }
    let destination = directory.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&destination)
        && !metadata.file_type().is_file()
    {
        return Err(format!(
            "Biblioteca Android não pode substituir link ou diretório: {}",
            destination.display()
        ));
    }
    Ok(destination)
}

pub(crate) fn copy_android_library(
    output: &Path,
    abi: &str,
    source: &Path,
    name: &str,
) -> Result<()> {
    let destination = android_library_path(output, abi, name)?;
    let directory = destination.parent().ok_or("Destino Android inválido")?;
    fs::create_dir_all(directory).map_err(err)?;
    // Replace the library path instead of truncating a possible hard-linked source.
    let temporary = directory.join(format!(
        ".{name}.controlab-{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(err)?;
    drop(handle);
    let result = (|| {
        fs::copy(source, &temporary).map_err(err)?;
        #[cfg(windows)]
        if destination.exists() {
            fs::remove_file(&destination).map_err(err)?;
        }
        fs::rename(&temporary, &destination).map_err(err)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn android(root: &Path, args: &[String]) -> Result<()> {
    let opts = Options::parse(args, &["--abis", "--profile", "--output"], &["--offline"])?;
    opts.no_positionals()?;
    let output = opts.values.get("--output").ok_or("--output obrigatório")?;
    let output = android_output(root, Path::new(output))?;
    let profile = opts.get("--profile", "debug");
    if !["debug", "release"].contains(&profile) {
        return Err("--profile aceita debug ou release".into());
    }
    let mut targets = vec![];
    for abi in opts.get("--abis", "arm64-v8a").split(',') {
        let target = match abi {
            "arm64-v8a" => "aarch64-linux-android",
            "x86_64" => "x86_64-linux-android",
            _ => return Err("ABIs aceitas: arm64-v8a, x86_64".into()),
        };
        if targets.iter().any(|(a, _)| *a == abi) {
            return Err(format!("ABI repetida: {abi}"));
        }
        android_library_path(&output, abi, "libcontrolab_native.so")?;
        targets.push((abi, target));
    }
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(err)?;
    if !installed.status.success() {
        return Err("Não foi possível consultar alvos Rust instalados".into());
    }
    let installed = String::from_utf8(installed.stdout).map_err(err)?;
    for (_, target) in &targets {
        if !installed.lines().any(|line| line == *target) {
            return Err(format!(
                "Alvo Rust ausente (instale explicitamente antes de compilar): {target}"
            ));
        }
    }
    let host = if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else if cfg!(target_os = "macos") {
        "darwin-x86_64"
    } else if cfg!(windows) {
        "windows-x86_64"
    } else {
        return Err("Host NDK não suportado".into());
    };
    let ndk = find_ndk(root)?;
    let toolchain = ndk.join("toolchains/llvm/prebuilt").join(host).join("bin");
    let mut linkers = vec![];
    for (_, triple) in &targets {
        let linker = toolchain.join(format!(
            "{triple}24-clang{}",
            if cfg!(windows) { ".cmd" } else { "" }
        ));
        if !linker.is_file() {
            return Err(format!("Linker NDK ausente: {}", linker.display()));
        }
        linkers.push(linker);
    }
    let target_dir = root.join("build/native-android");
    for ((abi, triple), linker) in targets.into_iter().zip(linkers) {
        let mut paths = vec![toolchain.clone()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        let path: OsString = env::join_paths(paths).map_err(err)?;
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root)
            .args(["build", "--locked", "--target", triple, "--target-dir"])
            .arg(&target_dir)
            .args(["-p", "controlab-native", "--lib"])
            .env(
                format!(
                    "CARGO_TARGET_{}_LINKER",
                    triple.to_uppercase().replace('-', "_")
                ),
                &linker,
            )
            .env(format!("CC_{}", triple.replace('-', "_")), &linker)
            .env(
                format!("AR_{}", triple.replace('-', "_")),
                toolchain.join(if cfg!(windows) {
                    "llvm-ar.exe"
                } else {
                    "llvm-ar"
                }),
            )
            .env("PATH", path);
        if profile == "release" {
            cmd.arg("--release");
        }
        if opts.flag("--offline") {
            cmd.arg("--offline");
        }
        checked(&mut cmd)?;
        let destination = output.join(abi);
        copy_android_library(
            &output,
            abi,
            &target_dir
                .join(triple)
                .join(profile)
                .join("libcontrolab_native.so"),
            "libcontrolab_native.so",
        )?;
        let cxx = toolchain
            .parent()
            .ok_or("Toolchain inválida")?
            .join("sysroot/usr/lib")
            .join(triple)
            .join("libc++_shared.so");
        if cxx.exists() {
            copy_android_library(&output, abi, &cxx, "libc++_shared.so")?;
        }
        println!(
            "[ControLAB] {abi}: interface Rust e interpretador .m em {}",
            destination.display()
        );
    }
    Ok(())
}
