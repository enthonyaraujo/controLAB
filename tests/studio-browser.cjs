/* End-to-end workspace checks using an isolated local server and temporary .m files. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const net = require('node:net');
const { spawn } = require('node:child_process');
const puppeteer = require('puppeteer');
const root = path.join(__dirname, '..');
async function freePort() {
  const server = net.createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}
(async () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'controlab-studio-test-'));
  const port = await freePort();
  const fixture = path.join(temp, 'matrizes.m');
  fs.mkdirSync(path.join(temp, 'pasta-a'));
  fs.mkdirSync(path.join(temp, 'pasta-b'));
  fs.writeFileSync(fixture, 'A=[1 2;3 4]; B=[5 6;7 8]; C=A*B; disp(C);\n');
  const venv = path.join(root, '.venv', ...(process.platform === 'win32' ? ['Scripts', 'python.exe'] : ['bin', 'python']));
  const python = fs.existsSync(venv) ? venv : (process.platform === 'win32' ? 'python' : 'python3');
  const server = spawn(python, ['src/python/web_server.py', '--port', String(port)], { cwd: root, stdio: 'ignore' });
  let browser;
  try {
    const url = `http://127.0.0.1:${port}`;
    const deadline = Date.now() + 10000;
    while (true) {
      try { if ((await fetch(url)).ok) break; } catch {}
      if (Date.now() > deadline) throw new Error('Servidor de teste não iniciou.');
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    browser = await puppeteer.launch({ headless: true });
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.setViewport({ width: 1440, height: 900 });
    await page.goto(url, { waitUntil: 'networkidle0' });
    await page.waitForSelector('#studio-source');
    await page.locator('#studio-folder-input').fill(temp);
    await page.click('#studio-folder-form button');
    await page.waitForSelector(`button[title="${fixture}"]`);
    // Successful navigation builds history; failed navigation must not change it.
    const subfolder = path.join(temp, 'pasta-a');
    await page.click(`button[title="${subfolder}"]`);
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, subfolder);
    await page.click('#studio-back');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, temp);
    await page.click('#studio-forward');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, subfolder);
    await page.click('#studio-up');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, temp);
    await page.click('#studio-select-folder');
    await page.waitForSelector('#studio-folder-dialog[open]');
    await page.waitForFunction(() => !document.querySelector('#studio-picker-confirm').disabled);
    await page.locator('#studio-picker-path').fill(path.join(temp, 'pasta-b'));
    await page.click('#studio-picker-form button[type="submit"]');
    await page.waitForFunction((folder) => document.querySelector('#studio-picker-path').value === folder && !document.querySelector('#studio-picker-confirm').disabled, {}, path.join(temp, 'pasta-b'));
    await page.click('#studio-picker-confirm');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, path.join(temp, 'pasta-b'));
    await page.click('#studio-back');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, temp);
    await page.click('#studio-select-folder');
    await page.waitForSelector('#studio-folder-dialog[open]');
    await page.click('#studio-picker-cancel');
    assert.equal(await page.$eval('#studio-folder-input', (node) => node.value), temp);
    await page.locator('#studio-folder-input').fill(path.join(temp, 'missing'));
    await page.click('#studio-folder-form button');
    await page.waitForFunction(() => document.querySelector('#studio-status').textContent === 'Pasta indisponível');
    await page.click('#studio-refresh');
    await page.waitForFunction((folder) => document.querySelector('#studio-folder-input').value === folder, {}, temp);
    await page.click(`button[title="${fixture}"]`);
    await page.waitForFunction(() => document.querySelector('#studio-editor-title').textContent.includes('matrizes.m'));
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#home-console-logs').textContent.includes('43  50'));
    assert.match(await page.$eval('#home-workspace-list', (node) => node.textContent), /C19\s+22/);
    await page.locator('#home-console-input').fill('det(A)');
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.querySelector('#home-console-logs').textContent.includes('-2'));
    // Editing and saving the opened file must write the actual document.
    await page.locator('#studio-source').fill('answer=42; disp(answer);');
    await page.keyboard.down('Control'); await page.keyboard.press('s'); await page.keyboard.up('Control');
    await page.waitForFunction(() => document.querySelector('#studio-status').textContent.includes('Salvo: matrizes.m'));
    assert.equal(fs.readFileSync(fixture, 'utf8'), 'answer=42; disp(answer);');
    // New unsaved document, browser-local file import, and non-destructive close.
    await page.click('#studio-new');
    await page.locator('#studio-source').fill('x=0:0.1:1; plot(x,x.^2);');
    await page.click('#studio-run');
    const figureTarget = await browser.waitForTarget((target) => target.opener() === page.target() && target.url().includes('/figure.html'), { timeout: 10000 });
    const figurePage = await figureTarget.page();
    await figurePage.waitForSelector('#figure-content svg polyline');
    assert.equal(await page.$eval('#studio-editor-pane', (node) => node.hidden), false);
    assert.equal(await page.$$eval('#studio-figures svg', (nodes) => nodes.length), 0);
    const hasExport = await figurePage.$eval('.studio-figure-tools button', (node) => node.textContent);
    assert.equal(hasExport, 'Salvar SVG');
    await figurePage.close();
    assert.equal(await page.$eval('#studio-figure-count', (node) => node.textContent), '1');
    await page.click('#studio-figures-view');
    const reopenedTarget = await browser.waitForTarget((target) => target.opener() === page.target() && target.url().includes('/figure.html'), { timeout: 10000 });
    const reopenedFigure = await reopenedTarget.page();
    await reopenedFigure.waitForSelector('#figure-content svg polyline');
    await reopenedFigure.close();
    await page.click('#studio-editor-view');
    await page.locator('#studio-source').fill('value=7;');
    page.once('dialog', (dialog) => dialog.dismiss());
    await page.click('.studio-tab[aria-selected="true"] .studio-close-tab');
    assert.equal(await page.$eval('#studio-source', (node) => node.value), 'value=7;');
    await page.click('#studio-save');
    await page.waitForSelector('#studio-save-dialog[open]');
    await page.locator('#studio-save-name').fill('novo.m');
    await page.click('#studio-save-dialog button[value="save"]');
    await page.waitForFunction(() => document.querySelector('#studio-status').textContent.includes('Salvo: novo.m'));
    assert.equal(fs.readFileSync(path.join(temp, 'novo.m'), 'utf8'), 'value=7;');
    // Error locations remain actionable, and clear removes actual runtime variables.
    await page.locator('#studio-source').fill('missing_function(1)');
    await page.click('#studio-run');
    await page.waitForFunction(() => document.querySelector('#studio-status').textContent === 'Erro de execução');
    await page.click('#studio-clear');
    await page.waitForFunction(() => document.querySelector('#home-workspace-list').textContent.includes('Nenhuma variável'));
    await page.click('#studio-clear-output');
    assert.equal(await page.$eval('#home-console-logs', (node) => node.textContent), '');
    const screenshots = process.env.CONTROLAB_SCREENSHOTS || temp;
    fs.mkdirSync(screenshots, { recursive: true });
    // Browser-local imports use the same editor and can be executed.
    const input = await page.$('#studio-file-input');
    await input.uploadFile(fixture);
    await page.waitForFunction(() => document.querySelector('#studio-source').value.includes('answer=42'));
    await page.click('#studio-run');
    await page.waitForFunction(() => document.querySelector('#home-console-logs').textContent.includes('42'));
    for (const theme of ['dark', 'light']) {
      await page.evaluate((value) => document.documentElement.setAttribute('data-theme', value), theme);
      await page.screenshot({ path: path.join(screenshots, `studio-${theme}.png`) });
    }
    for (const width of [360, 768, 1024, 1440]) {
      await page.setViewport({ width, height: width < 768 ? 800 : 900 });
      const bounds = await page.evaluate(() => {
        const rect = document.querySelector('#studio-source').getBoundingClientRect();
        const console = document.querySelector('#home-console-area').getBoundingClientRect();
        return { width: rect.width, height: rect.height, console: console.height, overflow: document.documentElement.scrollWidth > innerWidth + 1 };
      });
      assert(bounds.width > 100 && bounds.height > 60 && bounds.console >= 90, `${width}px: ${JSON.stringify(bounds)}`);
      assert(!bounds.overflow, `Overflow horizontal em ${width}px`);
      await page.screenshot({ path: path.join(screenshots, `studio-${width}.png`) });
    }
    await page.setViewport({ width: 360, height: 800 });
    await page.click('#studio-files');
    assert(await page.$eval('.home-left-col', (node) => node.getBoundingClientRect().width > 0));
    await page.click('#studio-files');
    await page.click('#studio-variables');
    assert(await page.$eval('.home-right-col', (node) => node.getBoundingClientRect().width > 0));
    await page.click('#studio-variables');
    await page.setViewport({ width: 1440, height: 900 });
    await page.click('.home-top-tabs [data-nav="time"]');
    await page.waitForFunction(() => document.getElementById('page-scientific').classList.contains('active'));
    assert.deepEqual(errors, []);
    console.log('Editor .m: navegação de pastas, histórico, seletor e cancelamento; abrir, editar, salvar, F5, console persistente, gráficos, erros, limpeza, temas, navegação e quatro larguras aprovados.');
  } finally {
    if (browser) await browser.close();
    server.kill();
    // Only the synthetic fixture directory created by this test is removed.
    fs.rmSync(temp, { recursive: true, force: true });
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
