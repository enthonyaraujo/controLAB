(function registerScientificModules(global) {
  const MODULES = {
    time: {
      action: 'time_response',
      title: 'Resposta no Domínio do Tempo',
      kicker: 'Transitório & Regime Permanente',
      description: 'Simule degrau, impulso e rampa em malha fechada e calcule as métricas transitórias e os erros estacionários.',
      filename: 'resposta-temporal',
      fields: [
        {
          name: 'expr',
          label: 'Função de transferência G(s)',
          type: 'text',
          value: '4 / (s^2 + 2*s + 4)',
          placeholder: 'Ex.: 4 / (s^2 + 2*s + 4)',
          help: 'Use s como variável. A resposta temporal considera realimentação unitária negativa.',
        },
        {
          name: 'final_time',
          label: 'Tempo final (s)',
          type: 'number',
          value: '10',
          min: '0.05',
          max: '10000',
          step: '0.1',
        },
        {
          name: 'settling_threshold',
          label: 'Critério de acomodação',
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
        ['Sistema tipo 1', '5 / (s * (s + 2))'],
      ],
    },
    frequency: {
      action: 'frequency_response',
      title: 'Resposta em Frequência',
      kicker: 'Bode & Nyquist',
      description: 'Compare a curva real às assíntotas de Bode, visualize Nyquist e obtenha margens e frequências críticas.',
      filename: 'resposta-em-frequencia',
      fields: [
        {
          name: 'expr',
          label: 'Função de transferência G(s)',
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
          step: '0.01',
        },
        {
          name: 'omega_max',
          label: 'Frequência máxima (rad/s)',
          type: 'number',
          value: '100',
          min: '0.00001',
          step: '1',
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
          name: 'critical_gain', label: 'Ganho crítico Kcr', type: 'number', value: '6', min: '0.000001', step: '0.1',
          when: { design_type: ['pid'], method: ['zn_critical'] },
        },
        {
          name: 'critical_period', label: 'Período crítico Pcr (s)', type: 'number', value: '2', min: '0.000001', step: '0.1',
          when: { design_type: ['pid'], method: ['zn_critical'] },
        },
        {
          name: 'process_gain', label: 'Ganho do processo K', type: 'number', value: '1', min: '0.000001', step: '0.1',
          when: { design_type: ['pid'], method: ['zn_reaction', 'cohen_coon', 'chr'] },
        },
        {
          name: 'delay', label: 'Atraso aparente L (s)', type: 'number', value: '0.5', min: '0.000001', step: '0.1',
          when: { design_type: ['pid'], method: ['zn_reaction', 'cohen_coon', 'chr'] },
        },
        {
          name: 'time_constant', label: 'Constante de tempo T (s)', type: 'number', value: '4', min: '0.000001', step: '0.1',
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
          name: 'compensator_zero', label: 'Frequência do zero', type: 'number', value: '1', min: '0.000001', step: '0.1',
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'compensator_pole', label: 'Frequência do polo', type: 'number', value: '5', min: '0.000001', step: '0.1',
          help: 'No avanço, polo > zero. No atraso, polo < zero.',
          when: { design_type: ['lead_lag'] },
        },
        {
          name: 'compensator_gain', label: 'Ganho do compensador', type: 'number', value: '1', min: '0.000001', step: '0.1',
          when: { design_type: ['lead_lag'] },
        },
        { name: 'final_time', label: 'Tempo final da comparação (s)', type: 'number', value: '20', min: '0.05', step: '0.5' },
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
          placeholder: 'Ex.: -4, -5', help: 'Opcional. Informe um polo por estado, separados por vírgula.',
        },
        {
          name: 'observer_poles', label: 'Polos do observador A - LC', type: 'text', value: '-6, -7',
          placeholder: 'Ex.: -6, -7', help: 'Opcional. Informe um polo por estado, separados por vírgula.',
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

  function initialize({ renderMath, showToast }) {
    const form = element('scientific-form');
    const runButton = element('btn-run-scientific');
    if (!form || !runButton) return null;

    let currentKey = null;
    let currentResult = null;
    let requestId = 0;

    function renderFields(config) {
      form.replaceChildren();
      config.fields.forEach((field) => {
        const group = document.createElement('label');
        group.className = 'scientific-field';
        if (field.when) group.dataset.when = JSON.stringify(field.when);
        appendTextNode(group, 'span', 'form-label', field.label);

        let input;
        if (field.type === 'select') {
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
          input.rows = field.rows || 3;
          input.value = field.value || '';
        } else {
          input = document.createElement('input');
          input.type = field.type || 'text';
          input.value = field.value || '';
          ['placeholder', 'min', 'max', 'step'].forEach((property) => {
            if (field[property] !== undefined) input[property] = field[property];
          });
        }
        input.name = field.name;
        input.className = 'form-input code-font';
        input.autocomplete = 'off';
        group.appendChild(input);
        if (field.help) appendTextNode(group, 'small', 'input-help', field.help);
        form.appendChild(group);
      });

      if (config.examples?.length) {
        const examples = document.createElement('div');
        examples.className = 'scientific-examples';
        appendTextNode(examples, 'span', 'hint-title', 'Exemplos rápidos');
        const chips = document.createElement('div');
        chips.className = 'chip-group';
        config.examples.forEach(([label, expression]) => {
          const chip = appendTextNode(chips, 'button', 'chip', label);
          chip.type = 'button';
          chip.addEventListener('click', () => {
            const expressionInput = form.elements.namedItem('expr') || form.elements.namedItem('plant_expr');
            if (expressionInput) expressionInput.value = expression;
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
    }

    function open(moduleKey) {
      const config = MODULES[moduleKey];
      if (!config) return false;
      if (currentKey !== moduleKey) {
        currentKey = moduleKey;
        element('scientific-title').textContent = config.title;
        element('scientific-kicker').textContent = config.kicker;
        element('scientific-description').textContent = config.description;
        renderFields(config);
        showState('empty');
      }
      return true;
    }

    function showState(state) {
      element('scientific-loading').hidden = state !== 'loading';
      element('scientific-empty').hidden = state !== 'empty';
      element('scientific-error').hidden = state !== 'error';
      element('scientific-results').hidden = state !== 'results';
      runButton.disabled = state === 'loading';
    }

    function payloadFromForm(config) {
      const payload = {};
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
      element('scientific-result-title').textContent = result.title || MODULES[currentKey].title;
      element('scientific-result-description').textContent = result.description || '';

      const latexGrid = element('scientific-latex');
      latexGrid.replaceChildren();
      (result.latex || []).forEach((item) => {
        const card = document.createElement('article');
        card.className = 'scientific-result-card scientific-latex-card';
        appendTextNode(card, 'span', 'scientific-card-label', item.label || 'Equação');
        const math = document.createElement('div');
        math.className = 'scientific-math';
        card.appendChild(math);
        renderMath(math, item.value || '', true);
        latexGrid.appendChild(card);
      });

      const metricsGrid = element('scientific-metrics');
      metricsGrid.replaceChildren();
      (result.metrics || []).forEach((metric) => {
        const card = document.createElement('article');
        card.className = 'scientific-metric-card';
        appendTextNode(card, 'span', 'scientific-card-label', metric.label || 'Métrica');
        appendTextNode(card, 'strong', '', `${metric.value ?? '—'}${metric.unit ? ` ${metric.unit}` : ''}`);
        metricsGrid.appendChild(card);
      });

      const plotCard = element('scientific-plot-card');
      const plot = element('scientific-plot');
      plotCard.hidden = !result.image;
      if (result.image) plot.src = result.image;
      element('btn-scientific-png').hidden = !result.image;
      element('btn-scientific-svg').hidden = !result.svg;

      const detailsCard = element('scientific-details-card');
      detailsCard.hidden = !result.details;
      if (result.details) element('scientific-details').textContent = JSON.stringify(result.details, null, 2);
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
    global.addEventListener('keydown', (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 'Enter'
        && element('page-scientific')?.classList.contains('active')) {
        event.preventDefault();
        run();
      }
    });

    element('btn-scientific-png')?.addEventListener('click', async () => {
      if (!currentResult?.image) return;
      await global.api.saveImage({ base64: currentResult.image, defaultName: `${MODULES[currentKey].filename}.png` });
    });
    element('btn-scientific-svg')?.addEventListener('click', async () => {
      if (!currentResult?.svg) return;
      await global.api.saveSVG({ svg: currentResult.svg, defaultName: `${MODULES[currentKey].filename}.svg` });
    });

    return Object.freeze({ open, run });
  }

  global.ControLABScientific = Object.freeze({ initialize, modules: MODULES });
}(window));
