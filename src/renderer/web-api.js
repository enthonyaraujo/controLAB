(function configurePlatformApi() {
  if (window.api) return;

  const nativeLgr = window.Capacitor?.isNativePlatform?.()
    ? window.Capacitor.Plugins?.LgrPython
    : null;

  async function readJsonResponse(response) {
    const body = await response.text();
    try {
      return JSON.parse(body);
    } catch {
      throw new Error(`O backend retornou uma resposta inválida (HTTP ${response.status}).`);
    }
  }

  async function request(action, payload = {}) {
    if (nativeLgr) {
      return nativeLgr.dispatch({ payload: { action, ...payload } });
    }

    const response = await fetch('/api', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action, ...payload }),
    });
    const data = await readJsonResponse(response);
    if (!response.ok) {
      throw new Error(data.error || `Falha HTTP ${response.status}.`);
    }
    return data;
  }

  function downloadBlob(blob, defaultName) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = defaultName;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
    return { success: true };
  }

  async function copyImageToClipboard(base64) {
    if (!navigator.clipboard || !window.ClipboardItem) {
      return { success: false, error: 'A cópia exige HTTPS ou localhost neste navegador.' };
    }
    const blob = await (await fetch(base64)).blob();
    await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
    return { success: true };
  }

  window.api = {
    calculateLGR: (payload) => request('calculate', payload),
    previewTransferFunction: (payload) => request('preview', payload),
    getPresets: () => request('presets'),
    saveImage: async ({ base64, defaultName }) => {
      if (nativeLgr) {
        return nativeLgr.saveImage({ base64, defaultName: defaultName || 'lgr_grafico.png' });
      }
      const parts = base64.split(',');
      const binary = atob(parts[1]);
      const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
      return downloadBlob(new Blob([bytes], { type: 'image/png' }), defaultName);
    },
    saveSVG: async ({ svg, defaultName }) => {
      if (nativeLgr) {
        return nativeLgr.saveSVG({ svg, defaultName: defaultName || 'lgr_grafico.svg' });
      }
      return downloadBlob(
        new Blob([svg], { type: 'image/svg+xml;charset=utf-8' }),
        defaultName,
      );
    },
    copyImageToClipboard,
    openExternal: async (url) => {
      window.open(url, '_blank', 'noopener,noreferrer');
      return { success: true };
    },
    checkUpdates: async (token) => {
      const headers = { Accept: 'application/vnd.github.v3+json' };
      if (token) headers.Authorization = `Bearer ${token}`;
      const response = await fetch('https://api.github.com/repos/enthonyaraujo/controlab/releases/latest', { headers });
      if (!response.ok) {
        return { success: false, status: response.status, error: `GitHub retornou status HTTP ${response.status}` };
      }
      const release = await response.json();
      return { success: true, release };
    },
    getSystemInfo: async () => {
      if (global.Capacitor?.isNativePlatform?.()) {
        return {
          os: 'android',
          osName: 'Android Nativo',
          packageType: 'apk',
          packageLabel: 'Android (.apk)',
          arch: 'arm64',
          appVersion: '2.0.0',
        };
      }
      const ua = (navigator.userAgent || '').toLowerCase();
      if (ua.includes('android')) {
        return {
          os: 'android',
          osName: 'Android',
          packageType: 'apk',
          packageLabel: 'Android (.apk)',
          arch: '',
          appVersion: '2.0.0',
        };
      }
      if (ua.includes('win')) {
        return {
          os: 'windows',
          osName: 'Windows',
          packageType: 'exe',
          packageLabel: 'Instalador Windows (.exe)',
          arch: 'x64',
          appVersion: '2.0.0',
        };
      }
      if (ua.includes('linux')) {
        return {
          os: 'linux',
          osName: 'Linux',
          packageType: 'deb',
          packageLabel: 'Debian / Ubuntu (.deb)',
          arch: 'x64',
          appVersion: '2.0.0',
        };
      }
      return {
        os: 'web',
        osName: 'Navegador Web',
        packageType: 'web',
        packageLabel: 'Web',
        arch: '',
        appVersion: '2.0.0',
      };
    },
  };

  if (!nativeLgr && 'serviceWorker' in navigator && window.location.protocol.startsWith('http')) {
    window.addEventListener('load', async () => {
      try {
        const registration = await navigator.serviceWorker.register('/sw.js');
        await registration.update();
      } catch (error) {
        console.warn('Não foi possível atualizar o modo offline:', error);
      }
    });
  }
}());
