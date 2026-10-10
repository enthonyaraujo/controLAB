(function registerNavigation(global) {
  function optionalElement(id) {
    return document.getElementById(id);
  }

  function initializeNavigation({ renderMathInContainer, showToast, onOpenLgr, onOpenRouth, onOpenScientific }) {
    const pageHome = optionalElement('page-home');
    const pageLgr = optionalElement('page-lgr');
    const pageRouth = optionalElement('page-routh');
    const pageScientific = optionalElement('page-scientific');
    const btnNavHome = optionalElement('btn-nav-home');
    const globalBrand = optionalElement('global-brand');
    const globalBrandTitle = optionalElement('global-brand-title');
    const globalNavbarCenter = optionalElement('global-navbar-center');
    const globalNavbarTitle = optionalElement('global-navbar-title') || optionalElement('global-navbar-subtitle');
    const globalNavbar = document.querySelector('.global-navbar');
    const renderedPages = new WeakSet();

    function renderPageOnce(page) {
      if (!page || renderedPages.has(page)) return;
      renderMathInContainer(page);
      renderedPages.add(page);
    }

    function navigateTo(pageId) {
      const isHome = pageId === 'home' || !pageId;
      const isLgr = pageId === 'lgr';
      const isRouth = pageId === 'routh';
      const isScientific = ['time', 'frequency', 'controllers', 'state-space'].includes(pageId);
      const isSciShell = isScientific || isLgr || isRouth || isHome;

      pageHome?.classList.toggle('active', isHome);
      pageLgr?.classList.toggle('active', isLgr);
      pageRouth?.classList.toggle('active', isRouth);
      pageScientific?.classList.toggle('active', isScientific);

      const hasBack = !isHome && !isSciShell;
      if (btnNavHome) btnNavHome.style.display = hasBack ? 'inline-flex' : 'none';
      if (globalBrand) globalBrand.style.display = isHome && !isSciShell ? 'flex' : 'none';
      if (globalBrandTitle) {
        globalBrandTitle.textContent = isHome ? 'ControLAB' : '';
      }
      if (globalNavbar) {
        globalNavbar.classList.toggle('has-back', hasBack);
        globalNavbar.style.display = isSciShell ? 'none' : '';
      }
      document.body.classList.toggle('in-scientific', isSciShell);

      // Sincroniza abas, trilho e navegação inferior do software científico
      document.querySelectorAll('[data-nav]').forEach((el) => {
        const target = el.dataset.nav;
        if (!target || target === 'more') return;
        const matches = target === (isHome ? 'home' : pageId);
        el.classList.toggle('active', matches);
      });

      const sciAppBarTitle = optionalElement('sci-app-bar-title');
      const sciAppBarIcon = document.querySelector('.sci-app-bar-icon use');
      const navTitles = {
        time: 'Resposta no tempo',
        frequency: 'Resposta em frequência',
        controllers: 'Projeto de controladores',
        'state-space': 'Espaço de estados',
      };
      const navIcons = {
        time: '#s-time',
        frequency: '#s-freq',
        controllers: '#s-ctrl',
        'state-space': '#s-state',
      };
      if (sciAppBarTitle && navTitles[pageId]) {
        sciAppBarTitle.textContent = navTitles[pageId];
      }
      if (sciAppBarIcon && navIcons[pageId]) {
        sciAppBarIcon.setAttribute('href', navIcons[pageId]);
      }

      if (globalNavbarCenter) {
        globalNavbarCenter.style.display = isHome ? 'none' : 'flex';
      }
      if (globalNavbarTitle) {
        if (isLgr) {
          globalNavbarTitle.textContent = 'Lugar Geométrico das Raízes';
        } else if (isRouth) {
          globalNavbarTitle.textContent = 'Critério de Routh-Hurwitz';
        } else if (isScientific) {
          const titles = {
            time: 'Resposta no Domínio do Tempo',
            frequency: 'Resposta em Frequência',
            controllers: 'Projeto de Controladores',
            'state-space': 'Espaço de Estados',
          };
          globalNavbarTitle.textContent = titles[pageId];
        } else {
          globalNavbarTitle.textContent = '';
        }
      }

      if (isLgr) {
        renderPageOnce(pageLgr);
        onOpenLgr();
      } else if (isRouth) {
        renderPageOnce(pageRouth);
        if (onOpenRouth) onOpenRouth();
      } else if (isScientific) {
        renderPageOnce(pageScientific);
        onOpenScientific?.(pageId);
        pageScientific?.scrollTo?.({ top: 0 });
      } else {
        if (pageHome) pageHome.scrollTop = 0;
        renderPageOnce(pageHome);
      }
    }

    // Delegação de cliques para botões e links com [data-nav] e ações
    document.addEventListener('click', (event) => {
      const settingsBtn = event.target.closest('[data-action="settings"]');
      if (settingsBtn) {
        event.preventDefault();
        window.ControLABSettings?.open?.();
        return;
      }
      const navBtn = event.target.closest('[data-nav]');
      if (!navBtn) return;
      const target = navBtn.dataset.nav;
      if (!target || target === 'more') return;
      event.preventDefault();
      optionalElement('sci-more-dropdown')?.setAttribute('hidden', '');
      navigateTo(target);
    });

    optionalElement('btn-open-lgr')?.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      navigateTo('lgr');
    });
    optionalElement('card-module-lgr')?.addEventListener('click', (event) => {
      event.preventDefault();
      navigateTo('lgr');
    });

    optionalElement('btn-open-routh')?.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      navigateTo('routh');
    });
    optionalElement('card-module-routh')?.addEventListener('click', (event) => {
      event.preventDefault();
      navigateTo('routh');
    });

    document.querySelectorAll('.module-card.card-active[data-page]').forEach((card) => {
      if (['lgr', 'routh'].includes(card.dataset.page)) return;
      const open = (event) => {
        event?.preventDefault();
        navigateTo(card.dataset.page);
      };
      card.addEventListener('click', open);
      card.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          open(event);
        }
      });
      card.querySelector('.btn-module-primary')?.addEventListener('click', (event) => {
        event.stopPropagation();
        open(event);
      });
    });

    btnNavHome?.addEventListener('click', () => navigateTo('home'));
    optionalElement('btn-sidebar-back-home')?.addEventListener('click', () => navigateTo('home'));

    // Linhas móveis do Hub (< 600px)
    document.querySelectorAll('.hub-mobile-row[data-page]').forEach((row) => {
      row.addEventListener('click', (event) => {
        event.preventDefault();
        navigateTo(row.dataset.page);
      });
    });

    // Itens de Cálculos Recentes
    document.querySelectorAll('.hub-recent-row[data-nav]').forEach((row) => {
      row.addEventListener('click', (event) => {
        event.preventDefault();
        navigateTo(row.dataset.nav);
      });
    });

    // Exemplos rápidos da página inicial
    document.querySelectorAll('.hub-example-row[data-example]').forEach((row) => {
      row.addEventListener('click', () => {
        navigateTo(row.dataset.example);
      });
    });

    // Atalhos de Teclado Científicos (Ctrl+1 a Ctrl+6)
    window.addEventListener('keydown', (event) => {
      if ((event.ctrlKey || event.metaKey) && !event.shiftKey && !event.altKey) {
        const keyMap = {
          '1': 'time',
          '2': 'routh',
          '3': 'lgr',
          '4': 'frequency',
          '5': 'controllers',
          '6': 'state-space',
        };
        if (keyMap[event.key]) {
          event.preventDefault();
          navigateTo(keyMap[event.key]);
        }
      }
    });

    function setupHomeScreen() {
      // Helper para sincronizar estado ativo entre top tabs, tablet rail e bottom nav
      function setActiveHomeTab(tabKey) {
        document.querySelectorAll('.home-tab, .home-rail-item, .home-bottom-nav-item').forEach((el) => {
          if (el.dataset.homeTab) {
            el.classList.toggle('active', el.dataset.homeTab === tabKey);
          }
        });
      }

      function handleTabAction(tabKey) {
        if (tabKey === 'inicio') {
          setActiveHomeTab('inicio');
          const homePage = optionalElement('page-home');
          if (homePage) homePage.scrollTop = 0;
        } else if (tabKey === 'console') {
          setActiveHomeTab('console');
          const input = optionalElement('home-console-input');
          input?.scrollIntoView?.({ behavior: 'smooth', block: 'center' });
          input?.focus();
          showToast('Console interativo selecionado.', 'info', 2000);
        } else if (tabKey === 'modulos') {
          setActiveHomeTab('modulos');
          const modGrid = optionalElement('home-modules-grid');
          modGrid?.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
          showToast('Módulos científicos do ControLAB.', 'info', 2000);
        } else if (tabKey === 'mais') {
          window.ControLABSettings?.open?.();
        } else {
          const names = {
            editor: 'Editor de scripts',
            pacotes: 'Gerenciador de pacotes',
            blocos: 'Diagrama de blocos',
            ajuda: 'Ajuda e documentação',
          };
          showToast(`O recurso "${names[tabKey] || tabKey}" está em desenvolvimento.`, 'info', 2800);
        }
      }

      // Abas da barra superior, trilho lateral e navegação inferior
      document.querySelectorAll('.home-tab, .home-rail-item, .home-bottom-nav-item').forEach((tab) => {
        tab.addEventListener('click', () => {
          const tabKey = tab.dataset.homeTab;
          if (tabKey) handleTabAction(tabKey);
        });
      });

      // Botão "Ver todos" nos módulos (modo mobile)
      const btnSeeAll = optionalElement('btn-see-all-modules');
      const modulesGrid = optionalElement('home-modules-grid');
      if (btnSeeAll && modulesGrid) {
        btnSeeAll.addEventListener('click', () => {
          const isExpanded = modulesGrid.classList.toggle('modules-expanded');
          btnSeeAll.textContent = isExpanded ? 'Ver menos' : 'Ver todos';
        });
      }

      // Botões de ação em "Começar"
      optionalElement('btn-home-new-script')?.addEventListener('click', () => {
        showToast('Novo script iniciado no workspace.', 'info', 2500);
        optionalElement('home-console-input')?.focus();
      });
      optionalElement('btn-home-open-file')?.addEventListener('click', () => {
        showToast('Selecione um arquivo .m ou modelo do ControLAB.', 'info', 2500);
      });
      optionalElement('btn-home-import')?.addEventListener('click', () => {
        showToast('Importador de arquivos .m / .mat pronto.', 'info', 2500);
      });

      // Cards de módulos
      document.querySelectorAll('.home-module-card').forEach((card) => {
        card.addEventListener('click', () => {
          const nav = card.dataset.nav;
          if (nav) {
            navigateTo(nav);
          } else {
            const moduleName = card.dataset.module || card.querySelector('.mod-card-title')?.textContent || 'Módulo';
            showToast(`O módulo "${moduleName}" está em desenvolvimento.`, 'info', 3000);
          }
        });
        card.addEventListener('keydown', (event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            card.click();
          }
        });
      });

      // Exemplos rápidos
      document.querySelectorAll('.home-example-item').forEach((item) => {
        item.addEventListener('click', () => {
          const example = item.dataset.example;
          const expr = item.dataset.expr;
          if (example) {
            navigateTo(example);
            if (expr) {
              window.setTimeout(() => {
                const exprInput = document.querySelector('#scientific-form input[name="expr"]') || optionalElement('input-expr');
                if (exprInput) {
                  exprInput.value = expr;
                  exprInput.dispatchEvent(new Event('input', { bubbles: true }));
                  exprInput.dispatchEvent(new Event('change', { bubbles: true }));
                }
              }, 100);
            }
          }
        });
      });

      // Formatador de tempo relativo para Recentes
      function formatRelativeTime(timestamp) {
        if (!timestamp) return 'recente';
        const diffSec = Math.floor((Date.now() - timestamp) / 1000);
        if (diffSec < 60) return 'agora';
        const diffMin = Math.floor(diffSec / 60);
        if (diffMin < 60) return `${diffMin} min`;
        const diffH = Math.floor(diffMin / 60);
        if (diffH < 24) return `${diffH} h`;
        const diffD = Math.floor(diffH / 24);
        if (diffD === 1) return 'ontem';
        return `${diffD} d`;
      }

      function loadRecentItems() {
        try {
          const raw = localStorage.getItem('controlab_recent_items');
          if (raw) {
            const parsed = JSON.parse(raw);
            if (Array.isArray(parsed)) return parsed;
          }
        } catch {}
        return [];
      }

      function saveRecentItems(items) {
        try {
          localStorage.setItem('controlab_recent_items', JSON.stringify((items || []).slice(0, 30)));
        } catch {}
      }

      function handleRecentItemClick(item) {
        if (item.nav) {
          navigateTo(item.nav);
          if (item.expr) {
            window.setTimeout(() => {
              const exprInput = document.querySelector('#scientific-form input[name="expr"]') || optionalElement('input-expr');
              if (exprInput) {
                exprInput.value = item.expr;
                exprInput.dispatchEvent(new Event('input', { bubbles: true }));
                exprInput.dispatchEvent(new Event('change', { bubbles: true }));
              }
            }, 100);
          }
        } else if (item.file) {
          showToast(`Abrindo ${item.file}...`, 'info', 2000);
          const input = optionalElement('home-console-input');
          if (input) {
            input.value = `run('${item.file}')`;
            input.focus();
          }
        }
      }

      function renderRecentItems(items) {
        const list = optionalElement('home-recent-list');
        if (!list) return;
        list.innerHTML = '';
        if (!items || items.length === 0) {
          const emptyLi = document.createElement('li');
          emptyLi.className = 'home-empty-hint';
          emptyLi.textContent = 'Nenhum cálculo recente';
          list.appendChild(emptyLi);
          return;
        }

        items.slice(0, 8).forEach((item) => {
          const li = document.createElement('li');
          li.className = 'home-recent-item';
          if (item.nav) li.dataset.nav = item.nav;
          if (item.expr) li.dataset.expr = item.expr;
          if (item.file) li.dataset.file = item.file;

          const spanName = document.createElement('span');
          spanName.className = 'recent-name';
          spanName.textContent = item.name || item.expr || item.file;

          const spanTime = document.createElement('span');
          spanTime.className = 'recent-time';
          spanTime.textContent = item.time || formatRelativeTime(item.timestamp);

          li.appendChild(spanName);
          li.appendChild(spanTime);
          li.addEventListener('click', () => handleRecentItemClick(item));
          list.appendChild(li);
        });
      }

      function addRecentItem(entry) {
        if (!entry || (!entry.name && !entry.expr && !entry.file)) return;
        const items = loadRecentItems();
        const key = entry.name || entry.expr || entry.file;
        const filtered = items.filter((it) => (it.name || it.expr || it.file) !== key);
        const newItem = {
          name: key,
          expr: entry.expr || (entry.type === 'math' ? key : ''),
          nav: entry.nav || 'time',
          file: entry.file,
          type: entry.type || 'math',
          timestamp: entry.timestamp || Date.now(),
        };
        filtered.unshift(newItem);
        saveRecentItems(filtered);
        renderRecentItems(filtered);
      }

      // Registra globalmente para ser chamado por qualquer módulo
      window.addRecentItem = addRecentItem;
      window.ControLABRecents = Object.freeze({
        addRecent: addRecentItem,
        getRecents: loadRecentItems,
        clearRecents: () => {
          saveRecentItems([]);
          renderRecentItems([]);
        },
      });

      // Botão para limpar itens recentes
      const btnClearRecent = optionalElement('btn-clear-recent');
      if (btnClearRecent) {
        btnClearRecent.addEventListener('click', () => {
          saveRecentItems([]);
          renderRecentItems([]);
          showToast('Histórico recente limpo.', 'info', 2000);
        });
      }

      // Renderiza itens recentes persistidos
      renderRecentItems(loadRecentItems());

      // Renderizador da lista de arquivos reais
      function renderFileList(files) {
        const list = optionalElement('home-file-list');
        if (!list) return;
        list.innerHTML = '';
        if (!files || files.length === 0) {
          const emptyLi = document.createElement('li');
          emptyLi.className = 'home-empty-hint';
          emptyLi.textContent = 'Nenhum arquivo na pasta';
          list.appendChild(emptyLi);
          return;
        }

        files.forEach((file) => {
          const li = document.createElement('li');
          li.className = 'home-file-item';
          li.dataset.file = file.name;
          if (file.path) li.dataset.path = file.path;
          li.dataset.type = file.type || 'arquivo';

          const spanName = document.createElement('span');
          spanName.className = 'file-name';
          spanName.textContent = file.name;

          const spanType = document.createElement('span');
          spanType.className = 'file-type';
          spanType.textContent = file.type || 'arquivo';

          li.appendChild(spanName);
          li.appendChild(spanType);

          li.addEventListener('click', () => {
            showToast(`Arquivo "${file.name}" selecionado.`, 'info', 2000);
            addRecentItem({
              name: file.name,
              file: file.name,
              type: file.type || 'arquivo',
              timestamp: Date.now(),
            });
            const input = optionalElement('home-console-input');
            if (input) {
              if (file.type === 'script' || file.name.endsWith('.m')) {
                input.value = `run('${file.name}')`;
              } else if (file.type === 'dados' || file.name.endsWith('.mat')) {
                input.value = `load('${file.name}')`;
              } else {
                input.value = file.name;
              }
              input.focus();
            }
          });

          list.appendChild(li);
        });
      }

      // Atualiza estado da pasta
      async function refreshHomeState(folder) {
        if (!window.api?.getHomeState) return;
        try {
          const state = await window.api.getHomeState(folder);
          if (!state) return;
          if (state.folder) {
            const folderText = optionalElement('home-folder-path-text');
            if (folderText) folderText.textContent = state.folder;
          }
          if (Array.isArray(state.files)) {
            renderFileList(state.files);
          }
          if (Array.isArray(state.workspace)) {
            updateWorkspaceUI(state.workspace);
          }
        } catch {}
      }

      // Clique na pasta atual abre o diálogo de seleção
      optionalElement('home-folder-path')?.addEventListener('click', async () => {
        if (window.api?.selectWorkspaceFolder) {
          const res = await window.api.selectWorkspaceFolder();
          if (res?.success && res.folder) {
            showToast(`Pasta de trabalho alterada.`, 'success', 2500);
            refreshHomeState(res.folder);
          }
        }
      });

      // Console e Workspace
      const consoleInput = optionalElement('home-console-input');
      const consoleLogs = optionalElement('home-console-logs');
      const cmdHistory = [];
      let historyIdx = -1;

      function appendLog(text, type = 'output') {
        if (!consoleLogs) return;
        const line = document.createElement('div');
        line.className = `home-console-log-line ${type}`;
        line.textContent = text;
        consoleLogs.appendChild(line);
        consoleLogs.scrollTop = consoleLogs.scrollHeight;
      }

      function updateWorkspaceUI(vars) {
        const list = optionalElement('home-workspace-list');
        if (!list || !Array.isArray(vars)) return;
        list.innerHTML = '';
        if (vars.length === 0) {
          const emptyLi = document.createElement('li');
          emptyLi.className = 'home-empty-hint';
          emptyLi.textContent = 'Nenhuma variável no workspace';
          list.appendChild(emptyLi);
          return;
        }
        vars.forEach((v) => {
          const li = document.createElement('li');
          li.className = 'home-workspace-item';
          li.dataset.var = v.name;
          const spanName = document.createElement('span');
          spanName.className = 'var-name';
          spanName.textContent = v.name;
          const spanType = document.createElement('span');
          spanType.className = 'var-type';
          spanType.textContent = v.type;
          li.appendChild(spanName);
          li.appendChild(spanType);
          li.addEventListener('click', () => executeConsoleCommand(v.name));
          list.appendChild(li);
        });
      }

      async function executeConsoleCommand(rawCmd) {
        const cmd = (rawCmd || '').trim();
        if (!cmd) return;
        appendLog(`>> ${cmd}`, 'cmd');

        if (cmd === 'clc') {
          if (consoleLogs) consoleLogs.innerHTML = '';
          return;
        }

        if (cmd === 'clear') {
          updateWorkspaceUI([]);
        }

        addRecentItem({
          name: cmd,
          expr: cmd,
          nav: 'console',
          type: 'cmd',
          timestamp: Date.now(),
        });

        try {
          if (window.api?.consoleExec) {
            const res = await window.api.consoleExec(cmd);
            if (res.error) {
              appendLog(res.error, 'error');
            } else if (res.output) {
              appendLog(res.output, 'output');
            }
            if (res.folder) {
              const folderText = optionalElement('home-folder-path-text');
              if (folderText) folderText.textContent = res.folder;
            }
            if (Array.isArray(res.files)) {
              renderFileList(res.files);
            }
            if (res.workspace) {
              updateWorkspaceUI(res.workspace);
            }
          } else {
            appendLog(`Comando "${cmd}" executado.`, 'output');
          }
        } catch (err) {
          appendLog(err.message || String(err), 'error');
        }
      }

      // Botão para limpar o workspace
      const btnClearWorkspace = optionalElement('btn-clear-workspace');
      if (btnClearWorkspace) {
        btnClearWorkspace.addEventListener('click', async () => {
          updateWorkspaceUI([]);
          if (window.api?.consoleExec) {
            try {
              await window.api.consoleExec('clear');
            } catch {}
          }
          showToast('Workspace limpo.', 'info', 2000);
        });
      }

      if (consoleInput) {
        consoleInput.addEventListener('keydown', async (event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            const cmd = consoleInput.value.trim();
            if (!cmd) return;
            cmdHistory.push(cmd);
            historyIdx = cmdHistory.length;
            consoleInput.value = '';
            await executeConsoleCommand(cmd);
          } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            if (cmdHistory.length > 0 && historyIdx > 0) {
              historyIdx--;
              consoleInput.value = cmdHistory[historyIdx];
            }
          } else if (event.key === 'ArrowDown') {
            event.preventDefault();
            if (historyIdx < cmdHistory.length - 1) {
              historyIdx++;
              consoleInput.value = cmdHistory[historyIdx];
            } else {
              historyIdx = cmdHistory.length;
              consoleInput.value = '';
            }
          }
        });
      }

      // --- Split Panes (Apenas Desktop >= 1024px) ---
      function setupHomeSplitPanes() {
          const splitterConsole = optionalElement('home-splitter-console');
          const splitterLeft = optionalElement('home-splitter-left');
          const splitterRight = optionalElement('home-splitter-right');
          const homeMainContent = document.querySelector('.home-main-content');
          const consoleArea = optionalElement('home-console-area');
          const leftCol = document.querySelector('.home-left-col');
          const rightCol = document.querySelector('.home-right-col');

          if (!splitterConsole && !splitterLeft && !splitterRight) return;

          function isDesktop() {
            return window.innerWidth >= 1024;
          }

          function getMaxConsoleHeight() {
            if (!homeMainContent) return 300;
            const mainRect = homeMainContent.getBoundingClientRect();
            const modGrid = optionalElement('home-modules-grid');
            const centerCol = document.querySelector('.home-center-col');

            if (modGrid && centerCol) {
              const centerScroll = centerCol.scrollTop || 0;
              const modRect = modGrid.getBoundingClientRect();
              if (modRect.height > 0) {
                const modBottomInMain = (modRect.bottom - mainRect.top) + centerScroll;
                const buffer = 16;
                const maxH = mainRect.height - modBottomInMain - buffer;
                return Math.max(70, Math.floor(maxH));
              }
            }

            return Math.max(70, Math.floor(mainRect.height - 420));
          }

          // Recupera tamanhos salvos no localStorage (usados apenas no desktop)
          try {
            const maxAllowedH = getMaxConsoleHeight();
            const savedConsoleH = parseInt(localStorage.getItem('controlab_desktop_console_height'), 10);
            if (savedConsoleH && savedConsoleH >= 70) {
              const clampedH = Math.min(savedConsoleH, maxAllowedH);
              document.documentElement.style.setProperty('--home-console-height', `${clampedH}px`);
            } else {
              const defaultH = Math.min(160, maxAllowedH);
              document.documentElement.style.setProperty('--home-console-height', `${defaultH}px`);
            }
            const savedLeftW = parseInt(localStorage.getItem('controlab_desktop_left_col_width'), 10);
            if (savedLeftW && savedLeftW >= 140 && savedLeftW <= 450) {
              document.documentElement.style.setProperty('--home-left-col-width', `${savedLeftW}px`);
            }
            const savedRightW = parseInt(localStorage.getItem('controlab_desktop_right_col_width'), 10);
            if (savedRightW && savedRightW >= 200 && savedRightW <= 500) {
              document.documentElement.style.setProperty('--home-right-col-width', `${savedRightW}px`);
            }
          } catch {}

          // 1. Splitter Console (Horizontal / Arraste vertical)
          if (splitterConsole && homeMainContent && consoleArea) {
            let startY = 0;
            let startHeight = 0;
            let isDragging = false;
            let cachedMaxH = 300;

            function onPointerMove(e) {
              if (!isDragging) return;
              const deltaY = startY - e.clientY;
              const newH = Math.min(Math.max(70, startHeight + deltaY), cachedMaxH);
              document.documentElement.style.setProperty('--home-console-height', `${Math.round(newH)}px`);
            }

            function onPointerUp() {
              if (!isDragging) return;
              isDragging = false;
              document.body.classList.remove('is-resizing-splitter-h');
              splitterConsole.classList.remove('active');
              window.removeEventListener('pointermove', onPointerMove);
              window.removeEventListener('pointerup', onPointerUp);
              window.removeEventListener('pointercancel', onPointerUp);

              const curH = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-console-height'), 10);
              if (curH) {
                try { localStorage.setItem('controlab_desktop_console_height', curH); } catch {}
              }
            }

            splitterConsole.addEventListener('pointerdown', (e) => {
              if (!isDesktop() || e.button !== 0) return;
              e.preventDefault();
              isDragging = true;
              startY = e.clientY;
              startHeight = consoleArea.getBoundingClientRect().height;
              cachedMaxH = getMaxConsoleHeight();
              document.body.classList.add('is-resizing-splitter-h');
              splitterConsole.classList.add('active');
              window.addEventListener('pointermove', onPointerMove);
              window.addEventListener('pointerup', onPointerUp);
              window.addEventListener('pointercancel', onPointerUp);
            });

            splitterConsole.addEventListener('dblclick', () => {
              if (!isDesktop()) return;
              const maxAllowedH = getMaxConsoleHeight();
              const defaultH = Math.min(160, maxAllowedH);
              document.documentElement.style.setProperty('--home-console-height', `${defaultH}px`);
              try { localStorage.removeItem('controlab_desktop_console_height'); } catch {}
              showToast('Console redefinido para a altura padrão.', 'info', 1500);
            });

            splitterConsole.addEventListener('keydown', (e) => {
              if (!isDesktop()) return;
              const step = e.shiftKey ? 30 : 15;
              const curH = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-console-height'), 10) || 160;
              const maxAllowedH = getMaxConsoleHeight();
              if (e.key === 'ArrowUp') {
                e.preventDefault();
                const newH = Math.min(curH + step, maxAllowedH);
                document.documentElement.style.setProperty('--home-console-height', `${newH}px`);
                try { localStorage.setItem('controlab_desktop_console_height', newH); } catch {}
              } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                const newH = Math.max(curH - step, 70);
                document.documentElement.style.setProperty('--home-console-height', `${newH}px`);
                try { localStorage.setItem('controlab_desktop_console_height', newH); } catch {}
              }
            });
          }

          // 2. Splitter Coluna Esquerda (Vertical / Arraste horizontal)
          if (splitterLeft && leftCol) {
            let startX = 0;
            let startWidth = 0;
            let isDragging = false;

            function onPointerMove(e) {
              if (!isDragging) return;
              const deltaX = e.clientX - startX;
              const newW = Math.min(Math.max(140, startWidth + deltaX), 450);
              document.documentElement.style.setProperty('--home-left-col-width', `${Math.round(newW)}px`);
            }

            function onPointerUp() {
              if (!isDragging) return;
              isDragging = false;
              document.body.classList.remove('is-resizing-splitter-v');
              splitterLeft.classList.remove('active');
              window.removeEventListener('pointermove', onPointerMove);
              window.removeEventListener('pointerup', onPointerUp);
              window.removeEventListener('pointercancel', onPointerUp);

              const curW = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-left-col-width'), 10);
              if (curW) {
                try { localStorage.setItem('controlab_desktop_left_col_width', curW); } catch {}
              }
            }

            splitterLeft.addEventListener('pointerdown', (e) => {
              if (!isDesktop() || e.button !== 0) return;
              e.preventDefault();
              isDragging = true;
              startX = e.clientX;
              startWidth = leftCol.getBoundingClientRect().width;
              document.body.classList.add('is-resizing-splitter-v');
              splitterLeft.classList.add('active');
              window.addEventListener('pointermove', onPointerMove);
              window.addEventListener('pointerup', onPointerUp);
              window.addEventListener('pointercancel', onPointerUp);
            });

            splitterLeft.addEventListener('dblclick', () => {
              if (!isDesktop()) return;
              document.documentElement.style.setProperty('--home-left-col-width', '240px');
              try { localStorage.removeItem('controlab_desktop_left_col_width'); } catch {}
              showToast('Coluna esquerda redefinida.', 'info', 1500);
            });

            splitterLeft.addEventListener('keydown', (e) => {
              if (!isDesktop()) return;
              const step = e.shiftKey ? 30 : 15;
              const curW = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-left-col-width'), 10) || 240;
              if (e.key === 'ArrowRight') {
                e.preventDefault();
                const newW = Math.min(curW + step, 450);
                document.documentElement.style.setProperty('--home-left-col-width', `${newW}px`);
                try { localStorage.setItem('controlab_desktop_left_col_width', newW); } catch {}
              } else if (e.key === 'ArrowLeft') {
                e.preventDefault();
                const newW = Math.max(curW - step, 140);
                document.documentElement.style.setProperty('--home-left-col-width', `${newW}px`);
                try { localStorage.setItem('controlab_desktop_left_col_width', newW); } catch {}
              }
            });
          }

          // 3. Splitter Coluna Direita (Vertical / Arraste horizontal)
          if (splitterRight && rightCol) {
            let startX = 0;
            let startWidth = 0;
            let isDragging = false;

            function onPointerMove(e) {
              if (!isDragging) return;
              const deltaX = startX - e.clientX;
              const newW = Math.min(Math.max(200, startWidth + deltaX), 500);
              document.documentElement.style.setProperty('--home-right-col-width', `${Math.round(newW)}px`);
            }

            function onPointerUp() {
              if (!isDragging) return;
              isDragging = false;
              document.body.classList.remove('is-resizing-splitter-v');
              splitterRight.classList.remove('active');
              window.removeEventListener('pointermove', onPointerMove);
              window.removeEventListener('pointerup', onPointerUp);
              window.removeEventListener('pointercancel', onPointerUp);

              const curW = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-right-col-width'), 10);
              if (curW) {
                try { localStorage.setItem('controlab_desktop_right_col_width', curW); } catch {}
              }
            }

            splitterRight.addEventListener('pointerdown', (e) => {
              if (!isDesktop() || e.button !== 0) return;
              e.preventDefault();
              isDragging = true;
              startX = e.clientX;
              startWidth = rightCol.getBoundingClientRect().width;
              document.body.classList.add('is-resizing-splitter-v');
              splitterRight.classList.add('active');
              window.addEventListener('pointermove', onPointerMove);
              window.addEventListener('pointerup', onPointerUp);
              window.addEventListener('pointercancel', onPointerUp);
            });

            splitterRight.addEventListener('dblclick', () => {
              if (!isDesktop()) return;
              document.documentElement.style.setProperty('--home-right-col-width', '300px');
              try { localStorage.removeItem('controlab_desktop_right_col_width'); } catch {}
              showToast('Coluna direita redefinida.', 'info', 1500);
            });

            splitterRight.addEventListener('keydown', (e) => {
              if (!isDesktop()) return;
              const step = e.shiftKey ? 30 : 15;
              const curW = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-right-col-width'), 10) || 300;
              if (e.key === 'ArrowLeft') {
                e.preventDefault();
                const newW = Math.min(curW + step, 500);
                document.documentElement.style.setProperty('--home-right-col-width', `${newW}px`);
                try { localStorage.setItem('controlab_desktop_right_col_width', newW); } catch {}
              } else if (e.key === 'ArrowRight') {
                e.preventDefault();
                const newW = Math.max(curW - step, 200);
                document.documentElement.style.setProperty('--home-right-col-width', `${newW}px`);
                try { localStorage.setItem('controlab_desktop_right_col_width', newW); } catch {}
              }
            });
          }

          window.addEventListener('resize', () => {
            if (!isDesktop()) return;
            const maxAllowedH = getMaxConsoleHeight();
            const curH = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--home-console-height'), 10) || 160;
            if (curH > maxAllowedH) {
              document.documentElement.style.setProperty('--home-console-height', `${maxAllowedH}px`);
              try { localStorage.setItem('controlab_desktop_console_height', maxAllowedH); } catch {}
            }
          });

          window.ControLABHomeSplitters = Object.freeze({
            setConsoleHeight: (h) => {
              const maxAllowedH = getMaxConsoleHeight();
              const clamped = Math.min(Math.max(70, h), maxAllowedH);
              document.documentElement.style.setProperty('--home-console-height', `${clamped}px`);
              try { localStorage.setItem('controlab_desktop_console_height', clamped); } catch {}
            },
            setLeftColWidth: (w) => document.documentElement.style.setProperty('--home-left-col-width', `${w}px`),
            setRightColWidth: (w) => document.documentElement.style.setProperty('--home-right-col-width', `${w}px`),
            resetAll: () => {
              const maxAllowedH = getMaxConsoleHeight();
              const defaultH = Math.min(160, maxAllowedH);
              document.documentElement.style.setProperty('--home-console-height', `${defaultH}px`);
              document.documentElement.style.setProperty('--home-left-col-width', '240px');
              document.documentElement.style.setProperty('--home-right-col-width', '300px');
              try {
                localStorage.removeItem('controlab_desktop_console_height');
                localStorage.removeItem('controlab_desktop_left_col_width');
                localStorage.removeItem('controlab_desktop_right_col_width');
              } catch {}
            },
          });
        }

        setupHomeSplitPanes();

        // Sincroniza estado inicial via backend
        refreshHomeState();
      }

    setupHomeScreen();

    const initialHash = (window.location.hash || '').replace('#', '').trim();
    if (['time', 'routh', 'lgr', 'frequency', 'controllers', 'state-space'].includes(initialHash)) {
      navigateTo(initialHash);
      try {
        const params = new URLSearchParams(window.location.search);
        if (params.get('theme')) {
          document.documentElement.setAttribute('data-theme', params.get('theme'));
        }
        if (params.get('tab')) {
          const tabKey = params.get('tab');
          document.querySelector(`#page-${initialHash} .sci-mobile-tab[data-tab="${tabKey}"], #sci-mobile-tabs [data-tab="${tabKey}"]`)?.click();
          document.querySelector(`#page-${initialHash} .sci-tablet-switcher [data-view="${tabKey}"], #sci-tablet-switcher [data-view="${tabKey}"]`)?.click();
        }
        if (params.get('calc') === '1') {
          const delayMs = parseInt(params.get('delay') || '1600', 10);
          if (typeof Image !== 'undefined') {
            const delayImg = new Image();
            delayImg.src = `/api/delay?ms=${delayMs}`;
            delayImg.style.display = 'none';
            document.body.appendChild(delayImg);
          }
          if (initialHash === 'time') optionalElement('btn-run-scientific')?.click();
          if (initialHash === 'routh') optionalElement('btn-calculate-routh')?.click();
          if (initialHash === 'lgr') optionalElement('btn-calculate')?.click();
          if (params.get('tab')) {
            const tabKey = params.get('tab');
            window.setTimeout(() => {
              document.querySelector(`#page-${initialHash} .sci-mobile-tab[data-tab="${tabKey}"], #sci-mobile-tabs [data-tab="${tabKey}"]`)?.click();
              document.querySelector(`#page-${initialHash} .sci-tablet-switcher [data-view="${tabKey}"], #sci-tablet-switcher [data-view="${tabKey}"]`)?.click();
            }, 700);
          }
        }
      } catch (e) {}
    } else {
      navigateTo('home');
    }
    return Object.freeze({ navigateTo });
  }

  global.LGRNavigation = Object.freeze({ initialize: initializeNavigation });
}(window));
