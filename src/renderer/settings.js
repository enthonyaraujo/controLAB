/**
 * ControLAB - Configurações, Atualizações e Informações do Repositório
 */

(function registerSettings(global) {
  const CURRENT_VERSION = '3.0.2';
  const GITHUB_REPO_URL = 'https://github.com/enthonyaraujo/controlab';
  const GITHUB_RELEASES_API = 'https://api.github.com/repos/enthonyaraujo/controlab/releases/latest';

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

  function formatSpeed(bytesPerSec) {
    if (!bytesPerSec || bytesPerSec <= 0) return '';
    const mb = bytesPerSec / (1024 * 1024);
    if (mb >= 1) return `${mb.toFixed(1)} MB/s`;
    const kb = bytesPerSec / 1024;
    return `${kb.toFixed(0)} KB/s`;
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

  try {
    localStorage.removeItem('controlab_github_token');
  } catch {
    // Ignora se localStorage indisponível
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
        <div class="target-package-card" id="target-package-card">
          <div class="target-package-info">
            <span class="target-pkg-badge">Detectado para seu sistema</span>
            <div class="target-pkg-title">${systemInfo.osName} • ${systemInfo.packageLabel}</div>
            <div class="target-pkg-filename">${targetAsset.name} (${formatBytes(targetAsset.size)})</div>
          </div>
          <div class="target-pkg-action-wrap" id="target-pkg-action-wrap">
            <button class="btn-download-primary btn-inapp-update" id="btn-inapp-update" data-url="${targetAsset.browser_download_url}" data-filename="${targetAsset.name}" data-size="${targetAsset.size || 0}" type="button">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
              <span>Atualizar no Aplicativo</span>
            </button>
          </div>
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

  function initializeSettings(options = {}) {
    const showToast = options?.showToast;
    const modalSettings = optionalElement('modal-settings');
    const btnOpenSettings = optionalElement('btn-open-settings');
    const btnCloseSettings = optionalElement('btn-close-settings');
    const btnCheckUpdates = optionalElement('btn-check-updates');
    const updateResultBox = optionalElement('update-result-box');
    const btnCheckText = optionalElement('btn-check-updates-text');
    const iconRefresh = optionalElement('icon-update-refresh');
    const detectedEnvBadge = optionalElement('detected-env-badge');
    const updateIndicatorDot = optionalElement('update-indicator-dot');

    let currentSystemInfo = {
      os: 'unknown',
      osName: 'Detectando...',
      packageType: 'web',
      packageLabel: 'Sistema',
    };

    // Resolver detecção do sistema operacional e verificar atualizações automaticamente na inicialização
    resolveSystemInfo().then((info) => {
      currentSystemInfo = info;
      if (detectedEnvBadge) {
        detectedEnvBadge.textContent = `${info.osName} • ${info.packageLabel}`;
        detectedEnvBadge.title = `Ambiente detectado: ${info.osName} (${info.packageLabel})`;
      }
      performStartupUpdateCheck();
    }).catch(() => {
      performStartupUpdateCheck();
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

      const btnInappUpdate = updateResultBox.querySelector('#btn-inapp-update');
      const targetPkgCard = updateResultBox.querySelector('#target-package-card');
      const targetActionWrap = updateResultBox.querySelector('#target-pkg-action-wrap');

      if (btnInappUpdate && targetPkgCard && targetActionWrap) {
        btnInappUpdate.addEventListener('click', async (ev) => {
          ev.preventDefault();

          if (!global.api?.downloadUpdatePackage) {
            const url = btnInappUpdate.dataset.url;
            if (url) openExternal(url);
            return;
          }

          const url = btnInappUpdate.dataset.url;
          const filename = btnInappUpdate.dataset.filename;
          const expectedSize = parseInt(btnInappUpdate.dataset.size || '0', 10);

          targetPkgCard.classList.add('in-progress');
          targetActionWrap.innerHTML = `
            <div class="inapp-progress-wrap">
              <div class="inapp-progress-header">
                <span class="inapp-progress-title">Baixando atualizacao...</span>
                <span class="inapp-progress-percent" id="inapp-progress-percent">0%</span>
              </div>
              <div class="inapp-progress-track">
                <div class="inapp-progress-fill" id="inapp-progress-fill" style="width: 0%"></div>
              </div>
              <div class="inapp-progress-footer">
                <span class="inapp-progress-stats" id="inapp-progress-stats">Iniciando download...</span>
                <button class="btn-inapp-cancel" id="btn-inapp-cancel" type="button">Cancelar</button>
              </div>
            </div>
          `;

          const progressFill = targetActionWrap.querySelector('#inapp-progress-fill');
          const progressPercent = targetActionWrap.querySelector('#inapp-progress-percent');
          const progressStats = targetActionWrap.querySelector('#inapp-progress-stats');
          const btnCancel = targetActionWrap.querySelector('#btn-inapp-cancel');

          btnCancel?.addEventListener('click', async () => {
            if (global.api?.cancelUpdateDownload) {
              await global.api.cancelUpdateDownload();
            }
          });

          const unsubscribe = global.api.onUpdateDownloadProgress ? global.api.onUpdateDownloadProgress((prog) => {
            if (prog.percent != null && progressFill && progressPercent) {
              progressFill.style.width = `${prog.percent}%`;
              progressPercent.textContent = `${prog.percent}%`;
            }
            if (progressStats) {
              const speedText = prog.bytesPerSecond > 0 ? ` • ${formatSpeed(prog.bytesPerSecond)}` : '';
              progressStats.textContent = `${formatBytes(prog.transferred)} de ${formatBytes(prog.total || expectedSize)}${speedText}`;
            }
          }) : null;

          try {
            const res = await global.api.downloadUpdatePackage({
              url,
              filename,
              expectedSize,
            });

            unsubscribe?.();

            if (res && res.success && res.filePath) {
              const downloadedPath = res.filePath;
              targetPkgCard.classList.remove('in-progress');
              targetPkgCard.classList.add('install-ready');

              targetActionWrap.innerHTML = `
                <div class="inapp-install-ready-wrap">
                  <div class="inapp-install-header">
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                      <polyline points="22 4 12 14.01 9 11.01"/>
                    </svg>
                    <div>
                      <div class="inapp-install-title">Download Concluido com Sucesso</div>
                      <div class="inapp-install-subtitle">${filename} pronto para ser instalado.</div>
                    </div>
                  </div>
                  <div class="inapp-install-actions">
                    <button class="btn-install-execute" id="btn-install-execute" type="button">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="7 10 12 15 17 10"/>
                        <line x1="12" y1="15" x2="12" y2="3"/>
                      </svg>
                      <span>${currentSystemInfo?.os === 'android' ? 'Instalar Atualizacao' : 'Instalar e Reiniciar'}</span>
                    </button>
                    ${currentSystemInfo?.os === 'android' ? '' : `
                    <button class="btn-open-folder" id="btn-open-folder" type="button" title="Abrir pasta onde o arquivo foi baixado">
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                      </svg>
                      <span>Abrir Pasta</span>
                    </button>
                    `}
                  </div>
                  <div class="inapp-install-hint" id="inapp-install-hint"></div>
                </div>
              `;

              const btnInstallExecute = targetActionWrap.querySelector('#btn-install-execute');
              const btnOpenFolder = targetActionWrap.querySelector('#btn-open-folder');
              const installHint = targetActionWrap.querySelector('#inapp-install-hint');

              btnOpenFolder?.addEventListener('click', () => {
                global.api.openUpdateFolder?.(downloadedPath);
              });

              btnInstallExecute?.addEventListener('click', async () => {
                btnInstallExecute.disabled = true;
                btnInstallExecute.innerHTML = `
                  <svg class="spinning" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="23 4 23 10 17 10"/>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
                  </svg>
                  <span>Iniciando Instalador...</span>
                `;
                if (installHint) {
                  if (currentSystemInfo?.os === 'android') {
                    installHint.textContent = 'Aguarde. O instalador de pacotes do Android sera aberto na tela.';
                  } else {
                    installHint.textContent = 'Aguarde. Se solicitado pelo sistema operacional, confirme a autorizacao na tela.';
                  }
                }

                const installRes = await global.api.installUpdatePackage({ filePath: downloadedPath });
                if (!installRes || !installRes.success) {
                  btnInstallExecute.disabled = false;
                  btnInstallExecute.innerHTML = `
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                      <polyline points="7 10 12 15 17 10"/>
                      <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    <span>Tentar Instalar Novamente</span>
                  `;
                  if (installHint) {
                    if (currentSystemInfo?.os === 'android') {
                      installHint.textContent = `Instalacao nao iniciou: ${installRes?.error || 'Acao cancelada'}. Verifique as permissoes de instalacao de fontes desconhecidas do aplicativo.`;
                    } else {
                      installHint.textContent = `Instalacao nao concluiu: ${installRes?.error || 'Acao cancelada'}. Voce pode clicar em "Abrir Pasta" para executar manualmente.`;
                    }
                  }
                } else if (installRes.fallbackNote) {
                  if (installHint) {
                    installHint.textContent = installRes.fallbackNote;
                  }
                }
              });
            } else if (res && res.canceled) {
              targetPkgCard.classList.remove('in-progress');
              targetActionWrap.innerHTML = `
                <button class="btn-download-primary btn-inapp-update" id="btn-inapp-update" data-url="${url}" data-filename="${filename}" data-size="${expectedSize}" type="button">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7 10 12 15 17 10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                  </svg>
                  <span>Atualizar no Aplicativo</span>
                </button>
              `;
              bindUpdateBoxEvents();
            } else {
              targetPkgCard.classList.remove('in-progress');
              targetActionWrap.innerHTML = `
                <div class="inapp-error-box">
                  <span class="inapp-error-msg">Nao foi possivel baixar: ${res?.error || 'Erro desconhecido'}.</span>
                  <button class="btn-action-primary btn-asset-dl" data-url="${url}" type="button">Baixar pelo Navegador</button>
                </div>
              `;
              bindUpdateBoxEvents();
            }
          } catch (err) {
            unsubscribe?.();
            targetPkgCard.classList.remove('in-progress');
            targetActionWrap.innerHTML = `
              <div class="inapp-error-box">
                <span class="inapp-error-msg">Falha no download: ${err.message}.</span>
                <button class="btn-action-primary btn-asset-dl" data-url="${url}" type="button">Baixar pelo Navegador</button>
              </div>
            `;
            bindUpdateBoxEvents();
          }
        });
      }

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

    async function performStartupUpdateCheck() {
      try {
        let release = null;
        if (global.api?.checkUpdates) {
          const result = await global.api.checkUpdates();
          if (result && result.success) {
            release = result.release;
            if (result.systemInfo) currentSystemInfo = result.systemInfo;
          }
        } else {
          const headers = { Accept: 'application/vnd.github.v3+json' };
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
            if (showToast) {
              showToast(`Nova versão (v${latestTag}) disponível! Clique para atualizar.`, 'info', 7000);
            }
          }
        }
      } catch {
        // Checagem de inicialização silenciosa em caso de dispositivo offline
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

      try {
        let release = null;
        if (global.api?.checkUpdates) {
          const result = await global.api.checkUpdates();
          if (!result.success) {
            if (result.status === 403) {
              throw new Error('Limite de requisições da API do GitHub atingido temporariamente. Tente novamente mais tarde.');
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
          const response = await fetch(GITHUB_RELEASES_API, { headers });
          if (!response.ok) {
            if (response.status === 403) {
              throw new Error('Limite de requisições da API do GitHub atingido temporariamente. Tente novamente mais tarde.');
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
