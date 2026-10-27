# <img src="src/renderer/app-icon.svg" width="24" height="24" align="absmiddle"> <strong>ControLAB — Suíte de Sistemas de Controle</strong>

[English](README.md) | **Português (Brasil)**

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)

---

<p align="center">
  <a href="docs/screenshots/preview_dark.png">
    <img src="docs/screenshots/preview_dark.png" alt="Hub principal de módulos do ControLAB v3.1.1" width="100%">
  </a>
  <br>
</p>

<p align="center">
  <a href="docs/screenshots/preview_white.png">
    <img src="docs/screenshots/preview_white.png" alt="Hub principal de módulos do ControLAB v3.1.1" width="100%">
  </a>
  <br>
</p>

---

## Instalação e Download

Para obter a versão mais recente, acesse a página de [GitHub Releases](https://github.com/enthonyaraujo/controlab/releases) e baixe o pacote correspondente ao seu sistema operacional:

- **Windows:** Instalador independente (`.exe`, NSIS). Instala o aplicativo diretamente no sistema operacional, cria atalhos na Área de Trabalho e no Menu Iniciar e permite a desinstalação nativa pelo Painel de Controle.
- **Linux:** Pacotes nativos para Debian/Ubuntu (`.deb`), Fedora/Red Hat/openSUSE (`.rpm`) e um pacote universal independente (`.AppImage`).
- **Android:** Aplicativo nativo compilado como pacote `.apk`, com um motor Python embarcado via Chaquopy, permitindo o funcionamento totalmente offline em smartphones e tablets.

---

## Desenvolvimento e Estrutura do Projeto

- `src/python/`: Arquivos-fonte canônicos dos motores matemáticos, utilitários, ponte JSON e servidor web.
- `src/main/`, `src/preload/` e `src/renderer/`: Aplicativo Electron e interface compartilhada com as versões Web e Android.
- `android/app/src/main/python/`: Cópias usadas pelo Chaquopy, sincronizadas por `node scripts/sync-mobile.js`. Faça as alterações nos arquivos originais em `src/python/`.
- `scripts/`: Scripts de execução, sincronização, empacotamento e validação do aplicativo.
- `tests/`: Testes de Python e do renderer.
- `docs/test-systembars/`: Evidências da matriz de testes do Android geradas por `scripts/test-systembars-matrix.js`, com os resultados armazenados em `resultado-matriz.json`.

Com as dependências instaladas, execute `npm start` para abrir o Electron, `npm run web` para abrir a versão no navegador ou `npm run web:mobile` para permitir o acesso pela rede local.

Execute `npm test` para rodar as verificações de JavaScript e os testes de Python. O executor `scripts/run-python.js` adiciona `src/python/` ao `PYTHONPATH`, inclusive nos subprocessos dos testes.

Para rodar os testes diretamente no Linux, use:

`PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests`

A CLI de Routh-Hurwitz continua disponível por:

`node scripts/run-python.js src/python/routh_hurwitz.py "s^2 + 3*s + 2"`

Ou:

`./run.sh --cli "s^2 + 3*s + 2"`

As antigas interfaces Streamlit/Tkinter e o iniciador interativo foram removidos.

## Editor e Execução de Scripts .m

A tela inicial oferece um navegador de arquivos, um editor com abas e realce de sintaxe, um console, um workspace de variáveis e a visualização de figuras.

Abra um arquivo `.m` pelo diretório atual ou clicando em **Abrir**. Salve as alterações com **Salvar**. Pressione **F5** para executar o documento ativo.

O console mantém as variáveis durante a mesma sessão. Por exemplo, depois de executar `A=[1 2;3 4];`, digite `det(A)` no console.

**Limpar** remove todas as variáveis da sessão, enquanto **Limpar saída** limpa apenas a saída do console.

Comandos como `plot(x,y)` e `step(G)` geram figuras que podem ser exportadas como arquivos SVG.

A barra de navegação de diretórios oferece os controles **Voltar**, **Avançar**, **Subir um nível** e **Selecionar pasta**, além de opções para atualizar e editar o caminho atual.

Atalhos de teclado:

- **Alt+← / Alt+→:** Volta e avança no histórico de diretórios.
- **Alt+↑:** Sobe um nível de diretório.

No Electron, a seleção de pastas usa a caixa de diálogo nativa do sistema operacional. Na versão Web, o seletor de pastas navega pelos diretórios expostos pelo backend local.

O motor de execução é uma cópia independente do núcleo Rust do openMAT, localizada em `vendor/openmat/`, com sua licença e procedência documentadas nesse diretório.

O diretório original do projeto openMAT não é necessário para compilar ou executar o ControLAB.

Durante o desenvolvimento, Rust e Cargo são necessários para compilar o motor, além das dependências de Python e Node.js já existentes:

```bash
npm run build:openmat
npm start

# Ou execute no navegador com o backend local:
npm run web
```

A primeira compilação pode precisar baixar as dependências Rust especificadas em `Cargo.lock`. Depois que as dependências estiverem em cache, a compilação offline fica disponível com:

`npm run build:openmat -- --offline`

O executável é gerado em `build/runtime/` e incluído nos pacotes do aplicativo desktop.

Quem instala as versões pré-compiladas não precisa de Rust.

No Windows, feche o aplicativo antes de atualizar um executável que esteja em uso.

### Testes e Validação

Execute `npm test` para compilar o motor e rodar as verificações de JavaScript e Python.

Execute `npm run test:studio:browser` para testar o fluxo do editor usando um servidor local isolado e o Chromium já fornecido pelo Puppeteer.

O teste no navegador não instala navegadores adicionais.

### Implementação Atual e Limitações

Os scripts `.m` rodam no Electron, na versão Web com backend local e no Android.

No Android, o motor é compilado com o Android NDK para `arm64-v8a` e `x86_64` e incluído no APK como `libcontrolab_runtime.so`, então os scripts rodam totalmente offline. As figuras são exibidas dentro do editor. Os módulos científicos em Python continuam funcionando de forma independente.

O interpretador oferece suporte ao subconjunto da linguagem MATLAB documentado em `vendor/openmat/README.md`.

Os scripts são limitados a **1 MiB**, e a execução está sujeita a limites de tempo, carga de trabalho, memória lógica e eventos.

As variáveis ficam disponíveis durante a sessão ativa, mas não são restauradas automaticamente depois que o aplicativo é fechado.
