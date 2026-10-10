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
      });

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

      // Variáveis do workspace atuais
      document.querySelectorAll('.home-workspace-item').forEach((item) => {
        item.addEventListener('click', () => {
          const varName = item.dataset.var;
          if (varName) {
            executeConsoleCommand(varName);
          }
        });
      });

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
