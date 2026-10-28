# Migração nativa — 28/10/2026

Branch: `enthony/migrate-rust`. Interface Rust/egui/eframe 0.36.2; engine
Python e núcleo `.m` preservados. O launcher nativo não inicia Electron,
Capacitor, HTTP ou WebView. Código legado permanece como referência.

## Implementação

- Mesmos tokens de cor, fontes locais e estrutura Studio: navegação, arquivos,
  abas de documentos, editor, console persistente, variáveis e figuras.
- Seis módulos científicos usam o dispatch canônico Python; entradas, condições
  de visibilidade, limites e presets reproduzem os formulários existentes.
- Worker assíncrono serial, JSON estrito e limitado, timeout desktop e limpeza
  dos processos filhos. Sessão `.m` permanece entre comandos.
- Gráficos Matplotlib decodificados como texturas nativas, fórmulas compatíveis
  compostas por MathText, exportação SVG com confirmação de substituição.
- Activity Android nativa/JNI/Chaquopy, fontes Python canônicas, armazenamento
  privado persistente, build do motor `.m` e interface para ARM64.

## Evidências executadas

| Verificação | Resultado |
|---|---|
| `cargo test --manifest-path native/Cargo.toml --offline` | 9 testes aprovados |
| `cargo clippy --manifest-path native/Cargo.toml --offline --all-targets -- -D warnings` | Aprovado sem avisos |
| `cargo fmt --manifest-path native/Cargo.toml --check` | Aprovado |
| `PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests` | 107 testes aprovados |
| `python3 scripts/run-native.py --offline` com captura automática | Build + janela Linux + cálculo temporal real aprovados |
| `native/android/build_native.py --abis arm64-v8a --profile debug --offline` com saída no build Android | Biblioteca UI e executável `.m` ARM64 compilados/linkados |
| `py_compile`, `bash -n run.sh`, `git diff --check` | Aprovados |

Os testes Rust renderizam a interface em 390/768/1440 px e dois temas, incluindo
as abas móveis, estados vazio/carregando/erro/resultado e geometria tessellada;
verificam ainda payloads, números inválidos, recuperação de flags, texturas PNG
com e sem prefixo data URI e integração real Rust→Python→`.m` persistente.
Os testes Python cobrem os seis motores, normalização científica, erros e limites
do protocolo e renderização das fórmulas. Matplotlib emitiu avisos de fallback
de pesos de fontes; testes legados também emitiram FutureWarning de root_locus.

Capturas reais do aplicativo Linux:

- [Studio escuro](native/studio-dark.png)
- [Studio claro](native/studio-light.png)
- [Resposta temporal](native/time-response.png)

## Limites confirmados

- A distribuição Gradle 8.14.3 não está instalada. APK, compilação Java,
  empacotamento Chaquopy e aparelho/emulador não foram validados.
- O check cruzado Windows não executou: `windows-sys 0.48.0` ausente no cache
  offline. Não há prova de execução nem instalador Windows nesta sessão.
- Não foram gerados instaladores desktop autocontidos. É necessário fornecer
  Python com os pacotes científicos, a ponte e o runtime `.m` com o binário Rust.
- O seletor de arquivos usa a interface egui e caminhos. Integração Android com
  SAF/compartilhamento público ainda não está implementada.
- A UI nativa está em português. O seletor de inglês existente na UI anterior
  ainda precisa ser migrado. MathText cobre um subconjunto de LaTeX; expressões
  não suportadas permanecem legíveis no resultado estruturado como fonte TeX.
- Identidade visual preservada (cores/fontes/organização), sem promessa de
  equivalência pixel a pixel entre CSS e widgets egui.

Nenhuma dependência foi instalada globalmente. Sem commit, push ou publicação.

## Atualização — build direto pelo Cargo

A pedido do usuário, o workspace da raiz agora compila os dois executáveis:

```bash
cargo build --offline
cargo run --offline --bin controlab
cargo build --offline --release
```

`target/debug/` (ou `target/release/`) contém `controlab` e `controlab-runtime`
(com `.exe` no Windows). O pacote `native/runtime` aponta diretamente para o
adaptador vendorizado e depende das mesmas crates openMAT; nenhuma fonte do
motor foi alterada. `Cargo.lock` fica na raiz. `run.sh` usa o mesmo fluxo Cargo.
O launcher Python intermediário mencionado na validação histórica acima foi
substituído; Python continua necessário para os módulos científicos.

Validado nesta atualização:

- `cargo build --offline`: interface e runtime compilados.
- `cargo run --offline --bin controlab`: janela Linux e cálculo temporal real,
  com captura automática; o runtime `.m` é localizado junto ao executável.
- `CONTROLAB_RUNTIME="$PWD/target/debug/controlab-runtime" cargo test --offline --workspace`:
  13 testes aprovados (9 nativos + 4 do adaptador de documentos), incluindo
  execução `.m` persistente com o runtime gerado pelo workspace.
- `cargo clippy --offline -p controlab-native --all-targets -- -D warnings`,
  `cargo fmt -p controlab-native --check`, `bash -n run.sh` e `git diff --check`:
  aprovados.
- `cargo fmt --all --check` aponta formatação preexistente em fontes vendorizadas;
  elas foram preservadas. O check de formatação da interface passou.

O build otimizado está configurado pelo Cargo, mas não foi executado nesta
atualização. Os limites de APK/Windows/instaladores listados acima permanecem.
