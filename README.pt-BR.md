# ControLAB — Sistemas de Controle

Interface nativa em **Rust + egui/eframe**, engine científica em **Python** e
interpretador `.m` em **Rust/openMAT**. O aplicativo não usa navegador,
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

O workspace gera `controlab` e `controlab-runtime` em `target/debug/`. Para uma
compilação otimizada, use `cargo build --release --workspace` e
`cargo run --release --bin controlab`. `--offline` pode ser acrescentado quando
as dependências Cargo já estiverem no cache. `./run.sh` usa o mesmo fluxo.

O Python da `.venv` é detectado automaticamente. `CONTROLAB_PYTHON` seleciona
outro interpretador; `CONTROLAB_RUNTIME` seleciona o executável `.m`.
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
selecionar outra pasta. Não há publicação, instalador ou atualização automática
nesses comandos.

## Estrutura e validação

- `native/`: interface Rust; `native/runtime/`: adaptação de build do motor `.m`.
- `src/python/`: engine científica e transporte local por stdin/stdout, sem servidor HTTP.
- `assets/`: fontes, licenças, ícones e catálogo compartilhados; `manifest.json` registra seus hashes.
- `vendor/openmat/`: núcleo Rust independente, com licença e procedência próprias.
- `scripts/`: empacotamento e verificações em Python; builds Rust pelo Cargo.

```bash
cargo test --workspace
cargo clippy -p controlab-native --all-targets -- -D warnings
PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests
python3 scripts/check_native_sources.py
```

O fluxo CI atual valida Linux. A integração Android nativa em `native/android`
é mantida para continuidade, mas Android e Windows estão fora da validação
desta etapa por decisão do usuário. Registros históricos em `CHANGELOG.md` e
`docs/validation` podem mencionar a interface antiga, cujo código foi removido.
