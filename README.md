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
