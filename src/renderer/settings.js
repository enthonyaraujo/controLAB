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
    const ua = (navigator.userAgent || '').toLowerCase();
    if (ua.includes('android')) return 'android';
    if (ua.includes('win')) return 'windows';
    if (ua.includes('linux')) return 'linux';
    if (ua.includes('mac')) return 'mac';
    return 'web';
  }

  async function resolveSystemInfo() {
    if (global.api?.getSystemInfo) {
      try {
        const info = await global.api.getSystemInfo();
        if (info && info.os) return info;
      } catch {
        // Fallback para detecção no navegador
      }
    }

    if (global.Capacitor?.isNativePlatform?.()) {
      return {
        os: 'android',
        osName: 'Android Nativo',
        packageType: 'apk',
        packageLabel: 'Android (.apk)',
      };
    }

    const ua = (navigator.userAgent || '').toLowerCase();
    if (ua.includes('android')) {
      return {
        os: 'android',
        osName: 'Android',
        packageType: 'apk',
        packageLabel: 'Android (.apk)',
      };
    }
    if (ua.includes('win')) {
      return {
        os: 'windows',
        osName: 'Windows',
        packageType: 'exe',
        packageLabel: 'Instalador Windows (.exe)',
      };
    }
    if (ua.includes('linux')) {
      return {
        os: 'linux',
        osName: 'Linux',
        packageType: 'deb',
        packageLabel: 'Debian / Ubuntu (.deb)',
      };
    }
    if (ua.includes('mac')) {
      return {
        os: 'mac',
        osName: 'macOS',
        packageType: 'dmg',
        packageLabel: 'macOS (.dmg)',
      };
    }

    return {
      os: 'web',
      osName: 'Navegador Web',
      packageType: 'web',
      packageLabel: 'Web',
    };
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

  function findTargetAsset(assets, systemInfo) {
    if (!assets || assets.length === 0) return null;
    const pkg = (systemInfo?.packageType || '').toLowerCase();

    let matched = null;
    if (pkg === 'deb') {
      matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.deb'))
        || assets.find((a) => (a.name || '').toLowerCase().endsWith('.appimage'));
    } else if (pkg === 'appimage') {
      matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.appimage'))
        || assets.find((a) => (a.name || '').toLowerCase().endsWith('.deb'));
    } else if (pkg === 'rpm') {
      matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.rpm'))
        || assets.find((a) => (a.name || '').toLowerCase().endsWith('.appimage'));
    } else if (pkg === 'exe') {
      matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.exe'));
    } else if (pkg === 'apk') {
      matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.apk'));
    }

    if (!matched) {
      const os = (systemInfo?.os || '').toLowerCase();
      if (os === 'linux') {
        matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.deb'))
          || assets.find((a) => (a.name || '').toLowerCase().endsWith('.appimage'))
          || assets.find((a) => (a.name || '').toLowerCase().endsWith('.rpm'));
      } else if (os === 'windows') {
        matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.exe'));
      } else if (os === 'android') {
        matched = assets.find((a) => (a.name || '').toLowerCase().endsWith('.apk'));
      }
    }

    return matched || assets[0] || null;
  }

  function renderAssetItem(asset, isTarget = false) {
    const name = asset.name || '';
    const sizeStr = formatBytes(asset.size);
    let label = name;
    if (name.endsWith('.AppImage')) label = `Linux Universal (.AppImage) - ${sizeStr}`;
    else if (name.endsWith('.deb')) label = `Debian / Ubuntu (.deb) - ${sizeStr}`;
    else if (name.endsWith('.rpm')) label = `Fedora / RedHat (.rpm) - ${sizeStr}`;
    else if (name.endsWith('.exe')) label = `Instalador Windows (.exe) - ${sizeStr}`;
    else if (name.endsWith('.apk')) label = `Android Nativo (.apk) - ${sizeStr}`;

    return `
      <div class="asset-download-item ${isTarget ? 'recommended-asset' : ''}">
        <div class="asset-info">
          <span class="asset-name">${label}</span>
          ${isTarget ? '<span class="badge-recommended">Compatível com seu sistema</span>' : ''}
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
  }

  function renderUpdateAvailableBox(release, systemInfo) {
    const latestTag = (release.tag_name || '').replace(/^v/, '');
    const releaseUrl = release.html_url || `${GITHUB_REPO_URL}/releases`;
    const publishDate = formatDate(release.published_at);
    const assets = release.assets || [];
    const targetAsset = findTargetAsset(assets, systemInfo);
    const otherAssets = targetAsset ? assets.filter((a) => a !== targetAsset) : assets;

    let targetCardHtml = '';
    if (targetAsset) {
      targetCardHtml = `
        <div class="target-package-card">
          <div class="target-package-info">
            <span class="target-pkg-badge">Detectado para seu sistema</span>
            <div class="target-pkg-title">${systemInfo.osName} • ${systemInfo.packageLabel}</div>
            <div class="target-pkg-filename">${targetAsset.name} (${formatBytes(targetAsset.size)})</div>
          </div>
          <button class="btn-download-primary btn-asset-dl" data-url="${targetAsset.browser_download_url}" type="button">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            <span>Baixar Atualização</span>
          </button>
        </div>
      `;
    } else {
      targetCardHtml = `
        <p class="no-assets-msg">Atualização disponível (v${latestTag}), mas nenhum arquivo binário direto para ${systemInfo.packageLabel} foi identificado no GitHub.</p>
        <a href="${releaseUrl}" class="btn-download-update" target="_blank" rel="noopener noreferrer">
          <span>Abrir Release no GitHub &rarr;</span>
        </a>
      `;
    }

    let otherAssetsHtml = '';
    if (otherAssets.length > 0) {
      otherAssetsHtml = `
        <details class="other-assets-details">
          <summary class="other-assets-summary">
            <span>Ver instaladores para outras plataformas (${otherAssets.length})</span>
          </summary>
          <div class="assets-download-grid">
            ${otherAssets.map((a) => renderAssetItem(a, false)).join('')}
          </div>
        </details>
      `;
    }

    return `
      <div class="update-msg-title">Nova versão disponível: v${latestTag} ${publishDate ? `(${publishDate})` : ''}</div>
      <p>Uma versão mais recente do ControLAB está pronta para instalação.</p>
      ${targetCardHtml}
      ${otherAssetsHtml}
      <div class="release-footer-links">
        <a href="${releaseUrl}" class="external-link-row" target="_blank" rel="noopener noreferrer">
          <span>Ver notas da versão no GitHub &rarr;</span>
        </a>
      </div>
    `;
  }

  function renderUpToDateBox(systemInfo) {
    return `
      <div class="update-msg-title">Você já está com a versão mais recente (v${CURRENT_VERSION})!</div>
      <p>Seu sistema (${systemInfo.osName} • ${systemInfo.packageLabel}) está atualizado com as últimas melhorias de engenharia e controle.</p>
    `;
  }

  function initializeSettings() {
    const modalSettings = optionalElement('modal-settings');
    const btnOpenSettings = optionalElement('btn-open-settings');
    const btnCloseSettings = optionalElement('btn-close-settings');
    const btnCheckUpdates = optionalElement('btn-check-updates');
    const updateResultBox = optionalElement('update-result-box');
    const btnCheckText = optionalElement('btn-check-updates-text');
    const iconRefresh = optionalElement('icon-update-refresh');
    const detectedEnvBadge = optionalElement('detected-env-badge');
    const updateIndicatorDot = optionalElement('update-indicator-dot');

    const inputToken = optionalElement('input-github-token');
    const btnSaveToken = optionalElement('btn-save-token');
    const btnClearToken = optionalElement('btn-clear-token');
    const tokenFeedback = optionalElement('token-feedback-msg');

    let currentSystemInfo = {
      os: 'unknown',
      osName: 'Detectando...',
      packageType: 'web',
      packageLabel: 'Sistema',
    };

    // Resolver detecção do sistema operacional e pacote
    resolveSystemInfo().then((info) => {
      currentSystemInfo = info;
      if (detectedEnvBadge) {
        detectedEnvBadge.textContent = `${info.osName} • ${info.packageLabel}`;
        detectedEnvBadge.title = `Ambiente detectado: ${info.osName} (${info.packageLabel})`;
      }
      performSilentCheck();
    });

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

    function bindUpdateBoxEvents() {
      if (!updateResultBox) return;

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
    }

    async function performSilentCheck() {
      try {
        const token = getStoredToken();
        let release = null;
        if (global.api?.checkUpdates) {
          const result = await global.api.checkUpdates(token);
          if (result && result.success) {
            release = result.release;
            if (result.systemInfo) currentSystemInfo = result.systemInfo;
          }
        } else {
          const headers = { Accept: 'application/vnd.github.v3+json' };
          if (token) headers.Authorization = `Bearer ${token}`;
          const response = await fetch(GITHUB_RELEASES_API, { headers });
          if (response.ok) release = await response.json();
        }

        if (release) {
          const latestTag = (release.tag_name || '').replace(/^v/, '');
          const isNewer = latestTag && compareSemver(latestTag, CURRENT_VERSION) > 0;
          if (isNewer) {
            if (updateIndicatorDot) updateIndicatorDot.classList.add('active');
            if (updateResultBox) {
              updateResultBox.style.display = 'block';
              updateResultBox.className = 'update-result-box update-available';
              updateResultBox.innerHTML = renderUpdateAvailableBox(release, currentSystemInfo);
              bindUpdateBoxEvents();
            }
          }
        }
      } catch {
        // Checagem silenciosa sem poluição de UI
      }
    }

    async function checkForUpdates() {
      if (!updateResultBox) return;

      updateResultBox.style.display = 'block';
      updateResultBox.className = 'update-result-box loading';
      updateResultBox.innerHTML = '<span>Consultando os lançamentos mais recentes no GitHub...</span>';

      if (btnCheckText) btnCheckText.textContent = 'Verificando...';
      if (iconRefresh) iconRefresh.classList.add('spinning');
      if (btnCheckUpdates) btnCheckUpdates.disabled = true;

      const token = getStoredToken();

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
          if (result.systemInfo) {
            currentSystemInfo = result.systemInfo;
            if (detectedEnvBadge) {
              detectedEnvBadge.textContent = `${currentSystemInfo.osName} • ${currentSystemInfo.packageLabel}`;
            }
          }
        } else {
          const headers = { Accept: 'application/vnd.github.v3+json' };
          if (token) headers.Authorization = `Bearer ${token}`;
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
        const isNewer = latestTag && compareSemver(latestTag, CURRENT_VERSION) > 0;

        if (isNewer) {
          if (updateIndicatorDot) updateIndicatorDot.classList.add('active');
          updateResultBox.className = 'update-result-box update-available';
          updateResultBox.innerHTML = renderUpdateAvailableBox(release, currentSystemInfo);
        } else {
          if (updateIndicatorDot) updateIndicatorDot.classList.remove('active');
          updateResultBox.className = 'update-result-box up-to-date';
          updateResultBox.innerHTML = renderUpToDateBox(currentSystemInfo);
        }

        bindUpdateBoxEvents();

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
