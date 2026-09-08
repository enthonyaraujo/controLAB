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

      // 4. Preencher o Memorial de Cálculo dos 7 Passos
      populateMemorialSteps(det);

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
      ignoredTags: ['script', 'noscript', 'style', 'textarea', 'pre', 'code'],
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
    const polosStr = det.polos.map((p) => p.str).join(', ');
    const zerosStr = det.Z > 0 ? det.zeros.map((z) => z.str).join(', ') : 'Nenhum zero finito';

    el1.innerHTML = `
      <div class="step-item-box">
        <strong>Número de Polos ($P$):</strong> ${det.P} &nbsp;|&nbsp; <strong>Número de Zeros ($Z$):</strong> ${det.Z}
      </div>
      <p>• <strong>Polos de Malha Aberta ($K=0$):</strong> <code>${polosStr}</code></p>
      <p>• <strong>Zeros de Malha Aberta ($K\\to\\infty$):</strong> <code>${zerosStr}</code></p>
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
