/**
 * ControLAB - Configurações, Atualizações e Informações do Repositório
 */

(function registerSettings(global) {
  const CURRENT_VERSION = '1.2.0';
  const GITHUB_REPO_URL = 'https://github.com/enthonyaraujo/lgr';
  const GITHUB_RELEASES_API = 'https://api.github.com/repos/enthonyaraujo/lgr/releases/latest';

  function optionalElement(id) {
    return document.getElementById(id);
  }

  function compareSemver(v1, v2) {
    const clean1 = (v1 || '0').replace(/^v/, '').split('.').map((n) => parseInt(n, 10) || 0);
    const clean2 = (v2 || '0').replace(/^v/, '').split('.').map((n) => parseInt(n, 10) || 0);
    const maxLen = Math.max(clean1.length, clean2.length);

    for (let i = 0; i < maxLen; i += 1) {
      const part1 = clean1[i] || 0;
      const part2 = clean2[i] || 0;
      if (part1 > part2) return 1;
      if (part1 < part2) return -1;
    }
    return 0;
  }

  function openExternal(url) {
    if (!url) return;
    if (global.api?.openExternal) {
      global.api.openExternal(url);
    } else {
      window.open(url, '_blank', 'noopener,noreferrer');
    }
  }

  function initializeSettings() {
    const modalSettings = optionalElement('modal-settings');
    const btnOpenSettings = optionalElement('btn-open-settings');
    const btnCloseSettings = optionalElement('btn-close-settings');
    const btnCheckUpdates = optionalElement('btn-check-updates');
    const updateResultBox = optionalElement('update-result-box');
    const btnCheckText = optionalElement('btn-check-updates-text');
    const iconRefresh = optionalElement('icon-update-refresh');

    function openModal() {
      if (!modalSettings) return;
      modalSettings.classList.add('active');
      modalSettings.setAttribute('aria-hidden', 'false');
      document.body.style.overflow = 'hidden';
    }

    function closeModal() {
      if (!modalSettings) return;
      modalSettings.classList.remove('active');
      modalSettings.setAttribute('aria-hidden', 'true');
      document.body.style.overflow = '';
    }

    btnOpenSettings?.addEventListener('click', openModal);
    btnCloseSettings?.addEventListener('click', closeModal);

    modalSettings?.addEventListener('click', (event) => {
      if (event.target === modalSettings) {
        closeModal();
      }
    });

    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape' && modalSettings?.classList.contains('active')) {
        closeModal();
      }
    });

    modalSettings?.querySelectorAll('a[href^="http"]').forEach((link) => {
      link.addEventListener('click', (event) => {
        event.preventDefault();
        openExternal(link.getAttribute('href'));
      });
    });

    async function checkForUpdates() {
      if (!updateResultBox) return;

      updateResultBox.style.display = 'block';
      updateResultBox.className = 'update-result-box loading';
      updateResultBox.innerHTML = '<span>Consultando os lançamentos mais recentes no GitHub...</span>';

      if (btnCheckText) btnCheckText.textContent = 'Verificando...';
      if (iconRefresh) iconRefresh.classList.add('spinning');
      if (btnCheckUpdates) btnCheckUpdates.disabled = true;

      try {
        const response = await fetch(GITHUB_RELEASES_API, {
          headers: { Accept: 'application/vnd.github.v3+json' },
        });

        if (!response.ok) {
          throw new Error(`O GitHub retornou HTTP ${response.status}`);
        }

        const release = await response.json();
        const latestTag = (release.tag_name || '').replace(/^v/, '');
        const releaseUrl = release.html_url || `${GITHUB_REPO_URL}/releases`;

        if (latestTag && compareSemver(latestTag, CURRENT_VERSION) > 0) {
          updateResultBox.className = 'update-result-box update-available';
          updateResultBox.innerHTML = `
            <div class="update-msg-title">Nova versão disponível: v${latestTag}</div>
            <p>Uma versão mais recente do ControLAB já está disponível para download.</p>
            <a href="${releaseUrl}" class="btn-download-update" data-external="true">
              <span>Baixar Versão v${latestTag} no GitHub</span>
            </a>
          `;
          updateResultBox.querySelector('a')?.addEventListener('click', (ev) => {
            ev.preventDefault();
            openExternal(releaseUrl);
          });
        } else {
          updateResultBox.className = 'update-result-box up-to-date';
          updateResultBox.innerHTML = `
            <div class="update-msg-title">Você já está na versão mais recente!</div>
            <p>A versão <strong>v${CURRENT_VERSION}</strong> instalada no seu dispositivo é a mais atual.</p>
          `;
        }
      } catch (error) {
        updateResultBox.className = 'update-result-box update-fallback';
        updateResultBox.innerHTML = `
          <div class="update-msg-title">Não foi possível consultar a API do GitHub agora</div>
          <p>Verifique sua conexão à internet ou acesse a página de releases diretamente.</p>
          <a href="${GITHUB_REPO_URL}/releases" class="btn-download-update" data-external="true">
            <span>Acessar Releases no GitHub</span>
          </a>
        `;
        updateResultBox.querySelector('a')?.addEventListener('click', (ev) => {
          ev.preventDefault();
          openExternal(`${GITHUB_REPO_URL}/releases`);
        });
      } finally {
        if (btnCheckText) btnCheckText.textContent = 'Verificar Novamente';
        if (iconRefresh) iconRefresh.classList.remove('spinning');
        if (btnCheckUpdates) btnCheckUpdates.disabled = false;
      }
    }

    btnCheckUpdates?.addEventListener('click', checkForUpdates);

    return Object.freeze({
      open: openModal,
      close: closeModal,
      checkForUpdates,
      currentVersion: CURRENT_VERSION,
    });
  }

  global.ControLABSettings = Object.freeze({ initialize: initializeSettings });
}(window));
