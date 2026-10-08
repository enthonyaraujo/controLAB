(function registerScientificModules(global) {
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
      examples: [
        ['Segunda ordem', '4 / (s^2 + 2*s + 4)'],
        ['Primeira ordem', '1 / (2*s + 1)'],
        ['Tipo 1', '5 / (s * (s + 2))'],
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
      examples: [
        ['Primeira ordem', '1 / (s + 1)'],
        ['Segunda ordem', '25 / (s^2 + 4*s + 25)'],
        ['Tipo 1', '10 / (s * (s + 2) * (s + 5))'],
      ],
    },
    controllers: {
      action: 'controller_design',
      title: 'Projeto de Controladores',
      kicker: 'PID & Compensadores Lead-Lag',
      description: 'Sintonize controladores clássicos ou aplique compensação de fase e compare a dinâmica antes e depois.',
      filename: 'projeto-de-controlador',
      primaryKeys: ['Tipo de projeto', 'Método', 'Estrutura', 'Estabilidade'],
      fields: [
        {
          name: 'plant_expr',
          label: 'Planta G(s)',
          type: 'text',
          value: '1 / (s * (s + 1) * (s + 5))',
          placeholder: 'Ex.: 1 / (s * (s + 1) * (s + 5))',
        },
        {
          name: 'design_type',
          label: 'Tipo de projeto',
          type: 'select',
          value: 'pid',
          options: [['pid', 'Sintonia P / PI / PID'], ['lead_lag', 'Compensador avanço / atraso']],
        },
        {
          name: 'method',
          label: 'Método de sintonia',
          type: 'select',
          value: 'zn_critical',
          options: [
            ['zn_critical', 'Ziegler-Nichols — oscilação crítica'],
            ['zn_reaction', 'Ziegler-Nichols — curva de reação'],
            ['cohen_coon', 'Cohen-Coon'],
            ['chr', 'CHR'],
          ],
          when: { design_type: ['pid'] },
        },
        {
          name: 'controller_type',
          label: 'Estrutura do controlador',
          type: 'select',
          value: 'PID',
          options: [['P', 'P'], ['PI', 'PI'], ['PID', 'PID']],
          when: { design_type: ['pid'] },
        },
        {
          name: 'critical_gain', label: 'Ganho crítico Kcr', type: 'number', value: '6', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_critical'] },
        },
        {
          name: 'critical_period', label: 'Período crítico Pcr (s)', type: 'number', value: '2', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_critical'] },
        },
        {
          name: 'process_gain', label: 'Ganho do processo K', type: 'number', value: '1', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_reaction', 'cohen_coon', 'chr'] },
        },
        {
          name: 'delay', label: 'Atraso aparente L (s)', type: 'number', value: '0.5', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_reaction', 'cohen_coon', 'chr'] },
        },
        {
          name: 'time_constant', label: 'Constante de tempo T (s)', type: 'number', value: '4', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_reaction', 'cohen_coon', 'chr'] },
        },
        {
          name: 'chr_response', label: 'Configuração CHR', type: 'select', value: '0',
          options: [['0', '0% de sobressinal'], ['20', '20% de sobressinal']],
          when: { design_type: ['pid'], method: ['chr'] },
        },
        {
          name: 'compensator_type', label: 'Tipo de compensador', type: 'select', value: 'lead',
          options: [['lead', 'Avanço de fase (lead)'], ['lag', 'Atraso de fase (lag)']],
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'design_domain', label: 'Domínio de referência', type: 'select', value: 'frequency',
          options: [['frequency', 'Frequência / Bode'], ['root_locus', 'Plano s / LGR']],
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'compensator_zero', label: 'Frequência do zero', type: 'number', value: '1', min: '0.000001',
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'compensator_pole', label: 'Frequência do polo', type: 'number', value: '5', min: '0.000001',
          help: 'No avanço, polo > zero. No atraso, polo < zero.',
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'compensator_gain', label: 'Ganho do compensador', type: 'number', value: '1', min: '0.000001',
          when: { design_type: ['lead_lag'] },
        },
        { name: 'final_time', label: 'Tempo final (s)', type: 'number', value: '20', min: '0.05' },
      ],
      examples: [
        ['Terceira ordem', '1 / (s * (s + 1) * (s + 5))'],
        ['Primeira ordem', '1 / (4*s + 1)'],
        ['Segunda ordem', '1 / (s^2 + 3*s + 2)'],
      ],
    },
    'state-space': {
      action: 'state_space',
      title: 'Espaço de Estados',
      kicker: 'Modelagem Matricial Moderna',
      description: 'Converta modelos, verifique controlabilidade e observabilidade e projete realimentação e observadores.',
      filename: 'espaco-de-estados',
      primaryKeys: ['Posto de controlabilidade', 'Controlável', 'Posto de observabilidade', 'Observável'],
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
            previewStatus.textContent = 'Expressão válida';
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
            previewStatus.textContent = 'Expressão válida';
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

    function renderFields(config) {
      if (previewContainer && form.parentElement) {
        previewContainer.style.display = 'none';
        form.parentElement.appendChild(previewContainer);
      }
      form.replaceChildren();
      config.fields.forEach((field) => {
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
            hiddenInput.value = field.value || field.options[0][0];
            
            field.options.forEach(([val, label]) => {
              const pill = document.createElement('button');
              pill.type = 'button';
              pill.className = `tab-seg ${val === hiddenInput.value ? 'active' : ''}`;
              pill.textContent = label;
              pill.addEventListener('click', () => {
                hiddenInput.value = val;
                segContainer.querySelectorAll('.tab-seg').forEach(p => p.classList.remove('active'));
                pill.classList.add('active');
                updateConditionalFields();
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
            option.selected = value === field.value;
            input.appendChild(option);
          });
        } else if (field.type === 'textarea') {
          input = document.createElement('textarea');
          input.rows = field.rows || 2;
          input.value = field.value || '';
        } else {
          input = document.createElement('input');
          input.type = field.type || 'text';
          input.value = field.value || '';
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

    function open(moduleKey) {
      const config = MODULES[moduleKey];
      if (!config) return false;
      if (currentKey !== moduleKey) {
        currentKey = moduleKey;
        lastPreviewExpr = null;
        const kickerEl = element('scientific-kicker');
        if (kickerEl) kickerEl.textContent = config.kicker;
        const titleEl = element('scientific-title');
        if (titleEl) titleEl.textContent = config.title;
        const descEl = element('scientific-description');
        if (descEl) descEl.textContent = config.description;
        renderFields(config);
        showState('empty');
      }
      return true;
    }

    function showState(state) {
      const loadingEl = element('scientific-loading');
      if (loadingEl) loadingEl.hidden = state !== 'loading';
      const emptyEl = element('scientific-empty');
      if (emptyEl) emptyEl.hidden = state !== 'empty';
      const errorEl = element('scientific-error');
      if (errorEl) errorEl.hidden = state !== 'error';
      const resultsEl = element('scientific-results');
      if (resultsEl) resultsEl.hidden = state !== 'results';
      runButton.disabled = state === 'loading';
    }

    function payloadFromForm(config) {
      const payload = {
        theme: document.documentElement.getAttribute('data-theme') || 'dark',
        response_type: currentResponseType,
      };
      config.fields.forEach((field) => {
        const input = form.elements.namedItem(field.name);
        if (!input) return;
        if (field.type === 'number') {
          if (input.value !== '') payload[field.name] = Number(input.value);
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

      // 1. STATUS CHIP (Estável / Instável)
      const statusChip = element('scientific-status-chip');
      const statusText = element('scientific-status-text');
      const isStable = result.details?.stable ?? (result.metrics?.find(m => m.label === 'Estabilidade')?.value === 'Estável');
      if (statusChip && statusText) {
        statusChip.className = `status-chip ${isStable ? 'status-stable' : 'status-unstable'}`;
        statusText.textContent = isStable ? 'Estável' : 'Instável';
        statusChip.style.display = 'inline-flex';
      }

      // 2. PLOT & TABS (Degrau / Rampa / Impulso para Resposta no Tempo)
      const plotCard = element('scientific-plot-card');
      const plot = element('scientific-plot');
      const plotTabs = element('scientific-plot-tabs');
      plotCard.hidden = !result.image;

      if (result.plots && currentKey === 'time') {
        if (plotTabs) {
          plotTabs.style.display = 'inline-flex';
          plotTabs.querySelectorAll('.tab-seg').forEach((tab) => {
            const resp = tab.dataset.resp;
            tab.classList.toggle('active', resp === currentResponseType);
            tab.onclick = () => {
              currentResponseType = resp;
              plotTabs.querySelectorAll('.tab-seg').forEach(t => t.classList.remove('active'));
              tab.classList.add('active');
              if (result.plots[resp]) {
                plot.src = result.plots[resp].image;
              }
            };
          });
        }
        plot.src = result.plots[currentResponseType]?.image || result.image;
      } else {
        if (plotTabs) plotTabs.style.display = 'none';
        if (result.image) plot.src = result.image;
      }

      element('btn-scientific-png').hidden = !result.image;
      element('btn-scientific-svg').hidden = !result.svg;

      // 3. TOP 4 PRIMARY METRICS (Métricas de destaque)
      const primaryContainer = element('scientific-primary-metrics');
      const allMetricsContainer = element('scientific-metrics');
      if (primaryContainer) primaryContainer.replaceChildren();
      if (allMetricsContainer) allMetricsContainer.replaceChildren();

      const allMetrics = result.metrics || [];
      const primaryKeys = config?.primaryKeys || [];
      const primaryMetrics = [];
      const secondaryMetrics = [];

      allMetrics.forEach((metric) => {
        const matchesPrimary = primaryKeys.some((k) => metric.label.toLowerCase().includes(k.toLowerCase()));
        if (matchesPrimary && primaryMetrics.length < 4) {
          primaryMetrics.push(metric);
        } else {
          secondaryMetrics.push(metric);
        }
      });

      // Se não atingiu 4, completa com as primeiras secundárias
      while (primaryMetrics.length < Math.min(4, allMetrics.length) && secondaryMetrics.length > 0) {
        primaryMetrics.push(secondaryMetrics.shift());
      }

      // Renderiza os 4 cards principais de métricas
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

      // Renderiza o restante em "Todas as métricas"
      allMetrics.forEach((metric) => {
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
        (result.latex || []).forEach((item) => {
          const card = document.createElement('article');
          card.className = 'scientific-latex-card';
          appendTextNode(card, 'span', 'scientific-card-label', item.label || 'Equação');
          const math = document.createElement('div');
          math.className = 'scientific-math';
          card.appendChild(math);
          renderMath(math, item.value || '', true);
          latexGrid.appendChild(card);
        });
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
      const currentRequest = ++requestId;
      showState('loading');
      try {
        const result = await global.api.runScientificAnalysis(config.action, payloadFromForm(config));
        if (currentRequest !== requestId) return;
        if (!result?.success) throw new Error(result?.error || 'Não foi possível concluir a análise.');
        renderResult(result);
        showToast(`${config.title} concluída.`, 'success');
      } catch (error) {
        if (currentRequest !== requestId) return;
        element('scientific-error').textContent = error?.message || String(error);
        showState('error');
        showToast(`Erro: ${error?.message || error}`, 'error', 4500);
      }
    }

    form.addEventListener('submit', (event) => {
      event.preventDefault();
      run();
    });
    form.addEventListener('change', updateConditionalFields);
    form.addEventListener('input', (event) => {
      if (event.target.name === 'expr' || event.target.name === 'plant_expr') {
        scheduleScientificPreview();
      }
    });
    global.addEventListener('keydown', (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 'Enter'
        && element('page-scientific')?.classList.contains('active')) {
        event.preventDefault();
        run();
      }
    });

    element('btn-scientific-png')?.addEventListener('click', async () => {
      if (!currentResult) return;
      const activeImg = currentResult.plots?.[currentResponseType]?.image || currentResult.image;
      if (!activeImg) return;
      await global.api.saveImage({ base64: activeImg, defaultName: `${MODULES[currentKey].filename}.png` });
    });
    element('btn-scientific-svg')?.addEventListener('click', async () => {
      if (!currentResult) return;
      const activeSvg = currentResult.plots?.[currentResponseType]?.svg || currentResult.svg;
      if (!activeSvg) return;
      await global.api.saveSVG({ svg: activeSvg, defaultName: `${MODULES[currentKey].filename}.svg` });
    });

    function recalculate() {
      if (currentResult && element('page-scientific')?.classList.contains('active')) {
        run();
      }
    }

    return Object.freeze({ open, run, recalculate });
  }

  global.ControLABScientific = Object.freeze({ initialize, modules: MODULES });
}(window));
