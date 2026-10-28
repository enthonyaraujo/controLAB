# Responsividade nativa — celular e tablet

## Referência e implementação

Pedido: terminar a responsividade Android do trabalho em `enthony/migrate-rust`,
preservando os fluxos e a organização da interface Electron. Foram consultados
os estilos canônicos `src/renderer/styles.css`, `styles/studio.css`, os controles
de `studio.js` e capturas do mesmo renderer em Chromium local com toque ativado.
Não houve alteração nas engines Python/openMAT nem nas fontes da UI Electron.

| Faixa | Comportamento preservado |
|---|---|
| Menos de 600 px | Navegação inferior rolável; Parâmetros/Gráfico/Resultados exclusivos; Calcular fixo; símbolos matemáticos |
| De 600 até menos de 1024 px | Rail de módulos; parâmetros laterais; Figura/Resultados no painel principal |
| 1024 px ou mais | Navegação superior e painéis simultâneos; abas cabem antes do botão Configurações |
| Studio abaixo de 768 px | Ferramentas roláveis, arquivos/variáveis em gavetas, welcome em uma coluna, ações empilhadas |
| Studio de 768 a 1000 px | Colunas de arquivos e variáveis com 170/200 px, mantendo espaço para editor/console |

A classificação usa a largura integral não arredondada de entrada do egui.
Isso evita que o zoom ou um recorte de tela transforme uma janela de 600 px em
celular. As áreas seguras delimitam o conteúdo, sem alterar o breakpoint.

Durante foco em edição por toque, os rodapés cedem espaço e a fileira de
símbolos permanece fixa. A inserção respeita caret, seleção e Unicode. As
entradas voltam à área visível quando o viewport encolhe. O gráfico individual
aceita pinch com dois dedos; figuras empilhadas e resultados mantêm rolagem.
No LGR, as abas de memorial/teoria são roláveis, e o resultado do tablet pode
ser aberto mesmo quando o memorial estava selecionado.

Foram corrigidos ainda títulos sobrepostos aos controles, labels longos,
gavetas inacessíveis, dimensão do console em pouca altura e transbordamento de
Configurações/confirmar/links e ações. Cores, fontes e componentes existentes
foram preservados.

## Validação executada

- `cargo test --offline --workspace`: **32 aprovados** (28 nativos + 4 runtime).
- `cargo clippy --offline -p controlab-native --all-targets -- -D warnings`:
  aprovado sem avisos.
- `cargo build --offline`, formatação da interface e `git diff --check`: aprovados.
- Compilação/link Android `arm64-v8a` offline: UI e runtime `.m` aprovados.
  Permanecem os três avisos Android de código exclusivo do desktop não usado
  (`Picked`, campo de screenshot e `Formulas::busy`); não houve erro de build.
- Capturas reais do executável Linux em modo de toque, com X11, usando os seis
  módulos, páginas vazias e cálculos reais. A coleta Wayland foi interrompida
  após uma janela oculta não entregar o evento de screenshot; X11 concluiu a
  coleta. Isso não foi tratado como prova de execução Android.

A matriz cobre 360, 390, 599, 600, 767, 768, 844, 1023, 1024, 1280, 1440 e
1920 px; retrato/paisagem; claro/escuro; escalas 80/100/125%; recortes e insets.
As interações testadas incluem cliques reais em abas e gavetas, teclado de
símbolos, foco/viewport reduzido, seleção Unicode e pinch. Respostas reais dos
seis motores e suas fórmulas também são renderizadas em celular/tablet.

## Capturas

- [Celular — início](native-responsive/celular-inicio.png)
- [Celular — parâmetros](native-responsive/celular-parametros.png)
- [Celular — gráficos](native-responsive/celular-graficos.png)
- [Celular — resultados](native-responsive/celular-resultados.png)
- [Tablet — início](native-responsive/tablet-inicio.png)
- [Celular — configurações](native-responsive/celular-configuracoes.png)
- [Tablet — resultados](native-responsive/tablet-resultados.png)
- [Tablet — paisagem](native-responsive/tablet-paisagem.png)
- [Tablet — limite de 600 px](native-responsive/tablet-600.png)
- [Electron original — celular](native-responsive/referencia-electron-celular.png)
- [Electron original — tablet](native-responsive/referencia-electron-tablet.png)

Reproduzir uma captura após `cargo build`:

```bash
WINIT_UNIX_BACKEND=x11 CONTROLAB_TOUCH=1 \
CONTROLAB_SCREENSHOT=/tmp/controlab-celular.png \
CONTROLAB_SCREENSHOT_SIZE=390x844 \
CONTROLAB_SCREENSHOT_PAGE=time CONTROLAB_SCREENSHOT_TAB=0 \
CONTROLAB_SCREENSHOT_EMPTY=1 CONTROLAB_SCREENSHOT_THEME=light \
cargo run --offline --bin controlab
```

## Limites

O APK completo e a interação no aparelho/emulador Android continuam sem
validação. A verificação de teclado, insets e gestos aqui usa entrada sintética
do egui; a compilação ARM64 confirma o código de destino, não o comportamento
do sistema operacional. Os demais limites de distribuição da migração
permanecem registrados em `native-migration.md`. Nenhuma dependência foi
instalada, e não houve commit, push ou publicação.
