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
            ['zn_critical', 'Ziegler-Nichols — oscilação crítica (automático)'],
            ['zn_critical_manual', 'Ziegler-Nichols — oscilação crítica (manual)'],
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
          name: 'critical_gain', label: 'Ganho crítico Kcr', type: 'number', value: '30', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_critical_manual'] },
        },
        {
          name: 'critical_period', label: 'Período crítico Pcr (s)', type: 'number', value: '2.81', min: '0.000001',
          when: { design_type: ['pid'], method: ['zn_critical_manual'] },
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

    const moduleStates = {
      time: { result: null, formData: null, responseType: 'step', consoleText: '>> step(G, 10)' },
      frequency: { result: null, formData: null, responseType: 'step', consoleText: '>> bode(G)' },
      controllers: { result: null, formData: null, responseType: 'step', consoleText: '>> pid(G)' },
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

    function renderFields(config, savedData = null) {
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

        const initialVal = (savedData && savedData[field.name] !== undefined) ? savedData[field.name] : field.value;

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
            option.title = label;
            option.selected = String(value) === String(initialVal);
            input.appendChild(option);
          });
          const updateTooltip = () => {
            const opt = input.options[input.selectedIndex];
            if (opt) input.title = opt.textContent;
          };
          updateTooltip();
          input.addEventListener('change', updateTooltip);
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

    const MODULE_METAS = {
      time: {
        figureTitle: 'Figura 1 — Resposta temporal',
        hasResponseTabs: true,
        defaultConsole: '>> step(G, 10)',
        solverMeta: 'Módulo: Resposta no Tempo • Solver: scipy',
        cmd: 'step',
      },
      frequency: {
        figureTitle: 'Figura 1 — Diagrama de Bode',
        hasResponseTabs: false,
        defaultConsole: '>> bode(G)',
        solverMeta: 'Módulo: Resposta em Frequência • Solver: scipy / python-control',
        cmd: 'bode',
      },
      controllers: {
        figureTitle: 'Figura 1 — Desempenho e LGR do controlador',
        hasResponseTabs: false,
        defaultConsole: '>> pid(G)',
        solverMeta: 'Módulo: Projeto de Controladores • Solver: python-control',
        cmd: 'pid',
      },
      'state-space': {
        figureTitle: 'Figura 1 — Plano complexo e polos de estados',
        hasResponseTabs: false,
        defaultConsole: '>> ss(A, B, C, D)',
        solverMeta: 'Módulo: Espaço de Estados • Solver: scipy / sympy',
        cmd: 'ss',
      },
    };

    function resetResults(moduleKey) {
      const timeTable = element('time-results-table');
      let generalTable = element('scientific-general-results-table');

      const timeKeys = ['res-val-mp', 'res-val-tp', 'res-val-tr', 'res-val-ts', 'res-val-yf', 'res-val-zeta', 'res-val-wn', 'res-val-stab'];
      timeKeys.forEach(id => {
        document.querySelectorAll('#' + id).forEach(el => { el.textContent = '—'; });
      });
      const tMetrics = ['t-metric-mp', 't-metric-tp', 't-metric-ts'];
      tMetrics.forEach(id => {
        document.querySelectorAll('#' + id).forEach(el => { el.textContent = '—'; });
      });

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

      const primaryContainer = element('scientific-primary-metrics');
      if (primaryContainer) primaryContainer.replaceChildren();
      const allMetricsContainer = element('scientific-metrics');
      if (allMetricsContainer) allMetricsContainer.replaceChildren();
      const latexGrid = element('scientific-latex');
      if (latexGrid) latexGrid.replaceChildren();
      const detailsCard = element('scientific-details-card');
      if (detailsCard) detailsCard.hidden = true;

      const tabletRes = element('sci-tablet-results-container');
      if (tabletRes) {
        tabletRes.innerHTML = '';
        const activeTable = moduleKey === 'time' ? timeTable : generalTable;
        if (activeTable) {
          const clone = activeTable.cloneNode(true);
          clone.querySelectorAll('[id]').forEach(el => el.removeAttribute('id'));
          tabletRes.appendChild(clone);
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

      const statusState = element('sci-status-state');
      if (statusState) statusState.textContent = 'Pronto';

      const metaEl = element('sci-status-meta');
      if (metaEl) metaEl.textContent = meta.solverMeta;

      if (modState.result) {
        renderResult(modState.result);
      } else {
        const statusChip = element('scientific-status-chip');
        if (statusChip) statusChip.style.display = 'none';

        const plot = element('scientific-plot');
        if (plot) plot.src = '';

        showState('empty');
        resetResults(moduleKey);
      }

      return true;
    }

    function showState(state) {
      const loadingEl = element('scientific-loading');
      if (loadingEl) loadingEl.hidden = state !== 'loading';
      const emptyEl = element('scientific-empty');
      if (emptyEl) emptyEl.style.display = state === 'empty' ? 'flex' : 'none';
      const errorEl = element('scientific-error');
      if (errorEl) errorEl.hidden = state !== 'error';
      const resultsEl = element('scientific-results');
      if (resultsEl) resultsEl.hidden = state !== 'results';
      runButton.disabled = state === 'loading';
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
      if (plotCard) plotCard.hidden = !result.image;

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
              const figTitleEl = element('sci-figure-title');
              if (figTitleEl) {
                const titles = { step: 'Figura 1 — Resposta ao degrau', ramp: 'Figura 1 — Resposta à rampa', impulse: 'Figura 1 — Resposta ao impulso' };
                figTitleEl.textContent = titles[resp] || 'Figura 1';
              }
            };
          });
        }
        plot.src = result.plots[currentResponseType]?.image || result.image;
        const figTitleEl = element('sci-figure-title');
        if (figTitleEl) {
          const titles = { step: 'Figura 1 — Resposta ao degrau', ramp: 'Figura 1 — Resposta à rampa', impulse: 'Figura 1 — Resposta ao impulso' };
          figTitleEl.textContent = titles[currentResponseType] || 'Figura 1';
        }
      } else {
        if (plotTabs) plotTabs.style.display = 'none';
        if (result.image) plot.src = result.image;
      }

      if (element('btn-scientific-copy')) element('btn-scientific-copy').hidden = !result.image;
      if (element('btn-scientific-png')) element('btn-scientific-png').hidden = !result.image;
      if (element('btn-scientific-svg')) element('btn-scientific-svg').hidden = !result.svg;

      // Atualiza Tabela de Resultados
      function updateMetricsTable(res) {
        if (!res) return;
        const timeTable = element('time-results-table');
        let generalTable = element('scientific-general-results-table');

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
            tdL.textContent = m.label;
            const tdV = document.createElement('td');
            tdV.textContent = `${formatDisplayNumber(m.value)}${m.unit ? ' ' + m.unit : ''}`;
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
    let currentZoom = 1.0;
    const plotImgEl = element('scientific-plot');
    function applyZoom(z) {
      currentZoom = Math.max(0.6, Math.min(2.5, z));
      if (plotImgEl) {
        plotImgEl.style.transform = `scale(${currentZoom})`;
        plotImgEl.style.transformOrigin = 'center center';
        plotImgEl.style.transition = 'transform 120ms ease';
      }
    }
    element('ribbon-btn-calc')?.addEventListener('click', run);
    element('ribbon-btn-clear')?.addEventListener('click', () => {
      const exprInput = getExpressionInput();
      if (exprInput) {
        exprInput.value = '';
        exprInput.focus();
        exprInput.dispatchEvent(new Event('input', { bubbles: true }));
      }
    });
    element('ribbon-btn-zoom-in')?.addEventListener('click', () => applyZoom(currentZoom + 0.2));
    element('ribbon-btn-zoom-out')?.addEventListener('click', () => applyZoom(currentZoom - 0.2));
    element('ribbon-btn-fit')?.addEventListener('click', () => applyZoom(1.0));
    element('ribbon-btn-grid')?.addEventListener('click', () => {
      showToast('Grade ativada na figura.', 'info', 2000);
    });
    element('ribbon-btn-export')?.addEventListener('click', () => {
      element('btn-scientific-png')?.click();
    });
    element('ribbon-btn-code')?.addEventListener('click', async () => {
      const exprInput = getExpressionInput();
      const exprStr = exprInput?.value?.trim() || '4 / (s^2 + 2*s + 4)';
      const pyCode = `# ControLAB v3.1.1 - Script de Simulação\nimport control as ct\nimport matplotlib.pyplot as plt\n\ns = ct.tf('s')\nG = ${exprStr.replace(/\^/g, '**')}\nT = ct.feedback(G, 1)\nt, y = ct.step_response(T, T=10)\n\nplt.figure(figsize=(8, 4.5), dpi=120)\nplt.plot(t, y, label='Degrau unitário')\nplt.grid(True)\nplt.title('Resposta ao Degrau - ControLAB')\nplt.xlabel('Tempo (s)')\nplt.ylabel('Amplitude')\nplt.legend()\nplt.show()\n`;
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

    function recalculate() {
      if (currentResult && element('page-scientific')?.classList.contains('active')) {
        run();
      }
    }

    return Object.freeze({ open, run, recalculate });
  }

  global.ControLABScientific = Object.freeze({ initialize, modules: MODULES });
}(window));
