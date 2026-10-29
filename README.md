# ControLAB — Control Systems

A native **Rust + egui/eframe** interface, **Rust** scientific engine and
**Rust** `.m` interpreter (`mscript`). The application uses no browser, Electron,
Capacitor, JavaScript, TypeScript, HTML or CSS.

[Português (Brasil)](README.pt-BR.md) · [Native interface](native/README.md)

The former Python engine was retired after the migration; it remains in the Git
history. Its results and figures are frozen in `tests/reference/` and
`cargo xtask parity` compares the Rust engine against them.

## Linux development

Prepare Rust/Cargo and the Linux graphics development libraries required by
eframe, then build and run the Rust scientific backend:

```bash
cargo build --locked --workspace
CONTROLAB_SCIENTIFIC_BACKEND=rust cargo run --locked --bin controlab
```

The `controlab` executable, which includes the `.m` interpreter, is built under
`target/debug/`; `controlab-engine` provides a local JSON-lines scientific
transport. Use `--release` on build and run commands for an optimized build;
`--offline` uses cached Cargo dependencies. Build commands do not install
dependencies automatically.

## Independent Linux distribution

```bash
cargo xtask package --release
CONTROLAB_SCIENTIFIC_BACKEND=rust env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 cargo xtask validate-bundle dist/native/controlab --ui
```

Distribute the entire `dist/native/controlab` folder, which includes the UI,
scientific Rust executable and shared resources; the `.m` runtime runs inside
the UI. Validation exercises all six scientific modules, formula rendering,
resource integrity and, with `--ui`, the native interface and `.m` execution.
Target machines still need Linux graphics libraries and a glibc version
compatible with the build host. The output folder must be empty; use
`--output` to choose another folder. Use `--skip-build` only after building the
workspace with the matching profile.

## Installers and releases

GitHub Actions (`.github/workflows/build-all.yml`) builds, tests and uploads for
configured pushes and pull requests: `.deb`, `.rpm` and `.AppImage` (Ubuntu 24.04
runner, glibc 2.39+), a per-user NSIS installer for Windows and an Android APK
(arm64-v8a). Pushing a `vX.Y.Z` tag applies that version with
`cargo xtask version "$GITHUB_REF_NAME"` and publishes a GitHub Release with
all files. `Cargo.toml`'s `workspace.package.version` is the single version source.

From an existing bundle, with `dpkg-deb`, `rpmbuild`, `appimagetool` or
`makensis` installed:

```bash
cargo xtask installers deb rpm appimage --bundle dist/native/controlab --output release  # Linux
cargo xtask installers nsis --bundle dist/native/controlab --output release              # Windows
```

Installers are written to `release/`. Linux packages install into `/opt/controlab`
with a `controlab` launcher, menu entry and icons. Android requires JDK 17, an
Android SDK/NDK and the Rust target `aarch64-linux-android`; Gradle invokes
`cargo xtask android` to build the Rust library:

```bash
native/android/gradlew --project-dir native/android --no-daemon :app:assembleRelease -PnativeAbis=arm64-v8a
```

Without the secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`,
`ANDROID_KEY_ALIAS` and `ANDROID_KEY_PASSWORD`, the release APK is signed with
the runner's debug key, so installing a newer build over an older one requires
uninstalling first.

## Source and checks

- `native/`: Rust UI, including the JSON bridge for `.m` scripts (`native/src/scripts.rs`).
- `crates/control-engine/`: Rust scientific calculations, formula rendering and local JSON-lines transport; no HTTP server.
- `assets/`: canonical fonts, licenses, icons and translation data with a SHA-256 manifest.
- `crates/mscript/`: `.m` interpreter (numeric arrays, control models, plots, documents), run in-process by the UI.
- `xtask/`: Rust packaging, installers, version synchronization, source checks, bundle validation and Android library builds.
- `tests/reference/`: frozen Python results, curves and figures used by `cargo xtask parity`.
- `packaging/`: Linux desktop entry, Windows NSIS script and installer icons.

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo xtask check-sources
cargo xtask parity
```

CI runs the Rust workspace checks and packaged UI check on Linux; the Windows
job validates the six scientific modules, formula rendering and a silent
install. The APK is built in CI but not exercised on a device. Parity with the
retired Python engine is checked by `cargo xtask parity`: numeric results within
`atol=1e-9, rtol=1e-7`, exact texts, and every figure's labels and colors.
Notes and screenshots of the removed Electron/Capacitor interface are kept in
`docs/history`.
