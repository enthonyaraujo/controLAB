/**
 * ControLAB - Módulo de Análise de Estabilidade por Routh-Hurwitz
 * Gerencia o cálculo, interface interativa e renderização matemática da tabela de Routh.
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

    // Equação e Faixa de K
    const charPolyEl = optionalElement('routh-char-poly');
    const kRangeBanner = optionalElement('routh-k-range-banner');
    const kRangeExpr = optionalElement('routh-k-range-expr');
    const kCritInfo = optionalElement('routh-k-crit-info');

    // Tabela e Detalhes
    const tableContainer = optionalElement('routh-table-container');
    const specialCasesContainer = optionalElement('routh-special-cases');
    const stepsList = optionalElement('routh-steps-list');
    const rootsContainer = optionalElement('routh-exact-roots-container');
    const rootsList = optionalElement('routh-exact-roots-list');

    let isCalculating = false;

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
      try {
        const data = await window.api.calculateRouth({
          expr,
          has_k: true,
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

      // 3. Faixa de K
      if (kRangeBanner) {
        if (data.k_range && data.k_range.has_k) {
          kRangeBanner.style.display = 'block';
          if (kRangeExpr) {
            kRangeExpr.innerHTML = '';
            renderMath(kRangeExpr, data.k_range.range_latex, true);
          }
          if (kCritInfo) {
            if (data.k_range.critical_k && data.k_range.critical_k.length > 0) {
              const critItems = data.k_range.critical_k.map((ck) => {
                const omegaText = ck.omega_latex ? ` \\quad (\\text{com } ${ck.omega_latex})` : '';
                return `<li>$K_{crit} = ${ck.k_latex}${omegaText}$</li>`;
              }).join('');
              kCritInfo.innerHTML = `<strong>Ganhos Críticos de Oscilação:</strong><ul>${critItems}</ul>`;
              renderMathInContainer(kCritInfo);
            } else {
              kCritInfo.innerHTML = '';
            }
          }
        } else {
          kRangeBanner.style.display = 'none';
        }
      }

      // 4. Tabela de Routh
      if (tableContainer) {
        renderRouthTable(data.routh_table, data.cell_details);
      }

      // 5. Casos Especiais
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

      // 6. Passos Didáticos
      if (stepsList) {
        stepsList.innerHTML = data.steps.map((st, idx) => {
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

      // 7. Raízes Exatas (quando disponíveis)
      if (rootsContainer && rootsList) {
        if (data.exact_roots && data.exact_roots.length > 0) {
          rootsContainer.style.display = 'block';
          rootsList.innerHTML = data.exact_roots.map((r, i) => {
            const reTag = r.real > 0 ? '<span class="root-rhp">SPD</span>' : (r.real < 0 ? '<span class="root-lhp">SPE</span>' : '<span class="root-jw">jω</span>');
            return `<li class="root-item"><span class="root-index">s_${i+1}:</span> <code>${r.str}</code> ${reTag}</li>`;
          }).join('');
        } else {
          rootsContainer.style.display = 'none';
        }
      }

      // Renderiza expressões LaTeX em toda a área de resultados
      if (resultsContainer) {
        renderMathInContainer(resultsContainer);
      }
    }

    function renderRouthTable(rows, cellDetails) {
      if (!tableContainer || !rows) return;

      const numCols = rows[0]?.cells?.length || 0;
      let headersHtml = '<th>Potência</th>';
      for (let j = 0; j < numCols; j++) {
        headersHtml += `<th>Coluna ${j + 1}</th>`;
      }

      let rowsHtml = '';
      rows.forEach((r, rIdx) => {
        let cellsHtml = `<td class="cell-power">$${r.power}$</td>`;
        r.cells.forEach((cell, cIdx) => {
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
