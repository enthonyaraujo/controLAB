# ControLAB — Control Systems

A native **Rust + egui/eframe** interface, **Python** scientific engine and
**Rust/openMAT** `.m` interpreter. The application uses no browser, Electron,
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

Both native executables are built under `target/debug/`. Use `--release` on
build and run commands for an optimized build; `--offline` uses cached Cargo
dependencies. The local `.venv` interpreter is selected automatically.
`CONTROLAB_PYTHON` and `CONTROLAB_RUNTIME` override the scientific interpreter
and `.m` runtime. Build scripts never install dependencies automatically.

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

## Source and checks

- `native/`: Rust UI and Cargo runtime adapter.
- `src/python/`: scientific calculations and local JSON-lines transport; no HTTP server.
- `assets/`: canonical fonts, licenses, icons and translation data with a SHA-256 manifest.
- `vendor/openmat/`: independently copied Rust interpreter with its provenance and license.

```bash
cargo test --workspace
cargo clippy -p controlab-native --all-targets -- -D warnings
PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests
python3 scripts/check_native_sources.py
```

Current validation and CI focus on Linux. Native Android integration is retained
for future work; Android and Windows validation is out of scope for this stage.
Historical changelogs and validation notes describe earlier versions; the web
implementation and Node build tools have been removed.
