
<h1>
  <img src="assets/icons/app-icon.png" width="32" height="32" align="absmiddle" alt="ControLAB Logo">
  ControLAB
</h1>

[🇧🇷 Português](README.md) | [🇺🇸 English](README_EN.md)

Software for the analysis, simulation, and design of **Control Systems**, developed in **Rust** with a native **egui/eframe** interface.

<p align="center">
  <a href="docs/screenshots/01-studio.png">
    <img src="docs/screenshots/01-studio.png" alt="ControLAB Studio" width="100%">
  </a>
</p>

---

## Features

- **Linear Analysis:** Time-domain response, stability analysis (Routh-Hurwitz), Root Locus, and frequency response (Bode and Nyquist).
- **Control & State-Space:** Controller design and state-space modeling.
- **Block Diagrams:** Visual editing and reduction of block diagrams.
- **`.m` Studio:** Integrated script interpreter and editor (`mscript`).
- **Export & Visualization:** Interactive plots, mathematical formula rendering, and PNG/SVG export.

## Interface & Modules

| | |
|:---:|:---:|
| ![.m Studio](docs/screenshots/01-studio.png) | ![Applications Hub](docs/screenshots/02-apps-hub.png) |
| **.m Studio & Command Window** | **Applications Hub** |
| ![Block Diagrams](docs/screenshots/03-block-diagram.png) | ![Time-Domain Response](docs/screenshots/04-time-response.png) |
| **Block Diagrams** | **Time-Domain Response** |
| ![Stability Analysis](docs/screenshots/05-stability.png) | ![Root Locus](docs/screenshots/06-root-locus.png) |
| **Stability Analysis (Routh-Hurwitz)** | **Root Locus (RL)** |
| ![Frequency Response](docs/screenshots/07-frequency-response.png) | ![Controller Design](docs/screenshots/08-controllers.png) |
| **Frequency Response (Bode & Nyquist)** | **Controller Design** |
| ![State-Space Representation](docs/screenshots/09-state-space.png) | |
| **State-Space Representation** | |

## Getting Started

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/) installed.
- Graphics libraries required for `eframe` compilation (on Linux: graphics drivers, Wayland/X11, and libxkbcommon).

### Running the Application

```bash
# Clone the repository
git clone https://github.com/enthonyaraujo/controLAB.git
cd controLAB

# Build and run
cargo run --bin controlab

# Or run in optimized release mode
cargo run --release --bin controlab
```

> **Tip:** You can also use the `./run.sh` script.

## Contributing

Contributions are always welcome! To contribute:

1. **Fork** the repository.
2. Create a new branch for your changes:
   ```bash
   git checkout -b feat/my-feature
   ```
3. Commit your changes:
   ```bash
   git commit -m "feat: add my feature"
   ```
4. Push your changes to your fork:
   ```bash
   git push origin feat/my-feature
   ```
5. Open a **Pull Request**.

Before submitting, make sure all tests pass and the code follows the project's formatting standards:

```bash
cargo test --workspace
cargo fmt --all --check
```

## License

Distributed under the [MIT License](LICENSE).
