(function configurePlatformApi() {
  if (window.api) return;

  const nativeLgr = window.Capacitor?.isNativePlatform?.()
    ? window.Capacitor.Plugins?.LgrPython
    : null;

  async function request(action, payload = {}) {
    if (nativeLgr) {
      return nativeLgr.dispatch({ payload: { action, ...payload } });
    }

    const response = await fetch('/api', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action, ...payload }),
    });
    const data = await response.json();
    if (!response.ok && data.success !== false) {
      throw new Error(`Falha HTTP ${response.status}`);
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
  };

  if (!nativeLgr && 'serviceWorker' in navigator && window.location.protocol.startsWith('http')) {
    window.addEventListener('load', () => navigator.serviceWorker.register('/sw.js'));
  }
}());
