# ControLAB — Sistemas de Controle

Interface nativa em **Rust + egui/eframe**, engine científica em **Python** e
interpretador `.m` em **Rust** (`mscript`). O aplicativo não usa navegador,
Electron, Capacitor, JavaScript, TypeScript, HTML ou CSS.

[English](README.md) · [Interface e atalhos](native/README.md) · [Responsividade](docs/validation/native-responsive.md)

## Executar no Linux

Requer Rust/Cargo e Python com as bibliotecas científicas de `requirements.txt`.

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
cargo build --workspace
cargo run --bin controlab
```

O workspace gera `controlab`, que já inclui o interpretador `.m`, em `target/debug/`. Para uma
compilação otimizada, use `cargo build --release --workspace` e
`cargo run --release --bin controlab`. `--offline` pode ser acrescentado quando
as dependências Cargo já estiverem no cache. `./run.sh` usa o mesmo fluxo.

O Python da `.venv` é detectado automaticamente. `CONTROLAB_PYTHON` seleciona
outro interpretador.
Nenhuma dependência é instalada automaticamente pelos scripts do aplicativo.

## Funcionalidades

- Resposta no tempo, estabilidade/Routh-Hurwitz, lugar das raízes, Bode/Nyquist,
  controladores e espaço de estados.
- Studio `.m` com arquivos, editor, console e variáveis persistentes na sessão.
- Gráficos científicos, fórmulas, memoriais e exportação PNG/SVG.
- Temas claro/escuro/sistema e layouts de celular, tablet e desktop.

## Pacote Linux independente

```bash
.venv/bin/python -m pip install -r requirements-build.txt
.venv/bin/python scripts/package_native.py --release
env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 .venv/bin/python scripts/validate_native_bundle.py dist/native/controlab --ui
```

A pasta `dist/native/controlab` contém o aplicativo, runtime `.m`, Python e
bibliotecas científicas. Copie a pasta inteira; somente o binário da interface
não é uma distribuição completa. A máquina de destino precisa das bibliotecas
gráficas do Linux e de uma glibc compatível com o sistema usado no build.

A saída deve estar vazia para preservar arquivos existentes; `--output` permite
selecionar outra pasta.

## Instaladores e releases

O GitHub Actions (`.github/workflows/build-all.yml`) compila, testa e anexa como
artefatos a cada push: `.deb`, `.rpm` e `.AppImage` (runner Ubuntu 24.04, glibc
2.39+), instalador NSIS por usuário para Windows e APK Android (arm64-v8a). Uma
tag `vX.Y.Z` aplica essa versão (`scripts/set_version.py`) e publica um GitHub
Release com todos os arquivos. `native/Cargo.toml` é a fonte única da versão.

A partir de um pacote já gerado, com `dpkg-deb`, `rpmbuild`, `appimagetool` ou
`makensis` instalados:

```bash
python3 scripts/package_installers.py deb rpm appimage   # Linux
python scripts/package_installers.py nsis                # Windows
```

Os instaladores ficam em `release/`. Os pacotes Linux instalam em `/opt/controlab`,
com o comando `controlab`, entrada no menu e ícones. Sem os secrets
`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` e
`ANDROID_KEY_PASSWORD`, o APK release é assinado com a chave debug do runner;
instalar uma build nova sobre outra antiga exige desinstalar antes.

## Estrutura e validação

- `native/`: interface Rust, incluindo a ponte JSON dos scripts `.m` (`native/src/scripts.rs`).
- `src/python/`: engine científica e transporte local por stdin/stdout, sem servidor HTTP.
- `assets/`: fontes, licenças, ícones e catálogo compartilhados; `manifest.json` registra seus hashes.
- `crates/mscript/`: interpretador `.m` (arrays, modelos de controle, gráficos e documentos), executado dentro da interface.
- `scripts/`: empacotamento, versão e verificações em Python; builds Rust pelo Cargo.
- `packaging/`: entrada de menu Linux, script NSIS do Windows e ícones dos instaladores.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests
python3 scripts/check_native_sources.py
```

O CI executa todos os testes Rust/Python e a verificação da interface no Linux;
o job Windows valida a engine empacotada e uma instalação silenciosa. O APK é
gerado no CI, mas não é executado em aparelho. Documentos e capturas da interface
Electron/Capacitor removida ficam em `docs/history`.
