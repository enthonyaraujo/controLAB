/**
 * ControLAB - Módulo de Análise de Estabilidade por Routh-Hurwitz
 * Gerencia o cálculo, interface interativa, 3 regimes de K, gráfico no plano s e renderização matemática.
 */

(function registerRouthModule(global) {
  function optionalElement(id) {
    return document.getElementById(id);
  }

  function initializeRouth({ renderMath, renderMathInContainer, showToast }) {
    const inputExpr = optionalElement('input-routh-expr');
    const btnCalculate = optionalElement('btn-calculate-routh');
    const loadingEl = optionalElement('routh-loading');
    const resultsContainer = optionalElement('routh-results-container');
    const emptyState = optionalElement('routh-empty-state');

    // Badges e Veredito
    const verdictBadge = optionalElement('routh-verdict-badge');
    const verdictTitle = optionalElement('routh-verdict-title');
    const verdictSummary = optionalElement('routh-verdict-summary');
    const badgeDegree = optionalElement('routh-badge-degree');
    const badgeSignChanges = optionalElement('routh-badge-sign-changes');
    const badgeRhp = optionalElement('routh-badge-rhp');

    // Prévia Matemática da Sidebar
    const previewStatus = optionalElement('routh-preview-status');
    const previewMath = optionalElement('routh-latex-preview');
    let previewTimer = null;

    // Equação Característica
    const charPolyEl = optionalElement('routh-char-poly');

    // Regimes de K
    const kRangeBanner = optionalElement('routh-k-range-banner');
    const kStableExpr = optionalElement('routh-k-stable-expr');
    const kStableDesc = optionalElement('routh-k-stable-desc');
    const kMarginalExpr = optionalElement('routh-k-marginal-expr');
    const kMarginalDesc = optionalElement('routh-k-marginal-desc');
    const kUnstableExpr = optionalElement('routh-k-unstable-expr');
    const kUnstableDesc = optionalElement('routh-k-unstable-desc');

    // Simulador Interativo de Ganho K
    const kLiveBadge = optionalElement('routh-k-live-badge');
    const kSlider = optionalElement('routh-k-slider');
    const btnKPresetStable = optionalElement('btn-k-preset-stable');
    const btnKPresetCrit = optionalElement('btn-k-preset-crit');
    const btnKPresetUnstable = optionalElement('btn-k-preset-unstable');
    const kLiveStatus = optionalElement('routh-k-live-status');

    // Gráfico no Plano s
    const plotCard = optionalElement('routh-plot-card');
    const plotImage = optionalElement('routh-plot-image');
    const btnCopyImg = optionalElement('btn-routh-copy-img');
    const btnDownloadPng = optionalElement('btn-routh-download-png');

    // Tabela e Detalhes
    const tableContainer = optionalElement('routh-table-container');
    const specialCasesContainer = optionalElement('routh-special-cases');
    const stepsList = optionalElement('routh-steps-list');
    const rootsContainer = optionalElement('routh-exact-roots-container');
    const rootsList = optionalElement('routh-exact-roots-list');

    let isCalculating = false;
    let lastExpr = '';
    let lastPlotImage = null;
    let lastCritK = null;
    let liveKTimer = null;

    function updatePreview() {
      if (!previewMath) return;
      const raw = (inputExpr ? inputExpr.value : '').trim();
      if (!raw) {
        if (previewStatus) {
          previewStatus.textContent = 'Aguardando entrada';
          previewStatus.className = 'preview-status';
        }
        renderMath(previewMath, 'P(s) = 0', true);
        return;
      }

      let tex = raw
        .replace(/\*/g, ' ')
        .replace(/\^([0-9]+)/g, '^{$1}')
        .replace(/s([0-9]+)/g, 's^{$1}');

      if (tex.includes('/')) {
        const parts = tex.split('/');
        tex = `G(s) = \\frac{${parts[0].trim()}}{${parts.slice(1).join('/').trim()}}`;
      } else {
        tex = `P(s) = ${tex} = 0`;
      }

      if (previewStatus) {
        previewStatus.textContent = 'Expressão válida';
        previewStatus.className = 'preview-status valid';
      }
      renderMath(previewMath, tex, true);
    }

    function setLoading(isLoading) {
      isCalculating = isLoading;
      if (btnCalculate) btnCalculate.disabled = isLoading;
      if (loadingEl) loadingEl.style.display = isLoading ? 'flex' : 'none';
      if (isLoading && resultsContainer) resultsContainer.style.opacity = '0.5';
      else if (resultsContainer) resultsContainer.style.opacity = '1';
    }

    async function executeCalculation() {
      if (isCalculating) return;
      const expr = inputExpr ? inputExpr.value.trim() : '';
      if (!expr) {
        showToast('Digite uma função de transferência ou equação característica.', 'warning');
        return;
      }

      setLoading(true);
      lastExpr = expr;
      const currentTheme = document.documentElement.getAttribute('data-theme') || 'dark';

      try {
        const data = await window.api.calculateRouth({
          expr,
          has_k: true,
          theme: currentTheme,
        });

        if (!data.success) {
          throw new Error(data.error || 'Erro ao processar análise de Routh-Hurwitz.');
        }

        renderResults(data);
        if (emptyState) emptyState.style.display = 'none';
        if (resultsContainer) resultsContainer.style.display = 'flex';
        showToast('Estabilidade calculada com sucesso!', 'success');
      } catch (err) {
        showToast(`Erro na análise: ${err.message}`, 'error', 4500);
      } finally {
        setLoading(false);
      }
    }

    function renderResults(data) {
      // 1. Veredito e Badges
      if (verdictBadge && verdictTitle) {
        verdictTitle.textContent = data.verdict;
        verdictBadge.className = 'verdict-badge';
        if (data.verdict === 'Estável') {
          verdictBadge.classList.add('verdict-stable');
        } else if (data.verdict === 'Marginalmente Estável') {
          verdictBadge.classList.add('verdict-marginal');
        } else if (data.verdict === 'Instável') {
          verdictBadge.classList.add('verdict-unstable');
        } else {
          verdictBadge.classList.add('verdict-dependent');
        }
      }

      if (verdictSummary) {
        verdictSummary.textContent = data.summary;
      }

      if (badgeDegree) badgeDegree.textContent = `Grau ${data.degree}`;
      if (badgeSignChanges) badgeSignChanges.textContent = `${data.sign_changes} Trocas de Sinal`;
      if (badgeRhp) badgeRhp.textContent = `${data.rhp_poles} Raízes no SPD`;

      // 2. Equação Característica
      if (charPolyEl) {
        charPolyEl.innerHTML = '';
        renderMath(charPolyEl, data.char_poly_latex, true);
      }

      // 3. Regimes de Estabilidade por Ganho K
      if (kRangeBanner) {
        if (data.k_range && data.k_range.has_k) {
          kRangeBanner.style.display = 'block';

          // Regime Estável
          if (kStableExpr) {
            kStableExpr.innerHTML = '';
            renderMath(kStableExpr, data.k_range.stable_latex || '\\text{Sem faixa estável}', true);
          }
          if (kStableDesc) {
            kStableDesc.textContent = (data.k_range.stable_latex && data.k_range.stable_latex.includes('Sem'))
              ? 'Nenhum valor de K garante todos os polos estritamente no semiplano esquerdo.'
              : 'Todos os polos de malha fechada situam-se no SPE com amortecimento.';
          }

          // Regime Marginalmente Estável
          if (kMarginalExpr) {
            kMarginalExpr.innerHTML = '';
            renderMath(kMarginalExpr, data.k_range.marginal_latex || '\\text{Nenhum ponto marginal}', true);
          }
          if (kMarginalDesc) {
            if (data.k_range.marginal_cases && data.k_range.marginal_cases.length > 0) {
              const omegas = data.k_range.marginal_cases.map((m) => m.omega_latex).join(', ');
              kMarginalDesc.textContent = `Oscilações sustentadas no eixo imaginário com ${omegas}.`;
            } else {
              kMarginalDesc.textContent = 'Não há cruzamento com o eixo jω para ganhos positivos.';
            }
          }

          // Regime Instável
          if (kUnstableExpr) {
            kUnstableExpr.innerHTML = '';
            renderMath(kUnstableExpr, data.k_range.unstable_latex || '\\text{Sem faixa instável}', true);
          }
          if (kUnstableDesc) {
            kUnstableDesc.textContent = (data.k_range.unstable_latex && data.k_range.unstable_latex.includes('Sem'))
              ? 'O sistema permanece estável para todos os valores admissíveis de K.'
              : 'Polos no semiplano direito causam crescimento exponencial e instabilidade.';
          }

          // Configuração do Simulador Interativo
          setupKSimulator(data.k_range);
        } else {
          kRangeBanner.style.display = 'none';
        }
      }

      // 4. Gráfico no Plano Complexo s
      if (plotCard && plotImage) {
        if (data.plot_image) {
          plotCard.style.display = 'block';
          plotImage.src = data.plot_image;
          lastPlotImage = data.plot_image;
        } else {
          plotCard.style.display = 'none';
          lastPlotImage = null;
        }
      }

      // 5. Casos Especiais (Alerta quando ocorrer epsilon ou linha de zeros)
      if (specialCasesContainer) {
        if (data.special_cases && data.special_cases.length > 0) {
          specialCasesContainer.style.display = 'block';
          specialCasesContainer.innerHTML = data.special_cases.map((sc) => {
            return `
              <div class="special-case-card">
                <span class="special-case-tag">Caso Especial</span>
                <p>${sc.desc}</p>
              </div>
            `;
          }).join('');
          renderMathInContainer(specialCasesContainer);
        } else {
          specialCasesContainer.style.display = 'none';
        }
      }

      // 6. Tabela de Routh
      if (tableContainer) {
        renderRouthTable(data.routh_table);
      }

      // 7. Passo a Passo das Linhas
      if (stepsList) {
        stepsList.innerHTML = data.steps.map((st) => {
          const valsHtml = st.values.map((v) => `<span class="routh-step-term">$${v}$</span>`).join(' ');
          return `
            <div class="routh-step-item">
              <div class="routh-step-header">
                <span class="step-badge">$${st.power}$</span>
                <span class="step-desc">${st.desc}</span>
              </div>
              <div class="routh-step-values">${valsHtml}</div>
            </div>
          `;
        }).join('');
        renderMathInContainer(stepsList);
      }

      // 8. Raízes Numéricas Exatas (quando disponíveis)
      if (rootsContainer && rootsList) {
        if (data.exact_roots && data.exact_roots.length > 0) {
          rootsContainer.style.display = 'block';
          rootsList.innerHTML = data.exact_roots.map((r, i) => {
            const reTag = r.real > 0 ? '<span class="root-rhp">SPD</span>' : (r.real < 0 ? '<span class="root-lhp">SPE</span>' : '<span class="root-jw">jω</span>');
            return `<li class="root-item"><span class="root-index">s_{${i+1}}:</span> <code>${r.str}</code> ${reTag}</li>`;
          }).join('');
        } else {
          rootsContainer.style.display = 'none';
        }
      }

      // Renderiza todas as expressões LaTeX na área de resultados
      if (resultsContainer) {
        renderMathInContainer(resultsContainer);
      }
    }

    function setupKSimulator(kRange) {
      if (!kSlider) return;

      let critVal = null;
      if (kRange.critical_k && kRange.critical_k.length > 0) {
        critVal = kRange.critical_k[0].k_val;
      } else if (kRange.marginal_cases && kRange.marginal_cases.length > 0) {
        critVal = kRange.marginal_cases[0].k_val;
      }
      lastCritK = critVal;

      let defaultK = 1.0;
      if (critVal !== null && critVal > 0) {
        const maxVal = Math.max(16, Math.ceil(critVal * 2.2));
        kSlider.min = '0.1';
        kSlider.max = maxVal.toString();
        kSlider.step = Math.max(0.05, Math.round((critVal / 60) * 100) / 100).toString();
        defaultK = Math.max(0.2, Math.round((critVal * 0.5) * 10) / 10);
      } else {
        kSlider.min = '0.1';
        kSlider.max = '20';
        kSlider.step = '0.1';
        defaultK = 1.0;
      }

      kSlider.value = defaultK.toString();
      updateLiveK(defaultK);
    }

    function updateLiveK(kVal) {
      if (kLiveBadge) {
        kLiveBadge.textContent = `K = ${kVal}`;
      }

      clearTimeout(liveKTimer);
      liveKTimer = setTimeout(async () => {
        if (!window.api || !window.api.evaluateRouthK) return;
        try {
          const res = await window.api.evaluateRouthK({
            expr: lastExpr || (inputExpr ? inputExpr.value.trim() : ''),
            k_val: kVal,
          });

          if (!res || !res.success) return;
          renderLiveKStatus(res);
        } catch (err) {
          console.warn('Erro na avaliação de ponto K:', err);
        }
      }, 70);
    }

    function renderLiveKStatus(res) {
      if (!kLiveStatus) return;

      const verdictClass = res.verdict === 'Estável'
        ? 'status-pill-stable'
        : (res.verdict === 'Marginalmente Estável' ? 'status-pill-marginal' : 'status-pill-unstable');

      const speCount = (res.roots || []).length - res.spd_count - res.jw_count;
      const rootsHtml = (res.roots || []).map((r, i) => {
        const tag = r.zone === 'SPD'
          ? '<span class="root-rhp">SPD</span>'
          : (r.zone === 'SPE' ? '<span class="root-lhp">SPE</span>' : '<span class="root-jw">jω</span>');
        return `<span class="live-root-item"><code>s_{${i + 1}} = ${r.str}</code> ${tag}</span>`;
      }).join('');

      kLiveStatus.innerHTML = `
        <div class="live-status-row">
          <span class="live-status-badge ${verdictClass}">${res.verdict}</span>
          <span class="live-status-poles-summary">${speCount} polo(s) no SPE &bull; ${res.jw_count} no j&omega; &bull; ${res.spd_count} no SPD</span>
        </div>
        <div class="live-status-roots">${rootsHtml}</div>
      `;
    }

    // Eventos do Simulador de K
    kSlider?.addEventListener('input', (e) => {
      const val = parseFloat(e.target.value);
      updateLiveK(val);
    });

    btnKPresetStable?.addEventListener('click', () => {
      if (!kSlider) return;
      const target = (lastCritK !== null && lastCritK > 0)
        ? Math.max(0.1, Math.round((lastCritK * 0.5) * 10) / 10)
        : 1.0;
      kSlider.value = target.toString();
      updateLiveK(target);
    });

    btnKPresetCrit?.addEventListener('click', () => {
      if (!kSlider) return;
      if (lastCritK !== null && lastCritK > 0) {
        kSlider.value = lastCritK.toString();
        updateLiveK(lastCritK);
      } else {
        showToast('Não há ganho crítico de oscilação marginal neste sistema.', 'info');
      }
    });

    btnKPresetUnstable?.addEventListener('click', () => {
      if (!kSlider) return;
      const target = (lastCritK !== null && lastCritK > 0)
        ? Math.round((lastCritK * 1.5) * 10) / 10
        : 12.0;
      kSlider.value = target.toString();
      updateLiveK(target);
    });

    // Eventos de Exportação do Gráfico do Plano s
    btnCopyImg?.addEventListener('click', async () => {
      if (!lastPlotImage) return;
      try {
        const res = await window.api.copyImageToClipboard(lastPlotImage);
        if (res && res.error) throw new Error(res.error);
        showToast('Gráfico copiado para a área de transferência!', 'success');
      } catch (err) {
        showToast('Não foi possível copiar o gráfico: ' + err.message, 'error');
      }
    });

    btnDownloadPng?.addEventListener('click', async () => {
      if (!lastPlotImage) return;
      try {
        await window.api.saveImage({
          base64: lastPlotImage,
          defaultName: 'estabilidade_routh_plano_s.png',
        });
        showToast('Gráfico PNG exportado com sucesso!', 'success');
      } catch (err) {
        showToast('Erro ao exportar gráfico: ' + err.message, 'error');
      }
    });

    function renderRouthTable(rows) {
      if (!tableContainer || !rows) return;

      const numCols = rows[0]?.cells?.length || 0;
      let headersHtml = '<th>Potência</th>';
      for (let j = 0; j < numCols; j++) {
        headersHtml += `<th>Coluna ${j + 1}</th>`;
      }

      let rowsHtml = '';
      rows.forEach((r) => {
        let cellsHtml = `<td class="cell-power">$${r.power}$</td>`;
        r.cells.forEach((cell) => {
          const isFirstCol = cell.is_first_col;
          const signClass = isFirstCol && cell.sign ? `sign-${cell.sign === '+' ? 'pos' : (cell.sign === '-' ? 'neg' : 'zero')}` : '';
          const signBadge = isFirstCol && cell.sign && cell.sign !== 'expr' ? `<span class="sign-pill ${signClass}">${cell.sign}</span>` : '';

          cellsHtml += `
            <td class="routh-cell ${isFirstCol ? 'first-col' : ''}">
              <div class="cell-wrapper">
                <span class="cell-math">$${cell.latex}$</span>
                ${signBadge}
              </div>
            </td>
          `;
        });
        rowsHtml += `<tr>${cellsHtml}</tr>`;
      });

      tableContainer.innerHTML = `
        <table class="routh-matrix">
          <thead>
            <tr>${headersHtml}</tr>
          </thead>
          <tbody>
            ${rowsHtml}
          </tbody>
        </table>
      `;

      renderMathInContainer(tableContainer);
    }

    // Prévia em tempo real ao digitar
    inputExpr?.addEventListener('input', () => {
      clearTimeout(previewTimer);
      previewTimer = setTimeout(updatePreview, 120);
    });

    // Eventos de clique nos chips de exemplos
    document.querySelectorAll('.chip-routh').forEach((chip) => {
      chip.addEventListener('click', () => {
        const example = chip.dataset.example;
        if (example && inputExpr) {
          inputExpr.value = example;
          updatePreview();
          executeCalculation();
        }
      });
    });

    // Ação do botão calcular
    btnCalculate?.addEventListener('click', executeCalculation);

    // Atalho Enter no input
    inputExpr?.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        executeCalculation();
      }
    });

    // Atualiza a prévia matemática inicial
    updatePreview();

    // Se estiver vazio ao carregar, executa o primeiro exemplo por padrão ao abrir o módulo
    function ensureInitialCalculation() {
      updatePreview();
      if (resultsContainer && resultsContainer.style.display !== 'flex') {
        executeCalculation();
      }
    }

    return {
      executeCalculation,
      ensureInitialCalculation,
    };
  }

  global.ControLABRouth = {
    initialize: initializeRouth,
  };
})(window);
