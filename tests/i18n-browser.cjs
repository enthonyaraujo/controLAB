/* Run against the local web server: CONTROLAB_TEST_URL=http://127.0.0.1:8766 node tests/i18n-browser.cjs */
const assert = require('node:assert/strict');
const puppeteer = require('puppeteer');
(async () => {
  const browser = await puppeteer.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(process.env.CONTROLAB_TEST_URL || 'http://127.0.0.1:8766', { waitUntil: 'networkidle0' });
    assert.equal(await page.$eval('html', (node) => node.lang), 'pt-BR');
    await page.evaluate(() => window.ControLABSettings.open());
    await page.select('#settings-language', 'en');
    await page.waitForFunction(() => document.querySelector('#appearance-theme-label').textContent === 'Theme');
    assert.equal(await page.$eval('html', (node) => node.lang), 'en');
    await page.evaluate(() => {
      const text = document.createElement('p'); text.id = 'dynamic-i18n'; text.textContent = 'Entrada inválida.'; document.body.append(text);
      const code = document.createElement('pre'); code.id = 'protected-i18n'; code.textContent = 'disp("Ganho");'; document.body.append(code);
      document.querySelector('#studio-source').value = 'disp("Configurações");';
    });
    await page.waitForFunction(() => document.querySelector('#dynamic-i18n').textContent === 'Invalid input.');
    assert.equal(await page.$eval('#protected-i18n', (node) => node.textContent), 'disp("Ganho");');
    await page.screenshot({ path: '/tmp/controlab-english-settings.png' });
    await page.select('#settings-language', 'pt-BR');
    await page.waitForFunction(() => document.querySelector('#dynamic-i18n').textContent === 'Entrada inválida.');
    assert.equal(await page.$eval('#studio-source', (node) => node.value), 'disp("Configurações");');
    await page.select('#settings-language', 'en');
    await page.reload({ waitUntil: 'networkidle0' });
    assert.equal(await page.$eval('html', (node) => node.lang), 'en');
    for (const module of ['time', 'frequency', 'state-space', 'controllers', 'lgr', 'routh']) {
      await page.evaluate((name) => document.querySelector(`[data-nav="${name}"]`).click(), module);
      await page.waitForFunction(() => document.documentElement.lang === 'en');
      const leftovers = await page.evaluate(() => {
        const result = [];
        const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
        while (walker.nextNode()) {
          const node = walker.currentNode, text = node.textContent.trim();
          if (text && node.parentElement.getClientRects().length && !node.parentElement.closest('script,style,textarea,.katex,[data-no-i18n]') && /[ãõç]|\b(não|para|uma|pelo|pela)\b/.test(text)) result.push(text);
        }
        return [...new Set(result)];
      });
      assert.deepEqual(leftovers, [], `${module}: ${leftovers.join('; ')}`);
    }
    const scientific = await page.evaluate(async () => {
      const result = await window.api.runScientificAnalysis('time_response', { expr: '4/(s^2+2*s+4)', points: 200 });
      return { success: result.success, english: result.svg?.includes('Time (s)'), portuguese: result.svg?.includes('Tempo (s)') };
    });
    assert.deepEqual(scientific, { success: true, english: true, portuguese: false });
    await page.setViewport({ width: 390, height: 844 });
    await page.evaluate(() => window.ControLABSettings.open());
    assert.equal(await page.$eval('#settings-language', (node) => node.value), 'en');
    await page.screenshot({ path: '/tmp/controlab-english-mobile.png' });
    assert.deepEqual(errors, []);
    console.log('Language switching, persistence, dynamic UI, six modules and user content verified.');
  } finally { await browser.close(); }
})().catch((error) => { console.error(error); process.exitCode = 1; });
