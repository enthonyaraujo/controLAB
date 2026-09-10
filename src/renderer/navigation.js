(function registerNavigation(global) {
  function optionalElement(id) {
    return document.getElementById(id);
  }

  function initializeNavigation({ renderMathInContainer, showToast, onOpenLgr, onOpenRouth }) {
    const pageHome = optionalElement('page-home');
    const pageLgr = optionalElement('page-lgr');
    const pageRouth = optionalElement('page-routh');
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

      pageHome?.classList.toggle('active', isHome);
      pageLgr?.classList.toggle('active', isLgr);
      pageRouth?.classList.toggle('active', isRouth);

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
        } else {
          globalNavbarTitle.textContent = 'Hub de Módulos';
        }
      }

      if (isLgr) {
        onOpenLgr();
      } else if (isRouth) {
        if (onOpenRouth) onOpenRouth();
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
    optionalElement('btn-routh-back-home')?.addEventListener('click', () => navigateTo('home'));

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
