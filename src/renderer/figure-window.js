(function initializeFigureWindow() {
  'use strict';
  let received = false;
  const timer = setTimeout(() => {
    if (!received) document.getElementById('figure-status').textContent = 'Não foi possível carregar a figura. Abra-a novamente pelo editor.';
  }, 12000);
  function show(data) {
    if (received || !data?.plot?.series) return;
    received = true; clearTimeout(timer);
    document.documentElement.dataset.theme = data.theme === 'light' ? 'light' : 'dark';
    const title = data.plot.title || 'Figura';
    document.title = `${title} — ControLAB`;
    document.getElementById('figure-title').textContent = title;
    window.ControLABFigures.render(document.getElementById('figure-content'), [data.plot]);
  }
  document.getElementById('figure-close').addEventListener('click', () => {
    window.close();
    // A page that was not opened as a pop-up (e.g. inside the Android WebView) cannot close itself.
    setTimeout(() => {
      if (!window.closed) {
        if (history.length > 1) history.back(); else location.replace('index.html');
      }
    }, 200);
  });
  if (window.figureApi) {
    window.figureApi.getData().then(show).catch(() => {});
  } else {
    const id = new URLSearchParams(location.search).get('id');
    const listener = (event) => {
      if (event.origin !== location.origin || event.source !== window.opener || event.data?.type !== 'controlab-figure-data' || event.data.id !== id) return;
      show(event.data);
      window.removeEventListener('message', listener);
      window.opener = null;
    };
    window.addEventListener('message', listener);
    window.opener?.postMessage({ type: 'controlab-figure-ready', id }, location.origin);
  }
})();
