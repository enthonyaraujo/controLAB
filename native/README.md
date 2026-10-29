# ControLAB nativo — Rust + egui/eframe

A interface nativa executa em Linux, Windows e Android, sem navegador, WebView,
HTML, CSS ou JavaScript. O motor científico continua em `src/python`; scripts
`.m` são executados pelo interpretador `crates/mscript`, dentro do aplicativo.

## Desktop

Requer Rust/Cargo e o ambiente Python científico já usado pelo projeto. Cargo compila
as dependências especificadas em `Cargo.lock`; `--offline` exige cache completo.

```bash
# Na raiz do repositório: interface e runtime .m
cargo build --offline
cargo run --offline

# Compilação otimizada
cargo build --offline --release
cargo run --offline --release
# Linux também aceita:
./run.sh
```

O workspace Cargo compila a interface e o interpretador `.m` (`crates/mscript`),
ligado à própria interface. `cargo run` executa a interface (`default-run`); o
executável fica em `target/debug/` ou `target/release/`. O caminho padrão não
depende de Node ou de um launcher Python. O Python da `.venv` é escolhido
automaticamente. É possível selecionar outro com `CONTROLAB_PYTHON`, e
`CONTROLAB_NATIVE_BRIDGE` seleciona a ponte Python. A comunicação com Python usa
JSON por stdin/stdout de um processo persistente, sem abrir portas de rede. Os
scripts `.m` são executados na mesma thread de trabalho, sem processo separado
(`native/src/scripts.rs`), com limites de tempo, memória e eventos do
interpretador. Cálculos são serializados nessa thread; erro/timeout não bloqueia
os frames da UI.

A interface mantém a identidade visual estabelecida: IBM Plex Sans e JetBrains Mono em todos os pesos, os ícones SVG
originais (rasterizados com resvg), as abas superiores, os ribbons, os três
painéis dos módulos (Parâmetros, Figura e Resultados), o Studio `.m`, os toasts
e o modal de Configurações & Sobre. Os temas Claro, Escuro e Sistema e a escala
da interface (90–150%) são salvos no diretório de configuração do usuário, com
a pasta de trabalho e os itens recentes. Abaixo de 600 px os módulos usam as
abas Parâmetros/Gráfico/Resultados; tablets usam rail e Figura/Resultados; abaixo de 768 px o Studio oculta as colunas
laterais, como no CSS anterior. F5 executa o documento; Ctrl/Cmd+S, +O e +N
salvam, abrem e criam scripts; Ctrl/Cmd+Enter calcula o módulo atual. Abrir,
salvar, selecionar pasta e exportar figuras usam os diálogos do sistema.

Os gráficos científicos são gerados pelo mesmo Matplotlib em Python, como
imagens locais, com zoom (botões, roda do mouse e arraste) e exportação em PNG
ou SVG; Copiar envia a figura para a área de transferência. As fórmulas, prévias
e memoriais são compostos pelo MathText com as fontes do KaTeX vendorizadas
(`assets/fonts/katex`), em modo display, com matrizes e setas AMS
adaptadas; a imagem é branca e recebe a cor do contexto na interface. Fórmulas
não suportadas continuam legíveis como texto. Não há navegador, KaTeX em
JavaScript nem instalação externa de LaTeX.

Para comparar visualmente com a versão anterior, `CONTROLAB_SCREENSHOT=<arquivo.png>`
captura a janela em 1920×1048 e encerra; `CONTROLAB_SCREENSHOT_PAGE`,
`CONTROLAB_SCREENSHOT_THEME`, `CONTROLAB_SCREENSHOT_EMPTY`,
`CONTROLAB_SCREENSHOT_LGR_VIEW`, `CONTROLAB_SCREENSHOT_SETTINGS` e
`CONTROLAB_SCREENSHOT_SCRIPT` escolhem o estado capturado.

## Android

Consulte [ANDROID.md](ANDROID.md): projeto Gradle separado, NativeActivity,
interface Rust e Chaquopy, sem Capacitor. Fontes Python são canônicas, sem cópias
sincronizadas por JavaScript.

## Validação

```bash
cargo test --workspace --offline
cargo clippy --workspace --offline --all-targets -- -D warnings
PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests
```

A implementação web e seus scripts foram removidos. O build usa Cargo e o
pacote Linux é produzido por `scripts/package_native.py`, que inclui uma engine
Python independente com recursos em `assets/`. O cliente detecta a engine ao
lado do aplicativo em `engine/controlab-engine`; `CONTROLAB_ENGINE` permite
selecionar outra engine empacotada. O transporte continua local por stdin/stdout.
Veja a [separação e validação Linux](../docs/validation/native-linux-only.md).

A interface nativa está em português; a migração do seletor de inglês ainda está
pendente. Evidências e limites completos: [validação](../docs/validation/native-migration.md).

## Responsividade Android

A interface segue os breakpoints do Electron: celular abaixo de 600 px,
tablet entre 600 e 1024 px e três painéis a partir de 1024 px. O Studio mantém
os próprios limites de 768/1000 px, com gavetas no celular, ferramentas
roláveis e console limitado pela altura disponível. Navegação, áreas seguras,
abas científicas, edição com teclado e pinch do gráfico têm testes de toque.
Veja [matriz, capturas e limites de validação](../docs/validation/native-responsive.md).
