/**
 * LGR Studio - Renderer Application Logic
 */

document.addEventListener('DOMContentLoaded', () => {
  // State
  let currentMode = 'expr';
  let currentImageData = null;
  let currentSVGData = null;
  let previewTimer = null;
  let previewRequestId = 0;

  // Zoom / Pan State
  let zoomLevel = 1.0;
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let startX = 0;
  let startY = 0;

  // DOM Elements
  const themeToggleBtn = document.getElementById('theme-toggle');
  const modeTabs = document.querySelectorAll('.mode-tab');
  const formPanels = document.querySelectorAll('.form-panel');
  const viewTabs = document.querySelectorAll('.view-tab');
  const viewPanels = document.querySelectorAll('.view-panel');
  
  const inputExpr = document.getElementById('input-expr');
  const inputNumerator = document.getElementById('input-numerator');
  const inputDenominator = document.getElementById('input-denominator');
  const inputNum = document.getElementById('input-num');
  const inputDen = document.getElementById('input-den');
  const inputK = document.getElementById('input-k');
  const inputZeros = document.getElementById('input-zeros');
  const inputPoles = document.getElementById('input-poles');
  const inputTitle = document.getElementById('input-title');
  const btnCalculate = document.getElementById('btn-calculate');
  const latexPreview = document.getElementById('latex-preview');
  const previewStatus = document.getElementById('preview-status');
  const previewError = document.getElementById('preview-error');

  const plotViewport = document.getElementById('plot-viewport');
  const plotImg = document.getElementById('plot-image');
  const plotLoading = document.getElementById('plot-loading');

  const latexExpanded = document.getElementById('latex-expanded');
  const latexFactored = document.getElementById('latex-factored');

  const badgePoles = document.getElementById('badge-poles');
  const badgeZeros = document.getElementById('badge-zeros');
  const badgeBranches = document.getElementById('badge-branches');
  const badgeCentroid = document.getElementById('badge-centroid');

  const toastEl = document.getElementById('toast');

  // =========================================================================
  // TEMA (DARK / LIGHT)
  // =========================================================================
  function initTheme() {
    const savedTheme = localStorage.getItem('lgr-theme') || 'dark';
    document.documentElement.setAttribute('data-theme', savedTheme);
    updateThemeIcon(savedTheme);
  }

  function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme') || 'dark';
    const next = current === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('lgr-theme', next);
    updateThemeIcon(next);
  }

  function updateThemeIcon(theme) {
    const isDark = theme === 'dark';
    themeToggleBtn.setAttribute('title', isDark ? 'Alternar para Modo Claro' : 'Alternar para Modo Escuro');
    themeToggleBtn.setAttribute('aria-label', isDark ? 'Alternar para Modo Claro' : 'Alternar para Modo Escuro');
  }

  themeToggleBtn.addEventListener('click', toggleTheme);
  initTheme();

  // =========================================================================
  // NAVEGAÇÃO ENTRE PÁGINAS (HUB / MÓDULOS)
  // =========================================================================
  const pageHome = document.getElementById('page-home');
  const pageLgr = document.getElementById('page-lgr');
  const btnNavHome = document.getElementById('btn-nav-home');
  const navModulePill = document.getElementById('nav-module-pill');
  const navModuleName = document.getElementById('nav-module-name');
  const btnSidebarBackHome = document.getElementById('btn-sidebar-back-home');
  const btnOpenLgr = document.getElementById('btn-open-lgr');
  const cardModuleLgr = document.getElementById('card-module-lgr');
  const globalBrand = document.getElementById('global-brand');

  function navigateTo(pageId) {
    if (pageId === 'lgr') {
      if (pageHome) pageHome.classList.remove('active');
      if (pageLgr) pageLgr.classList.add('active');
      if (btnNavHome) btnNavHome.style.display = 'inline-flex';
      if (navModulePill) {
        navModulePill.style.display = 'inline-flex';
        if (navModuleName) navModuleName.textContent = 'Lugar Geométrico das Raízes (LGR)';
      }
      renderMathInContainer(pageLgr);
    } else {
      if (pageLgr) pageLgr.classList.remove('active');
      if (pageHome) {
        pageHome.classList.add('active');
        pageHome.scrollTop = 0;
      }
      if (btnNavHome) btnNavHome.style.display = 'none';
      if (navModulePill) navModulePill.style.display = 'none';
      renderMathInContainer(pageHome);
    }
  }

  if (btnOpenLgr) {
    btnOpenLgr.addEventListener('click', (e) => {
      e.stopPropagation();
      navigateTo('lgr');
    });
  }

  if (cardModuleLgr) {
    cardModuleLgr.addEventListener('click', () => {
      navigateTo('lgr');
    });
  }

  if (btnNavHome) {
    btnNavHome.addEventListener('click', () => navigateTo('home'));
  }

  if (btnSidebarBackHome) {
    btnSidebarBackHome.addEventListener('click', () => navigateTo('home'));
  }

  if (globalBrand) {
    globalBrand.addEventListener('click', () => navigateTo('home'));
    globalBrand.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        navigateTo('home');
      }
    });
  }

  // Cards e botões com aviso de módulo em desenvolvimento
  const soonCards = document.querySelectorAll('.module-card.card-soon');
  soonCards.forEach((card) => {
    card.addEventListener('click', () => {
      const moduleName = card.dataset.module || 'selecionado';
      showToast(`O módulo "${moduleName}" está em desenvolvimento e estará disponível nas próximas atualizações.`, 'info', 3200);
    });
  });

  const soonButtons = document.querySelectorAll('.btn-module-soon');
  soonButtons.forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const moduleName = btn.dataset.module || 'selecionado';
      showToast(`O módulo "${moduleName}" está em desenvolvimento e estará disponível nas próximas atualizações.`, 'info', 3200);
    });
  });

  // Inicializa na Página Inicial (Hub)
  navigateTo('home');

  // =========================================================================
  // NAVEGAÇÃO POR ABAS (MODO DE ENTRADA & VIEWS)
  // =========================================================================
  modeTabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      modeTabs.forEach((t) => t.classList.remove('active'));
      formPanels.forEach((p) => p.classList.remove('active'));

      tab.classList.add('active');
      currentMode = tab.dataset.mode;
      const targetPanel = document.getElementById(`form-${currentMode}`);
      if (targetPanel) targetPanel.classList.add('active');
      schedulePreview();
    });
  });

  viewTabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      viewTabs.forEach((t) => t.classList.remove('active'));
      viewPanels.forEach((p) => p.classList.remove('active'));

      tab.classList.add('active');
      const targetView = document.getElementById(`view-${tab.dataset.view}`);
      if (targetView) {
        targetView.classList.add('active');
        renderMathInContainer(targetView);
      }
      if (tab.dataset.view !== 'memorial') {
        toggleMemorialSidebar(false);
      }
    });
  });

  // Chips de Exemplos Rápidos
  document.querySelectorAll('.chip').forEach((chip) => {
    chip.addEventListener('click', () => {
      inputExpr.value = chip.dataset.example;
      // Muda para aba de expressão se não estiver
      document.querySelector('.mode-tab[data-mode="expr"]').click();
      calculate();
    });
  });

  // =========================================================================
  // CÁLCULO E PLOTAGEM DO LGR
  // =========================================================================
  function getPayload() {
    const title = inputTitle.value.trim() || 'Lugar Geométrico das Raízes';
    
    if (currentMode === 'expr') {
      return {
        mode: 'expr',
        expr: inputExpr.value.trim(),
        title,
      };
    } else if (currentMode === 'parts') {
      return {
        mode: 'parts',
        numerator: inputNumerator.value.trim(),
        denominator: inputDenominator.value.trim(),
        title,
      };
    } else if (currentMode === 'coeffs') {
      return {
        mode: 'coeffs',
        num: inputNum.value.trim(),
        den: inputDen.value.trim(),
        title,
      };
    } else if (currentMode === 'zpk') {
      return {
        mode: 'zpk',
        k: Number.isFinite(Number.parseFloat(inputK.value)) ? Number.parseFloat(inputK.value) : 1.0,
        zeros: inputZeros.value.trim(),
        poles: inputPoles.value.trim(),
        title,
      };
    }
    return { mode: 'expr', expr: inputExpr.value.trim(), title };
  }

  async function calculate() {
    showLoading(true);

    try {
      const payload = getPayload();
      const res = await window.api.calculateLGR(payload);

      if (!res.success) {
        showToast(`Erro: ${res.error}`, 'error', 4000);
        showLoading(false);
        return;
      }

      // 1. Atualizar Imagem e Vetorial
      currentImageData = res.image;
      currentSVGData = res.svg;
      plotImg.src = res.image;
      resetZoom();

      // 2. Renderizar Fórmulas LaTeX via KaTeX
      renderMath(latexExpanded, res.latex_exp);
      renderMath(latexFactored, res.latex_fac);

      // 3. Atualizar Badges de Resumo
      const det = res.detalhes;
      badgePoles.innerHTML = `Polos: <b>${det.P}</b>`;
      badgeZeros.innerHTML = `Zeros: <b>${det.Z}</b>`;
      badgeBranches.innerHTML = `Ramos: <b>${det.ramos}</b>`;
      badgeCentroid.innerHTML = det.centroide !== null 
        ? `Centroide $\\sigma_a$: <b>${det.centroide.toFixed(2)}</b>` 
        : `Centroide: <b>N/A</b>`;
      renderMathInContainer(badgeCentroid);

      // 4. Preencher o Memorial de Cálculo dos 7 Passos e Deduções
      populateMemorialSteps(det);
      populateMemorialDeductions(det.passo_a_passo);

      showToast('LGR calculado e traçado com sucesso!', 'success');

    } catch (err) {
      showToast(`Erro inesperado: ${err.message}`, 'error', 4000);
    } finally {
      showLoading(false);
    }
  }

  function renderMath(element, texString, displayMode = false) {
    if (!element || !texString) return;
    try {
      if (window.katex) {
        window.katex.render(texString, element, {
          throwOnError: false,
          displayMode,
          strict: 'warn',
          trust: false,
        });
      } else {
        element.textContent = texString;
      }
    } catch (e) {
      element.textContent = texString;
    }
  }

  const mathDelimiters = [
    { left: '$$', right: '$$', display: true },
    { left: '\\[', right: '\\]', display: true },
    { left: '$', right: '$', display: false },
    { left: '\\(', right: '\\)', display: false },
  ];

  function renderMathInContainer(container) {
    if (!container || !window.renderMathInElement) return;
    window.renderMathInElement(container, {
      delimiters: mathDelimiters,
      throwOnError: false,
      strict: 'warn',
      trust: false,
      ignoredTags: ['script', 'noscript', 'style', 'textarea', 'pre'],
    });
  }

  function schedulePreview() {
    window.clearTimeout(previewTimer);
    previewTimer = window.setTimeout(previewTransferFunction, 220);
  }

  async function previewTransferFunction() {
    const requestId = ++previewRequestId;
    previewStatus.textContent = 'Validando…';
    previewStatus.className = 'preview-status loading';
    previewError.hidden = true;

    try {
      const res = await window.api.previewTransferFunction(getPayload());
      if (requestId !== previewRequestId) return;
      if (!res.success) throw new Error(res.error || 'Entrada inválida.');

      renderMath(latexPreview, res.latex_fac, true);
      previewStatus.textContent = 'Expressão válida';
      previewStatus.className = 'preview-status valid';
    } catch (err) {
      if (requestId !== previewRequestId) return;
      latexPreview.textContent = 'G(s) = ?';
      previewStatus.textContent = 'Revise a entrada';
      previewStatus.className = 'preview-status invalid';
      previewError.textContent = err.message;
      previewError.hidden = false;
    }
  }

  // =========================================================================
  // PREENCHIMENTO DO MEMORIAL DE CÁLCULO DOS 7 PASSOS
  // =========================================================================
  function formatComplexToLatex(str) {
    if (!str) return '';
    return str.replace(/([+-]?\s*\d+(?:\.\d+)?)j/g, (match, val) => {
      const trimmed = val.replace(/\s+/g, '');
      const sign = trimmed.startsWith('-') ? '-' : (trimmed.startsWith('+') ? '+' : '');
      const num = trimmed.replace(/^[+-]/, '');
      return `${sign ? sign + ' ' : ''}j${num}`;
    }).trim();
  }

  function populateMemorialSteps(det) {
    // Passo 1, 2 e 3
    const el1 = document.getElementById('step-content-1');
    const polosStr = det.polos.length > 0
      ? det.polos.map((p) => `$${formatComplexToLatex(p.str)}$`).join(', ')
      : 'Nenhum polo finito';
    const zerosStr = det.Z > 0
      ? det.zeros.map((z) => `$${formatComplexToLatex(z.str)}$`).join(', ')
      : 'Nenhum zero finito';

    el1.innerHTML = `
      <div class="step-item-box">
        <strong>Número de Polos ($P$):</strong> ${det.P} &nbsp;|&nbsp; <strong>Número de Zeros ($Z$):</strong> ${det.Z}
      </div>
      <p>• <strong>Polos de Malha Aberta ($K=0$):</strong> ${polosStr}</p>
      <p>• <strong>Zeros de Malha Aberta ($K\\to\\infty$):</strong> ${zerosStr}</p>
      <p>• <strong>Total de Ramos do LGR ($n = \\max(P, Z)$):</strong> ${det.ramos}</p>
      <p>• <strong>Simetria:</strong> O LGR é perfeitamente simétrico em relação ao Eixo Real ($\\sigma$).</p>
    `;

    // Passo 4: Assíntotas e Centroide
    const el4 = document.getElementById('step-content-4');
    if (det.P > det.Z) {
      const numAssintotas = det.P - det.Z;
      const angulosTexto = det.angulos_assintotas
        .map((a) => `<span class="math-pill">$\\theta_{${a.k}} = ${a.graus.toFixed(1)}^\\circ$</span>`)
        .join(' ');

      el4.innerHTML = `
        <div class="step-item-box info">
          <strong>Ramos que tendem ao infinito ($P - Z$):</strong> ${numAssintotas} ramo(s)
        </div>
        <p>• <strong>Centroide das Assíntotas ($\\sigma_a$):</strong> $\\sigma_a = ${det.centroide.toFixed(3)}$</p>
        <p>• <strong>Ângulos das Assíntotas ($\\theta_k = \\frac{(2k+1)180^\\circ}{P-Z}$):</strong> ${angulosTexto}</p>
      `;
    } else {
      el4.innerHTML = `
        <div class="step-item-box">
          Como $P \\le Z$, todos os ramos terminam nos zeros finitos. Não há assíntotas direcionadas ao infinito.
        </div>
      `;
    }

    // Passo 5: Break-in / Breakaway
    const el5 = document.getElementById('step-content-5');
    if (det.break_points && det.break_points.length > 0) {
      const iconPin = `<span class="step-badge-icon badge-icon-emerald"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg></span>`;
      const breakItems = det.break_points.map(
        (bp) => `<div class="step-item-box success">${iconPin} Ponto no Eixo Real: $s = ${bp.s.toFixed(3)}$ &nbsp;|&nbsp; Ganho Crítico: $K = ${bp.K.toFixed(2)}$</div>`
      ).join('');
      el5.innerHTML = `
        <p>Pontos onde $\\frac{dK}{ds} = 0 \\iff N'(s)D(s) - N(s)D'(s) = 0$:</p>
        ${breakItems}
      `;
    } else {
      el5.innerHTML = `
        <div class="step-item-box">
          Nenhum ponto de partida ou retorno no eixo real com ganho $K > 0$ válido.
        </div>
      `;
    }

    // Passo 6: Cruzamento do Eixo jw
    const el6 = document.getElementById('step-content-6');
    if (det.jw_cruzamentos && det.jw_cruzamentos.length > 0) {
      // Deduplicar pares simetricos (+w e -w) para nao duplicar informacao identica de conjugados
      const seenFrequencies = new Set();
      const uniqueJw = det.jw_cruzamentos.filter((jw) => {
        const absW = Math.abs(jw.w).toFixed(2);
        const kStr = jw.K.toFixed(2);
        const key = `${absW}_${kStr}`;
        if (seenFrequencies.has(key)) return false;
        seenFrequencies.add(key);
        return true;
      });

      const iconStability = `<span class="step-badge-icon badge-icon-amber"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg></span>`;
      const jwItems = uniqueJw.map(
        (jw) => `<div class="step-item-box warning">${iconStability} Cruzamento detectado em: $s = \\pm j${Math.abs(jw.w).toFixed(2)}$ &nbsp;|&nbsp; Ganho Limite de Estabilidade: $K_{crit} = ${jw.K.toFixed(2)}$</div>`
      ).join('');
      el6.innerHTML = `
        <p>Cruzamentos com o eixo imaginário ($j\\omega$):</p>
        ${jwItems}
      `;
    } else {
      el6.innerHTML = `
        <div class="step-item-box">
          Nenhum cruzamento com o eixo $j\\omega$ detectado na faixa de ganho analisada (o sistema permanece no semiplano atual).
        </div>
      `;
    }

    // Passo 7: Ângulos de Partida e Chegada
    const el7 = document.getElementById('step-content-7');
    const temPartida = det.angulos_partida && det.angulos_partida.length > 0;
    const temChegada = det.angulos_chegada && det.angulos_chegada.length > 0;

    if (temPartida || temChegada) {
      let html = '';
      if (temPartida) {
        const iconPartida = `<span class="step-badge-icon badge-icon-purple"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m16 12-4-4-4 4"/><path d="M12 16V8"/></svg></span>`;
        const partidaItems = det.angulos_partida.map(
          (ap) => `<div class="step-item-box info">${iconPartida} Para o polo complexo $p = ${formatComplexToLatex(ap.polo_str)}$: Ângulo de partida $\\theta_d = ${ap.angulo.toFixed(1)}^\\circ$</div>`
        ).join('');
        html += `
          <p>• <strong>Ângulo de Partida em Polos Complexos ($\\theta_d$):</strong></p>
          <p class="input-help" style="margin-bottom: 6px;">Condição angular: $\\theta_d = 180^\\circ + \\sum \\angle(p - z) - \\sum \\angle(p - p_{outros})$</p>
          ${partidaItems}
        `;
      }
      if (temChegada) {
        const iconChegada = `<span class="step-badge-icon badge-icon-emerald"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg></span>`;
        const chegadaItems = det.angulos_chegada.map(
          (ac) => `<div class="step-item-box success">${iconChegada} Para o zero complexo $z = ${formatComplexToLatex(ac.zero_str)}$: Ângulo de chegada $\\theta_a = ${ac.angulo.toFixed(1)}^\\circ$</div>`
        ).join('');
        html += `
          <p style="margin-top: ${temPartida ? '14px' : '0'};">• <strong>Ângulo de Chegada em Zeros Complexos ($\\theta_a$):</strong></p>
          <p class="input-help" style="margin-bottom: 6px;">Condição angular: $\\theta_a = 180^\\circ + \\sum \\angle(z - p) - \\sum \\angle(z - z_{outros})$</p>
          ${chegadaItems}
        `;
      }
      el7.innerHTML = html;
    } else {
      el7.innerHTML = `
        <div class="step-item-box">
          Não há polos ou zeros complexos conjugados nesta função de transferência (todas as singularidades são puramente reais).
        </div>
      `;
    }

    const memorialContainer = document.getElementById('view-memorial');
    if (memorialContainer) {
      renderMathInContainer(memorialContainer);
    } else {
      [el1, el4, el5, el6, el7].forEach(renderMathInContainer);
    }
  }

  // =========================================================================
  // DEDUÇÕES MATEMÁTICAS ANALÍTICAS PASSO A PASSO
  // =========================================================================
  function populateMemorialDeductions(pap) {
    const container = document.getElementById('sidebar-deductions-content');
    if (!container) return;
    if (!pap || Object.keys(pap).length === 0) {
      container.innerHTML = `
        <div class="deduction-empty">
          <span>Nenhuma dedução calculada ainda. Clique em Traçar LGR para processar.</span>
        </div>
      `;
      return;
    }

    const p1 = pap.passo_1_2_3;
    const p4 = pap.passo_4;
    const p5 = pap.passo_5;
    const p6 = pap.passo_6;
    const p7 = pap.passo_7;

    let html = '';

    // ==========================================
    // DEDUÇÃO PASSOS 1, 2 e 3
    // ==========================================
    html += `
      <div class="deduction-section" id="deduction-step-1" data-step-nav="1">
        <div class="deduction-section-header">
          <span class="deduction-badge">1, 2, 3</span>
          <h4>Polos, Zeros, Ramos e Eixo Real</h4>
        </div>
        <div class="deduction-section-body">
          <div class="deduction-item">
            <div class="deduction-item-title">1. Equação Característica em Malha Fechada</div>
            <p>A condição fundamental de malha fechada $1 + K G(s) = 0$ resulta na equação característica polinomial:</p>
            <div class="math-display-box">$$${p1.eq_caracteristica}$$</div>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">2. Fatoração e Singularidades de Malha Aberta</div>
            <p>Forma fatorada da função de malha aberta $G(s)$:</p>
            <div class="math-display-box">$$${p1.eq_fatorada}$$</div>
            <p>• Polos ($K = 0$): $P = ${p1.P}$ raiz(es) de $D(s) = 0$</p>
            <p>• Zeros ($K \\to \\infty$): $Z = ${p1.Z}$ raiz(es) de $N(s) = 0$</p>
            <p>• Total de Ramos do LGR: $n = \\max(P, Z) = ${p1.ramos}$ ramo(s)</p>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">3. Segmentos no Eixo Real (Regra da Contagem Ímpar)</div>
            <p>Um ponto sobre o eixo real pertence ao LGR se o total de polos e zeros reais à sua direita for <strong>ímpar</strong>:</p>
            <table class="deduction-table">
              <thead>
                <tr>
                  <th>Intervalo Real</th>
                  <th>À Direita</th>
                  <th>Status no LGR</th>
                </tr>
              </thead>
              <tbody>
                ${p1.segmentos_eixo_real && p1.segmentos_eixo_real.length > 0 ? p1.segmentos_eixo_real.map((s) => `
                  <tr>
                    <td>$${s.intervalo}$</td>
                    <td>${s.contagem_direita}</td>
                    <td><span class="${s.pertence ? 'badge-valid' : 'badge-invalid'}">${s.pertence ? 'Pertence ao LGR' : 'Fora do LGR'}</span></td>
                  </tr>
                `).join('') : '<tr><td colspan="3">Nenhuma singularidade real para delimitar intervalos.</td></tr>'}
              </tbody>
            </table>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">4. Simetria Reflexiva do LGR</div>
            <p>${p1.simetria_justificativa}</p>
          </div>
        </div>
      </div>
    `;

    // ==========================================
    // DEDUÇÃO PASSO 4: Assíntotas e Centroide
    // ==========================================
    html += `
      <div class="deduction-section" id="deduction-step-4" data-step-nav="4">
        <div class="deduction-section-header">
          <span class="deduction-badge">Passo 4</span>
          <h4>Assíntotas e Centroide</h4>
        </div>
        <div class="deduction-section-body">
          ${p4.tem_assintotas ? `
            <div class="deduction-item">
              <div class="deduction-item-title">1. Centroide das Assíntotas ($\\sigma_a$)</div>
              <p>Fórmula analítica fundamental baseada na conservação da soma das raízes:</p>
              <div class="math-display-box">$$${p4.formula_centroide}$$</div>
              <p>Substituição numérica com soma dos polos e soma dos zeros:</p>
              <div class="math-display-box">$$${p4.substituicao_centroide}$$</div>
            </div>

            <div class="deduction-item">
              <div class="deduction-item-title">2. Ângulos das Assíntotas ($\\theta_k$)</div>
              <p>Condição angular para $k = 0, 1, \\dots, ${p4.n_assintotas - 1}$:</p>
              <div class="math-display-box">$$${p4.formula_angulos}$$</div>
              <p>Desenvolvimento analítico de cada ramo assintótico que tende ao infinito:</p>
              ${p4.angulos_deduzidos.map((a) => `
                <div class="math-display-box" style="margin: 4px 0;">$$${a.formula}$$</div>
              `).join('')}
            </div>
          ` : `
            <div class="step-item-box">
              Como $P \\le Z$, todos os ramos terminam nos zeros finitos. Não há assíntotas direcionadas ao infinito.
            </div>
          `}
        </div>
      </div>
    `;

    // ==========================================
    // DEDUÇÃO PASSO 5: Break-in / Breakaway
    // ==========================================
    html += `
      <div class="deduction-section" id="deduction-step-5" data-step-nav="5">
        <div class="deduction-section-header">
          <span class="deduction-badge">Passo 5</span>
          <h4>Break-in / Breakaway (Pontos de Quebra)</h4>
        </div>
        <div class="deduction-section-body">
          <div class="deduction-item">
            <div class="deduction-item-title">1. Condição de Raiz Múltipla ($\\frac{dK}{ds} = 0$)</div>
            <p>Expressão do ganho $K$ em função de $s$ na malha fechada:</p>
            <div class="math-display-box">$$${p5.formula_derivada}$$</div>
            <p>Anulando o numerador da derivada, obtém-se a condição polinomial:</p>
            <div class="math-display-box">$$${p5.condicao}$$</div>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">2. Derivação dos Polinômios</div>
            <p>Derivadas analíticas de $N(s)$ e $D(s)$:</p>
            <div class="math-display-box">$$N'(s) = \\frac{d}{ds} N(s) = ${p5.derivada_N}$$</div>
            <div class="math-display-box">$$D'(s) = \\frac{d}{ds} D(s) = ${p5.derivada_D}$$</div>
            <p>Polinômio de quebra expandido $P_{break}(s) = 0$:</p>
            <div class="math-display-box">$$${p5.polinomio_break}$$</div>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">3. Resolução das Raízes e Verificação de Validade</div>
            <p>Critérios de aceitação: $s \\in \\mathbb{R}$, ganho positivo $K(s) > 0$ e pertencimento aos ramos reais do LGR:</p>
            <table class="deduction-table">
              <thead>
                <tr>
                  <th>Raiz Candidata</th>
                  <th>Ganho $K(s)$</th>
                  <th>Validação e Justificativa</th>
                </tr>
              </thead>
              <tbody>
                ${p5.raizes_analisadas && p5.raizes_analisadas.length > 0 ? p5.raizes_analisadas.map((r) => `
                  <tr>
                    <td>$s = ${formatComplexToLatex(r.s_str)}$</td>
                    <td>${r.K !== undefined && r.K !== null ? `$K = ${r.K.toFixed(2)}$` : 'N/A'}</td>
                    <td>
                      <span class="${r.valido ? 'badge-valid' : 'badge-invalid'}">
                        ${r.valido ? 'Válido' : 'Descartado'}
                      </span>
                      <div style="font-size: 0.74rem; margin-top: 2px;">${r.motivo}</div>
                    </td>
                  </tr>
                `).join('') : '<tr><td colspan="3">Nenhuma raiz candidata encontrada.</td></tr>'}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    `;

    // ==========================================
    // DEDUÇÃO PASSO 6: Cruzamento do Eixo jw
    // ==========================================
    html += `
      <div class="deduction-section" id="deduction-step-6" data-step-nav="6">
        <div class="deduction-section-header">
          <span class="deduction-badge">Passo 6</span>
          <h4>Cruzamento com o Eixo Imaginário ($j\\omega$)</h4>
        </div>
        <div class="deduction-section-body">
          <div class="deduction-item">
            <div class="deduction-item-title">1. Substituição de Frequência Limite ($s = j\\omega$)</div>
            <p>Equação característica em malha fechada:</p>
            <div class="math-display-box">$$${p6.eq_caracteristica}$$</div>
            <p>Substituindo $s = j\\omega$ com potências de $j$ ($j^2 = -1$, $j^3 = -j$, $j^4 = 1$):</p>
            <div class="math-display-box">$$${p6.substituicao_jw}$$</div>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">2. Decomposição Algébrica (Parte Real e Imaginária)</div>
            <p>Equações desacopladas que devem anular-se simultaneamente:</p>
            <div class="math-display-box">$$\\text{Re}(\\omega, K): \\quad ${p6.parte_real_eq}$$</div>
            <div class="math-display-box">$$\\text{Im}(\\omega, K): \\quad ${p6.parte_imaginaria_eq}$$</div>
          </div>

          <div class="deduction-item">
            <div class="deduction-item-title">3. Soluções Reais de Cruzamento ($K_{crit} > 0, \\omega > 0$)</div>
            ${p6.tem_cruzamento ? `
              <p>Resolvendo o sistema algébrico para frequências críticas no limiar de estabilidade:</p>
              ${p6.cruzamentos.map((c) => `
                <div class="step-item-box warning" style="margin: 6px 0;">
                  Ponto crítico detectado em: $${c.s_str}$ com ganho limite $${c.K_str}$
                </div>
              `).join('')}
            ` : `
              <p>Não há soluções reais com $\\omega > 0$ e $K > 0$ satisfazendo simultaneamente a parte real e imaginária. O sistema não cruza o eixo imaginário na faixa analisada.</p>
            `}
          </div>
        </div>
      </div>
    `;

    // ==========================================
    // DEDUÇÃO PASSO 7: Ângulos de Partida e Chegada
    // ==========================================
    html += `
      <div class="deduction-section" id="deduction-step-7" data-step-nav="7">
        <div class="deduction-section-header">
          <span class="deduction-badge">Passo 7</span>
          <h4>Ângulos de Partida ($\\theta_d$) e Chegada ($\\theta_a$)</h4>
        </div>
        <div class="deduction-section-body">
          ${(p7.tem_partida || p7.tem_chegada) ? `
            ${p7.deducoes_partida && p7.deducoes_partida.length > 0 ? p7.deducoes_partida.map((dp) => `
              <div class="deduction-item">
                <div class="deduction-item-title">• Polo Complexo: $p = ${formatComplexToLatex(dp.polo_str)}$</div>
                <p>Condição angular de partida:</p>
                <div class="math-display-box">$$${dp.formula_aplicada}$$</div>
                <p>Vetores angulares para cada zero $\\phi_z$:</p>
                <p style="font-size: 0.8rem;">${dp.termos_zeros.length > 0 ? dp.termos_zeros.map((tz) => `$\\angle(p - (${formatComplexToLatex(tz.zero_str)})) = ${tz.angulo.toFixed(1)}^\\circ$`).join(', ') : 'Nenhum zero finito'}</p>
                <p>Vetores angulares para os outros polos $\\theta_p$:</p>
                <p style="font-size: 0.8rem;">${dp.termos_polos.map((tp) => `$\\angle(p - (${formatComplexToLatex(tp.polo_str)})) = ${tp.angulo.toFixed(1)}^\\circ$`).join(', ')}</p>
                <p>Substituição vetorial:</p>
                <div class="math-display-box">$$${dp.calculo_substituicao}$$</div>
              </div>
            `).join('') : ''}
            ${p7.deducoes_chegada && p7.deducoes_chegada.length > 0 ? p7.deducoes_chegada.map((dc) => `
              <div class="deduction-item">
                <div class="deduction-item-title">• Zero Complexo: $z = ${formatComplexToLatex(dc.zero_str)}$</div>
                <p>Condição angular de chegada:</p>
                <div class="math-display-box">$$${dc.formula_aplicada}$$</div>
                <p>Vetores angulares para cada polo $\\phi_p$:</p>
                <p style="font-size: 0.8rem;">${dc.termos_polos.map((tp) => `$\\angle(z - (${formatComplexToLatex(tp.polo_str)})) = ${tp.angulo.toFixed(1)}^\\circ$`).join(', ')}</p>
                <p>Substituição vetorial:</p>
                <div class="math-display-box">$$${dc.calculo_substituicao}$$</div>
              </div>
            `).join('') : ''}
          ` : `
            <div class="step-item-box">
              Não há singularidades complexas conjugadas nesta função. Todas as singularidades são puramente reais, não se aplicando ângulos tangenciais de partida ou chegada.
            </div>
          `}
        </div>
      </div>
    `;

    container.innerHTML = html;
    renderMathInContainer(container);
  }

  // =========================================================================
  // TOGGLE E NAVEGAÇÃO DA SIDEBAR DE DEDUÇÕES DO MEMORIAL
  // =========================================================================
  const btnToggleDeductions = document.getElementById('btn-toggle-deductions');
  const memorialSidebar = document.getElementById('memorial-sidebar');
  const btnCloseSidebar = document.getElementById('btn-close-sidebar');
  const sidebarBackdrop = document.getElementById('sidebar-backdrop');
  const viewMemorial = document.getElementById('view-memorial');

  function toggleMemorialSidebar(forceOpen = null) {
    if (!memorialSidebar) return;
    const shouldOpen = forceOpen !== null ? forceOpen : !memorialSidebar.classList.contains('open');
    memorialSidebar.classList.toggle('open', shouldOpen);
    memorialSidebar.setAttribute('aria-hidden', String(!shouldOpen));
    if (sidebarBackdrop) {
      sidebarBackdrop.classList.toggle('active', shouldOpen);
    }
    if (viewMemorial) {
      viewMemorial.classList.toggle('sidebar-open', shouldOpen);
    }
    if (btnToggleDeductions) {
      btnToggleDeductions.setAttribute('aria-expanded', String(shouldOpen));
    }
  }

  function filterSidebarStep(navStep) {
    const pills = document.querySelectorAll('.step-nav-pill');
    pills.forEach((p) => {
      p.classList.toggle('active', p.getAttribute('data-nav') === navStep);
    });

    const sections = document.querySelectorAll('.deduction-section');
    sections.forEach((sec) => {
      if (navStep === 'all' || sec.getAttribute('data-step-nav') === navStep) {
        sec.style.display = '';
      } else {
        sec.style.display = 'none';
      }
    });

    const contentContainer = document.getElementById('sidebar-deductions-content');
    if (contentContainer) {
      contentContainer.scrollTop = 0;
    }
  }

  function openSidebarAtStep(stepNum) {
    toggleMemorialSidebar(true);
    filterSidebarStep(String(stepNum));
    const targetEl = document.getElementById(`deduction-step-${stepNum}`);
    if (targetEl) {
      setTimeout(() => {
        targetEl.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }, 100);
    }
  }

  if (btnToggleDeductions) {
    btnToggleDeductions.addEventListener('click', () => toggleMemorialSidebar());
  }
  if (btnCloseSidebar) {
    btnCloseSidebar.addEventListener('click', () => toggleMemorialSidebar(false));
  }
  if (sidebarBackdrop) {
    sidebarBackdrop.addEventListener('click', () => toggleMemorialSidebar(false));
  }

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && memorialSidebar && memorialSidebar.classList.contains('open')) {
      toggleMemorialSidebar(false);
    }
  });

  document.querySelectorAll('.btn-step-breakdown').forEach((btn) => {
    btn.addEventListener('click', () => {
      const step = btn.getAttribute('data-step');
      openSidebarAtStep(step);
    });
  });

  document.querySelectorAll('.step-nav-pill').forEach((pill) => {
    pill.addEventListener('click', () => {
      filterSidebarStep(pill.getAttribute('data-nav'));
    });
  });

  // =========================================================================
  // ZOOM E PAN INTERATIVO (MOUSE, PINCH-TO-ZOOM TOUCH E SCROLL MOBILE)
  // =========================================================================
  const btnScrollTop = document.getElementById('btn-scroll-top');
  let initialPinchDistance = 0;
  let initialPinchZoom = 1.0;
  let initialPinchMidX = 0;
  let initialPinchMidY = 0;
  let startPinchPanX = 0;
  let startPinchPanY = 0;
  let lastTapTime = 0;

  function updateImageTransform() {
    if (zoomLevel <= 1.0) {
      zoomLevel = 1.0;
      panX = 0;
      panY = 0;
    }
    plotImg.style.transform = `translate(${panX}px, ${panY}px) scale(${zoomLevel})`;

    const isZoomed = zoomLevel > 1.0;
    plotViewport.classList.toggle('zoomed', isZoomed);
    plotViewport.style.touchAction = isZoomed ? 'none' : 'pan-y';

    const btnZoomOut = document.getElementById('btn-zoom-out');
    if (btnZoomOut) {
      btnZoomOut.disabled = !isZoomed;
      btnZoomOut.style.opacity = isZoomed ? '1' : '0.45';
      btnZoomOut.style.cursor = isZoomed ? 'pointer' : 'not-allowed';
    }
  }

  function resetZoom() {
    zoomLevel = 1.0;
    panX = 0;
    panY = 0;
    updateImageTransform();
  }

  document.getElementById('btn-zoom-in').addEventListener('click', () => {
    zoomLevel = Math.min(zoomLevel * 1.25, 5.0);
    updateImageTransform();
  });

  document.getElementById('btn-zoom-out').addEventListener('click', () => {
    zoomLevel = Math.max(zoomLevel / 1.25, 1.0);
    if (zoomLevel === 1.0) {
      panX = 0;
      panY = 0;
    }
    updateImageTransform();
  });

  document.getElementById('btn-zoom-reset').addEventListener('click', resetZoom);

  // Scroll do Mouse para Zoom
  plotViewport.addEventListener('wheel', (e) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.15 : 0.85;
    zoomLevel = Math.min(Math.max(zoomLevel * factor, 1.0), 6.0);
    if (zoomLevel === 1.0) {
      panX = 0;
      panY = 0;
    }
    updateImageTransform();
  }, { passive: false });

  // Pan com Mouse no Desktop
  plotViewport.addEventListener('mousedown', (e) => {
    if (zoomLevel <= 1.0 || e.button !== 0) return;
    isPanning = true;
    startX = e.clientX - panX;
    startY = e.clientY - panY;
  });

  window.addEventListener('mousemove', (e) => {
    if (!isPanning || zoomLevel <= 1.0) return;
    panX = e.clientX - startX;
    panY = e.clientY - startY;
    updateImageTransform();
  });

  window.addEventListener('mouseup', () => {
    isPanning = false;
  });

  // GESTOS TOUCH (MOBILE / TABLET: PINCH TO ZOOM, PAN & SCROLL)
  plotViewport.addEventListener('touchstart', (e) => {
    if (e.touches.length === 2) {
      // Início do Pinch-to-zoom com 2 dedos
      const t1 = e.touches[0];
      const t2 = e.touches[1];
      initialPinchDistance = Math.hypot(t1.clientX - t2.clientX, t1.clientY - t2.clientY);
      initialPinchZoom = zoomLevel;
      initialPinchMidX = (t1.clientX + t2.clientX) / 2;
      initialPinchMidY = (t1.clientY + t2.clientY) / 2;
      startPinchPanX = panX;
      startPinchPanY = panY;
    } else if (e.touches.length === 1) {
      const touch = e.touches[0];
      startX = touch.clientX - panX;
      startY = touch.clientY - panY;

      // Duplo-toque rápido para alternar zoom 2x / reset
      const now = Date.now();
      if (now - lastTapTime < 300) {
        if (zoomLevel > 1.0) {
          resetZoom();
        } else {
          zoomLevel = 2.0;
          const rect = plotViewport.getBoundingClientRect();
          panX = (rect.width / 2 - (touch.clientX - rect.left)) * 0.6;
          panY = (rect.height / 2 - (touch.clientY - rect.top)) * 0.6;
          updateImageTransform();
        }
        lastTapTime = 0;
      } else {
        lastTapTime = now;
      }
    }
  }, { passive: true });

  plotViewport.addEventListener('touchmove', (e) => {
    if (e.touches.length === 2) {
      // Gesto de Pinça (Pinch-to-zoom)
      e.preventDefault();
      const t1 = e.touches[0];
      const t2 = e.touches[1];
      const currentDist = Math.hypot(t1.clientX - t2.clientX, t1.clientY - t2.clientY);
      if (initialPinchDistance > 0) {
        const factor = currentDist / initialPinchDistance;
        zoomLevel = Math.min(Math.max(initialPinchZoom * factor, 1.0), 5.0);
        if (zoomLevel > 1.0) {
          const currentMidX = (t1.clientX + t2.clientX) / 2;
          const currentMidY = (t1.clientY + t2.clientY) / 2;
          panX = startPinchPanX + (currentMidX - initialPinchMidX);
          panY = startPinchPanY + (currentMidY - initialPinchMidY);
        } else {
          panX = 0;
          panY = 0;
        }
        updateImageTransform();
      }
    } else if (e.touches.length === 1 && zoomLevel > 1.0) {
      // Pan com 1 dedo quando ampliado
      e.preventDefault();
      const touch = e.touches[0];
      panX = touch.clientX - startX;
      panY = touch.clientY - startY;
      updateImageTransform();
    }
    // Quando zoomLevel === 1.0 e 1 dedo: NÃO chama preventDefault -> permite o scroll natural da página!
  }, { passive: false });

  plotViewport.addEventListener('touchend', (e) => {
    if (e.touches.length === 0) {
      if (zoomLevel <= 1.05) {
        resetZoom();
      }
    } else if (e.touches.length === 1) {
      // Transição de 2 dedos para 1 dedo
      const touch = e.touches[0];
      startX = touch.clientX - panX;
      startY = touch.clientY - panY;
    }
  });

  // Botão Flutuante de Retorno ao Topo no Mobile
  function checkScrollTop() {
    const scrollPos = window.scrollY || document.documentElement.scrollTop || document.body.scrollTop || 0;
    if (btnScrollTop) {
      btnScrollTop.classList.toggle('visible', scrollPos > 280);
    }
  }

  window.addEventListener('scroll', checkScrollTop, { passive: true });
  if (btnScrollTop) {
    btnScrollTop.addEventListener('click', () => {
      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
  }

  // =========================================================================
  // EXPORTAÇÃO E DOWNLOAD
  // =========================================================================
  document.getElementById('btn-download-png').addEventListener('click', async () => {
    if (!currentImageData) return;
    try {
      const res = await window.api.saveImage({
        base64: currentImageData,
        defaultName: 'lugar_geometrico_das_raizes.png',
      });
      if (res && res.success !== false) {
        showToast(res.message || 'Imagem PNG salva com sucesso!', 'success');
      }
    } catch (err) {
      showToast(`Erro ao salvar: ${err.message}`, 'error', 3500);
    }
  });

  document.getElementById('btn-download-svg').addEventListener('click', async () => {
    if (!currentSVGData) return;
    try {
      const res = await window.api.saveSVG({
        svg: currentSVGData,
        defaultName: 'lugar_geometrico_das_raizes.svg',
      });
      if (res && res.success !== false) {
        showToast(res.message || 'Gráfico Vetorial SVG salvo com sucesso!', 'success');
      }
    } catch (err) {
      showToast(`Erro ao salvar SVG: ${err.message}`, 'error', 3500);
    }
  });

  document.getElementById('btn-copy-img').addEventListener('click', async () => {
    if (!currentImageData) return;
    try {
      const res = await window.api.copyImageToClipboard(currentImageData);
      if (res.success) {
        showToast('Gráfico copiado para a Área de Transferência!', 'success');
      } else if (res.error) {
        showToast(`Não foi possível copiar: ${res.error}`, 'error', 4000);
      }
    } catch (err) {
      showToast(`Erro ao copiar: ${err.message}`, 'error', 3500);
    }
  });

  // =========================================================================
  // HELPERS DE UI
  // =========================================================================
  function showLoading(show) {
    if (show) {
      plotLoading.classList.add('active');
      btnCalculate.disabled = true;
    } else {
      plotLoading.classList.remove('active');
      btnCalculate.disabled = false;
    }
  }

  function showToast(msg, type = 'info', duration = 2800) {
    let iconSvg = '';
    if (type === 'success') {
      iconSvg = '<svg class="toast-svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>';
    } else if (type === 'error') {
      iconSvg = '<svg class="toast-svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#f43f5e" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>';
    } else {
      iconSvg = '<svg class="toast-svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>';
    }
    toastEl.innerHTML = `${iconSvg}<span>${msg}</span>`;
    toastEl.className = `toast active toast-${type}`;
    if (toastEl._timer) clearTimeout(toastEl._timer);
    toastEl._timer = setTimeout(() => {
      toastEl.classList.remove('active');
    }, duration);
  }

  // Atalho de Teclado: Ctrl+Enter / Cmd+Enter
  window.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      calculate();
    }
  });

  btnCalculate.addEventListener('click', calculate);

  [
    inputExpr,
    inputNumerator,
    inputDenominator,
    inputNum,
    inputDen,
    inputK,
    inputZeros,
    inputPoles,
  ].forEach((input) => input.addEventListener('input', schedulePreview));

  renderMathInContainer(document.body);

  // Inicialização imediata
  schedulePreview();
  calculate();
});
