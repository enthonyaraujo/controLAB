# Android nativo

A interface Android usa o mesmo cliente Rust/egui do desktop, renderizado por
OpenGL ES em `android.app.NativeActivity`. Não há WebView, Capacitor ou servidor
HTTP neste caminho. O projeto Gradle independente fica em `native/android`.

O APK inclui três componentes:

- `libcontrolab_native.so`: a interface Rust e o ciclo de vida `android-activity`;
- Python 3.10/Chaquopy 17.0.0 e os módulos canônicos de `src/python`, com as mesmas
  versões científicas de `android/app/requirements-android.txt`;
- `libcontrolab_runtime.so`: o executável Rust do motor `.m`, extraído para
  `nativeLibraryDir` e iniciado pelo `script_service.py` canônico.

O worker Rust anexa sua thread à JVM, chama `NativeApplication.dispatchJson` por
JNI e recebe JSON de `native_presentation.dispatch_android` (que preserva a compatibilidade de
`android_bridge.dispatch_json` e acrescenta a composição local de fórmulas). A aplicação Java serializa
as chamadas do motor e mantém Python durante recriações da Activity. O Rust
retém uma referência global à Application, sem reter janelas antigas.

As preferências e a pasta inicial de scripts usam `Application.getFilesDir()`.
Esses arquivos persistem entre reinícios e atualizações; a desinstalação remove
os dados privados. A exploração por caminhos fica limitada aos diretórios que
o aplicativo pode acessar. Importação/exportação pelo seletor de documentos e
compartilhamento pela galeria exigem integração Android específica posterior.

## Compilar

Pré-requisitos já instalados: JDK 17, Android SDK API 36, NDK, Rust/Cargo,
alvo `aarch64-linux-android` e Python 3.10 para o Chaquopy. O script de compilação
não instala alvos, NDK ou Python. A primeira execução do wrapper Gradle e a
resolução de dependências podem requerer rede.

Na raiz do repositório:

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export LGR_ANDROID_PYTHON=/caminho/para/python3.10
native/android/gradlew --project-dir native/android :app:assembleDebug
```

A tarefa Gradle compila a biblioteca Rust e o executável `.m` antes de empacotar
o APK, por padrão somente para `arm64-v8a`. Saída:
`native/android/app/build/outputs/apk/debug/app-debug.apk`.

Para adicionar um emulador x86_64, instale previamente o alvo Rust
`x86_64-linux-android` e selecione as ABIs:

```bash
native/android/gradlew --project-dir native/android :app:assembleDebug \
  -PnativeAbis=arm64-v8a,x86_64
```

`-PnativeOffline=true` passa `--offline` ao Cargo; `--offline` do Gradle exige
que suas dependências já estejam no cache. `-PnativeBuildPython=/caminho/python3`
seleciona o Python que executa `build_native.py` (independente do Python 3.10
selecionado por `LGR_ANDROID_PYTHON`).

Para verificar apenas os artefatos Rust, sem executar o Gradle:

```bash
python3 native/android/build_native.py --abis arm64-v8a --profile debug \
  --output native/android/app/build/generated/native/debug --offline
```

`assembleRelease` usa os mesmos passos com otimização Rust. A configuração não
inclui chave de assinatura de produção; distribuição e atualização de uma
instalação existente exigem assinatura apropriada. O `applicationId` permanece
`br.com.enthony.lgrstudio`; a versão declarada permanece 5.0.0/código 25.

## Verificação e limites

Em 28/10/2026, `build_native.py --abis arm64-v8a --profile debug --offline`
compilou e linkou a interface com eframe 0.36.2/JNI 0.22.4 e o motor `.m`, usando
o NDK 27.3.13750724 e API 24. A inspeção ELF confirmou AArch64 e as exportações
`ANativeActivity_onCreate` e `android_main`. A sintaxe do script Python e o XML
do manifesto também foram verificados.

O APK não foi montado: a distribuição Gradle 8.14.3 não estava instalada no
ambiente e seu download automático foi interrompido para respeitar o escopo
sem instalação de dependências. Portanto, configuração Gradle, compilação Java,
empacotamento Chaquopy e execução no Android continuam sem validação real.

A aceitação no aparelho/emulador deve cobrir inicialização, teclado virtual,
rotação e retomada, os seis módulos de análise, gráficos, execução de scripts,
salvamento e reabertura de arquivos privados e preservação das preferências.
Nenhum teste em aparelho/emulador ou APK instalado foi realizado nesta mudança.

Referências: [Chaquopy: configuração e fontes Python](https://chaquo.com/chaquopy/doc/current/android.html)
e a documentação local do crate `android-activity` usado no `Cargo.lock`.
