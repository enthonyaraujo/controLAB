# Android nativo

A interface Android usa o mesmo cliente Rust/egui do desktop, renderizado por
OpenGL ES em `android.app.NativeActivity`. Não há WebView, Capacitor ou servidor
HTTP neste caminho. O projeto Gradle independente fica em `native/android`.

O APK inclui uma única biblioteca, `libcontrolab_native.so`: a interface Rust,
o motor científico (`crates/control-engine`), o interpretador `.m`
(`crates/mscript`) e o ciclo de vida `android-activity`. Não há Python nem
Chaquopy.

Cálculos e scripts `.m` rodam na thread de trabalho do Rust, que chama
`controlab_engine::dispatch` diretamente, sem JNI nem processo separado.
`NativeApplication` existe apenas como Application do processo.

As preferências e a pasta inicial de scripts usam `Application.getFilesDir()`.
Esses arquivos persistem entre reinícios e atualizações; a desinstalação remove
os dados privados. A exploração por caminhos fica limitada aos diretórios que
o aplicativo pode acessar. Importação/exportação pelo seletor de documentos e
compartilhamento pela galeria exigem integração Android específica posterior.

## Barras do sistema e teclado

A janela é edge-to-edge (targetSdk 36) e a superfície nativa ocupa a tela toda.
O egui 0.36 tem `RawInput::safe_area_insets` e tira essas margens de
`Context::content_rect`, mas o egui-winit só as preenche no iOS, e o winit 0.30
descarta `ContentRectChanged`/`InsetsChanged` no Android. Por isso:

- `src/android/insets.rs` lê por JNI
  `getWindow().getDecorView().getRootWindowInsets()`: `systemBars() | displayCutout()`
  e `ime()` a partir da API 30; antes disso, `getSystemWindowInset*` (que já inclui o
  teclado) e o recorte de `getDisplayCutout()` na API 28–29;
- `src/safe_area.rs` converte pixels físicos em pontos do egui (escala nativa × zoom
  da página) e entrega a `SafeArea` ao egui em `App::raw_input_hook`, antes de cada
  passada. `Controlab::frame` desenha a interface inteira dentro de `content_rect`;
  `Area`, popups, menus e tooltips já se limitam a esse retângulo. Só os fundos do
  cabeçalho, da navegação inferior e das laterais vão até as bordas;
- a leitura não acontece em todo quadro: só ao iniciar, quando o tamanho da janela ou
  o foco mudam (rotação, retorno do segundo plano), no primeiro quadro após uma pausa
  (rotação de 180°), durante 1 s após cada mudança (animações) e a cada 0,5 s enquanto
  um campo de texto tem foco (teclado abrindo ou fechando);
- `NativeApplication.setSystemBarIcons` define, na thread de UI, ícones claros ou
  escuros conforme a luminância do fundo sob cada barra
  (`WindowInsetsController.setSystemBarsAppearance`; flags `View` antes da API 30).
  O cabeçalho é verde-azulado nos dois temas, então a barra de status usa ícones
  claros; a barra de navegação acompanha o tema.


Pré-requisitos já instalados: JDK 17, Android SDK API 36, NDK, Rust/Cargo,
alvo `aarch64-linux-android`. `cargo xtask android` não instala alvos nem
NDK. A primeira execução do wrapper Gradle e a
resolução de dependências podem requerer rede.

Na raiz do repositório:

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
native/android/gradlew --project-dir native/android :app:assembleDebug
```

A tarefa Gradle executa `cargo xtask android` (interface, motor e interpretador `.m`) antes de empacotar
o APK, por padrão somente para `arm64-v8a`. Saída:
`native/android/app/build/outputs/apk/debug/app-debug.apk`.

Para adicionar um emulador x86_64, instale previamente o alvo Rust
`x86_64-linux-android` e selecione as ABIs:

```bash
native/android/gradlew --project-dir native/android :app:assembleDebug \
  -PnativeAbis=arm64-v8a,x86_64
```

`-PnativeOffline=true` passa `--offline` ao Cargo; `--offline` do Gradle exige
que suas dependências já estejam no cache.

Para verificar apenas os artefatos Rust, sem executar o Gradle:

```bash
cargo xtask android --abis arm64-v8a --profile debug \
  --output native/android/app/build/generated/native/debug --offline
```

`assembleRelease` usa os mesmos passos com otimização Rust. A configuração não
inclui chave de assinatura de produção; distribuição e atualização de uma
instalação existente exigem assinatura apropriada. O `applicationId` permanece
`br.com.enthony.lgrstudio`; versionName e versionCode (`X*10000 + Y*100 + Z`) vêm de `native/Cargo.toml`.
Sem `CONTROLAB_KEYSTORE` (e `CONTROLAB_KEYSTORE_PASSWORD`, `CONTROLAB_KEY_ALIAS`,
`CONTROLAB_KEY_PASSWORD`), `assembleRelease` assina com a chave debug.

## Verificação e limites

Em 28/10/2026, o antigo `build_native.py --abis arm64-v8a --profile debug --offline`
(hoje `cargo xtask android`) compilou e linkou a interface com eframe 0.36.2/JNI
0.22.4 e o motor `.m`, usando o NDK 27.3.13750724 e API 24. A inspeção ELF
confirmou AArch64 e as exportações `ANativeActivity_onCreate` e `android_main`.
O XML do manifesto também foi verificado. Essa verificação é anterior à
inclusão do motor científico Rust na biblioteca.

O APK não foi montado: a distribuição Gradle 8.14.3 não estava instalada no
ambiente e seu download automático foi interrompido para respeitar o escopo
sem instalação de dependências. Portanto, configuração Gradle, compilação Java
e execução no Android continuam sem validação real.

A aceitação no aparelho/emulador deve cobrir inicialização, teclado virtual,
rotação e retomada, os seis módulos de análise, gráficos, execução de scripts,
salvamento e reabertura de arquivos privados e preservação das preferências.
Nenhum teste em aparelho/emulador ou APK instalado foi realizado nesta mudança.

Referência: a documentação local do crate `android-activity` usado no `Cargo.lock`.

As fontes usadas por gráficos e fórmulas são incorporadas à biblioteca Rust a
partir de `assets/`. O projeto Capacitor antigo foi removido.
A validação Android foi suspensa nesta etapa por decisão do usuário; o foco
atual é Linux. Não considerar as alterações recentes de empacotamento como
APK validado.
