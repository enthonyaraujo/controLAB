# Changelog — ControLAB

Todas as mudanças relevantes e lançamentos da suíte ControLAB são documentados neste arquivo.
O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e este projeto segue [Versionamento Semântico](https://semver.org/lang/pt-BR/).

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
