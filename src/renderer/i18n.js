/* Shared offline localization for the DOM, accessible names and generated UI. */
(function registerI18n(global) {
  'use strict';
  const KEY = 'controlab-language';
  const catalog = global.ControLABEnglish || {};
  const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const phrases = Object.keys(catalog).filter((key) => key !== catalog[key]).sort((a, b) => b.length - a.length);
  const pattern = new RegExp(`(?<![\\p{L}\\p{N}_])(?:${phrases.map(escape).join('|')})(?![\\p{L}\\p{N}_])`, 'gu');
  let language = 'pt-BR';
  try { if (localStorage.getItem(KEY) === 'en') language = 'en'; } catch (_) { /* private storage */ }
  const originals = new WeakMap();
  const attributes = ['title', 'aria-label', 'placeholder', 'alt', 'data-tooltip', 'aria-description', 'aria-valuetext'];
  const protectedContent = 'script, style, textarea, pre, code, .katex, [data-no-i18n], .studio-file-name, .studio-tab-name, .home-workspace-name, .home-workspace-value, .studio-variable, #home-folder-path-text';
  function translate(text, locale = language) {
    if (locale !== 'en' || typeof text !== 'string') return text;
    return catalog[text] || text.replace(pattern, (match) => catalog[match]);
  }
  function update(node, key, read, write) {
    const value = read();
    if (!value) return;
    let state = originals.get(node);
    if (!state) { state = {}; originals.set(node, state); }
    const previous = state[key];
    const source = previous && previous.output === value ? previous.source : value;
    const output = translate(source);
    state[key] = { source, output };
    if (output !== value) write(output);
  }
  function visit(node) {
    const parent = node.nodeType === 3 ? node.parentElement : node;
    if (!parent || parent.closest?.(protectedContent)) return;
    if (node.nodeType === 3) {
      update(node, 'text', () => node.nodeValue, (v) => { node.nodeValue = v; });
    } else if (node.nodeType === 1) {
      attributes.forEach((name) => update(node, name, () => node.getAttribute(name), (v) => node.setAttribute(name, v)));
      Array.from(node.childNodes).forEach(visit);
    }
  }
  function apply() {
    document.documentElement.lang = language;
    visit(document.documentElement);
    const selector = document.getElementById('settings-language');
    if (selector) selector.value = language;
  }
  function setLanguage(value) {
    if (!['pt-BR', 'en'].includes(value)) return;
    language = value;
    try { localStorage.setItem(KEY, value); } catch (_) { /* preference remains in memory */ }
    apply();
    global.api?.setLanguage?.(language);
    global.dispatchEvent(new CustomEvent('controlab:languagechange', { detail: { language } }));
  }
  global.ControLABI18n = Object.freeze({ translate, getLanguage: () => language, setLanguage, apply });
  // Translate browser-owned confirmation messages without touching script contents.
  ['confirm', 'alert'].forEach((name) => {
    const original = global[name].bind(global);
    global[name] = (message) => original(translate(message));
  });
  document.documentElement.lang = language;
  document.addEventListener('DOMContentLoaded', () => {
    apply();
    global.api?.setLanguage?.(language);
    const observer = new MutationObserver((records) => {
      records.forEach((record) => {
        if (record.type === 'childList') record.addedNodes.forEach(visit);
        else visit(record.target);
      });
    });
    observer.observe(document.documentElement, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: attributes });
    document.getElementById('settings-language')?.addEventListener('change', (event) => setLanguage(event.target.value));
  });
  global.addEventListener('storage', (event) => {
    if (event.key === KEY) { language = event.newValue === 'en' ? 'en' : 'pt-BR'; apply(); }
  });
}(window));
