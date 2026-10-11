# Changelog — ControLAB

Todas as mudanças relevantes e lançamentos da suíte ControLAB são documentados neste arquivo.
O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e este projeto segue [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [4.0.2] - 2026-10-11

### Corrigido
- **Serialização JSON e Estabilidade no Motor Chaquopy Android (`android_bridge.py`, `LgrPythonPlugin.java`):**
  - Sanitização recursiva de dados em `android_bridge.py` (`_json_safe`), tratando `inf` / `-inf` como `"∞"` / `"-∞"`, `nan` como `None`, tipos escalares do NumPy para primitivos Python e números complexos para dicionários `{real, imag}` com `allow_nan=False`.
  - Elimina o Erro 500 no Android ao pressionar "Calcular" nos módulos de *Resposta no Tempo*, *Resposta em Frequência*, *Projeto de Controladores* e *Espaço de Estados* decorrente de falha de desserialização em `org.json.JSONObject`.
  - Melhoria na propagação e diagnóstico de mensagens de erro no plugin nativo Java.
- **Scroll por Toque no Android (Lugar Geométrico das Raízes e Estabilidade / Routh-Hurwitz):**
  - Ajuste de overflow e layout responsivo móvel (`@media (max-width: 1000px)`) para `#page-lgr.active` e `#page-routh.active`, habilitando rolagem vertical nativa por toque nos contêineres de Parâmetros e Resultados.
  - Layout desktop (> 1000px) mantido 100% inalterado e preservado.

---

## [4.0.1] - 2026-10-11

### Corrigido
- **Áreas Seguras e Edge-to-Edge no Android (Capacitor):**
  - Integração com o plugin Capacitor SystemBars (`insetsHandling = 'css'`) e variáveis `--sat`, `--sab`, `--sal`, `--sar` centralizadas para ajuste sob a barra de status e barra de navegação/gestos sem sobrepor o conteúdo.
  - Correção do contraste de ícones das barras do sistema nos temas claro e escuro.
  - Preservação exata de 100% das proporções, margens e paddings originais na versão desktop.
- **Scroll por Toque no Android:**
  - Remoção de regras restritivas de altura e flex-shrink em contêineres móveis, restaurando rolagem natural por toque com `touch-action: pan-y pan-x` e `-webkit-overflow-scrolling: touch` em todos os painéis e telas.
- **Cálculo nos Módulos Científicos no Android:**
  - Resolução robusta do plugin nativo `LgrPython` no Android WebView, impedindo fallback indevido para requisições HTTP (`fetch('/api')`) que causavam erro 500 no dispositivo móvel.

---

## [4.0.0] - 2026-10-11

### Adicionado
- **Novo Menu Início e Hub Central com Split Pane:**
  - Navegação redesenhada com divisor interativo (Split Pane), pré-visualização em tempo real de cada módulo científico e layout responsivo para desktop, tablets e smartphones.
  - Ícones vetoriais customizados de Engenharia de Controle para todos os cards de módulos.
  - Painel de visualização de resultados em split pane persistente e adaptável sem necessidade de scroll vertical forçado no maior nível de expansão.
- **Suíte de Testes Numéricos Integrada ao CI:**
  - Nova suíte de testes de validação analítica e numérica em `tests/test_numerico.py` padronizada sobre `unittest.TestCase`, cobrindo resposta no tempo, critério de Routh-Hurwitz, LGR, diagramas de Bode/Nyquist, projeto de controladores (Ziegler-Nichols) e espaço de estados.

### Aprimorado
- **Design System e Refinamento de Interface:**
  - Padronização estética de botões de ação com contornos discretos, tipografia refinada e contraste balanceado em modo claro (`#ebeef2`) e escuro (`#222831`).
  - Suporte completo a redimensionamento de interface (*UI Scale*) proporcional baseado em unidades `rem`, funcionando de forma consistente desde a tela inicial até os módulos de cálculo.
  - Remoção de badges redundantes nos cards de módulos, conferindo um visual mais limpo e profissional.
- **Plataformas e Empacotamento:**
  - Atualização do Android para `versionCode 13` e `versionName 4.0.0`.
  - Cache do Service Worker atualizado para `controlab-shell-v4.0.0-1`.
  - Sincronização dos metadados de versão em todos os módulos e bridges da aplicação.

---

## [3.1.1] - 2026-10-08

### Corrigido
- **Tema Dark em Gráficos Científicos (`plot_theme.py`, `routh_hurwitz.py`, `lgr_engine.py`):**
  - Correção de legibilidade no título e itens de legendas em gráficos no Critério de Routh-Hurwitz e Lugar Geométrico das Raízes sob tema Dark. O título da legenda e os rótulos agora utilizam a cor e peso corretos de contraste com quadro de fundo dedicado (`bg_card`).
- **Navegação Sem Disparo Automático de Cálculo:**
  - Desativação do cálculo automático ao abrir os módulos de *Critério de Routh-Hurwitz* e *Lugar Geométrico das Raízes*. Os cálculos agora só são executados sob comando explícito do usuário, com telas iniciais limpas e informativas.

### Aprimorado
- **Padronização de Nomenclatura no Hub Central:**
  - O módulo de raízes foi formalmente renomeado de *"Lugar das Raízes"* para **"Lugar Geométrico das Raízes"** no Hub Central e no site do projeto.
- **Prévias Matemáticas e Interface de Parâmetros:**
  - Inclusão de prévia KaTeX em tempo real nos módulos de *Resposta no Domínio do Tempo*, *Resposta em Frequência* e *Projeto de Controladores*.
  - Remoção de exemplos rápidos que poluíam os formulários científicos desses três módulos.
  - Limpeza dos botões de ação e remoção de atalhos redundantes.
- **Plataformas e Empacotamento:**
  - Atualização do Android para `versionCode 12` e `versionName 3.1.1`.
  - Cache do Service Worker atualizado para `controlab-shell-v3.1.1-1`.
  - Atualização dos metadados de versão em toda a suíte.

---

## [3.1.0] - 2026-10-03

### Adicionado
- **Módulo de Tema Unificado para Gráficos Científicos (`plot_theme.py`):**
  - Integração visual com os tokens de estilo do frontend (temas Dark e Light).
  - Remoção de bordas/spines redundantes (superior e direita) e títulos duplicados sobre as figuras.
  - Gráficos gerados com fundo transparente (`transparent=True`), integrando-se perfeitamente aos cards da aplicação.
  - Paletas de alta legibilidade para curvas de resposta, linhas de referência, regiões de estabilidade e assíntotas.
  - Suporte a exportação vetorial SVG limpa com textos preservados.
- **Fontes Offline Nativas no Pacote:**
  - Inclusão dos pacotes completos das fontes `IBM Plex Sans` e `JetBrains Mono` em formatos WOFF2 e TTF.
  - Eliminação de qualquer dependência de rede externa para tipografia técnica e fórmulas.
  - Registro automático das fontes nas rotinas de renderização do Matplotlib.

### Aprimorado
- **Interface e Experiência do Usuário (UI/UX):**
  - Redesign completo do sistema de tokens CSS, hierarquia tipográfica, contraste e espaçamentos.
  - Workspaces científicos atualizados com cards de cálculo, chips de estabilidade destacados e visualizador interativo de respostas temporais (Degrau, Impulso e Rampa).
  - Adaptação responsiva refinada para smartphones, tablets e desktops em diferentes resoluções.
  - Aprimoramento do Service Worker (`controlab-shell-v3.1.0-1`) para cache offline consistente.
- **Módulos Científicos em Python:**
  - Padronização em todos os motores de cálculo: Lugar Geométrico das Raízes (LGR), Resposta no Tempo, Resposta em Frequência (Bode & Nyquist), Projeto de Controladores (PID & Lead-Lag), Critério de Routh-Hurwitz e Espaço de Estados.
  - Sincronização automática dos scripts Python para o ambiente móvel Android via Chaquopy.
- **Plataformas e Empacotamento:**
  - Atualização da versão do Android para `versionCode 11` e `versionName 3.1.0`.
  - Atualização dos scripts de validação de empacotamento multiplataforma (Linux, Windows, Android e Web).
  - Atualização da documentação oficial do repositório e landing page de distribuição.

---

## [3.0.3] - 2026-09-29

### Adicionado
- Sincronização do motor científico com a camada mobile Android via Chaquopy.
- Inclusão de suíte automatizada de testes do renderer e requisições científicas.

### Aprimorado
- Refinamento do ambiente educacional e responsividade em múltiplos dispositivos.
- Otimização do pipeline de prévias e concorrência para evitar recálculos redundantes.
- Ajustes finos na geometria dos ramos e assíntotas no motor de Lugar Geométrico das Raízes.

---

## [3.0.2] - 2026-09-12

### Aprimorado
- Centralização e expansão proporcional dos cards de módulos no Hub Central.
- Ajustes de branding na barra superior e alinhamento visual com emblema.

---

## [3.0.1] - 2026-09-05

### Aprimorado
- Modernização do layout e hierarquia visual do Hub Central.
- Correção de exibição de fórmulas KaTeX em telas compactas.

---

## [3.0.0] - 2026-08-20

### Adicionado
- Lançamento da suíte unificada ControLAB integrando todos os 5 módulos principais de Engenharia de Controle.
- Suporte a empacotamento completo para Linux (.AppImage, .deb, .rpm), Windows (.exe NSIS) e Android (.apk).
