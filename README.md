# 📈 LGR Studio — Lugar Geométrico das Raízes (Root Locus)

> **Interface moderna, interativa e 100% offline para cálculo analítico e traçado didático do Lugar Geométrico das Raízes (LGR) com renderização matemática em tempo real.**

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)]()

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
