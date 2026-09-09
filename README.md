# <img src="src/renderer/app-icon.svg" width="24" height="24" align="absmiddle"> <strong>ControLAB — Suíte de Sistemas de Controle</strong>

> **Interface moderna, interativa e 100% offline para cálculo analítico, simulação e ensino de Sistemas de Controle com renderização matemática em tempo real.**

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)]()

---

## Instalação e Download

Para obter a versão mais recente, acesse a seção de [Releases no GitHub](https://github.com/enthonyaraujo/lgr/releases) e baixe o pacote correspondente ao seu sistema operacional:

- **Windows:** Instalador local único (`.exe` NSIS). Instala a aplicação diretamente no sistema operacional, criando atalhos oficiais na Área de Trabalho e no Menu Iniciar, com suporte nativo a desinstalação pelo Painel de Controle.
- **Linux:** Pacotes nativos Debian/Ubuntu (`.deb`), Fedora/RedHat/openSUSE (`.rpm`) e pacote universal autônomo (`.AppImage`).
- **Android:** Aplicativo nativo compilado em pacote (`.apk`) com motor Python embutido via Chaquopy, funcionando 100% offline no celular ou tablet.
- **Web / PWA:** Execução direta no navegador com suporte a Progressive Web App (PWA) e cache local completo via Service Worker.

---

## Visão Geral

O **ControLAB** é uma plataforma educacional e analítica de Engenharia de Controle e Sistemas Dinâmicos. A ferramenta combina o rigor matemático do ecossistema científico em **Python** (`control`, `NumPy`, `SciPy`, `Matplotlib` e `SymPy`) com a agilidade de uma interface moderna em **Electron/HTML5/CSS3**.

A aplicação organiza os domínios clássicos e modernos de controle a partir de um Hub Central e oferece memorial passo a passo, equações renderizadas e gráficos vetoriais interativos.

---

## Módulos do Sistema

A partir da Página Inicial (Hub Central), o usuário pode acessar os módulos especializados:

1. **Lugar Geométrico das Raízes (LGR):**
   - Cálculo analítico e traçado em tempo real das trajetórias de polos em malha fechada para ganhos $0 \le K < \infty$.
   - Memorial passo a passo dos 7 passos clássicos (Ogata, Nise, Dorf).
   - Manipulação gráfica com zoom até 6.0x, pan livre, exportação em SVG vetorial e PNG a 300 DPI.
2. **Análise de Estabilidade:**
   - Construção analítica da tabela do Critério de Routh-Hurwitz.
   - Tratamento de casos especiais: primeiro elemento nulo ($\epsilon \to 0$) e linha inteiramente nula.
   - Determinação da faixa de ganho estável $K_{min} < K < K_{max}$ e frequências de oscilação crítica.
3. **Resposta no Domínio do Tempo:**
   - Simulação transitória e regime permanente para entradas degrau, impulso e rampa.
   - Extração automática de métricas: sobressinal percentual ($\%OS$), tempo de acomodação ($t_s$), tempo de pico ($t_p$), tempo de subida ($t_r$), fator de amortecimento ($\zeta$) e frequência natural ($\omega_n$).
   - Coeficientes de erro estático ($K_p$, $K_v$, $K_a$) e erro em regime permanente ($e_{ss}$).
4. **Resposta em Frequência:**
   - Diagramas de Bode de magnitude (dB) e fase (graus) com escalas logarítmicas assintóticas e reais.
   - Diagrama polar de Nyquist e contorno de estabilidade.
   - Cálculo das margens de ganho ($MG$) e de fase ($MF$), além das frequências de cruzamento.
5. **Projeto de Controladores:**
   - Sintonia de controladores PID pelos métodos clássicos de Ziegler-Nichols (resposta ao degrau e ganho crítico).
   - Síntese de compensadores por avanço de fase (Lead) e atraso de fase (Lag) via LGR.
   - Comparativo de desempenho em malha fechada antes e após a compensação.
6. **Espaço de Estados:**
   - Modelagem matricial contínua $(\mathbf{A}, \mathbf{B}, \mathbf{C}, \mathbf{D})$ e conversão para função de transferência.
   - Testes de controlabilidade ($\mathcal{C}$) e observabilidade ($\mathcal{O}$).
   - Alocação de polos por realimentação de estados via fórmula de Ackermann e síntese de observadores de Luenberger.

---

## Recursos Principais

- **Detecção Inteligente do Sistema e Atualizações Condicionais:**
  - Identificação automática do sistema operacional e do tipo de pacote em execução (`.deb`, `.rpm`, `.AppImage`, `.exe`, `.apk`).
  - Verificador de atualizações integrado via GitHub Releases: **só exibe botões de download e instaladores se houver de fato uma versão mais recente disponível** para a plataforma do usuário.
  - Indicador luminoso discreto na barra superior ao identificar atualizações publicadas.
  - Suporte opcional a Token Pessoal do GitHub para limites estendidos de requisições.
- **Fórmulas Matemáticas em Tempo Real (KaTeX Offline):**
  - Renderização tipográfica de equações, intervalos do eixo real, singularidades complexas e derivadas sem necessidade de conexão à internet.
- **Visualizador Gráfico do LGR:**
  - Gráficos científicos com zoom inteligente até 6.0x e arrasto livre (pan).
  - Exportação direta em PNG de alta resolução (300 DPI) e formato vetorial SVG.
  - Cópia instantânea do gráfico para a área de transferência do sistema.
- **Design System Acessível:**
  - Tema Escuro (*Dark Mode*) de alto contraste e Tema Claro (*Light Mode*) com alternância em tempo real.
  - Interface responsiva adaptada para desktop, tablet e celular.

---

## Os 7 Passos Clássicos do LGR

O módulo do LGR detalha cada uma das etapas teóricas consagradas:

1. **Passos 1, 2 e 3 (Ramos, Polos, Zeros e Simetria):**
   - Ponto de partida dos ramos nos polos ($K = 0$) e término nos zeros ou infinito ($K \to \infty$).
   - Contagem total de ramos: $n = \max(P, Z)$.
   - Simetria do traçado em relação ao Eixo Real ($\sigma$).
   - Segmentos reais à esquerda de contagem ímpar de singularidades reais.
2. **Passo 4 (Assíntotas e Centroide):**
   - Centroide no eixo real:
     $$\sigma_a = \frac{\sum_{i=1}^P \text{Re}(p_i) - \sum_{j=1}^Z \text{Re}(z_j)}{P - Z}$$
   - Ângulos das assíntotas:
     $$\theta_k = \frac{(2k + 1) \cdot 180^\circ}{P - Z}, \quad k = 0, \dots, P - Z - 1$$
3. **Passo 5 (Pontos de Partida e Retorno - Breakaway / Break-in):**
   - Solução analítica da equação: $\frac{dK}{ds} = 0 \iff N'(s)D(s) - N(s)D'(s) = 0$.
   - Validação de raízes reais sobre o LGR com ganho real positivo $K > 0$.
4. **Passo 6 (Cruzamento do Eixo Imaginário $j\omega$):**
   - Verificação de oscilação sustentada e ganho crítico de estabilidade $K_{crit}$.
5. **Passo 7 (Ângulos de Partida e Chegada):**
   - Ângulos de saída em polos complexos e chegada em zeros complexos por somatório de fases.

---

## Formatos de Entrada

O parser simbólico aceita múltiplos métodos de definição do sistema:

| Modo | Exemplo de Entrada | Descrição |
| :--- | :--- | :--- |
| **Expressão Completa** | `(s + 2) / (s * (s + 1) * (s + 4))` | Entrada com multiplicação explícita ou implícita, potências `^` e Unicode (`s²`, `s³`) |
| **Fração (Num / Den)** | Num: `s + 2` <br> Den: `s(s + 1)(s + 4)` | Campos dedicados para numerador e denominador |
| **Coeficientes Polinomiais** | $N(s)$: `1, 2` <br> $D(s)$: `1, 5, 4, 0` | Listas de coeficientes ordenados por potências decrescentes de $s$ |
| **Polos e Zeros (ZPK)** | Zeros: `-2` <br> Polos: `0, -1, -4` <br> Ganho: `1.0` | Entrada direta de raízes reais e complexas (`-1+2j`) |
| **Presets Clássicos** | Menu de sistemas predefinidos | Exemplos clássicos da literatura técnica |

---

## Estrutura do Projeto

```text
lgr/
├── src/
│   ├── main/
│   │   └── main.js              # Processo Principal do Electron, IPC e detecção de sistema
│   ├── preload/
│   │   └── preload.js           # ContextBridge seguro (APIs expostas para o renderer)
│   └── renderer/
│       ├── index.html           # Interface visual estruturada (Hub e LGR)
│       ├── styles.css           # Design System responsivo (Dark/Light mode)
│       ├── app.js               # Controlador do LGR (KaTeX, Zoom, Pan, Memorial)
│       ├── navigation.js        # Gerenciador de navegação entre páginas
│       ├── settings.js          # Modal de configurações, detecção de OS e atualizações
│       ├── web-api.js           # Adaptador de API para navegadores e Capacitor
│       ├── manifest.webmanifest # Manifesto PWA
│       ├── sw.js                # Service Worker para funcionamento 100% offline
│       └── vendor/katex/        # Biblioteca KaTeX e fontes matemáticas locais
├── scripts/
│   ├── start-electron.js        # Inicializador seguro do Electron
│   ├── run-python.js            # Executor do interpretador Python no ambiente virtual
│   ├── sync-mobile.js           # Sincronização do motor científico com o Android
│   ├── build-python.js          # Empacotamento do motor desktop via PyInstaller
│   ├── run-gradle.js            # Execução de rotinas Gradle de compilação
│   ├── install-apk.js           # Instalação automatizada via adb
│   ├── validate-packaging.js    # Verificação de configurações de empacotamento
│   └── check-javascript.js     # Checagem estática de integridade do código JavaScript
├── android/                     # Projeto Android nativo (Capacitor e Chaquopy)
├── tests/                       # Suíte de testes unitários e de integração
├── lgr_engine.py                # Motor de cálculo dos 7 passos clássicos
├── python_bridge.py             # Ponte IPC JSON stdio
├── web_server.py                # Servidor HTTP local para modo navegador
├── package.json                 # Manifesto do projeto e comandos de build
├── requirements.txt             # Dependências científicas em Python
├── requirements-build.txt       # Dependências de compilação (PyInstaller)
├── capacitor.config.json        # Configuração do aplicativo móvel Android
├── electron-builder.config.cjs  # Configuração de empacotamento desktop (Linux e Windows NSIS)
└── run.sh                       # Script de inicialização rápida em ambiente Unix
```

---

## Validação e Testes

Para rodar a suíte completa de testes de integridade de JavaScript e testes unitários de controle em Python:

```bash
npm test
```

Para sincronizar o motor Python e os recursos da interface com o projeto Android:

```bash
npm run mobile:sync
```
