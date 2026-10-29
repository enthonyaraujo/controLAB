# ControLAB nativo — Rust + egui/eframe

A interface nativa executa em Linux, Windows e Android, sem navegador, WebView,
HTML, CSS ou JavaScript. O motor científico Rust está em `crates/control-engine`;
scripts `.m` são executados pelo interpretador `crates/mscript`, dentro do aplicativo.
Desktop executa o serviço científico Rust local; Android usa a mesma biblioteca
diretamente na thread de trabalho.

## Desktop

Requer Rust/Cargo. Cargo compila as dependências especificadas em `Cargo.lock`;
`--offline` exige cache completo.

```bash
# Na raiz do repositório: interface, serviço científico e runtime .m
cargo build --locked --offline
cargo run --locked --offline -p controlab-native

# Compilação otimizada
cargo build --locked --offline --release
cargo run --locked --offline --release -p controlab-native
# Linux também aceita:
./run.sh
```

O workspace Cargo compila a interface, o serviço científico (`crates/control-engine`)
e o interpretador `.m` (`crates/mscript`), ligado à própria interface. Os executáveis
ficam em `target/debug/` ou `target/release/`. O cliente procura `CONTROLAB_ENGINE`,
`engine/controlab-engine` ao lado do aplicativo, depois
`target/debug/controlab-engine` ou `target/release/controlab-engine` do repositório
(no Windows, com sufixo `.exe`). Um caminho explícito inválido ou uma falha do
motor produz erro. `CONTROLAB_SCIENTIFIC_BACKEND` é opcional e aceita somente
`rust`; outros valores são rejeitados. A comunicação usa
JSON UTF-8 por linha em stdin/stdout de um processo persistente, sem portas de
rede, com fila de 16 pedidos, limite de 8 MiB por pedido e 32 MiB por resposta.
`CONTROLAB_REQUEST_TIMEOUT` define o prazo em segundos (60 por padrão, limitado
a 1–300); timeout encerra o grupo de processos e o próximo pedido inicia outro. Os
scripts `.m` são executados na mesma thread de trabalho, sem processo separado
(`native/src/scripts.rs`), com limites de tempo, memória e eventos do
interpretador. Cálculos são serializados nessa thread; erro/timeout não bloqueia
os frames da UI.

A navegação tem duas seções, como no MATLAB: **Início**, com o Studio `.m`, e
**Apps**, a galeria dos seis apps de análise (resposta no tempo, estabilidade,
lugar das raízes, resposta em frequência, controladores e espaço de estados).
Sem script aberto, a Command Window (o console `.m`) ocupa o centro do Início;
ao abrir um script, o editor ocupa o centro e a Command Window desce para o
painel inferior.

A interface mantém a identidade visual estabelecida: IBM Plex Sans e JetBrains Mono em todos os pesos, os ícones SVG
originais (rasterizados com resvg), as abas superiores, os ribbons, os três
painéis dos apps (Parâmetros, Figura e Resultados), o Studio `.m`, os toasts
e o modal de Configurações & Sobre. Os temas Claro, Escuro e Sistema e a escala
da interface (90–150%) são salvos no diretório de configuração do usuário, com
a pasta de trabalho e os itens recentes. Abaixo de 600 px os módulos usam as
abas Parâmetros/Gráfico/Resultados; tablets usam rail e Figura/Resultados; abaixo de 768 px o Studio oculta as colunas
laterais, como no CSS anterior. F5 executa o documento; Ctrl/Cmd+S, +O e +N
salvam, abrem e criam scripts; Ctrl/Cmd+Enter calcula o app atual. Abrir,
salvar, selecionar pasta e exportar figuras usam os diálogos do sistema.

Os gráficos científicos são compostos em Rust e exibidos como imagens locais,
com zoom (botões, roda do mouse e arraste) e exportação em PNG ou SVG; Copiar
envia a figura para a área de transferência. As fórmulas, prévias e memoriais
são compostos pelo motor TeX nativo com fontes incorporadas, incluindo KaTeX
(`assets/fonts/katex`). A imagem branca recebe a cor do contexto na interface;
fórmulas não suportadas continuam legíveis como texto.

Para comparar visualmente com a versão anterior, `CONTROLAB_SCREENSHOT=<arquivo.png>`
captura a janela em 1920×1048 e encerra; `CONTROLAB_SCREENSHOT_PAGE`,
`CONTROLAB_SCREENSHOT_THEME`, `CONTROLAB_SCREENSHOT_EMPTY`,
`CONTROLAB_SCREENSHOT_LGR_VIEW`, `CONTROLAB_SCREENSHOT_SETTINGS` e
`CONTROLAB_SCREENSHOT_SCRIPT` escolhem o estado capturado.

## Android

Consulte [ANDROID.md](ANDROID.md): projeto Gradle separado, NativeActivity,
interface e motor científico Rust, sem Capacitor ou Python. JNI obtém somente
o diretório privado do aplicativo; os pedidos científicos executam na thread
de trabalho com cancelamento e prazo. Gradle chama `cargo run --locked
--manifest-path <raiz>/Cargo.toml -p xtask -- android`, que compila as ABIs
selecionadas com `-PnativeAbis=arm64-v8a,x86_64`. Fontes e traduções
são incorporadas ao motor Rust.

## Validação

```bash
cargo test --workspace --offline
cargo clippy --workspace --offline --all-targets -- -D warnings
cargo run --locked --offline -p xtask -- parity --data-only
```

O build usa Cargo e as ferramentas do workspace `xtask`. O cliente
carrega o executável científico em `engine/controlab-engine`; `CONTROLAB_ENGINE`
permite selecionar outro caminho. O transporte desktop permanece local por
stdin/stdout.
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
