# ControLAB — Control Systems

A native **Rust + egui/eframe** interface, **Python** scientific engine and
**Rust** `.m` interpreter (`mscript`). The application uses no browser, Electron,
Capacitor, JavaScript, TypeScript, HTML or CSS.

[Português (Brasil)](README.pt-BR.md) · [Native interface](native/README.md)

## Linux development

Prepare Rust/Cargo and Python scientific dependencies:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
cargo build --workspace
cargo run --bin controlab
```

The `controlab` executable, which includes the `.m` interpreter, is built under
`target/debug/`. Use `--release` on build and run commands for an optimized
build; `--offline` uses cached Cargo dependencies. The local `.venv` interpreter
is selected automatically; `CONTROLAB_PYTHON` overrides it. Build scripts never
install dependencies automatically.

## Independent Linux distribution

```bash
.venv/bin/python -m pip install -r requirements-build.txt
.venv/bin/python scripts/package_native.py --release
env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 .venv/bin/python scripts/validate_native_bundle.py dist/native/controlab --ui
```

Distribute the entire `dist/native/controlab` folder, which includes the UI,
`.m` runtime and scientific Python engine. Target machines still need Linux
graphics libraries and a glibc version compatible with the build host.
The output folder must be empty; use `--output` to choose another folder.

## Installers and releases

GitHub Actions (`.github/workflows/build-all.yml`) builds, tests and uploads on
every push: `.deb`, `.rpm` and `.AppImage` (Ubuntu 24.04 runner, glibc 2.39+),
a per-user NSIS installer for Windows and an Android APK (arm64-v8a). Pushing a
`vX.Y.Z` tag sets that version (`scripts/set_version.py`) and publishes a GitHub
Release with all files. `native/Cargo.toml` is the single version source.

From an existing bundle, with `dpkg-deb`, `rpmbuild`, `appimagetool` or
`makensis` installed:

```bash
python3 scripts/package_installers.py deb rpm appimage   # Linux
python scripts/package_installers.py nsis                # Windows
```

Installers are written to `release/`. Linux packages install into `/opt/controlab`
with a `controlab` launcher, menu entry and icons. Without the secrets
`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` and
`ANDROID_KEY_PASSWORD`, the release APK is signed with the runner's debug key,
so installing a newer build over an older one requires uninstalling first.

## Source and checks

- `native/`: Rust UI, including the JSON bridge for `.m` scripts (`native/src/scripts.rs`).
- `src/python/`: scientific calculations and local JSON-lines transport; no HTTP server.
- `assets/`: canonical fonts, licenses, icons and translation data with a SHA-256 manifest.
- `crates/mscript/`: `.m` interpreter (numeric arrays, control models, plots, documents), run in-process by the UI.
- `scripts/`: packaging, version and source checks in Python; Rust builds use Cargo.
- `packaging/`: Linux desktop entry, Windows NSIS script and installer icons.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests
python3 scripts/check_native_sources.py
```

CI runs the full Rust/Python test suite and UI check on Linux; the Windows job
validates the packaged engine and a silent install. The APK is built in CI but
not exercised on a device.
Notes and screenshots of the removed Electron/Capacitor interface are kept in
`docs/history`.
