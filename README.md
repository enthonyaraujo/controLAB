# 📈 LGR Studio — Lugar Geométrico das Raízes (Root Locus)

> **Interface moderna, interativa e 100% offline para cálculo analítico e traçado didático do Lugar Geométrico das Raízes (LGR) com renderização matemática em tempo real.**

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)]()

---

## 📑 Sumário

- [Visão Geral](#-visão-geral)
- [Recursos Principais](#-recursos-principais)
- [Os 7 Passos Clássicos do LGR](#-os-7-passos-clássicos-do-lgr)
- [Formatos de Entrada Suportados](#-formatos-de-entrada-suportados)
- [Requisitos do Sistema](#-requisitos-do-sistema)
- [Instalação](#-instalação)
- [Como Executar](#-como-executar)
  - [1. Interface Desktop Electron (Padrão)](#1-interface-desktop-electron-padrão)
  - [2. Navegador Local Privado (127.0.0.1)](#2-navegador-local-privado-127001)
  - [3. Acesso Móvel na Rede Local (Wi-Fi)](#3-acesso-móvel-na-rede-local-wi-fi)
  - [4. Interfaces Alternativas (Streamlit / Tkinter)](#4-interfaces-alternativas-streamlit--tkinter)
- [Gerar APK e Executáveis](#-gerar-apk-e-executáveis)
  - [Android — APK e instalação no tablet](#android--apk-e-instalação-no-tablet)
  - [Linux — AppImage e DEB](#linux--appimage-e-deb)
  - [Windows — instalador e executável portátil](#windows--instalador-e-executável-portátil)
- [Estrutura do Projeto](#-estrutura-do-projeto)
- [Validação e Testes Automatizados](#-validação-e-testes-automatizados)
- [Privacidade e Execução 100% Local](#-privacidade-e-execução-100-local)
- [Regra de Integridade Algorítmica](#-regra-de-integridade-algorítmica)

---

## 🌟 Visão Geral

O **LGR Studio** é uma aplicação desktop e web projetada para estudantes, engenheiros e pesquisadores de **Engenharia de Controle** e **Sistemas Dinâmicos**. 

Ele automatiza e explica visualmente o traçado do **Lugar Geométrico das Raízes (Root Locus)** para qualquer função de transferência em malha aberta:

$$G(s)H(s) = \frac{N(s)}{D(s)} = K \frac{\prod_{j=1}^{Z} (s - z_j)}{\prod_{i=1}^{P} (s - p_i)}$$

O motor científico é executado integralmente em **Python** (`control`, `NumPy`, `SciPy`, `Matplotlib` e `SymPy`), enquanto a interface em **Electron/HTML5/CSS3** oferece uma experiência moderna com visual escuro/claro, pré-visualização LaTeX instantânea e memorial de cálculo passo a passo.

---

## ✨ Recursos Principais

- **🎨 Design Moderno & Acessível:**
  - Tema Escuro (*Dark Mode*) com tipografia branca de alto contraste e realces em azul.
  - Alternador dinâmico para Tema Claro (*Light Mode*).
  - Layout totalmente responsivo para desktop, tablet e celular.
- **📐 Fórmulas em Tempo Real (KaTeX 100% Offline):**
  - Pré-visualização da função de transferência enquanto você digita.
  - Exibição simultânea da forma polinomial expandida e da forma fatorada (polos e zeros).
- **🔍 Visualizador Gráfico Interativo:**
  - **Zoom inteligente:** ampliação de até `6.0x` com trava de limite mínimo em `100%` (não diminui além do tamanho visível original).
  - Arraste livre (*pan*) ativado automaticamente ao ampliar.
  - Exportação direta em **PNG de Alta Resolução (300 DPI)** e **Vetorial SVG**.
  - Botão de **Copiar Gráfico** para a Área de Transferência.
- **📑 Memorial de Cálculo dos 7 Passos:**
  - Detalhamento analítico completo de cada etapa do método clássico.
- **📱 Suporte Multiplataforma:**
  - Aplicativo Android offline por Capacitor e Chaquopy.
  - Aplicativo desktop por Electron, com AppImage/DEB para Linux e NSIS/portátil para Windows.
  - Interface web responsiva e PWA para acesso rápido pelo navegador.

---

## 📚 Os 7 Passos Clássicos do LGR

O algoritmo calcula e ilustra rigorosamente os 7 passos clássicos da teoria de controle (Ogata, Nise, Dorf):

1. **Passos 1, 2 e 3 (Ramos, Polos, Zeros e Simetria):**
   - Início dos ramos nos polos de malha aberta ($K = 0$) e término nos zeros finitos ou no infinito ($K \to \infty$).
   - Total de ramos: $n = \max(P, Z)$.
   - Simetria do LGR em relação ao Eixo Real ($\sigma$).
   - Segmentos no eixo real à esquerda de um número ímpar de polos e zeros reais.
2. **Passo 4 (Assíntotas e Centroide):**
   - Centroide das assíntotas:
     $$\sigma_a = \frac{\sum_{i=1}^P \text{Re}(p_i) - \sum_{j=1}^Z \text{Re}(z_j)}{P - Z}$$
   - Ângulos das assíntotas:
     $$\theta_k = \frac{(2k + 1) \cdot 180^\circ}{P - Z}, \quad k = 0, \dots, P - Z - 1$$
3. **Passo 5 (Pontos de Partida e Retorno - Breakaway / Break-in):**
   - Raízes da derivada: $\frac{dK}{ds} = 0 \iff N'(s)D(s) - N(s)D'(s) = 0$.
   - Verificação de raízes reais sobre o LGR com ganho $K_{break} = -\frac{D(s)}{N(s)} > 0$.
4. **Passo 6 (Cruzamento do Eixo Imaginário $j\omega$):**
   - Limite de estabilidade em malha fechada ($s = \pm j\omega_{cruz}$) e ganho crítico $K_{crit}$.
5. **Passo 7 (Ângulos de Partida e Chegada):**
   - Ângulo de partida em polos complexos: $\theta_d = 180^\circ + \sum \angle(p - z_j) - \sum_{k \neq p} \angle(p - p_k)$.
   - Ângulo de chegada em zeros complexos: $\theta_a = 180^\circ - \sum_{j \neq z} \angle(z - z_j) + \sum \angle(z - p_k)$.

---

## 🎯 Formatos de Entrada Suportados

A aplicação possui parser tolerante e inteligente via SymPy:

| Modo | Exemplo de Sintaxe | Descrição |
| :--- | :--- | :--- |
| **Expressão Completa** | `(s + 2) / (s * (s + 1) * (s + 4))` | Digitação livre com `*`, `^`, `s²`, `s³`, `÷`, `×` |
| **Numerador / Denominador** | Num: `s + 2` <br> Den: `s(s + 1)(s + 4)` | Campos separados para numerador e denominador |
| **Coeficientes Polinomiais** | $N(s)$: `1, 2` <br> $D(s)$: `1, 5, 4, 0` | Listas de coeficientes em ordem decrescente de potência |
| **Forma Fatorada (ZPK)** | Zeros: `-2` <br> Polos: `0, -1, -4` <br> Ganho: `1.0` | Inserção direta de polos/zeros reais e complexos (`-1+2j`) |
| **Biblioteca de Presets** | Menu suspenso de casos clássicos | Presets de livros consagrados (Ogata, Nise, Dorf) |

---

## 💻 Requisitos do Sistema

- **Node.js:** `v22.12` ou superior
- **Python para desenvolvimento desktop/web:** `v3.12` ou superior
- **Sistemas Operacionais:** Linux, Windows 10/11 e Android 7.0 ou superior

Para gerar APK também são necessários JDK 17+, Android Studio ou Android SDK 36 e Android Platform Tools (`adb`). O Python 3.10 usado dentro do APK é fornecido pelo Chaquopy e não precisa ser instalado no tablet.

---

## 📦 Instalação

### Linux / macOS:
```bash
# 1. Clonar o repositório
git clone https://github.com/enthonyaraujo/lgr.git
cd lgr

# 2. Instalar dependências Node.js
npm install

# 3. Criar e ativar ambiente virtual Python
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -r requirements.txt
```

### Windows (PowerShell):
```powershell
git clone https://github.com/enthonyaraujo/lgr.git
cd lgr

npm install
py -3.12 -m venv .venv
.venv\Scripts\Activate.ps1
python -m pip install -r requirements.txt
```

---

## 🚀 Como Executar

### 1. Interface Desktop Electron (Padrão)
```bash
npm start
```
*ou via script unificado no Linux/macOS:*
```bash
./run.sh
```

### 2. Navegador Local Privado (127.0.0.1)
Executa um servidor HTTP local fechado para a sua própria máquina:
```bash
npm run web
```
Acesse [http://127.0.0.1:8501](http://127.0.0.1:8501).

### 3. Acesso Móvel na Rede Local (Wi-Fi)
Permite acessar a interface pelo celular ou tablet na mesma rede local:
```bash
npm run web:mobile
```
Abra `http://<IP-DO-COMPUTADOR>:8501` no dispositivo móvel.

### 4. Interfaces Alternativas (Streamlit / Tkinter)
```bash
# Interface Web Streamlit
npm run web:streamlit
# ou: ./run.sh --web

# Interface Desktop Nativa Tkinter
npm run gui
# ou: ./run.sh --gui
```

## 📦 Gerar APK e executáveis

Os artefatos são autocontidos: o APK leva o motor Python pelo Chaquopy e os pacotes Electron levam um executável do motor criado pelo PyInstaller. O mesmo `lgr_engine.py` é sincronizado antes de cada empacotamento, sem alterar os sete passos do LGR.

### Android — APK e instalação no tablet

Requisitos adicionais: Android Studio/SDK 36, JDK 17 ou mais recente e `adb` no `PATH`. Ative a depuração USB no tablet e autorize o computador.

```bash
# Sincroniza interface e motor Python sem gerar APK
npm run mobile:sync

# Gera android/app/build/outputs/apk/debug/app-debug.apk
npm run mobile:apk

# Gera e instala o APK no dispositivo USB conectado
npm run mobile:install

# Opcional: abre o projeto no Android Studio
npm run mobile:open
```

Para instalar manualmente um APK já gerado:

```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

O APK é compatível com tablets e celulares Android 7.0/API 24 ou mais recentes, em `arm64-v8a`. O emulador `x86_64` também está habilitado. O cálculo é local e não exige que o desktop esteja ligado.

O Chaquopy 17 é mantido e, para Android, disponibiliza wheels próprios das bibliotecas científicas. A wheel SciPy 1.8.1 ainda aparece no PyPA como `PYSEC-2023-102`; o aviso descreve um possível vazamento de memória e foi retirado no GitHub Advisory. Essa versão é isolada ao APK porque ainda é a última wheel Android compatível fornecida pelo repositório Chaquopy; desktop e web continuam usando SciPy atual.

### Linux — AppImage e DEB

Execute em Linux:

```bash
source .venv/bin/activate
python -m pip install -r requirements-build.txt
npm run package:linux
```

São gerados em `release/` um AppImage portátil e um pacote DEB. O empacotamento Linux deve ser executado no Linux porque o motor Python incluído é específico da plataforma.

### Windows — instalador e executável portátil

Execute no próprio Windows, pois o motor PyInstaller é específico do sistema operacional:

```powershell
.venv\Scripts\Activate.ps1
python -m pip install -r requirements-build.txt
npm run package:windows
```

O comando gera em `release/` um instalador NSIS e um executável portátil. Não tente gerar o pacote Windows no Linux: o script interrompe com uma explicação para impedir que um binário Python incompatível seja incluído.

### Resumo dos comandos de empacotamento

| Plataforma | Comando | Resultado |
| :--- | :--- | :--- |
| Android | `npm run mobile:apk` | `android/app/build/outputs/apk/debug/app-debug.apk` |
| Android + USB | `npm run mobile:install` | Gera e instala no dispositivo autorizado |
| Linux | `npm run package:linux` | AppImage e DEB em `release/` |
| Windows | `npm run package:windows` | NSIS e portátil em `release/` |

Os comandos desktop exigem previamente `python -m pip install -r requirements-build.txt`. Cada pacote desktop deve ser gerado no próprio sistema operacional de destino.

---

## 📁 Estrutura do Projeto

```text
lgr/
├── src/
│   ├── main/
│   │   └── main.js              # Processo Principal do Electron e IPC
│   ├── preload/
│   │   └── preload.js           # ContextBridge isolado e seguro
│   └── renderer/
│       ├── index.html           # Interface visual compartilhada
│       ├── styles.css           # Design System responsivo (Dark/Light mode)
│       ├── app.js               # Lógica do renderer (KaTeX, Zoom, Pan, Memorial)
│       ├── web-api.js           # Adaptador de API para navegadores
│       ├── manifest.webmanifest # Manifesto PWA
│       ├── sw.js                # Service Worker para cache local offline
│       └── vendor/katex/        # KaTeX e fontes 100% offline
├── scripts/
│   ├── start-electron.js        # Inicializador seguro do Electron
│   ├── run-python.js            # Wrapper para o interpretador Python virtualenv
│   ├── sync-mobile.js           # Sincroniza o motor canônico com o Android
│   ├── build-python.js          # Empacota o motor desktop com PyInstaller
│   ├── run-gradle.js            # Executa tarefas Gradle de forma multiplataforma
│   └── install-apk.js           # Instala o APK por adb
├── android/                     # Projeto Capacitor, plugin Chaquopy e Gradle
├── tests/
│   └── test_transfer_inputs.py  # Testes de regressão matemática e parser
├── lgr_engine.py                # Motor de cálculo dos 7 passos clássicos
├── python_bridge.py             # Ponte de comunicação IPC JSON stdio
├── web_server.py                # Servidor HTTP local nativo
├── app.py                       # Interface alternativa Streamlit
├── gui.py                       # Interface alternativa Tkinter
├── main.py                      # Launcher unificado em Python
├── package.json                 # Manifesto Node.js e scripts
├── requirements.txt             # Dependências científicas Python
├── requirements-build.txt       # PyInstaller usado no empacotamento desktop
├── capacitor.config.json        # Configuração do aplicativo Android
├── electron-builder.config.cjs  # Configuração dos pacotes Linux/Windows
└── run.sh                       # Script bash de execução rápida
```

---

## 🧪 Validação e Testes Automatizados

Para executar toda a suíte de testes (validação sintática de JavaScript e testes analíticos de regressão em Python):

```bash
npm test
```

---

## 🔒 Privacidade e Execução 100% Local

- **Zero dependências externas:** Todas as fontes e scripts do KaTeX estão empacotados localmente.
- **CSP Estrita:** Política de segurança que bloqueia conexões a domínios de terceiros.
- **Privacidade total:** Nenhum dado de entrada ou gráfico é enviado para a nuvem. Todos os cálculos ocorrem localmente no processador da sua máquina.

---

## ⚖️ Regra de Integridade Algorítmica

A função `lgr_completo` em `lgr_engine.py` representa a implementação matemática canônica dos **7 passos clássicos**. Melhorias de interface, estilização ou empacotamento não alteram as rotinas analíticas nem a metodologia de cálculo.
