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

  function apply(notify) {
    let urlTheme = null;
    try { urlTheme = new URLSearchParams(global.location.search).get('theme'); } catch (_) { /* sem query */ }
    const effective = urlTheme === 'light' || urlTheme === 'dark' ? urlTheme : resolve(getTheme());
    const changed = root.getAttribute('data-theme') !== effective;
    root.setAttribute('data-theme', effective);
    root.style.setProperty('--ui-scale', String(getScale() / 100));
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
  global.ControLABAppearance = Object.freeze({ getTheme, getScale, setTheme, setScale, themes: THEMES, scales: SCALES });
})(window);
