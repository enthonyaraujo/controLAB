(function registerNavigation(global) {
  function optionalElement(id) {
    return document.getElementById(id);
  }

  function initializeNavigation({ renderMathInContainer, showToast, onOpenLgr }) {
    const pageHome = optionalElement('page-home');
    const pageLgr = optionalElement('page-lgr');
    const btnNavHome = optionalElement('btn-nav-home');
    const navModulePill = optionalElement('nav-module-pill');
    const navModuleName = optionalElement('nav-module-name');
    const renderedPages = new WeakSet();

    function renderPageOnce(page) {
      if (!page || renderedPages.has(page)) return;
      renderMathInContainer(page);
      renderedPages.add(page);
    }

    function navigateTo(pageId) {
      const opensLgr = pageId === 'lgr';
      pageHome?.classList.toggle('active', !opensLgr);
      pageLgr?.classList.toggle('active', opensLgr);

      if (btnNavHome) btnNavHome.style.display = opensLgr ? 'inline-flex' : 'none';
      if (navModulePill) navModulePill.style.display = opensLgr ? 'inline-flex' : 'none';
      if (opensLgr && navModuleName) {
        navModuleName.textContent = 'Lugar Geométrico das Raízes (LGR)';
      }

      if (opensLgr) {
        onOpenLgr();
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
    btnNavHome?.addEventListener('click', () => navigateTo('home'));
    optionalElement('btn-sidebar-back-home')?.addEventListener('click', () => navigateTo('home'));

    const globalBrand = optionalElement('global-brand');
    globalBrand?.addEventListener('click', () => navigateTo('home'));
    globalBrand?.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        navigateTo('home');
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

    navigateTo('home');
    return Object.freeze({ navigateTo });
  }

  global.LGRNavigation = Object.freeze({ initialize: initializeNavigation });
}(window));
