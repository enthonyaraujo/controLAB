/**
 * ControLAB - Configurações, Atualizações e Informações do Repositório
 */

(function registerSettings(global) {
  const CURRENT_VERSION = '1.2.0';
  const GITHUB_REPO_URL = 'https://github.com/enthonyaraujo/lgr';
  const GITHUB_RELEASES_API = 'https://api.github.com/repos/enthonyaraujo/lgr/releases/latest';
  const STORAGE_TOKEN_KEY = 'controlab_github_token';

  function optionalElement(id) {
    return document.getElementById(id);
  }

  function detectPlatform() {
    if (global.Capacitor?.isNativePlatform?.()) return 'android';
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('android')) return 'android';
    if (ua.includes('win')) return 'windows';
    if (ua.includes('linux')) return 'linux';
    if (ua.includes('mac')) return 'mac';
    return 'web';
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '';
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  function formatDate(isoString) {
    if (!isoString) return '';
    try {
      const date = new Date(isoString);
      return date.toLocaleDateString('pt-BR', { day: '2-digit', month: '2-digit', year: 'numeric' });
    } catch {
      return '';
    }
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

  function getStoredToken() {
    try {
      return localStorage.getItem(STORAGE_TOKEN_KEY) || '';
    } catch {
      return '';
    }
  }

  function setStoredToken(token) {
    try {
      if (token) {
        localStorage.setItem(STORAGE_TOKEN_KEY, token.trim());
      } else {
        localStorage.removeItem(STORAGE_TOKEN_KEY);
      }
    } catch {
      // localStorage pode estar indisponível
    }
  }

  function renderAssetsList(assets, currentPlatform) {
    if (!assets || assets.length === 0) {
      return '<p class="no-assets-msg">Nenhum executável anexado a este lançamento.</p>';
    }

    const items = assets.map((asset) => {
      const name = asset.name || '';
      const sizeStr = formatBytes(asset.size);
      const isRecommended = (
        (currentPlatform === 'linux' && (name.endsWith('.AppImage') || name.endsWith('.deb'))) ||
        (currentPlatform === 'windows' && name.endsWith('.exe')) ||
        (currentPlatform === 'android' && name.endsWith('.apk'))
      );

      let label = name;
      if (name.endsWith('.AppImage')) label = `Linux Universal (.AppImage) — ${sizeStr}`;
      else if (name.endsWith('.deb')) label = `Debian / Ubuntu (.deb) — ${sizeStr}`;
      else if (name.endsWith('.rpm')) label = `Fedora / RedHat (.rpm) — ${sizeStr}`;
      else if (name.endsWith('.exe')) label = `Windows 64-bit (.exe) — ${sizeStr}`;
      else if (name.endsWith('.apk')) label = `Android Nativo (.apk) — ${sizeStr}`;

      return `
        <div class="asset-download-item ${isRecommended ? 'recommended-asset' : ''}">
          <div class="asset-info">
            <span class="asset-name">${label}</span>
            ${isRecommended ? '<span class="badge-recommended">Recomendado</span>' : ''}
          </div>
          <button class="btn-asset-dl" data-url="${asset.browser_download_url}" type="button">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            <span>Baixar</span>
          </button>
        </div>
      `;
    }).join('');

    return `<div class="assets-download-grid">${items}</div>`;
  }

  function initializeSettings() {
    const modalSettings = optionalElement('modal-settings');
    const btnOpenSettings = optionalElement('btn-open-settings');
    const btnCloseSettings = optionalElement('btn-close-settings');
    const btnCheckUpdates = optionalElement('btn-check-updates');
    const updateResultBox = optionalElement('update-result-box');
    const btnCheckText = optionalElement('btn-check-updates-text');
    const iconRefresh = optionalElement('icon-update-refresh');

    const inputToken = optionalElement('input-github-token');
    const btnSaveToken = optionalElement('btn-save-token');
    const btnClearToken = optionalElement('btn-clear-token');
    const tokenFeedback = optionalElement('token-feedback-msg');

    // Carregar token previamente salvo
    if (inputToken) {
      const saved = getStoredToken();
      if (saved) {
        inputToken.value = saved;
        if (tokenFeedback) {
          tokenFeedback.textContent = 'Token ativo no armazenamento local.';
          tokenFeedback.className = 'token-feedback active';
        }
      }
    }

    btnSaveToken?.addEventListener('click', () => {
      const val = inputToken ? inputToken.value.trim() : '';
      if (!val) {
        if (tokenFeedback) {
          tokenFeedback.textContent = 'Digite um token válido antes de salvar.';
          tokenFeedback.className = 'token-feedback error';
        }
        return;
      }
      setStoredToken(val);
      if (tokenFeedback) {
        tokenFeedback.textContent = 'Token salvo localmente com sucesso.';
        tokenFeedback.className = 'token-feedback active';
      }
    });

    btnClearToken?.addEventListener('click', () => {
      setStoredToken('');
      if (inputToken) inputToken.value = '';
      if (tokenFeedback) {
        tokenFeedback.textContent = 'Token removido. Usando acesso anônimo público.';
        tokenFeedback.className = 'token-feedback';
      }
    });

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

      const headers = { Accept: 'application/vnd.github.v3+json' };
      const token = getStoredToken();
      if (token) {
        headers.Authorization = `Bearer ${token}`;
      }

      const platform = detectPlatform();

      try {
        let release = null;
        if (global.api?.checkUpdates) {
          const result = await global.api.checkUpdates(token);
          if (!result.success) {
            if (result.status === 403) {
              throw new Error('Limite de requisições da API atingido. Configure um GitHub Token abaixo.');
            }
            throw new Error(result.error || `Erro HTTP ${result.status}`);
          }
          release = result.release;
        } else {
          const response = await fetch(GITHUB_RELEASES_API, { headers });
          if (!response.ok) {
            if (response.status === 403) {
              throw new Error('Limite de requisições da API atingido. Configure um GitHub Token abaixo.');
            }
            throw new Error(`O GitHub retornou HTTP ${response.status}`);
          }
          release = await response.json();
        }
        const latestTag = (release.tag_name || '').replace(/^v/, '');
        const releaseUrl = release.html_url || `${GITHUB_REPO_URL}/releases`;
        const publishDate = formatDate(release.published_at);
        const assets = release.assets || [];

        const isNewer = latestTag && compareSemver(latestTag, CURRENT_VERSION) > 0;

        if (isNewer) {
          updateResultBox.className = 'update-result-box update-available';
          updateResultBox.innerHTML = `
            <div class="update-msg-title">Nova versão disponível: v${latestTag} ${publishDate ? `(${publishDate})` : ''}</div>
            <p>Uma versão mais recente do ControLAB está pronta para download.</p>
            <div class="release-assets-section">
              <span class="assets-section-title">Instaladores disponíveis:</span>
              ${renderAssetsList(assets, platform)}
            </div>
            <div class="release-footer-links">
              <a href="${releaseUrl}" class="external-link-row" target="_blank" rel="noopener noreferrer">
                <span>Ver notas da versão no GitHub &rarr;</span>
              </a>
            </div>
          `;
        } else {
          updateResultBox.className = 'update-result-box up-to-date';
          updateResultBox.innerHTML = `
            <div class="update-msg-title">Você já está com a versão mais recente (v${CURRENT_VERSION})!</div>
            <p>Seu aplicativo está atualizado com as últimas melhorias de engenharia e controle.</p>
            ${assets.length > 0 ? `
              <div class="release-assets-section">
                <span class="assets-section-title">Instaladores da versão atual (v${CURRENT_VERSION}):</span>
                ${renderAssetsList(assets, platform)}
              </div>
            ` : ''}
          `;
        }

        // Adicionar eventos de download nos botões de asset
        updateResultBox.querySelectorAll('.btn-asset-dl').forEach((btn) => {
          btn.addEventListener('click', (ev) => {
            ev.preventDefault();
            const downloadUrl = btn.dataset.url;
            if (downloadUrl) {
              openExternal(downloadUrl);
            }
          });
        });

        updateResultBox.querySelectorAll('a[href^="http"]').forEach((link) => {
          link.addEventListener('click', (ev) => {
            ev.preventDefault();
            openExternal(link.getAttribute('href'));
          });
        });

      } catch (error) {
        updateResultBox.className = 'update-result-box update-fallback';
        updateResultBox.innerHTML = `
          <div class="update-msg-title">Não foi possível consultar a API do GitHub</div>
          <p>${error.message || 'Verifique sua conexão à internet.'}</p>
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
