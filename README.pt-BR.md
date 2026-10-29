# ControLAB — Sistemas de Controle

Interface nativa em **Rust + egui/eframe**, engine científica em **Rust** e
interpretador `.m` em **Rust** (`mscript`). O aplicativo não usa navegador,
Electron, Capacitor, JavaScript, TypeScript, HTML ou CSS.

[English](README.md) · [Interface e atalhos](native/README.md) · [Responsividade](docs/validation/native-responsive.md)

A engine Python anterior foi retirada ao fim da migração e permanece no
histórico Git. Seus resultados e gráficos estão congelados em `tests/reference/`
e `cargo xtask parity` compara a engine Rust com eles.

## Executar no Linux

Requer Rust/Cargo e as bibliotecas gráficas de desenvolvimento usadas pelo
eframe. Compile e execute com a engine científica Rust:

```bash
cargo build --locked --workspace
CONTROLAB_SCIENTIFIC_BACKEND=rust cargo run --locked --bin controlab
```

O workspace gera `controlab`, que inclui o interpretador `.m`, em `target/debug/`;
`controlab-engine` oferece transporte científico local por JSON-lines. Para uma
compilação otimizada, acrescente `--release` ao build e à execução. `--offline`
pode ser acrescentado quando as dependências Cargo já estiverem no cache.
Nenhuma dependência é instalada automaticamente pelos comandos de build.

## Funcionalidades

- Resposta no tempo, estabilidade/Routh-Hurwitz, lugar das raízes, Bode/Nyquist,
  controladores e espaço de estados.
- Studio `.m` com arquivos, editor, console e variáveis persistentes na sessão.
- Gráficos científicos, fórmulas, memoriais e exportação PNG/SVG.
- Temas claro/escuro/sistema e layouts de celular, tablet e desktop.

## Pacote Linux independente

```bash
cargo xtask package --release
CONTROLAB_SCIENTIFIC_BACKEND=rust env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 cargo xtask validate-bundle dist/native/controlab --ui
```

A pasta `dist/native/controlab` contém a interface, o executável científico Rust
e os recursos compartilhados; o runtime `.m` executa dentro da interface. Copie
a pasta inteira. A validação exercita os seis módulos científicos, renderização
de fórmulas, integridade dos recursos e, com `--ui`, interface nativa e scripts
`.m`. A máquina de destino precisa das bibliotecas gráficas do Linux e de uma
glibc compatível com o sistema usado no build.

A saída deve estar vazia para preservar arquivos existentes; `--output` permite
selecionar outra pasta. Use `--skip-build` somente após compilar o workspace
com o perfil correspondente.

## Instaladores e releases

O GitHub Actions (`.github/workflows/build-all.yml`) compila, testa e anexa como
artefatos nos pushes e pull requests configurados: `.deb`, `.rpm` e `.AppImage`
(runner Ubuntu 24.04, glibc 2.39+), instalador NSIS por usuário para Windows e
APK Android (arm64-v8a). Uma tag `vX.Y.Z` aplica essa versão com
`cargo xtask version "$GITHUB_REF_NAME"` e publica um GitHub Release com todos
os arquivos. `workspace.package.version` do `Cargo.toml` é a fonte única da versão.

A partir de um pacote já gerado, com `dpkg-deb`, `rpmbuild`, `appimagetool` ou
`makensis` instalados:

```bash
cargo xtask installers deb rpm appimage --bundle dist/native/controlab --output release  # Linux
cargo xtask installers nsis --bundle dist/native/controlab --output release              # Windows
```

Os instaladores ficam em `release/`. Os pacotes Linux instalam em `/opt/controlab`,
com o comando `controlab`, entrada no menu e ícones. Android requer JDK 17,
SDK/NDK Android e o alvo Rust `aarch64-linux-android`; o Gradle chama
`cargo xtask android` para compilar a biblioteca Rust:

```bash
native/android/gradlew --project-dir native/android --no-daemon :app:assembleRelease -PnativeAbis=arm64-v8a
```

Sem os secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`,
`ANDROID_KEY_ALIAS` e `ANDROID_KEY_PASSWORD`, o APK release é assinado com a
chave debug do runner; instalar uma build nova sobre outra antiga exige
desinstalar antes.

## Estrutura e validação

- `native/`: interface Rust, incluindo a ponte JSON dos scripts `.m` (`native/src/scripts.rs`).
- `crates/control-engine/`: cálculos científicos Rust, renderização de fórmulas e transporte local por JSON-lines, sem servidor HTTP.
- `assets/`: fontes, licenças, ícones e catálogo compartilhados; `manifest.json` registra seus hashes.
- `crates/mscript/`: interpretador `.m` (arrays, modelos de controle, gráficos e documentos), executado dentro da interface.
- `xtask/`: empacotamento, instaladores, sincronização de versão, verificação de fontes, validação de pacotes e compilação Android em Rust.
- `tests/reference/`: resultados, curvas e gráficos Python congelados, usados por `cargo xtask parity`.
- `packaging/`: entrada de menu Linux, script NSIS do Windows e ícones dos instaladores.

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo xtask check-sources
cargo xtask parity
```

O CI executa as verificações do workspace Rust e da interface empacotada no
Linux; o job Windows valida os seis módulos científicos, fórmulas e uma
instalação silenciosa. O APK é gerado no CI, mas não é executado em aparelho.
A paridade com a engine Python retirada é verificada por `cargo xtask parity`:
valores numéricos com `atol=1e-9, rtol=1e-7`, textos exatos e os rótulos e
cores de cada gráfico. Documentos e capturas da interface Electron/Capacitor
removida ficam em `docs/history`.

## Diagrama de Blocos

Em **Apps → Diagrama de Blocos**, abra o editor em uma janela independente
no mesmo processo. O editor inclui redução simbólica/numérica por Mason,
exemplos, tutoriais, histórico de edição, arquivos JSON e exportação SVG/PNG.
Os arquivos JSON existentes do BlockLab continuam compatíveis.

```bash
cargo run --bin controlab -- --diagram caminho/diagrama.json
cargo run --bin controlab -- --export-diagram caminho/diagrama.json resultado.svg
```

A integração incorpora as bibliotecas do BlockLab em `crates/blocklab-core`
e `crates/blocklab-gui`; não exige outra instalação ou o repositório original.
A conexão automática com os outros Apps não faz parte desta entrega.
