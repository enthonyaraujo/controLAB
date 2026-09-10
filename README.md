# <img src="src/renderer/app-icon.svg" width="24" height="24" align="absmiddle"> <strong>ControLAB — Suíte de Sistemas de Controle</strong>

> **Interface moderna, interativa e 100% offline para cálculo analítico, simulação e ensino de Sistemas de Controle com renderização matemática em tempo real.**

[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg?logo=python&logoColor=white)](https://www.python.org/)
[![Electron](https://img.shields.io/badge/Electron-44.1.0-47848F.svg?logo=electron&logoColor=white)](https://www.electronjs.org/)
[![KaTeX](https://img.shields.io/badge/KaTeX-Offline%20Math-00d1b2.svg)](https://katex.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Android%20%7C%20Linux%20%7C%20Windows%20%7C%20Web-lightgrey.svg)]()

---

<p align="center">
  <a href="docs/screenshots/principal.webp">
    <img src="docs/screenshots/principal.webp" alt="Visão Geral da Suíte ControLAB v3.0.0" width="100%">
  </a>
  <br>
  <em>Visão Geral da Suíte ControLAB v3.0.0 em execução no desktop. Clique em qualquer imagem para abrir em alta resolução.</em>
</p>

---

## Instalação e Download

Para obter a versão mais recente, acesse a seção de [Releases no GitHub](https://github.com/enthonyaraujo/controlab/releases) e baixe o pacote correspondente ao seu sistema operacional:

- **Windows:** Instalador local único (`.exe` NSIS). Instala a aplicação diretamente no sistema operacional, criando atalhos oficiais na Área de Trabalho e no Menu Iniciar, com suporte nativo a desinstalação pelo Painel de Controle.
- **Linux:** Pacotes nativos Debian/Ubuntu (`.deb`), Fedora/RedHat/openSUSE (`.rpm`) e pacote universal autônomo (`.AppImage`).
- **Android:** Aplicativo nativo compilado em pacote (`.apk`) com motor Python embutido via Chaquopy, funcionando 100% offline no celular ou tablet.
- **Web / PWA:** Execução direta no navegador com suporte a Progressive Web App (PWA) e cache local completo via Service Worker.

---

## Visão Geral

O **ControLAB** é uma plataforma educacional e analítica de Engenharia de Controle e Sistemas Dinâmicos. A ferramenta combina o rigor matemático do ecossistema científico em **Python** (`control`, `NumPy`, `SciPy`, `Matplotlib` e `SymPy`) com a agilidade de uma interface moderna em **Electron/HTML5/CSS3**.

A aplicação organiza os domínios clássicos e modernos de controle a partir de um Hub Central e oferece memorial passo a passo, equações renderizadas e gráficos vetoriais interativos.

---

## Interface da Aplicação

O ControLAB oferece uma interface escura, responsiva e 100% offline, projetada para estudo, simulação e projeto de sistemas dinâmicos. Veja abaixo as capturas de tela em resolução completa da tela principal e de cada módulo (clique em qualquer imagem para abrir em tamanho original):

### Hub Central de Módulos (Página Inicial)
Grid de acesso rápido aos domínios analíticos e de projeto de sistemas de controle.

<p align="center">
  <a href="docs/screenshots/hub-modulos.webp">
    <img src="docs/screenshots/hub-modulos.webp" alt="Hub Central de Módulos do ControLAB" width="100%">
  </a>
</p>

### 1. Resposta no Domínio do Tempo
Simulação da resposta transitória e em regime permanente para sinais degrau, impulso e rampa, com cálculo automático de métricas transitórias ($\%OS$, $t_s$, $t_p$, $t_r$), fator de amortecimento ($\zeta$), frequência natural ($\omega_n$) e mapa de polos.

<p align="center">
  <a href="docs/screenshots/resposta-tempo.webp">
    <img src="docs/screenshots/resposta-tempo.webp" alt="Resposta no Domínio do Tempo" width="100%">
  </a>
</p>

### 2. Lugar Geométrico das Raízes (LGR)
Traçado analítico de Evans com ramos, assíntotas, centroide, pontos de quebra/dispersão, cruzamento com o eixo imaginário e memorial completo deduzindo os 7 passos clássicos da literatura.

<p align="center">
  <a href="docs/screenshots/lugar-das-raizes.webp">
    <img src="docs/screenshots/lugar-das-raizes.webp" alt="Lugar Geométrico das Raízes" width="100%">
  </a>
</p>

### 3. Resposta em Frequência (Bode e Nyquist)
Diagramas assintóticos e reais de Bode (magnitude em dB e fase em graus), diagrama polar de Nyquist completo e determinação analítica das margens de ganho ($MG$) e de fase ($MF$), com respectivas frequências de cruzamento.

<p align="center">
  <a href="docs/screenshots/resposta-frequencia.webp">
    <img src="docs/screenshots/resposta-frequencia.webp" alt="Resposta em Frequência" width="100%">
  </a>
</p>

### 4. Projeto de Controladores (PID & Lead-Lag)
Sintonia de controladores PID pelos métodos clássicos de Ziegler-Nichols (resposta ao degrau e ganho crítico) e síntese analítica de compensadores por avanço (Lead), atraso (Lag) e avanço-atraso de fase via LGR com comparação antes e após a compensação.

<p align="center">
  <a href="docs/screenshots/projeto-controladores.webp">
    <img src="docs/screenshots/projeto-controladores.webp" alt="Projeto de Controladores" width="100%">
  </a>
</p>

### 5. Espaço de Estados
Modelagem matricial contínua $(\mathbf{A}, \mathbf{B}, \mathbf{C}, \mathbf{D})$, testes de controlabilidade ($\mathcal{C}$) e observabilidade ($\mathcal{O}$), alocação de polos por realimentação de estados via fórmula de Ackermann e síntese de observadores de Luenberger.

<p align="center">
  <a href="docs/screenshots/espaco-estados.webp">
    <img src="docs/screenshots/espaco-estados.webp" alt="Espaço de Estados" width="100%">
  </a>
</p>

---

## Módulos do Sistema

A partir da Página Inicial (Hub Central), o usuário pode acessar os módulos especializados:

1. **Lugar Geométrico das Raízes (LGR):** ([Ver captura](docs/screenshots/lugar-das-raizes.webp))
   - Cálculo analítico e traçado em tempo real das trajetórias de polos em malha fechada para ganhos $0 \le K < \infty$.
   - Memorial passo a passo dos 7 passos clássicos (Ogata, Nise, Dorf).
   - Manipulação gráfica com zoom até 6.0x, pan livre, exportação em SVG vetorial e PNG a 300 DPI.
2. **Análise de Estabilidade:**
   - Construção analítica da tabela do Critério de Routh-Hurwitz.
   - Tratamento de casos especiais: primeiro elemento nulo ($\epsilon \to 0$) e linha inteiramente nula.
   - Determinação da faixa de ganho estável $K_{min} < K < K_{max}$ e frequências de oscilação crítica.
3. **Resposta no Domínio do Tempo:** ([Ver captura](docs/screenshots/resposta-tempo.webp))
   - Simulação transitória e regime permanente para entradas degrau, impulso e rampa.
   - Extração automática de métricas: sobressinal percentual ($\%OS$), tempo de acomodação ($t_s$), tempo de pico ($t_p$), tempo de subida ($t_r$), fator de amortecimento ($\zeta$) e frequência natural ($\omega_n$).
   - Coeficientes de erro estático ($K_p$, $K_v$, $K_a$) e erro em regime permanente ($e_{ss}$).
4. **Resposta em Frequência:** ([Ver captura](docs/screenshots/resposta-frequencia.webp))
   - Diagramas de Bode de magnitude (dB) e fase (graus) com escalas logarítmicas assintóticas e reais.
   - Diagrama polar de Nyquist e contorno de estabilidade.
   - Cálculo das margens de ganho ($MG$) e de fase ($MF$), além das frequências de cruzamento.
5. **Projeto de Controladores:** ([Ver captura](docs/screenshots/projeto-controladores.webp))
   - Sintonia de controladores PID pelos métodos clássicos de Ziegler-Nichols (resposta ao degrau e ganho crítico).
   - Síntese de compensadores por avanço de fase (Lead) e atraso de fase (Lag) via LGR.
   - Comparativo de desempenho em malha fechada antes e após a compensação.
6. **Espaço de Estados:** ([Ver captura](docs/screenshots/espaco-estados.webp))
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
