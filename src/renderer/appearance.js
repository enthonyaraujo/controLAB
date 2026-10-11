(function registerAppearance(global) {
  'use strict';

  const THEME_KEY = 'lgr-theme';
  const SCALE_KEY = 'controlab-ui-scale';
  const THEMES = ['light', 'dark', 'system'];
  const SCALES = [90, 100, 110, 125, 150];
  const root = document.documentElement;
  const media = global.matchMedia ? global.matchMedia('(prefers-color-scheme: dark)') : null;

  function read(key) {
    try { return localStorage.getItem(key); } catch (_) { return null; }
  }

  function write(key, value) {
    try { localStorage.setItem(key, value); } catch (_) { /* armazenamento indisponível */ }
  }

  function getTheme() {
    const saved = read(THEME_KEY);
    return THEMES.includes(saved) ? saved : 'dark';
  }

  function getScale() {
    const saved = parseInt(read(SCALE_KEY), 10);
    return SCALES.includes(saved) ? saved : 100;
  }

  function resolve(mode) {
    if (mode === 'system') return media && !media.matches ? 'light' : 'dark';
    return mode;
  }

  /**
   * Ajusta o estilo dos ícones das barras do sistema com SystemBars.setStyle.
   * - Topo (Status Bar): Fundo teal em ambos os temas (#0B4F61 dark, #0F6E86 light) ->
   *   Requer ícones claros (estilo 'DARK') para manter contraste ideal.
   * - Navegação Inferior (Gesture / Navigation Bar):
   *   Tema escuro -> Fundo escuro -> ícones claros ('DARK').
   *   Tema claro -> Fundo claro -> ícones escuros ('LIGHT').
   */
  function updateSystemBars(effectiveTheme) {
    const applyBars = () => {
      const SystemBars = global.Capacitor?.Plugins?.SystemBars;
      if (!SystemBars?.setStyle) return;

      const isDark = effectiveTheme === 'dark';
      // Barra de status sobre o cabeçalho teal
      SystemBars.setStyle({ style: 'DARK', bar: 'StatusBar' }).catch(() => {});
      // Barra de navegação inferior sobre a barra de abas inferior
      SystemBars.setStyle({ style: isDark ? 'DARK' : 'LIGHT', bar: 'NavigationBar' }).catch(() => {});
    };

    if (global.Capacitor?.isNativePlatform?.()) {
      applyBars();
    } else {
      // Tenta imediatamente e também no DOMContentLoaded / timeout
      applyBars();
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', applyBars, { once: true });
      }
    }
  }

  /**
   * Monitora visualViewport para gerenciar teclado virtual e manter campo G(s)
   * e a fileira de símbolos (^ * / ( ) + - s) sempre visíveis.
   */
  function setupVisualViewportAndKeyboard() {
    function updateViewport() {
      const vv = global.visualViewport;
      const winHeight = global.innerHeight;
      const height = vv ? vv.height : winHeight;

      root.style.setProperty('--visual-viewport-height', `${height}px`);

      // Detecta teclado aberto se a altura visual for significativamente menor que a janela
      const activeEl = document.activeElement;
      const isInputFocused = activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA');
      const isKeyboardOpen = (winHeight - height) > 140 || (isInputFocused && height < winHeight * 0.85);

      if (isKeyboardOpen) {
        document.body.classList.add('keyboard-open');
        root.classList.add('keyboard-open');

        if (isInputFocused) {
          activeEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
      } else {
        document.body.classList.remove('keyboard-open');
        root.classList.remove('keyboard-open');
      }
    }

    if (global.visualViewport) {
      global.visualViewport.addEventListener('resize', updateViewport);
      global.visualViewport.addEventListener('scroll', updateViewport);
    }
    global.addEventListener('resize', updateViewport);

    document.addEventListener('focusin', (e) => {
      if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) {
        setTimeout(updateViewport, 120);
        setTimeout(updateViewport, 300);
      }
    });

    document.addEventListener('focusout', () => {
      setTimeout(updateViewport, 150);
    });

    updateViewport();
  }

  function apply(notify) {
    let urlTheme = null;
    try { urlTheme = new URLSearchParams(global.location.search).get('theme'); } catch (_) { /* sem query */ }
    const effective = urlTheme === 'light' || urlTheme === 'dark' ? urlTheme : resolve(getTheme());
    const changed = root.getAttribute('data-theme') !== effective;
    root.setAttribute('data-theme', effective);
    root.style.setProperty('--ui-scale', String(getScale() / 100));

    updateSystemBars(effective);

    if (notify && changed) {
      global.dispatchEvent(new CustomEvent('controlab:themechange', { detail: { theme: effective } }));
    }
  }

  function setTheme(mode) {
    if (!THEMES.includes(mode)) return;
    write(THEME_KEY, mode);
    apply(true);
  }

  function setScale(percent) {
    const value = parseInt(percent, 10);
    if (!SCALES.includes(value)) return;
    write(SCALE_KEY, String(value));
    apply(false);
  }

  if (media && media.addEventListener) {
    media.addEventListener('change', () => { if (getTheme() === 'system') apply(true); });
  }

  apply(false);

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      apply(false);
      setupVisualViewportAndKeyboard();
    }, { once: true });
  } else {
    setupVisualViewportAndKeyboard();
  }

  global.ControLABAppearance = Object.freeze({
    getTheme,
    getScale,
    setTheme,
    setScale,
    updateSystemBars,
    themes: THEMES,
    scales: SCALES
  });
})(window);
