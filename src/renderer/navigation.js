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

    document.querySelectorAll('.module-card.card-soon').forEach((card) => {
      card.addEventListener('click', () => {
        const moduleName = card.dataset.module || 'selecionado';
        showToast(`O módulo "${moduleName}" está em desenvolvimento.`, 'info', 3200);
      });
    });
    document.querySelectorAll('.btn-module-soon').forEach((button) => {
      button.addEventListener('click', (event) => {
        event.stopPropagation();
        const moduleName = button.dataset.module || 'selecionado';
        showToast(`O módulo "${moduleName}" está em desenvolvimento.`, 'info', 3200);
      });
    });

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
