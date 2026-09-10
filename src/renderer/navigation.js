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

      pageHome?.classList.toggle('active', isHome);
      pageLgr?.classList.toggle('active', isLgr);
      pageRouth?.classList.toggle('active', isRouth);
      pageScientific?.classList.toggle('active', isScientific);

      const hasBack = !isHome;
      if (btnNavHome) btnNavHome.style.display = hasBack ? 'inline-flex' : 'none';
      if (globalBrand) globalBrand.style.display = isHome ? 'flex' : 'none';
      if (globalBrandTitle) {
        globalBrandTitle.textContent = isHome ? 'ControLAB' : '';
      }
      if (globalNavbar) globalNavbar.classList.toggle('has-back', hasBack);
      if (globalNavbarCenter) {
        globalNavbarCenter.style.display = 'flex';
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
          globalNavbarTitle.textContent = 'Hub de Módulos';
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

    optionalElement('btn-open-lgr')?.addEventListener('click', (event) => {
      event.stopPropagation();
      navigateTo('lgr');
    });
    optionalElement('card-module-lgr')?.addEventListener('click', () => navigateTo('lgr'));

    optionalElement('btn-open-routh')?.addEventListener('click', (event) => {
      event.stopPropagation();
      navigateTo('routh');
    });
    optionalElement('card-module-routh')?.addEventListener('click', () => navigateTo('routh'));

    document.querySelectorAll('.module-card.card-active[data-page]').forEach((card) => {
      if (['lgr', 'routh'].includes(card.dataset.page)) return;
      const open = () => navigateTo(card.dataset.page);
      card.addEventListener('click', open);
      card.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          open();
        }
      });
      card.querySelector('.btn-module-primary')?.addEventListener('click', (event) => {
        event.stopPropagation();
        open();
      });
    });

    btnNavHome?.addEventListener('click', () => navigateTo('home'));
    optionalElement('btn-sidebar-back-home')?.addEventListener('click', () => navigateTo('home'));

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

    navigateTo('home');
    return Object.freeze({ navigateTo });
  }

  global.LGRNavigation = Object.freeze({ initialize: initializeNavigation });
}(window));
