# <img src="src/renderer/app-icon.svg" width="24" height="24" align="absmiddle"> <strong>ControLAB — Suíte de Sistemas de Controle</strong>

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)]()

---

<p align="center">
  <a href="docs/screenshots/preview.png">
    <img src="docs/screenshots/preview.png" alt="Hub Central de Módulos do ControLAB v3.1.1" width="100%">
  </a>
  <br>
</p>

---

## Instalação e Download

Para obter a versão mais recente, acesse a seção de [Releases no GitHub](https://github.com/enthonyaraujo/controlab/releases) e baixe o pacote correspondente ao seu sistema operacional:

- **Windows:** Instalador local único (`.exe` NSIS). Instala a aplicação diretamente no sistema operacional, criando atalhos oficiais na Área de Trabalho e no Menu Iniciar, com suporte nativo a desinstalação pelo Painel de Controle.
- **Linux:** Pacotes nativos Debian/Ubuntu (`.deb`), Fedora/RedHat/openSUSE (`.rpm`) e pacote universal autônomo (`.AppImage`).
- **Android:** Aplicativo nativo compilado em pacote (`.apk`) com motor Python embutido via Chaquopy, funcionando 100% offline no celular ou tablet.
  
---

## Desenvolvimento e organização

- `src/python/`: fontes canônicas dos motores matemáticos, utilitários, ponte JSON e servidor Web.
- `src/main/`, `src/preload/` e `src/renderer/`: aplicação Electron e interface compartilhada com Web/Android.
- `android/app/src/main/python/`: cópias para o Chaquopy, atualizadas por `node scripts/sync-mobile.js`; edite os originais em `src/python/`.
- `scripts/`: execução, sincronização, empacotamento e verificações.
- `tests/`: testes Python e do renderer.
- `docs/test-systembars/`: evidências da matriz Android geradas por `scripts/test-systembars-matrix.js`, com resultados em `resultado-matriz.json`.

Com as dependências instaladas, use `npm start` para Electron, `npm run web` para navegador e `npm run web:mobile` para acesso na rede local. `npm test` executa as verificações JavaScript e os testes Python. O executor `scripts/run-python.js` inclui `src/python/` no `PYTHONPATH`, inclusive para subprocessos dos testes. Para executar testes diretamente, use `PYTHONPATH=src/python .venv/bin/python -m unittest discover -s tests` no Linux.

A CLI de Routh-Hurwitz permanece disponível por `node scripts/run-python.js src/python/routh_hurwitz.py "s^2 + 3*s + 2"` ou `./run.sh --cli "s^2 + 3*s + 2"`. As interfaces anteriores Streamlit/Tkinter e o launcher interativo foram removidos.

## Editor e execução de scripts .m

A página inicial reúne arquivos, editor com abas e realce de sintaxe, console, variáveis e figuras. Abra um `.m` pela pasta atual ou pelo botão **Abrir**; salve pelo botão **Salvar**. **F5** executa o documento ativo. O console mantém as variáveis da mesma sessão: após executar `A=[1 2;3 4];`, digite `det(A)`. **Limpar** remove as variáveis; **Limpar saída** limpa apenas o console. `plot(x,y)` e `step(G)` exibem figuras exportáveis em SVG.

A barra de pastas oferece **Voltar**, **Avançar**, **Subir um nível** e **Selecionar pasta**, além de atualizar e editar o caminho. Alt+←/Alt+→ percorrem o histórico; Alt+↑ sobe um nível. No Electron, a seleção usa o diálogo nativo; na Web, o seletor navega pelas pastas do backend local.

O motor é uma cópia independente do núcleo Rust do openMAT em `vendor/openmat/`, com licença e procedência registradas nessa pasta. A pasta original do openMAT não é necessária para compilar ou executar o ControLAB.

No desenvolvimento, além das dependências Python/Node já utilizadas, é necessário Rust/Cargo para compilar o motor:

```bash
npm run build:openmat
npm start
# Ou, para usar no navegador com o backend local:
npm run web
```

O primeiro build pode resolver dependências Rust do `Cargo.lock`; com o cache disponível, use `npm run build:openmat -- --offline`. O executável é gerado em `build/runtime/` e incluído nos recursos dos pacotes desktop. Os usuários dos instaladores não precisam de Rust. Feche a aplicação para atualizar um executável em uso no Windows.

Validação: `npm test` compila o motor e executa as verificações JS/Python; `npm run test:studio:browser` testa o fluxo do editor em um servidor local isolado usando o Chromium já instalado pelo Puppeteer. O teste de navegador não instala browsers.

Esta etapa contempla Electron e Web com backend local. A execução `.m` offline no Android exige portar/compilar o motor para Android e permanece pendente; os módulos científicos Python existentes continuam independentes. O interpretador suporta o subconjunto de MATLAB descrito em `vendor/openmat/README.md`. Os scripts têm limite de 1 MiB e a execução usa limites de tempo, trabalho, memória lógica e eventos. Variáveis persistem durante a sessão, sem restauração automática após encerrar o aplicativo.
