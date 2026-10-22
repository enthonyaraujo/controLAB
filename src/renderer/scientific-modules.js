(function registerScientificModules(global) {
  const CONTROLLER_PRESETS = {
    lead_1: {
      plant_expr: '5 / (s * (s + 1))',
      controller_type: 'lead',
      input_type: 'step',
      settling_criterion: '2',
      ts: '2',
      mp: '16',
      xi: '0.5',
      wn: '4',
      lead_zero: '',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    lag_1: {
      plant_expr: '1 / ((s + 1) * (s + 2))',
      controller_type: 'lag',
      input_type: 'step',
      settling_criterion: '2',
      error_reduction_factor: '10',
      target_steady_state_error: '',
      lag_pole: '0.01',
      lag_gain: '1',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    p_1: {
      plant_expr: '1 / (s * (s + 1) * (s + 2))',
      controller_type: 'P',
      input_type: 'step',
      settling_criterion: '2',
      ts: '10',
      mp: '10',
      xi: '0.591',
      wn: '0.6768',
      lgr_branch_re: '',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    p_2: {
      plant_expr: '1 / ((s + 1) * (s + 5))',
      controller_type: 'P',
      input_type: 'step',
      settling_criterion: '2',
      ts: '2',
      mp: '10',
      xi: '0.591',
      wn: '3.3841',
      lgr_branch_re: '3',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    pd_1: {
      plant_expr: '40 / (s * (s + 2))',
      controller_type: 'PD',
      input_type: 'step',
      settling_criterion: '2',
      ts: '1',
      mp: '25',
      xi: '0.4',
      wn: '10',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    pd_2: {
      plant_expr: '1 / (s * (s + 4) * (s + 6))',
      controller_type: 'PD',
      input_type: 'step',
      settling_criterion: '2',
      ts: '1.7544',
      mp: '20',
      xi: '0.456',
      wn: '5',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'wn',
    },
    pi_1: {
      plant_expr: '10 / ((s + 2) * (s + 5))',
      controller_type: 'PI',
      input_type: 'step',
      settling_criterion: '5',
      ts: '1.5',
      mp: '0.63',
      xi: '0.857',
      wn: '2.3337',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
    pi_2: {
      plant_expr: '1 / (s * (s + 4))',
      controller_type: 'PI',
      input_type: 'ramp',
      settling_criterion: '5',
      ts: '4',
      mp: '20',
      xi: '0.456',
      wn: '1.6447',
      primary_damping_spec: 'xi',
      primary_freq_spec: 'ts',
    },
  };

  const CONTROLLER_FORMULAS = {
    lead: 'G_c(s) = \\frac{K(s+z)}{s+p}, \\quad z < p',
    lag: 'G_c(s) = \\frac{K(s+z)}{s+p}, \\quad p < z',
    P: 'G_c(s) = K_p, \\quad u(t) = K_p e(t)',
    PD: 'G_c(s) = K_p(1 + \\tau_d s), \\quad z = \\frac{1}{\\tau_d}',
    PI: 'G_c(s) = K_p\\frac{s + 1/\\tau_i}{s} = \\frac{K_p(s+z)}{s}, \\quad z = \\frac{1}{\\tau_i}',
  };

  const MODULES = {
    time: {
      action: 'time_response',
      title: 'Resposta no Domínio do Tempo',
      kicker: 'Transitório & Regime Permanente',
      description: 'Simule degrau, impulso e rampa em malha fechada e calcule as métricas transitórias e os erros estacionários.',
      filename: 'resposta-temporal',
      primaryKeys: ['Sobressinal (%OS)', 'Tempo de pico (tp)', 'Acomodação', 'ess degrau'],
      fields: [
        {
          name: 'expr',
          label: 'G(s)',
          type: 'text',
          value: '4 / (s^2 + 2*s + 4)',
          placeholder: 'Ex.: 4 / (s^2 + 2*s + 4)',
          help: 'Use s como variável. Realimentação unitária negativa.',
        },
        {
          name: 'final_time',
          label: 'Tempo final (s)',
          type: 'number',
          value: '10',
          min: '0.05',
          max: '10000',
        },
        {
          name: 'settling_threshold',
          label: 'Acomodação',
          type: 'select',
          value: '0.02',
          options: [
            ['0.02', '2%'],
            ['0.05', '5%'],
          ],
        },
      ],
    },
    frequency: {
      action: 'frequency_response',
      title: 'Resposta em Frequência',
      kicker: 'Bode & Nyquist',
      description: 'Compare a curva real às assíntotas de Bode, visualize Nyquist e obtenha margens e frequências críticas.',
      filename: 'resposta-em-frequencia',
      primaryKeys: ['Margem de ganho (MG)', 'Cruzamento de ganho (ωcg)', 'Margem de fase (MF)', 'Cruzamento de fase (ωcf)'],
      fields: [
        {
          name: 'expr',
          label: 'G(s)',
          type: 'text',
          value: '10 / (s * (s + 2) * (s + 5))',
          placeholder: 'Ex.: 10 / (s * (s + 2) * (s + 5))',
        },
        {
          name: 'omega_min',
          label: 'Frequência mínima (rad/s)',
          type: 'number',
          value: '0.01',
          min: '0.000001',
        },
        {
          name: 'omega_max',
          label: 'Frequência máxima (rad/s)',
          type: 'number',
          value: '100',
          min: '0.00001',
        },
      ],
    },
    controllers: {
      action: 'controller_design',
      title: 'Projeto de Controladores',
      kicker: 'Síntese em Série via LGR — Gma(s) = Gc(s)G(s)',
      description: 'Projete controladores por Avanço de Fase, Atraso de Fase, P, PD e PI no plano complexo s atendendo a especificações transitórias e de erro estacionário.',
      filename: 'projeto-de-controlador',
      primaryKeys: ['Polo dominante s', 'Ganho', 'Zero do controlador (z)', 'Polo do controlador (p)', 'Erro e(∞) compensado', 'Estabilidade'],
      fields: [
        {
          name: 'plant_expr',
          label: 'Planta G(s)',
          type: 'text',
          value: '5 / (s * (s + 1))',
          placeholder: 'Ex.: 5 / (s * (s + 1))',
          help: 'Digite qualquer função de transferência G(s) genérica.',
        },
        {
          name: 'controller_type',
          label: 'Tipo de Controlador Gc(s)',
          type: 'select',
          value: 'lead',
          help: 'Selecione entre Avanço de Fase, Atraso de Fase, P, PD ou PI em série com G(s).',
          options: [
            ['lead', '1. Avanço de Fase — Gc(s) = K(s+z)/(s+p), z < p'],
            ['lag', '2. Atraso de Fase — Gc(s) = K(s+z)/(s+p), p < z'],
            ['P', '3. Proporcional (P) — Gc(s) = Kp'],
            ['PD', '4. Proporcional e Derivativo (PD) — Gc(s) = Kp(1 + τd·s)'],
            ['PI', '5. Proporcional-Integral (PI) — Gc(s) = Kp(s+z)/s'],
          ],
          showControllerFormula: true,
        },
        {
          name: 'input_type',
          label: 'Entrada R(s)',
          type: 'select',
          value: 'step',
          help: 'Entrada de referência R(s): degrau unitário 1/s, rampa unitária 1/s² ou parábola unitária 1/s³.',
          options: [
            ['step', 'Degrau 1/s'],
            ['ramp', 'Rampa 1/s²'],
            ['parabola', 'Parábola 1/s³'],
          ],
        },
        {
          name: 'settling_criterion',
          label: 'Critério de ts',
          type: 'select',
          value: '2',
          help: 'Critério de 2% (ts = 4/(ξωn)) ou de 5% (ts = 3/(ξωn)).',
          options: [
            ['2', 'Critério 2%'],
            ['5', 'Critério 5%'],
          ],
        },
        {
          name: 'ts',
          label: 'Tempo de acomodação ts (s)',
          type: 'number',
          value: '2',
          min: '0.0001',
          placeholder: 'Ex.: 2',
          help: 'Calcula ωn automaticamente a partir de ts e ξ.',
          when: { controller_type: ['lead', 'P', 'PD', 'PI'] },
        },
        {
          name: 'mp',
          label: 'Sobressinal máximo Mp (%)',
          type: 'number',
          value: '16',
          min: '0',
          max: '99.99',
          placeholder: 'Ex.: 16',
          help: 'Converte automaticamente entre Mp(%) e ξ pela fórmula exponencial.',
          when: { controller_type: ['lead', 'P', 'PD', 'PI'] },
        },
        {
          name: 'xi',
          label: 'Coeficiente de amortecimento (ξ)',
          type: 'number',
          value: '0.5',
          min: '0.0001',
          max: '0.9999',
          placeholder: 'Ex.: 0.5',
          help: 'Calcula Mp(%) e ωn (ou ts) automaticamente ao editar.',
          when: { controller_type: ['lead', 'P', 'PD', 'PI'] },
        },
        {
          name: 'wn',
          label: 'Frequência natural ωn (rad/s)',
          type: 'number',
          value: '4',
          min: '0.0001',
          placeholder: 'Ex.: 4',
          help: 'Calcula ts automaticamente a partir de ωn e ξ.',
          when: { controller_type: ['lead', 'P', 'PD', 'PI'] },
          showDominantPolePreview: true,
        },
        {
          name: 'lead_zero',
          label: 'Zero do controlador z (opcional)',
          type: 'number',
          value: '',
          min: '0.0001',
          placeholder: 'Vazio = alocar em z = Re (θz = 90°)',
          help: 'Por padrão aloca o zero em z = Re (θz = 90°). Preencha para customizar z.',
          when: { controller_type: ['lead'] },
        },
        {
          name: 'error_reduction_factor',
          label: 'Fator de redução de e(∞)',
          type: 'number',
          value: '10',
          min: '1.0001',
          placeholder: 'Ex.: 10 (reduzir erro em 10 vezes)',
          help: 'Divide o erro em regime permanente atual por este fator.',
          when: { controller_type: ['lag'] },
        },
        {
          name: 'target_steady_state_error',
          label: 'Meta direta para e_novo(∞) (opcional)',
          type: 'number',
          value: '',
          min: '0.000001',
          placeholder: 'Ex.: 0.0667 (sobrepõe o fator acima se preenchido)',
          help: 'Se preenchido, define diretamente o valor desejado para o novo erro estacionário.',
          when: { controller_type: ['lag'] },
        },
        {
          name: 'lag_pole',
          label: 'Polo do controlador p (próximo da origem)',
          type: 'number',
          value: '0.01',
          min: '0.000001',
          placeholder: 'Padrão: 0.01',
          help: 'Polo adotado próximo da origem (p < z) para não afetar significativamente o LGR.',
          when: { controller_type: ['lag'] },
        },
        {
          name: 'lag_gain',
          label: 'Ganho K do Atraso de Fase',
          type: 'number',
          value: '1',
          min: '0.0001',
          placeholder: 'Padrão: 1',
          help: 'Considera K = 1 por padrão para preservar a resposta transitória.',
          when: { controller_type: ['lag'] },
        },
        {
          name: 'lgr_branch_re',
          label: 'Abscissa Re_LGR do ramo vertical (opcional)',
          type: 'number',
          value: '',
          min: '0.0001',
          placeholder: 'Auto (ou informe Re_LGR, ex.: 3 para s = -3 ± ωd·j)',
          help: 'Se a planta possuir ramo vertical no LGR em -Re_LGR ≠ -Re, ajusta ωd = Re_LGR · tan(θ).',
          when: { controller_type: ['P'] },
        },
        {
          name: 'primary_damping_spec',
          label: '',
          type: 'hidden',
          value: 'xi',
        },
        {
          name: 'primary_freq_spec',
          label: '',
          type: 'hidden',
          value: 'ts',
        },
      ],
    },
    'state-space': {
      action: 'state_space',
      title: 'Espaço de Estados',
      kicker: 'Modelagem Matricial Moderna',
      description: 'Converta modelos, verifique controlabilidade e observabilidade e projete realimentação e observadores.',
      primaryKeys: ['Autovalores de A', 'Estabilidade', 'Controlabilidade (posto)', 'Observabilidade (posto)', 'Forma canônica solicitada', 'Ganho K', 'Ganho L'],
      fields: [
        {
          name: 'mode', label: 'Forma de entrada', type: 'select', value: 'matrices',
          options: [['matrices', 'Matrizes A, B, C e D'], ['transfer', 'Função de transferência']],
        },
        {
          name: 'expr', label: 'Função de transferência G(s)', type: 'text', value: '1 / (s^2 + 3*s + 2)',
          placeholder: 'Ex.: 1 / (s^2 + 3*s + 2)', when: { mode: ['transfer'] },
        },
        {
          name: 'A', label: 'Matriz A', type: 'textarea', rows: 2, value: '[[0, 1], [-2, -3]]',
          help: 'Use colchetes ou separe as linhas por ponto e vírgula.', when: { mode: ['matrices'] },
        },
        { name: 'B', label: 'Matriz B', type: 'textarea', rows: 2, value: '[[0], [1]]', when: { mode: ['matrices'] } },
        { name: 'C', label: 'Matriz C', type: 'textarea', rows: 2, value: '[[1, 0]]', when: { mode: ['matrices'] } },
        { name: 'D', label: 'Matriz D', type: 'textarea', rows: 2, value: '[[0]]', when: { mode: ['matrices'] } },
        {
          name: 'canonical_form', label: 'Representação solicitada', type: 'select', value: 'controllable',
          options: [
            ['original', 'Original'],
            ['controllable', 'Canônica controlável'],
            ['observable', 'Canônica observável'],
            ['diagonal', 'Canônica diagonal'],
            ['jordan', 'Forma de Jordan'],
          ],
        },
        {
          name: 'desired_poles', label: 'Polos desejados para A - BK', type: 'text', value: '-4, -5',
          placeholder: 'Ex.: -4, -5', help: 'Opcional. Um polo por estado.',
        },
        {
          name: 'observer_poles', label: 'Polos do observador A - LC', type: 'text', value: '-6, -7',
          placeholder: 'Ex.: -6, -7', help: 'Opcional. Um polo por estado.',
        },
      ],
    },
  };

  function element(id) {
    return document.getElementById(id);
  }

  function appendTextNode(parent, tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    node.textContent = text;
    parent.appendChild(node);
    return node;
  }

  function formatDisplayNumber(value) {
    if (value === null || value === undefined || value === '—' || value === 'None') return '—';
    const str = String(value);
    const num = Number(str);
    if (!isNaN(num) && str.trim() !== '') {
      return num.toLocaleString('pt-BR', { maximumFractionDigits: 3 });
    }
    return str.replace('.', ',');
  }

  function initialize({ renderMath, showToast }) {
    const form = element('scientific-form');
    const runButton = element('btn-run-scientific');
    const previewContainer = element('scientific-function-preview');
    const previewStatus = element('scientific-preview-status');
    const previewMath = element('scientific-latex-preview');
    const previewError = element('scientific-preview-error');
    if (!form || !runButton) return null;

    let currentKey = null;
    let currentResult = null;
    let currentResponseType = 'step';
    let requestId = 0;
    let previewTimer = null;
    let previewRequestId = 0;
    let lastPreviewExpr = null;

    // Controlador Unificado de Zoom (Desktop e Touch Android)
    const createPlotZoom = global.createPlotZoomController || (typeof window !== 'undefined' && window.createPlotZoomController);
    let singleZoomCtrl = null;
    const stackedZoomCtrls = new Map();

    function getActiveZoomController() {
      if (stackedZoomCtrls.size > 0) {
        return stackedZoomCtrls.get(currentResponseType) || stackedZoomCtrls.values().next().value;
      }
      return singleZoomCtrl;
    }

    function setupSinglePlotCard(imageSrc = '') {
      const resultsContainer = element('scientific-results');
      if (!resultsContainer) return null;

      resultsContainer.classList.remove('multi-plots-scroll');
      stackedZoomCtrls.clear();

      let card = element('scientific-plot-card');
      let img = element('scientific-plot');

      if (!card || !img || !resultsContainer.contains(card)) {
        resultsContainer.replaceChildren();

        card = document.createElement('section');
        card.id = 'scientific-plot-card';
        card.className = 'scientific-plot-card plot-viewport';
        card.style.cssText = 'width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; overflow: hidden; position: relative;';

        img = document.createElement('img');
        img.id = 'scientific-plot';
        img.className = 'plot-img';
        img.alt = 'Gráfico da resposta';
        img.style.cssText = 'max-width: 100%; max-height: 100%; object-fit: contain; display: block;';
        if (imageSrc) img.src = imageSrc;
        card.appendChild(img);
        resultsContainer.appendChild(card);

        if (createPlotZoom) {
          singleZoomCtrl = createPlotZoom({
            viewport: card,
            image: img,
          });
        }
      } else {
        card.className = 'scientific-plot-card plot-viewport';
        card.style.cssText = 'width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; overflow: hidden; position: relative;';
        img.className = 'plot-img';
        img.style.cssText = 'max-width: 100%; max-height: 100%; object-fit: contain; display: block;';
        if (imageSrc) img.src = imageSrc;

        if (!singleZoomCtrl || (typeof singleZoomCtrl.getViewport === 'function' && singleZoomCtrl.getViewport() !== card)) {
          if (createPlotZoom) {
            singleZoomCtrl = createPlotZoom({
              viewport: card,
              image: img,
            });
          }
        } else {
          singleZoomCtrl.setImage(img);
        }
      }

      singleZoomCtrl?.resetZoom();
      return { card, img };
    }

    const moduleStates = {
      time: { result: null, formData: null, responseType: 'step', consoleText: '>> step(G, 10)' },
      frequency: { result: null, formData: null, responseType: 'step', consoleText: '>> bode(G)' },
      controllers: { result: null, formData: null, responseType: 'step', consoleText: '>> controller_design(G, Gc)' },
      'state-space': { result: null, formData: null, responseType: 'step', consoleText: '>> ss(A, B, C, D)' },
    };

    function saveCurrentState() {
      if (!currentKey || !moduleStates[currentKey]) return;
      const config = MODULES[currentKey];
      if (config) {
        const data = {};
        config.fields.forEach((field) => {
          const input = form.elements.namedItem(field.name);
          if (input) {
            data[field.name] = field.type === 'checkbox' ? input.checked : input.value;
          }
        });
        moduleStates[currentKey].formData = data;
      }
      moduleStates[currentKey].responseType = currentResponseType;
      const logEl = element('sci-console-text');
      if (logEl) {
        moduleStates[currentKey].consoleText = logEl.textContent;
      }
    }

    function formatExpressionToLatex(raw, prefix = 'G(s)') {
      if (!raw) return `${prefix} = 0`;
      let tex = raw.trim();
      tex = tex
        .replace(/\s+/g, ' ')
        .replace(/\*/g, ' ')
        .replace(/\^([0-9]+)/g, '^{$1}')
        .replace(/s([0-9]+)/g, 's^{$1}');

      if (tex.includes('/')) {
        const slashIdx = tex.indexOf('/');
        let num = tex.slice(0, slashIdx).trim();
        let den = tex.slice(slashIdx + 1).trim();
        if (num.startsWith('(') && num.endsWith(')')) num = num.slice(1, -1).trim();
        if (den.startsWith('(') && den.endsWith(')')) den = den.slice(1, -1).trim();
        return `${prefix} = \\frac{${num}}{${den}}`;
      }
      return `${prefix} = ${tex}`;
    }

    function getExpressionInput() {
      return form.elements.namedItem('expr') || form.elements.namedItem('plant_expr');
    }

    function scheduleScientificPreview() {
      const exprInput = getExpressionInput();
      if (!exprInput || !previewContainer) return;
      const raw = exprInput.value.trim();

      if (previewMath) {
        const instantTex = formatExpressionToLatex(raw);
        renderMath(previewMath, instantTex, true);
      }

      if (previewStatus) {
        if (!raw) {
          previewStatus.textContent = 'Aguardando entrada';
          previewStatus.className = 'preview-status';
        } else if (raw === '4 / (s^2 + 2*s + 4)' || raw === lastPreviewExpr) {
          previewStatus.textContent = 'válida';
          previewStatus.className = 'preview-status valid';
        } else {
          previewStatus.textContent = 'Validando…';
          previewStatus.className = 'preview-status loading';
        }
      }
      if (previewError) previewError.hidden = true;

      if (!raw) {
        lastPreviewExpr = '';
        return;
      }

      const reqId = ++previewRequestId;
      window.clearTimeout(previewTimer);
      previewTimer = window.setTimeout(async () => {
        if (reqId !== previewRequestId) return;
        if (raw === lastPreviewExpr) return;

        if (!global.api?.previewTransferFunction) {
          if (previewStatus) {
            previewStatus.textContent = 'válida';
            previewStatus.className = 'preview-status valid';
          }
          return;
        }

        try {
          const res = await global.api.previewTransferFunction({ mode: 'expr', expr: raw });
          if (reqId !== previewRequestId) return;
          if (!res || !res.success) {
            throw new Error(res?.error || 'Expressão matemática inválida.');
          }
          lastPreviewExpr = raw;
          if (previewMath) {
            renderMath(previewMath, res.latex_fac || res.latex_exp, true);
          }
          if (previewStatus) {
            previewStatus.textContent = 'válida';
            previewStatus.className = 'preview-status valid';
          }
          if (previewError) previewError.hidden = true;
        } catch (err) {
          if (reqId !== previewRequestId) return;
          if (previewStatus) {
            previewStatus.textContent = 'Revise a entrada';
            previewStatus.className = 'preview-status invalid';
          }
          if (previewError) {
            previewError.textContent = err.message;
            previewError.hidden = false;
          }
        }
      }, 300);
    }

    function getSystemTransferFunctionExpression() {
      const lgrInput = element('input-expr');
      if (lgrInput && lgrInput.value && lgrInput.value.trim()) {
        return lgrInput.value.trim();
      }
      const timeExpr = moduleStates.time?.formData?.expr;
      if (timeExpr && String(timeExpr).trim()) {
        return String(timeExpr).trim();
      }
      const freqExpr = moduleStates.frequency?.formData?.expr;
      if (freqExpr && String(freqExpr).trim()) {
        return String(freqExpr).trim();
      }
      const routhInput = element('routh-input-expr');
      if (routhInput && routhInput.value && routhInput.value.trim()) {
        return routhInput.value.trim();
      }
      return '5 / (s * (s + 1))';
    }

    function updateControllerFormulaPreview() {
      const box = element('ctrl-formula-preview-math');
      const ctrlInput = form.elements.namedItem('controller_type');
      if (!box || !ctrlInput) return;
      const tex = CONTROLLER_FORMULAS[ctrlInput.value] || CONTROLLER_FORMULAS.lead;
      renderMath(box, tex, true);
    }

    function updateDominantPolePreview() {
      const box = element('ctrl-dominant-pole-math');
      if (!box) return;
      const xiInput = form.elements.namedItem('xi');
      const wnInput = form.elements.namedItem('wn');
      const xiVal = parseFloat(String(xiInput?.value || '').replace(',', '.'));
      const wnVal = parseFloat(String(wnInput?.value || '').replace(',', '.'));
      if (xiVal > 0 && xiVal < 1 && wnVal > 0) {
        const re = xiVal * wnVal;
        const im = wnVal * Math.sqrt(Math.max(0, 1 - xiVal * xiVal));
        const fmt = (v) => Number(v.toFixed(4)).toString().replace('.', '{,}');
        renderMath(box, `s = -\\text{Re} \\pm \\text{Im}j = -${fmt(re)} \\pm ${fmt(im)}j`, false);
      } else {
        box.textContent = 's = -Re ± Im·j (preencha ξ, Mp, ts ou ωn)';
      }
    }

    function recomputeControllerTransients(changedField) {
      if (currentKey !== 'controllers') return;
      const tsInput = form.elements.namedItem('ts');
      const mpInput = form.elements.namedItem('mp');
      const xiInput = form.elements.namedItem('xi');
      const wnInput = form.elements.namedItem('wn');
      const critInput = form.elements.namedItem('settling_criterion');
      const primaryDamp = form.elements.namedItem('primary_damping_spec');
      const primaryFreq = form.elements.namedItem('primary_freq_spec');
      if (!tsInput || !mpInput || !xiInput || !wnInput) return;

      const parseNum = (inp) => {
        const raw = String(inp?.value ?? '').trim().replace(',', '.');
        if (!raw) return NaN;
        return Number(raw);
      };

      const critConst = String(critInput?.value || '2') === '5' ? 3 : 4;

      if (changedField === 'mp') {
        if (primaryDamp) primaryDamp.value = 'mp';
        const mpVal = parseNum(mpInput);
        if (mpVal > 0 && mpVal < 100) {
          const lnMp = Math.log(mpVal / 100);
          const xiCalc = -lnMp / Math.sqrt(Math.PI * Math.PI + lnMp * lnMp);
          xiInput.value = String(Number(xiCalc.toFixed(4)));
        } else if (mpVal === 0) {
          xiInput.value = '1';
        }
      } else if (changedField === 'xi') {
        if (primaryDamp) primaryDamp.value = 'xi';
        const xiVal = parseNum(xiInput);
        if (xiVal > 0 && xiVal < 1) {
          const mpCalc = Math.exp((-xiVal * Math.PI) / Math.sqrt(1 - xiVal * xiVal)) * 100;
          mpInput.value = String(Number(mpCalc.toFixed(3)));
        } else if (xiVal >= 1) {
          mpInput.value = '0';
        }
      }

      const xiNow = parseNum(xiInput);
      if (changedField === 'ts') {
        if (primaryFreq) primaryFreq.value = 'ts';
        const tsVal = parseNum(tsInput);
        if (tsVal > 0 && xiNow > 0) {
          const wnCalc = critConst / (xiNow * tsVal);
          wnInput.value = String(Number(wnCalc.toFixed(4)));
        }
      } else if (changedField === 'wn') {
        if (primaryFreq) primaryFreq.value = 'wn';
        const wnVal = parseNum(wnInput);
        if (wnVal > 0 && xiNow > 0) {
          const tsCalc = critConst / (xiNow * wnVal);
          tsInput.value = String(Number(tsCalc.toFixed(4)));
        }
      } else if (changedField === 'mp' || changedField === 'xi' || changedField === 'settling_criterion') {
        const freqMode = primaryFreq?.value || 'ts';
        if (freqMode === 'wn') {
          const wnVal = parseNum(wnInput);
          if (wnVal > 0 && xiNow > 0) {
            tsInput.value = String(Number((critConst / (xiNow * wnVal)).toFixed(4)));
          }
        } else {
          const tsVal = parseNum(tsInput);
          if (tsVal > 0 && xiNow > 0) {
            wnInput.value = String(Number((critConst / (xiNow * tsVal)).toFixed(4)));
          }
        }
      }

      updateDominantPolePreview();
    }

    function applyControllerPreset(presetKey) {
      const preset = CONTROLLER_PRESETS[presetKey];
      if (!preset) return;
      Object.entries(preset).forEach(([key, val]) => {
        const control = form.elements.namedItem(key);
        if (!control) return;
        control.value = val;
        const segContainer = control.parentElement?.querySelector('.form-segmented');
        if (segContainer) {
          segContainer.querySelectorAll('.tab-seg').forEach((btn) => {
            btn.classList.toggle('active', btn.dataset.val === String(val));
          });
        }
      });
      updateConditionalFields();
      updateControllerFormulaPreview();
      updateDominantPolePreview();
      scheduleScientificPreview();
    }

    function renderFields(config, savedData = null) {
      if (previewContainer && form.parentElement) {
        previewContainer.style.display = 'none';
        form.parentElement.appendChild(previewContainer);
      }
      form.replaceChildren();

      if (config.underDevelopment) {
        const devCard = document.createElement('div');
        devCard.className = 'sci-dev-card';
        devCard.innerHTML = `
          <span class="sci-dev-card-badge">Em desenvolvimento</span>
          <p class="sci-dev-card-desc">Parâmetros desativados. Toda esta aba de controladores está em processo de reformulação analítica e será recriada em breve.</p>
        `;
        form.appendChild(devCard);
        if (previewContainer) previewContainer.style.display = 'none';
        return;
      }
      config.fields.forEach((field) => {
        const initialVal = (savedData && savedData[field.name] !== undefined) ? savedData[field.name] : field.value;

        if (field.type === 'hidden') {
          const hiddenInput = document.createElement('input');
          hiddenInput.type = 'hidden';
          hiddenInput.name = field.name;
          hiddenInput.value = initialVal !== undefined && initialVal !== null ? initialVal : '';
          form.appendChild(hiddenInput);
          return;
        }

        const group = document.createElement('div');
        group.className = 'form-group scientific-field';
        if (field.when) group.dataset.when = JSON.stringify(field.when);

        const labelRow = document.createElement('div');
        labelRow.className = 'form-label-row';
        appendTextNode(labelRow, 'span', 'form-label', field.label);
        if (field.help) {
          const hint = document.createElement('span');
          hint.className = 'input-tooltip';
          hint.title = field.help;
          hint.textContent = '?';
          labelRow.appendChild(hint);
        }
        group.appendChild(labelRow);

        let input;
        if (field.type === 'select') {
          // Se tiver 2 ou 3 opções curtas, renderiza como segmented pill buttons
          if (field.options.length <= 3 && field.options.every(([_, l]) => l.length <= 15)) {
            const segContainer = document.createElement('div');
            segContainer.className = 'tabs-segmented form-segmented';
            
            const hiddenInput = document.createElement('input');
            hiddenInput.type = 'hidden';
            hiddenInput.name = field.name;
            hiddenInput.value = initialVal || field.options[0][0];
            
            field.options.forEach(([val, label]) => {
              const pill = document.createElement('button');
              pill.type = 'button';
              pill.dataset.val = String(val);
              pill.className = `tab-seg ${String(val) === String(hiddenInput.value) ? 'active' : ''}`;
              pill.textContent = label;
              pill.addEventListener('click', () => {
                hiddenInput.value = val;
                segContainer.querySelectorAll('.tab-seg').forEach(p => p.classList.remove('active'));
                pill.classList.add('active');
                updateConditionalFields();
                if (field.name === 'settling_criterion') {
                  recomputeControllerTransients('settling_criterion');
                }
              });
              segContainer.appendChild(pill);
            });
            group.appendChild(hiddenInput);
            group.appendChild(segContainer);
            form.appendChild(group);
            return;
          }

          input = document.createElement('select');
          field.options.forEach(([value, label]) => {
            const option = document.createElement('option');
            option.value = value;
            option.textContent = label;
            option.title = label;
            option.selected = String(value) === String(initialVal);
            input.appendChild(option);
          });
          const updateTooltip = () => {
            const opt = input.options[input.selectedIndex];
            if (opt) input.title = opt.textContent;
          };
          updateTooltip();
          input.addEventListener('change', () => {
            updateTooltip();
            if (field.name === 'controller_type') {
              updateControllerFormulaPreview();
            }
          });
        } else if (field.type === 'textarea') {
          input = document.createElement('textarea');
          input.rows = field.rows || 2;
          input.value = initialVal || '';
        } else {
          input = document.createElement('input');
          input.type = field.type || 'text';
          input.value = initialVal !== undefined && initialVal !== null ? initialVal : '';
          ['placeholder', 'min', 'max'].forEach((property) => {
            if (field[property] !== undefined) input[property] = field[property];
          });
          if (field.type === 'number') {
            input.step = 'any';
            input.inputMode = 'decimal';
          }
        }
        input.name = field.name;
        input.className = 'form-input code-font';
        input.autocomplete = 'off';
        group.appendChild(input);

        if (field.showControllerFormula) {
          const formulaBox = document.createElement('div');
          formulaBox.className = 'ctrl-formula-preview-card';
          const formulaLabel = document.createElement('span');
          formulaLabel.className = 'ctrl-formula-preview-label';
          formulaLabel.textContent = 'Lei do controlador em série:';
          const formulaMath = document.createElement('div');
          formulaMath.id = 'ctrl-formula-preview-math';
          formulaMath.className = 'ctrl-formula-preview-math';
          formulaBox.appendChild(formulaLabel);
          formulaBox.appendChild(formulaMath);
          group.appendChild(formulaBox);
        }

        if (field.showDominantPolePreview) {
          const domBox = document.createElement('div');
          domBox.className = 'ctrl-dominant-pole-badge';
          const domLabel = document.createElement('span');
          domLabel.className = 'ctrl-dominant-pole-label';
          domLabel.textContent = 'Polos dominantes desejados:';
          const domMath = document.createElement('div');
          domMath.id = 'ctrl-dominant-pole-math';
          domMath.className = 'ctrl-dominant-pole-math';
          domBox.appendChild(domLabel);
          domBox.appendChild(domMath);
          group.appendChild(domBox);
        }

        form.appendChild(group);

        if (field.name === 'expr' || field.name === 'plant_expr') {
          if (previewContainer) {
            form.appendChild(previewContainer);
          }
        }
      });

      if (config.examples?.length) {
        const examples = document.createElement('div');
        examples.className = 'scientific-examples';
        const chips = document.createElement('div');
        chips.className = 'chip-group';
        config.examples.forEach(([label, expression]) => {
          const chip = appendTextNode(chips, 'button', 'chip', label);
          chip.type = 'button';
          chip.addEventListener('click', () => {
            const expressionInput = form.elements.namedItem('expr') || form.elements.namedItem('plant_expr');
            if (expressionInput) {
              expressionInput.value = expression;
              scheduleScientificPreview();
              run();
            }
          });
        });
        examples.appendChild(chips);
        form.appendChild(examples);
      }
      updateConditionalFields();
      updateControllerFormulaPreview();
      updateDominantPolePreview();
    }

    function updateConditionalFields() {
      form.querySelectorAll('[data-when]').forEach((group) => {
        const conditions = JSON.parse(group.dataset.when);
        group.hidden = !Object.entries(conditions).every(([fieldName, accepted]) => {
          const control = form.elements.namedItem(fieldName);
          return control && accepted.includes(control.value);
        });
      });

      const exprInput = getExpressionInput();
      if (previewContainer) {
        const isExprVisible = exprInput && !exprInput.closest('.form-group')?.hidden;
        previewContainer.style.display = isExprVisible ? 'block' : 'none';
        if (isExprVisible) {
          scheduleScientificPreview();
        }
      }
    }

    const MODULE_METAS = {
      time: {
        figureTitle: 'Figura 1 — Resposta temporal',
        hasResponseTabs: true,
        defaultConsole: '>> step(G, 10)',
        solverMeta: 'Módulo: Resposta no Tempo • Solver: scipy',
        cmd: 'step',
        loadingText: 'Calculando Resposta no Tempo...',
      },
      frequency: {
        figureTitle: 'Figura 1 — Diagrama de Bode',
        hasResponseTabs: false,
        defaultConsole: '>> bode(G)',
        solverMeta: 'Módulo: Resposta em Frequência • Solver: scipy / python-control',
        cmd: 'bode',
        loadingText: 'Calculando Resposta em Frequência (Bode)...',
      },
      controllers: {
        figureTitle: 'Figura 1 — Lugar Geométrico das Raízes (LGR) e Resposta Compensada',
        hasResponseTabs: false,
        defaultConsole: '>> controller_design(G, Gc)',
        solverMeta: 'Módulo: Projeto de Controladores • Solver: LGR / sympy / python-control',
        cmd: 'controller_design',
        loadingText: 'Projetando controlador em série via LGR...',
      },
      'state-space': {
        figureTitle: 'Figura 1 — Plano complexo e polos de estados',
        hasResponseTabs: false,
        defaultConsole: '>> ss(A, B, C, D)',
        solverMeta: 'Módulo: Espaço de Estados • Solver: scipy / sympy',
        cmd: 'ss',
        loadingText: 'Calculando Espaço de Estados...',
      },
    };

    function resetResults(moduleKey) {
      const timeTable = element('time-results-table');
      let generalTable = document.querySelector('#sci-panel-results #scientific-general-results-table') || element('scientific-general-results-table');

      const timeKeys = ['res-val-mp', 'res-val-tp', 'res-val-tr', 'res-val-ts', 'res-val-yf', 'res-val-zeta', 'res-val-wn', 'res-val-stab'];
      timeKeys.forEach(id => {
        document.querySelectorAll('#' + id).forEach(el => { el.textContent = '—'; });
      });
      const tMetrics = ['t-metric-mp', 't-metric-tp', 't-metric-ts'];
      tMetrics.forEach(id => {
        document.querySelectorAll('#' + id).forEach(el => { el.textContent = '—'; });
      });

      if (MODULES[moduleKey]?.underDevelopment) {
        if (timeTable) timeTable.style.display = 'none';
        if (generalTable) generalTable.style.display = 'none';
        let devResults = element('sci-dev-results-placeholder');
        if (!devResults) {
          devResults = document.createElement('div');
          devResults.id = 'sci-dev-results-placeholder';
          devResults.className = 'sci-dev-results-placeholder';
          devResults.textContent = 'Resultados indisponíveis — módulo em desenvolvimento.';
          const parent = element('sci-panel-results')?.querySelector('.sci-panel-body');
          if (parent) parent.insertBefore(devResults, parent.firstChild);
        }
        devResults.style.display = 'flex';
      } else {
        const devResults = element('sci-dev-results-placeholder');
        if (devResults) devResults.style.display = 'none';
        if (moduleKey === 'time') {
          if (timeTable) timeTable.style.display = 'table';
          if (generalTable) generalTable.style.display = 'none';
        } else {
          if (timeTable) timeTable.style.display = 'none';
          if (!generalTable) {
            generalTable = document.createElement('table');
            generalTable.id = 'scientific-general-results-table';
            generalTable.className = 'results-table tb';
            const parent = element('sci-panel-results')?.querySelector('.sci-panel-body');
            if (parent) parent.insertBefore(generalTable, parent.firstChild);
          }
          generalTable.style.display = 'table';
          generalTable.innerHTML = '';
          const tbody = document.createElement('tbody');
          const defaultMetrics = MODULES[moduleKey]?.primaryKeys || [];
          defaultMetrics.forEach(key => {
            const tr = document.createElement('tr');
            const tdL = document.createElement('td');
            tdL.textContent = key;
            const tdV = document.createElement('td');
            tdV.textContent = '—';
            tr.appendChild(tdL);
            tr.appendChild(tdV);
            tbody.appendChild(tr);
          });
          generalTable.appendChild(tbody);
        }
      }

      const primaryContainer = element('scientific-primary-metrics');
      if (primaryContainer) primaryContainer.replaceChildren();
      const allMetricsContainer = element('scientific-metrics');
      if (allMetricsContainer) allMetricsContainer.replaceChildren();
      const latexGrid = element('scientific-latex');
      if (latexGrid) {
        latexGrid.replaceChildren();
        latexGrid.style.display = 'none';
      }
      const stepsPanel = element('scientific-steps-panel');
      const stepsBody = element('scientific-steps-body');
      if (stepsBody) stepsBody.replaceChildren();
      if (stepsPanel) stepsPanel.style.display = 'none';
      const detailsCard = element('scientific-details-card');
      if (detailsCard) detailsCard.hidden = true;
      element('scientific-results')?.classList.remove('multi-plots-scroll');

      const tabletRes = element('sci-tablet-results-container');
      if (tabletRes) {
        tabletRes.innerHTML = '';
        if (MODULES[moduleKey]?.underDevelopment) {
          const cloneDev = document.createElement('div');
          cloneDev.className = 'sci-dev-results-placeholder';
          cloneDev.textContent = 'Resultados indisponíveis — módulo em desenvolvimento.';
          tabletRes.appendChild(cloneDev);
        } else {
          const activeTable = moduleKey === 'time' ? timeTable : generalTable;
          if (activeTable) {
            const clone = activeTable.cloneNode(true);
            clone.removeAttribute('id');
            clone.querySelectorAll('[id]').forEach(el => el.removeAttribute('id'));
            tabletRes.appendChild(clone);
          }
        }
      }
    }

    function open(moduleKey) {
      const config = MODULES[moduleKey];
      if (!config) return false;
      const meta = MODULE_METAS[moduleKey] || {
        figureTitle: 'Figura 1',
        hasResponseTabs: false,
        defaultConsole: '>> pronto',
        solverMeta: 'Pronto',
        cmd: 'calc',
      };

      saveCurrentState();

      currentKey = moduleKey;
      const modState = moduleStates[moduleKey] || {
        result: null,
        formData: null,
        responseType: 'step',
        consoleText: meta.defaultConsole,
      };
      currentResult = modState.result;
      currentResponseType = modState.responseType || 'step';
      lastPreviewExpr = null;
      singleZoomCtrl?.resetZoom();
      stackedZoomCtrls.forEach(c => c.resetZoom());

      const kickerEl = element('scientific-kicker');
      if (kickerEl) kickerEl.textContent = config.kicker;
      const titleEl = element('scientific-title');
      if (titleEl) titleEl.textContent = config.title;
      const descEl = element('scientific-description');
      if (descEl) descEl.textContent = config.description;

      renderFields(config, modState.formData);

      // Resetar Figura & Título
      const figTitleEl = element('sci-figure-title');
      if (figTitleEl) figTitleEl.textContent = meta.figureTitle;

      const loadingTextEl = element('scientific-loading-text');
      if (loadingTextEl && meta.loadingText) {
        loadingTextEl.textContent = meta.loadingText;
      }

      const plotTabs = element('scientific-plot-tabs');
      if (plotTabs) {
        plotTabs.style.display = meta.hasResponseTabs ? 'inline-flex' : 'none';
        plotTabs.querySelectorAll('.tab-seg').forEach(t => {
          t.classList.toggle('active', t.dataset.resp === currentResponseType);
        });
      }

      // Console e Status próprios do módulo
      const logEl = element('sci-console-text');
      if (logEl) logEl.textContent = modState.consoleText || meta.defaultConsole;

      const isDev = Boolean(config.underDevelopment);
      const statusState = element('sci-status-state');
      if (statusState) statusState.textContent = isDev ? 'Em desenvolvimento' : 'Pronto';

      const metaEl = element('sci-status-meta');
      if (metaEl) metaEl.textContent = meta.solverMeta;

      element('ribbon-btn-calc')?.classList.toggle('disabled', isDev);
      element('ribbon-btn-clear')?.classList.toggle('disabled', isDev);
      element('ribbon-btn-export')?.classList.toggle('disabled', isDev);
      element('ribbon-btn-code')?.classList.toggle('disabled', isDev);

      const mathKeyboard = element('sci-virtual-math');
      if (mathKeyboard) mathKeyboard.style.display = isDev ? 'none' : '';

      const statusChip = element('scientific-status-chip');
      if (statusChip) statusChip.style.display = 'none';

      if (isDev) {
        setupSinglePlotCard('');
        showState('dev');
        resetResults(moduleKey);
      } else if (modState.result) {
        renderResult(modState.result);
      } else {
        setupSinglePlotCard('');
        showState('empty');
        resetResults(moduleKey);
      }

      return true;
    }

    function showState(state) {
      const loadingEl = element('scientific-loading');
      if (loadingEl) {
        loadingEl.hidden = state !== 'loading';
        loadingEl.style.display = state === 'loading' ? 'flex' : 'none';
      }
      const emptyEl = element('scientific-empty');
      if (emptyEl) emptyEl.style.display = state === 'empty' ? 'flex' : 'none';
      const devEl = element('scientific-dev');
      if (devEl) {
        devEl.hidden = state !== 'dev';
        devEl.style.display = state === 'dev' ? 'flex' : 'none';
      }
      const errorEl = element('scientific-error');
      if (errorEl) errorEl.hidden = state !== 'error';
      const resultsEl = element('scientific-results');
      if (resultsEl) {
        resultsEl.hidden = state !== 'results';
        resultsEl.style.display = state === 'results' ? '' : 'none';
      }
      runButton.disabled = state === 'loading' || state === 'dev';
      const ribbonBtn = element('ribbon-btn-calc');
      if (ribbonBtn) ribbonBtn.classList.toggle('loading', state === 'loading');
    }

    function payloadFromForm(config) {
      const payload = {
        theme: document.documentElement.getAttribute('data-theme') || 'dark',
        response_type: currentResponseType,
      };
      const plotViewport = element('scientific-plot-viewport');
      if (plotViewport) {
        const rect = plotViewport.getBoundingClientRect();
        if (rect.width > 100) {
          payload.width = Math.round(rect.width);
        }
      }
      payload.dpi = Math.min(2, Math.max(1, window.devicePixelRatio || 1)) * 100;

      config.fields.forEach((field) => {
        const input = form.elements.namedItem(field.name);
        if (!input) return;
        if (field.type === 'number') {
          if (input.value !== '') payload[field.name] = Number(String(input.value).trim().replace(',', '.'));
        } else if (field.type === 'checkbox') {
          payload[field.name] = input.checked;
        } else {
          payload[field.name] = input.value.trim();
        }
      });
      return payload;
    }

    function renderResult(result) {
      currentResult = result;
      const config = MODULES[currentKey];
      const meta = MODULE_METAS[currentKey] || {};

      // 1. PLOT & TABS (Degrau / Rampa / Impulso para Resposta no Tempo)
      const resultsContainer = element('scientific-results');
      const plotCard = element('scientific-plot-card');
      const plot = element('scientific-plot');
      const plotTabs = element('scientific-plot-tabs');
      if (plotCard) plotCard.hidden = !result.image;

      const plotKeys = (result.plots && typeof result.plots === 'object') ? Object.keys(result.plots) : [];
      const hasMultiplePlots = plotKeys.length > 1;

      if (hasMultiplePlots) {
        if (resultsContainer) {
          resultsContainer.replaceChildren();
          resultsContainer.classList.add('multi-plots-scroll');
          stackedZoomCtrls.clear();

          plotKeys.forEach((key, idx) => {
            const p = result.plots[key];
            if (!p || !p.image) return;

            const card = document.createElement('section');
            card.className = 'scientific-plot-card stacked-plot-card';
            card.id = `plot-card-${key}`;

            const header = document.createElement('div');
            header.className = 'stacked-plot-header';
            const titleSpan = document.createElement('span');
            titleSpan.className = 'stacked-plot-title';
            const titles = {
              step: 'Figura 1 — Resposta ao degrau',
              ramp: 'Figura 2 — Resposta à rampa',
              impulse: 'Figura 3 — Resposta ao impulso',
            };
            titleSpan.textContent = p.title ? `Figura — ${p.title}` : (titles[key] || `Figura — ${key}`);
            header.appendChild(titleSpan);
            card.appendChild(header);

            const body = document.createElement('div');
            body.className = 'stacked-plot-body plot-viewport';
            const img = document.createElement('img');
            img.className = 'plot-img';
            img.src = p.image;
            img.alt = p.title || key;
            if (idx === 0 || key === 'step') {
              img.id = 'scientific-plot';
            }
            body.appendChild(img);
            card.appendChild(body);

            if (createPlotZoom) {
              const ctrl = createPlotZoom({
                viewport: body,
                image: img,
              });
              stackedZoomCtrls.set(key, ctrl);
            }

            resultsContainer.appendChild(card);
          });
        }

        if (plotTabs) {
          plotTabs.style.display = 'inline-flex';
          plotTabs.querySelectorAll('.tab-seg').forEach((tab) => {
            const resp = tab.dataset.resp;
            tab.classList.toggle('active', resp === currentResponseType);
            tab.onclick = () => {
              currentResponseType = resp;
              plotTabs.querySelectorAll('.tab-seg').forEach(t => t.classList.remove('active'));
              tab.classList.add('active');
              const targetCard = element(`plot-card-${resp}`);
              if (targetCard) {
                targetCard.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
              }
              const figTitleEl = element('sci-figure-title');
              if (figTitleEl) {
                const titles = { step: 'Figura 1 — Resposta ao degrau', ramp: 'Figura 2 — Resposta à rampa', impulse: 'Figura 3 — Resposta ao impulso' };
                figTitleEl.textContent = titles[resp] || 'Figura 1';
              }
            };
          });
        }

        const figTitleEl = element('sci-figure-title');
        if (figTitleEl) {
          figTitleEl.textContent = 'Figuras — Resposta no tempo';
        }
      } else {
        if (resultsContainer) {
          setupSinglePlotCard(result.image);
        }
        if (plotTabs) plotTabs.style.display = 'none';
        const figTitleEl = element('sci-figure-title');
        if (figTitleEl) {
          figTitleEl.textContent = meta.figureTitle || (result.title ? `Figura 1 — ${result.title}` : 'Figura 1');
        }
      }

      if (element('btn-scientific-copy')) element('btn-scientific-copy').hidden = !result.image;
      if (element('btn-scientific-png')) element('btn-scientific-png').hidden = !result.image;
      if (element('btn-scientific-svg')) element('btn-scientific-svg').hidden = !result.svg;

      // Atualiza Tabela de Resultados
      function updateMetricsTable(res) {
        if (!res) return;
        const timeTable = element('time-results-table');
        let generalTable = document.querySelector('#sci-panel-results #scientific-general-results-table') || element('scientific-general-results-table');

        if (currentKey === 'time') {
          if (timeTable) timeTable.style.display = 'table';
          if (generalTable) generalTable.style.display = 'none';

          const step = res.details?.step || {};
          const isStab = res.details?.stable ?? true;

          const fmt = (val, digits = 2, unit = '') => {
            if (val === null || val === undefined || isNaN(val)) return '—';
            return `${Number(val).toLocaleString('pt-BR', { minimumFractionDigits: 0, maximumFractionDigits: digits })}${unit ? ' ' + unit : ''}`;
          };

          const mpStr = step.overshoot_percent != null ? fmt(step.overshoot_percent, 1, '%') : '0,0 %';
          const tpStr = step.peak_time != null ? fmt(step.peak_time, 2, 's') : '—';
          const trStr = step.rise_time != null ? fmt(step.rise_time, 2, 's') : '—';
          const tsStr = step.settling_time != null ? fmt(step.settling_time, 1, 's') : '—';
          const yfStr = step.final_value != null ? fmt(step.final_value, 3) : '—';
          const zetaStr = step.damping_ratio != null ? fmt(step.damping_ratio, 3) : '—';
          const wnStr = step.natural_frequency != null ? fmt(step.natural_frequency, 3, 'rad/s') : '—';
          const stabStr = isStab ? 'Estável' : 'Instável';

          const setT = (id, val) => {
            document.querySelectorAll('#' + id).forEach(el => { el.textContent = val; });
          };

          setT('res-val-mp', mpStr);
          setT('res-val-tp', tpStr);
          setT('res-val-tr', trStr);
          setT('res-val-ts', tsStr);
          setT('res-val-yf', yfStr);
          setT('res-val-zeta', zetaStr);
          setT('res-val-wn', wnStr);
          setT('res-val-stab', stabStr);

          setT('t-metric-mp', mpStr);
          setT('t-metric-tp', tpStr);
          setT('t-metric-ts', tsStr);

          const tabletRes = element('sci-tablet-results-container');
          if (tabletRes && timeTable) {
            tabletRes.innerHTML = '';
            const clone = timeTable.cloneNode(true);
            clone.removeAttribute('id');
            clone.querySelectorAll('[id]').forEach(el => el.removeAttribute('id'));
            tabletRes.appendChild(clone);
          }
        } else {
          if (timeTable) timeTable.style.display = 'none';
          if (!generalTable) {
            generalTable = document.createElement('table');
            generalTable.id = 'scientific-general-results-table';
            generalTable.className = 'results-table tb';
            const parent = element('sci-panel-results')?.querySelector('.sci-panel-body');
            if (parent) parent.insertBefore(generalTable, parent.firstChild);
          }
          generalTable.style.display = 'table';
          generalTable.innerHTML = '';
          const tbody = document.createElement('tbody');
          (res.metrics || []).forEach((m) => {
            if (m.header) {
              const tr = document.createElement('tr');
              const th = document.createElement('th');
              th.colSpan = 2;
              th.className = 'results-group-header';
              th.textContent = m.header;
              tr.appendChild(th);
              tbody.appendChild(tr);
              return;
            }
            const tr = document.createElement('tr');
            const tdL = document.createElement('td');
            tdL.className = 'metric-cell-label';
            tdL.textContent = m.label;
            const tdV = document.createElement('td');
            tdV.className = 'metric-cell-value';
            const valFormatted = `${formatDisplayNumber(m.value)}${m.unit ? ' ' + m.unit : ''}`;
            tdV.textContent = valFormatted;
            if (m.value != null) tdV.title = String(m.value);
            tr.appendChild(tdL);
            tr.appendChild(tdV);
            tbody.appendChild(tr);
          });
          generalTable.appendChild(tbody);

          // Atualiza resumo de 3 métricas do tablet
          const tMetrics = (res.metrics || []).filter(m => !m.header).slice(0, 3);
          const tLabels = document.querySelectorAll('.t-metric-label');
          const tCols = [element('t-metric-mp'), element('t-metric-tp'), element('t-metric-ts')];
          tMetrics.forEach((m, idx) => {
            if (tLabels[idx]) tLabels[idx].textContent = m.label.slice(0, 8);
            if (tCols[idx]) tCols[idx].textContent = `${formatDisplayNumber(m.value)}${m.unit ? ' ' + m.unit : ''}`;
          });

          const tabletRes = element('sci-tablet-results-container');
          if (tabletRes) {
            tabletRes.innerHTML = '';
            const clone = generalTable.cloneNode(true);
            clone.removeAttribute('id');
            clone.querySelectorAll('[id]').forEach(el => el.removeAttribute('id'));
            tabletRes.appendChild(clone);
          }
        }
      }
      updateMetricsTable(result);

      // Auto-alternância para aba Gráfico no celular
      if (typeof setMobileTab === 'function' && window.innerWidth < 600) {
        setMobileTab('plot');
      }

      // 3. TOP 4 PRIMARY METRICS (Métricas de destaque)
      const primaryContainer = element('scientific-primary-metrics');
      const allMetricsContainer = element('scientific-metrics');
      if (primaryContainer) primaryContainer.replaceChildren();
      if (allMetricsContainer) allMetricsContainer.replaceChildren();

      const allMetrics = (result.metrics || []).filter((m) => m && !m.header && m.label);
      const primaryKeys = config?.primaryKeys || [];
      const primaryMetrics = [];
      const secondaryMetrics = [];

      allMetrics.forEach((metric) => {
        const matchesPrimary = metric.label && primaryKeys.some((k) => metric.label.toLowerCase().includes(k.toLowerCase()));
        if (matchesPrimary && primaryMetrics.length < 4) {
          primaryMetrics.push(metric);
        } else {
          secondaryMetrics.push(metric);
        }
      });

      while (primaryMetrics.length < Math.min(4, allMetrics.length) && secondaryMetrics.length > 0) {
        primaryMetrics.push(secondaryMetrics.shift());
      }

      primaryMetrics.forEach((metric, index) => {
        const card = document.createElement('article');
        card.className = `metric-card ${index === 0 ? 'metric-card-accent' : ''}`;
        appendTextNode(card, 'span', 'metric-label', metric.label);
        const valueRow = document.createElement('div');
        valueRow.className = 'metric-value-row';
        const numSpan = document.createElement('strong');
        numSpan.className = 'metric-value code-font';
        numSpan.textContent = formatDisplayNumber(metric.value);
        valueRow.appendChild(numSpan);
        if (metric.unit) {
          const unitSpan = document.createElement('span');
          unitSpan.className = 'metric-unit';
          unitSpan.textContent = metric.unit;
          valueRow.appendChild(unitSpan);
        }
        card.appendChild(valueRow);
        primaryContainer?.appendChild(card);
      });

      (result.metrics || []).forEach((metric) => {
        if (!metric) return;
        if (metric.header) {
          const hdr = document.createElement('div');
          hdr.className = 'submetric-header';
          hdr.textContent = metric.header;
          allMetricsContainer?.appendChild(hdr);
          return;
        }
        if (!metric.label) return;
        const card = document.createElement('div');
        card.className = 'submetric-row';
        appendTextNode(card, 'span', 'submetric-label', metric.label);
        const valText = `${formatDisplayNumber(metric.value)}${metric.unit ? ' ' + metric.unit : ''}`;
        appendTextNode(card, 'span', 'submetric-value code-font', valText);
        allMetricsContainer?.appendChild(card);
      });

      // 4. MEMORIAL DE CÁLCULO (LaTeX)
      const latexGrid = element('scientific-latex');
      if (latexGrid) {
        latexGrid.replaceChildren();
        const latexItems = result.latex || [];
        if (latexItems.length > 0) {
          latexGrid.style.display = 'grid';
          latexItems.forEach((item, idx) => {
            const card = document.createElement('article');
            card.className = `scientific-latex-card ${currentKey === 'controllers' && idx < 2 ? 'ctrl-highlight-latex-card' : ''}`;
            appendTextNode(card, 'span', 'scientific-card-label', item.label || 'Equação');
            const math = document.createElement('div');
            math.className = 'scientific-math';
            card.appendChild(math);
            renderMath(math, item.value || '', true);
            latexGrid.appendChild(card);
          });
          const tabletRes = element('sci-tablet-results-container');
          if (tabletRes) {
            const latexClone = latexGrid.cloneNode(true);
            latexClone.removeAttribute('id');
            latexClone.querySelectorAll('[id]').forEach((el) => el.removeAttribute('id'));
            tabletRes.insertBefore(latexClone, tabletRes.firstChild);
          }
        } else {
          latexGrid.style.display = 'none';
        }
      }

      // 4b. PAINEL EXPANSÍVEL DE MEMÓRIA DE CÁLCULO PASSO A PASSO (Controladores)
      const stepsPanel = element('scientific-steps-panel');
      const stepsBody = element('scientific-steps-body');
      if (stepsPanel && stepsBody) {
        stepsBody.replaceChildren();
        const stepsList = result.details?.steps || [];
        if (Array.isArray(stepsList) && stepsList.length > 0) {
          stepsPanel.style.display = 'block';
          stepsPanel.style.overflow = 'visible';
          stepsPanel.open = true;
          stepsList.forEach((step) => {
            const stepCard = document.createElement('article');
            stepCard.className = 'ctrl-step-card';

            const stepHeader = document.createElement('div');
            stepHeader.className = 'ctrl-step-header';
            appendTextNode(stepHeader, 'span', 'ctrl-step-badge', `Passo ${step.number}`);
            appendTextNode(stepHeader, 'h4', 'ctrl-step-title', step.title || '');
            stepCard.appendChild(stepHeader);

            const stepContent = document.createElement('div');
            stepContent.className = 'ctrl-step-content';

            if (Array.isArray(step.equations) && step.equations.length > 0) {
              step.equations.forEach((eqLatex) => {
                const eqBox = document.createElement('div');
                eqBox.className = 'ctrl-step-eq';
                renderMath(eqBox, eqLatex, true);
                stepContent.appendChild(eqBox);
              });
            }

            if (Array.isArray(step.notes) && step.notes.length > 0) {
              const notesList = document.createElement('ul');
              notesList.className = 'ctrl-step-notes';
              step.notes.forEach((noteText) => {
                appendTextNode(notesList, 'li', 'ctrl-step-note-item', noteText);
              });
              stepContent.appendChild(notesList);
            }

            stepCard.appendChild(stepContent);
            stepsBody.appendChild(stepCard);
          });

          const tabletRes = element('sci-tablet-results-container');
          if (tabletRes) {
            const stepsClone = stepsPanel.cloneNode(true);
            stepsClone.removeAttribute('id');
            stepsClone.querySelectorAll('[id]').forEach((el) => el.removeAttribute('id'));
            tabletRes.appendChild(stepsClone);
          }
        } else {
          stepsPanel.style.display = 'none';
        }
      }

      // 5. DETALHES JSON
      const detailsCard = element('scientific-details-card');
      if (detailsCard) {
        detailsCard.hidden = !result.details;
        if (result.details) element('scientific-details').textContent = JSON.stringify(result.details, null, 2);
      }

      showState('results');
    }

    async function run() {
      const config = MODULES[currentKey];
      if (!config) return;
      if (config.underDevelopment) {
        showToast(`O módulo "${config.title}" está em desenvolvimento.`, 'info', 2800);
        return;
      }
      const meta = MODULE_METAS[currentKey] || { cmd: 'run', solverMeta: 'Pronto' };
      const loadingTextEl = element('scientific-loading-text');
      if (loadingTextEl) {
        loadingTextEl.textContent = meta.loadingText || `Calculando ${config.title}...`;
      }
      setMobileTab('plot');
      const currentRequest = ++requestId;
      const t0 = performance.now();
      showState('loading');
      const statusState = element('sci-status-state');
      if (statusState) statusState.textContent = 'Calculando…';
      try {
        const result = await global.api.runScientificAnalysis(config.action, payloadFromForm(config));
        const dt = ((performance.now() - t0) / 1000).toFixed(3).replace('.', ',');
        if (currentRequest !== requestId) return;
        if (!result?.success) throw new Error(result?.error || 'Não foi possível concluir a análise.');
        if (moduleStates[currentKey]) {
          moduleStates[currentKey].result = result;
        }
        renderResult(result);
        const meta = MODULE_METAS[currentKey] || { cmd: 'run', solverMeta: 'Pronto' };
        const logEl = element('sci-console-text');
        if (logEl) {
          let cmdText;
          if (currentKey === 'state-space') {
            const modeInput = form.elements.namedItem('mode');
            if (!modeInput || modeInput.value === 'matrices') {
              cmdText = `>> ss(A, B, C, D) — concluído em ${dt} s`;
            } else {
              const exprInput = getExpressionInput();
              const rawExpr = exprInput?.value?.trim() || config.filename;
              cmdText = `>> tf2ss(${rawExpr}) — concluído em ${dt} s`;
            }
          } else {
            const exprInput = getExpressionInput();
            const rawExpr = exprInput?.value?.trim() || config.filename;
            cmdText = `>> ${meta.cmd}(${rawExpr}) — concluído em ${dt} s`;
          }
          logEl.textContent = cmdText;
          if (moduleStates[currentKey]) {
            moduleStates[currentKey].consoleText = cmdText;
          }
        }
        if (statusState) statusState.textContent = 'Pronto';
        const metaEl = element('sci-status-meta');
        if (metaEl) metaEl.textContent = meta.solverMeta;
        if (typeof window.addRecentItem === 'function') {
          const exprInput = getExpressionInput();
          const exprStr = exprInput?.value?.trim() || config.title;
          window.addRecentItem({
            name: exprStr,
            expr: exprStr,
            nav: currentKey,
            type: 'math',
            timestamp: Date.now(),
          });
        }
        showToast(`${config.title} concluída.`, 'success');
      } catch (error) {
        if (currentRequest !== requestId) return;
        element('scientific-error').textContent = error?.message || String(error);
        showState('error');
        if (statusState) statusState.textContent = 'Pronto';
        showToast(`Erro: ${error?.message || error}`, 'error', 4500);
      }
    }

    form.addEventListener('submit', (event) => {
      event.preventDefault();
      run();
    });
    form.addEventListener('change', updateConditionalFields);
    form.addEventListener('input', (event) => {
      const targetName = event.target.name;
      if (targetName === 'expr' || targetName === 'plant_expr') {
        scheduleScientificPreview();
      } else if (currentKey === 'controllers' && ['ts', 'mp', 'xi', 'wn'].includes(targetName)) {
        recomputeControllerTransients(targetName);
      }
    });
    global.addEventListener('keydown', (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 'Enter'
        && element('page-scientific')?.classList.contains('active')) {
        event.preventDefault();
        run();
      }
    });

    // 6. Controles da Interface Científica (Ribbon, Mobile Tabs, Tablet Switcher, Símbolos)
    const mobileTabs = document.querySelectorAll('#sci-mobile-tabs .sci-tab-btn');
    function setMobileTab(tabKey) {
      mobileTabs.forEach(b => b.classList.toggle('active', b.dataset.tab === tabKey));
      const sciShell = document.querySelector('#page-scientific .sci-shell');
      if (sciShell) sciShell.dataset.activeTab = tabKey;
    }
    mobileTabs.forEach(b => {
      b.addEventListener('click', () => setMobileTab(b.dataset.tab));
    });
    setMobileTab('params');

    element('btn-mobile-calc')?.addEventListener('click', (e) => {
      e.preventDefault();
      run();
    });

    const tabSwitchBtns = document.querySelectorAll('#sci-tablet-switcher button');
    const tabletResultsContainer = element('sci-tablet-results-container');
    const plotViewport = element('scientific-plot-viewport');
    const tabletMetricsBar = element('tablet-metrics-bar');
    function setTabletView(view) {
      tabSwitchBtns.forEach(b => b.classList.toggle('active', b.dataset.view === view));
      const isResults = view === 'results';
      if (tabletResultsContainer) tabletResultsContainer.style.display = isResults ? 'block' : 'none';
      if (plotViewport) plotViewport.style.display = isResults ? 'none' : 'flex';
      if (tabletMetricsBar) tabletMetricsBar.style.display = isResults ? 'none' : 'grid';
    }
    tabSwitchBtns.forEach(b => {
      b.addEventListener('click', () => setTabletView(b.dataset.view));
    });

    // Barra de símbolos virtuais no celular
    document.querySelectorAll('#sci-virtual-math button').forEach(b => {
      b.addEventListener('click', (e) => {
        e.preventDefault();
        const sym = b.dataset.sym;
        const exprInput = getExpressionInput();
        if (!exprInput || !sym) return;
        const start = exprInput.selectionStart ?? exprInput.value.length;
        const end = exprInput.selectionEnd ?? exprInput.value.length;
        const val = exprInput.value;
        exprInput.value = val.slice(0, start) + sym + val.slice(end);
        const nextPos = start + sym.length;
        exprInput.selectionStart = nextPos;
        exprInput.selectionEnd = nextPos;
        exprInput.focus();
        exprInput.dispatchEvent(new Event('input', { bubbles: true }));
      });
    });

    // Ações do Ribbon
    element('ribbon-btn-calc')?.addEventListener('click', run);
    element('ribbon-btn-clear')?.addEventListener('click', () => {
      const exprInput = getExpressionInput();
      if (exprInput) {
        exprInput.value = '';
        exprInput.focus();
        exprInput.dispatchEvent(new Event('input', { bubbles: true }));
      }
      singleZoomCtrl?.resetZoom();
      stackedZoomCtrls.forEach(c => c.resetZoom());
    });
    element('ribbon-btn-zoom-in')?.addEventListener('click', () => {
      getActiveZoomController()?.zoomIn();
    });
    element('ribbon-btn-zoom-out')?.addEventListener('click', () => {
      getActiveZoomController()?.zoomOut();
    });
    element('ribbon-btn-fit')?.addEventListener('click', () => {
      singleZoomCtrl?.resetZoom();
      stackedZoomCtrls.forEach(c => c.resetZoom());
    });
    element('ribbon-btn-grid')?.addEventListener('click', () => {
      showToast('Grade ativada na figura.', 'info', 2000);
    });
    element('ribbon-btn-export')?.addEventListener('click', () => {
      element('btn-scientific-png')?.click();
    });
    element('ribbon-btn-code')?.addEventListener('click', async () => {
      const exprInput = getExpressionInput();
      const exprStr = exprInput?.value?.trim() || '4 / (s^2 + 2*s + 4)';
      const pyCode = `# ControLAB v4.2.2 - Script de Simulação\nimport control as ct\nimport matplotlib.pyplot as plt\n\ns = ct.tf('s')\nG = ${exprStr.replace(/\^/g, '**')}\nT = ct.feedback(G, 1)\nt, y = ct.step_response(T, T=10)\n\nplt.figure(figsize=(8, 4.5), dpi=120)\nplt.plot(t, y, label='Degrau unitário')\nplt.grid(True)\nplt.title('Resposta ao Degrau - ControLAB')\nplt.xlabel('Tempo (s)')\nplt.ylabel('Amplitude')\nplt.legend()\nplt.show()\n`;
      try {
        if (navigator.clipboard) {
          await navigator.clipboard.writeText(pyCode);
          showToast('Código Python copiado para a Área de Transferência!', 'success');
        } else {
          showToast('Código gerado com sucesso.', 'info');
        }
      } catch (e) {
        showToast('Código gerado com sucesso.', 'info');
      }
    });

    // Menu "⋮" e "Mais"
    const menuBtn = element('btn-sci-menu');
    const moreBtn = element('btn-mobile-more');
    const menuDropdown = element('sci-more-dropdown');
    function toggleMenu(e) {
      e?.stopPropagation();
      if (!menuDropdown) return;
      const isHidden = menuDropdown.hasAttribute('hidden');
      if (isHidden) {
        menuDropdown.removeAttribute('hidden');
      } else {
        menuDropdown.setAttribute('hidden', '');
      }
    }
    menuBtn?.addEventListener('click', toggleMenu);
    moreBtn?.addEventListener('click', toggleMenu);
    document.addEventListener('click', (e) => {
      if (menuDropdown && !menuDropdown.contains(e.target) && e.target !== menuBtn && e.target !== moreBtn) {
        menuDropdown.setAttribute('hidden', '');
      }
    });
    element('menu-btn-export-png')?.addEventListener('click', () => {
      menuDropdown?.setAttribute('hidden', '');
      element('btn-scientific-png')?.click();
    });
    element('menu-btn-export-svg')?.addEventListener('click', () => {
      menuDropdown?.setAttribute('hidden', '');
      element('btn-scientific-svg')?.click();
    });
    element('menu-btn-copy-code')?.addEventListener('click', () => {
      menuDropdown?.setAttribute('hidden', '');
      element('ribbon-btn-code')?.click();
    });
    element('menu-btn-settings')?.addEventListener('click', () => {
      menuDropdown?.setAttribute('hidden', '');
      global.ControLABSettings?.open?.();
    });

    document.querySelectorAll('.sci-settings-btn').forEach(btn => {
      btn.addEventListener('click', () => global.ControLABSettings?.open?.());
    });

    element('btn-scientific-copy')?.addEventListener('click', async () => {
      if (!currentResult) return;
      const activeImg = currentResult.plots?.[currentResponseType]?.image || currentResult.image;
      if (!activeImg) return;
      try {
        const res = await (global.api?.copyImageToClipboard ? global.api.copyImageToClipboard(activeImg) : Promise.reject(new Error('API indisponível')));
        if (res && res.success !== false) {
          showToast('Gráfico copiado para a Área de Transferência!', 'success');
        } else if (res?.error) {
          showToast(`Não foi possível copiar: ${res.error}`, 'error', 4000);
        }
      } catch (err) {
        showToast(`Erro ao copiar: ${err.message}`, 'error', 3500);
      }
    });

    element('btn-scientific-png')?.addEventListener('click', async () => {
      if (!currentResult) return;
      const activeImg = currentResult.plots?.[currentResponseType]?.image || currentResult.image;
      if (!activeImg) return;
      try {
        const res = await global.api.saveImage({
          base64: activeImg,
          defaultName: `${MODULES[currentKey].filename}.png`
        });
        if (res && res.success !== false) {
          showToast(res.message || 'Imagem PNG salva com sucesso!', 'success');
        }
      } catch (err) {
        showToast(`Erro ao salvar: ${err.message}`, 'error', 3500);
      }
    });

    element('btn-scientific-svg')?.addEventListener('click', async () => {
      if (!currentResult) return;
      const activeSvg = currentResult.plots?.[currentResponseType]?.svg || currentResult.svg;
      if (!activeSvg) return;
      try {
        const res = await global.api.saveSVG({
          svg: activeSvg,
          defaultName: `${MODULES[currentKey].filename}.svg`
        });
        if (res && res.success !== false) {
          showToast(res.message || 'Gráfico Vetorial SVG salvo com sucesso!', 'success');
        }
      } catch (err) {
        showToast(`Erro ao salvar SVG: ${err.message}`, 'error', 3500);
      }
    });

    const themeObserver = new MutationObserver(() => {
      if (currentResult && element('page-scientific')?.classList.contains('active')) {
        recalculate();
      }
    });
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });

    function setupScientificSplitPanes() {
      const workspaces = document.querySelectorAll('.scientific-workspace');
      if (!workspaces.length) return;

      workspaces.forEach((ws) => {
        const splitLeft = ws.querySelector('.sci-split-handle[data-split="left"]');
        const splitRight = ws.querySelector('.sci-split-handle[data-split="right"]');
        const paramsPanel = ws.querySelector('.sci-panel-params');
        const resultsPanel = ws.querySelector('.sci-panel-results');

        if (!splitLeft || !splitRight || !paramsPanel || !resultsPanel) return;

        try {
          if (window.innerWidth >= 1024) {
            const savedParams = parseInt(localStorage.getItem('controlab_sci_params_width'), 10);
            if (savedParams && savedParams >= 200 && savedParams <= 480) {
              paramsPanel.style.width = `${savedParams}px`;
              paramsPanel.style.flex = `0 0 ${savedParams}px`;
            }
            const savedResults = parseInt(localStorage.getItem('controlab_sci_results_width'), 10);
            if (savedResults && savedResults >= 200 && savedResults <= 560) {
              resultsPanel.style.width = `${savedResults}px`;
              resultsPanel.style.flex = `0 0 ${savedResults}px`;
            } else {
              // Padrão de primeira execução: tamanho máximo/amplo (480px)
              resultsPanel.style.width = '480px';
              resultsPanel.style.flex = '0 0 480px';
            }
          }
        } catch {}

        // Splitter Esquerdo (Params <-> Plot)
        let isDraggingLeft = false;

        function onPointerMoveLeft(e) {
          if (!isDraggingLeft) return;
          const wsRect = ws.getBoundingClientRect();
          const resultsW = resultsPanel.getBoundingClientRect().width;
          const maxParamsW = Math.max(200, Math.floor(wsRect.width - resultsW - 300));
          const desiredW = e.clientX - wsRect.left;
          const clampedW = Math.max(200, Math.min(Math.min(480, maxParamsW), desiredW));
          paramsPanel.style.width = `${Math.round(clampedW)}px`;
          paramsPanel.style.flex = `0 0 ${Math.round(clampedW)}px`;
        }

        function onPointerUpLeft() {
          if (!isDraggingLeft) return;
          isDraggingLeft = false;
          splitLeft.classList.remove('is-dragging');
          document.body.classList.remove('is-resizing-splitter-v');
          window.removeEventListener('pointermove', onPointerMoveLeft);
          window.removeEventListener('pointerup', onPointerUpLeft);
          window.removeEventListener('pointercancel', onPointerUpLeft);
          try {
            const finalW = parseInt(paramsPanel.style.width, 10);
            if (finalW) localStorage.setItem('controlab_sci_params_width', String(finalW));
          } catch {}
        }

        splitLeft.addEventListener('pointerdown', (e) => {
          if (window.innerWidth < 1024 || e.button !== 0) return;
          isDraggingLeft = true;
          splitLeft.classList.add('is-dragging');
          document.body.classList.add('is-resizing-splitter-v');
          window.addEventListener('pointermove', onPointerMoveLeft);
          window.addEventListener('pointerup', onPointerUpLeft);
          window.addEventListener('pointercancel', onPointerUpLeft);
          e.preventDefault();
        });

        splitLeft.addEventListener('dblclick', () => {
          if (window.innerWidth < 1024) return;
          paramsPanel.style.width = '';
          paramsPanel.style.flex = '';
          try { localStorage.removeItem('controlab_sci_params_width'); } catch {}
        });

        // Splitter Direito (Plot <-> Results)
        let isDraggingRight = false;

        function onPointerMoveRight(e) {
          if (!isDraggingRight) return;
          const wsRect = ws.getBoundingClientRect();
          const paramsW = paramsPanel.getBoundingClientRect().width;
          const maxResultsW = Math.max(200, Math.floor(wsRect.width - paramsW - 300));
          const desiredW = wsRect.right - e.clientX;
          const clampedW = Math.max(200, Math.min(Math.min(560, maxResultsW), desiredW));
          resultsPanel.style.width = `${Math.round(clampedW)}px`;
          resultsPanel.style.flex = `0 0 ${Math.round(clampedW)}px`;
        }

        function onPointerUpRight() {
          if (!isDraggingRight) return;
          isDraggingRight = false;
          splitRight.classList.remove('is-dragging');
          document.body.classList.remove('is-resizing-splitter-v');
          window.removeEventListener('pointermove', onPointerMoveRight);
          window.removeEventListener('pointerup', onPointerUpRight);
          window.removeEventListener('pointercancel', onPointerUpRight);
          try {
            const finalW = parseInt(resultsPanel.style.width, 10);
            if (finalW) localStorage.setItem('controlab_sci_results_width', String(finalW));
          } catch {}
        }

        splitRight.addEventListener('pointerdown', (e) => {
          if (window.innerWidth < 1024 || e.button !== 0) return;
          isDraggingRight = true;
          splitRight.classList.add('is-dragging');
          document.body.classList.add('is-resizing-splitter-v');
          window.addEventListener('pointermove', onPointerMoveRight);
          window.addEventListener('pointerup', onPointerUpRight);
          window.addEventListener('pointercancel', onPointerUpRight);
          e.preventDefault();
        });

        splitRight.addEventListener('dblclick', () => {
          if (window.innerWidth < 1024) return;
          resultsPanel.style.width = '480px';
          resultsPanel.style.flex = '0 0 480px';
          try { localStorage.removeItem('controlab_sci_results_width'); } catch {}
        });
      });

      window.addEventListener('resize', () => {
        if (window.innerWidth < 1024) {
          workspaces.forEach((ws) => {
            const p = ws.querySelector('.sci-panel-params');
            const r = ws.querySelector('.sci-panel-results');
            if (p) { p.style.width = ''; p.style.flex = ''; }
            if (r) { r.style.width = ''; r.style.flex = ''; }
          });
        }
      });
    }

    setupScientificSplitPanes();

    function recalculate() {
      if (currentResult && element('page-scientific')?.classList.contains('active')) {
        run();
      }
    }

    return Object.freeze({ open, run, recalculate });
  }

  global.ControLABScientific = Object.freeze({ initialize, modules: MODULES });
}(window));
