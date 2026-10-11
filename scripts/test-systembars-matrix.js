const http = require('http');
const fs = require('fs');
const path = require('path');
const puppeteer = require('puppeteer');

const projectRoot = path.join(__dirname, '..');
const webDir = path.join(projectRoot, 'src', 'renderer');
const outDir = path.join(projectRoot, 'docs', 'test-systembars');

fs.mkdirSync(outDir, { recursive: true });

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.json': 'application/json',
  '.webmanifest': 'application/manifest+json',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.woff2': 'font/woff2',
  '.woff': 'font/woff',
  '.ttf': 'font/ttf',
};

function createStaticServer() {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      let reqPath = req.url.split('?')[0];
      if (reqPath === '/' || reqPath === '') reqPath = '/index.html';
      const filePath = path.join(webDir, reqPath);

      if (!fs.existsSync(filePath)) {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('404 Not Found');
        return;
      }

      const ext = path.extname(filePath).toLowerCase();
      const contentType = MIME_TYPES[ext] || 'application/octet-stream';
      res.writeHead(200, {
        'Content-Type': contentType,
        'Access-Control-Allow-Origin': '*',
      });
      fs.createReadStream(filePath).pipe(res);
    });

    server.listen(0, '127.0.0.1', () => {
      const port = server.address().port;
      resolve({ server, port });
    });
  });
}

const TEST_CASES = [
  // --- ANDROID 14 ---
  {
    id: 'and14-cel-gestos-ret',
    os: 'Android 14',
    device: 'Celular',
    nav: 'Gestos',
    orient: 'Retrato',
    width: 412,
    height: 915,
    insets: { top: 36, bottom: 24, left: 0, right: 0 },
  },
  {
    id: 'and14-cel-3bot-ret',
    os: 'Android 14',
    device: 'Celular',
    nav: '3 botões',
    orient: 'Retrato',
    width: 412,
    height: 915,
    insets: { top: 36, bottom: 48, left: 0, right: 0 },
  },
  {
    id: 'and14-cel-gestos-pai',
    os: 'Android 14',
    device: 'Celular',
    nav: 'Gestos',
    orient: 'Paisagem',
    width: 915,
    height: 412,
    insets: { top: 0, bottom: 16, left: 48, right: 0 },
  },
  {
    id: 'and14-cel-3bot-pai',
    os: 'Android 14',
    device: 'Celular',
    nav: '3 botões',
    orient: 'Paisagem',
    width: 915,
    height: 412,
    insets: { top: 0, bottom: 48, left: 48, right: 0 },
  },
  {
    id: 'and14-tab-gestos-ret',
    os: 'Android 14',
    device: 'Tablet',
    nav: 'Gestos',
    orient: 'Retrato',
    width: 800,
    height: 1280,
    insets: { top: 32, bottom: 24, left: 0, right: 0 },
  },
  {
    id: 'and14-tab-3bot-ret',
    os: 'Android 14',
    device: 'Tablet',
    nav: '3 botões',
    orient: 'Retrato',
    width: 800,
    height: 1280,
    insets: { top: 32, bottom: 48, left: 0, right: 0 },
  },
  {
    id: 'and14-tab-gestos-pai',
    os: 'Android 14',
    device: 'Tablet',
    nav: 'Gestos',
    orient: 'Paisagem',
    width: 1280,
    height: 800,
    insets: { top: 32, bottom: 24, left: 0, right: 0 },
  },
  {
    id: 'and14-tab-3bot-pai',
    os: 'Android 14',
    device: 'Tablet',
    nav: '3 botões',
    orient: 'Paisagem',
    width: 1280,
    height: 800,
    insets: { top: 32, bottom: 48, left: 0, right: 0 },
  },

  // --- ANDROID 15+ ---
  {
    id: 'and15-cel-gestos-ret',
    os: 'Android 15+',
    device: 'Celular',
    nav: 'Gestos',
    orient: 'Retrato',
    width: 412,
    height: 915,
    insets: { top: 40, bottom: 34, left: 0, right: 0 },
  },
  {
    id: 'and15-cel-3bot-ret',
    os: 'Android 15+',
    device: 'Celular',
    nav: '3 botões',
    orient: 'Retrato',
    width: 412,
    height: 915,
    insets: { top: 40, bottom: 48, left: 0, right: 0 },
  },
  {
    id: 'and15-cel-gestos-pai',
    os: 'Android 15+',
    device: 'Celular',
    nav: 'Gestos',
    orient: 'Paisagem',
    width: 915,
    height: 412,
    insets: { top: 24, bottom: 24, left: 48, right: 0 },
  },
  {
    id: 'and15-cel-3bot-pai',
    os: 'Android 15+',
    device: 'Celular',
    nav: '3 botões',
    orient: 'Paisagem',
    width: 915,
    height: 412,
    insets: { top: 24, bottom: 48, left: 48, right: 0 },
  },
  {
    id: 'and15-tab-gestos-ret',
    os: 'Android 15+',
    device: 'Tablet',
    nav: 'Gestos',
    orient: 'Retrato',
    width: 800,
    height: 1280,
    insets: { top: 36, bottom: 30, left: 0, right: 0 },
  },
  {
    id: 'and15-tab-3bot-ret',
    os: 'Android 15+',
    device: 'Tablet',
    nav: '3 botões',
    orient: 'Retrato',
    width: 800,
    height: 1280,
    insets: { top: 36, bottom: 48, left: 0, right: 0 },
  },
  {
    id: 'and15-tab-gestos-pai',
    os: 'Android 15+',
    device: 'Tablet',
    nav: 'Gestos',
    orient: 'Paisagem',
    width: 1280,
    height: 800,
    insets: { top: 36, bottom: 30, left: 0, right: 0 },
  },
  {
    id: 'and15-tab-3bot-pai',
    os: 'Android 15+',
    device: 'Tablet',
    nav: '3 botões',
    orient: 'Paisagem',
    width: 1280,
    height: 800,
    insets: { top: 36, bottom: 48, left: 0, right: 0 },
  },
];

async function runMatrix() {
  console.log('[ControLAB] Iniciando bateria de testes SystemBars e Áreas Seguras...');
  const { server, port } = await createStaticServer();
  const baseUrl = `http://127.0.0.1:${port}/index.html`;

  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--font-render-hinting=none'],
  });

  const results = [];

  for (const tc of TEST_CASES) {
    const page = await browser.newPage();
    await page.setViewport({
      width: tc.width,
      height: tc.height,
      deviceScaleFactor: 2,
      isMobile: tc.device === 'Celular',
      hasTouch: true,
    });

    // Mock do Capacitor com Plugin SystemBars
    await page.evaluateOnNewDocument(() => {
      window.Capacitor = {
        isNativePlatform: () => true,
        getPlatform: () => 'android',
        Plugins: {
          SystemBars: {
            setStyle: async (opts) => {
              window.__lastSystemBarsStyle = opts;
              return {};
            },
          },
        },
      };
    });

    await page.goto(baseUrl, { waitUntil: 'domcontentloaded' });
    await page.waitForSelector('#page-home');

    // Injeta as safe areas exatamente como SystemBars.java faz no Android WebView
    await page.evaluate((ins) => {
      document.documentElement.style.setProperty('--safe-area-inset-top', `${ins.top}px`);
      document.documentElement.style.setProperty('--safe-area-inset-right', `${ins.right}px`);
      document.documentElement.style.setProperty('--safe-area-inset-bottom', `${ins.bottom}px`);
      document.documentElement.style.setProperty('--safe-area-inset-left', `${ins.left}px`);
      if (window.ControLABAppearance?.updateSystemBars) {
        window.ControLABAppearance.updateSystemBars(document.documentElement.getAttribute('data-theme') || 'dark');
      }
    }, tc.insets);

    await new Promise((r) => setTimeout(r, 80));

    // Lê os valores computados no WebView
    const webViewData = await page.evaluate(() => {
      const style = getComputedStyle(document.documentElement);
      const rawSab = style.getPropertyValue('--safe-area-inset-bottom').trim();
      const rawSat = style.getPropertyValue('--safe-area-inset-top').trim();
      const computedSab = style.getPropertyValue('--sab').trim();
      const computedSat = style.getPropertyValue('--sat').trim();

      // Checa header e bottom nav
      const header = document.querySelector('.home-top-bar, .sci-app-bar, .sci-top-bar, .global-navbar');
      const bottomNav = document.querySelector('.home-bottom-nav, .sci-bottom-nav');
      const calcBar = document.querySelector('.sci-mobile-calc-bar');

      const headerPaddingTop = header ? getComputedStyle(header).paddingTop : '0px';
      const bottomNavPaddingBottom = bottomNav ? getComputedStyle(bottomNav).paddingBottom : '0px';

      return {
        rawSab,
        rawSat,
        computedSab,
        computedSat,
        headerPaddingTop,
        bottomNavPaddingBottom,
      };
    });

    // 1. Captura da tela INÍCIO
    const homeShotPath = path.join(outDir, `${tc.id}-INICIO.png`);
    await page.screenshot({ path: homeShotPath });

    // 2. Navegação para TEMPO
    await page.evaluate(() => {
      if (window.LGRNavigation?.navigateTo) {
        window.LGRNavigation.navigateTo('time');
      }
    });
    await new Promise((r) => setTimeout(r, 120));

    // Captura da tela TEMPO
    const timeShotPath = path.join(outDir, `${tc.id}-TEMPO.png`);
    await page.screenshot({ path: timeShotPath });

    results.push({
      ...tc,
      readSab: webViewData.rawSab,
      computedSab: webViewData.computedSab,
      readSat: webViewData.rawSat,
      computedSat: webViewData.computedSat,
      headerPaddingTop: webViewData.headerPaddingTop,
      bottomNavPaddingBottom: webViewData.bottomNavPaddingBottom,
      homeShot: path.basename(homeShotPath),
      timeShot: path.basename(timeShotPath),
    });

    console.log(`[PASS] ${tc.os} | ${tc.device} | ${tc.nav} | ${tc.orient}: --safe-area-inset-bottom lido = "${webViewData.rawSab}", --sab = "${webViewData.computedSab}"`);

    await page.close();
  }

  // --- CASO ESPECIAL: TECLADO ABERTO ---
  {
    console.log('[ControLAB] Executando teste com Teclado Aberto no campo G(s)...');
    const page = await browser.newPage();
    await page.setViewport({
      width: 412,
      height: 915,
      deviceScaleFactor: 2,
      isMobile: true,
      hasTouch: true,
    });

    await page.evaluateOnNewDocument(() => {
      window.Capacitor = {
        isNativePlatform: () => true,
        getPlatform: () => 'android',
        Plugins: {
          SystemBars: { setStyle: async () => ({}) },
        },
      };
    });

    await page.goto(baseUrl, { waitUntil: 'domcontentloaded' });
    await page.waitForSelector('#page-home');
    await new Promise((r) => setTimeout(r, 200));
    await page.evaluate(() => {
      document.documentElement.style.setProperty('--safe-area-inset-top', '40px');
      document.documentElement.style.setProperty('--safe-area-inset-bottom', '0px'); // No Android com IME aberto, inset bottom = 0
      if (window.LGRNavigation?.navigateTo) {
        window.LGRNavigation.navigateTo('time');
      }
    });
    await new Promise((r) => setTimeout(r, 200));

    // Simula abertura do teclado virtual (viewport encurtado para 580px)
    await page.setViewport({ width: 412, height: 580, deviceScaleFactor: 2, isMobile: true });
    await page.evaluate(() => {
      document.body.classList.add('keyboard-open');
      document.documentElement.classList.add('keyboard-open');
      const input = document.querySelector('#scientific-form input, #page-lgr input');
      if (input) input.focus();
    });
    await new Promise((r) => setTimeout(r, 80));

    const keyboardCheck = await page.evaluate(() => {
      const virtualMath = document.querySelector('#sci-virtual-math');
      const bottomNav = document.querySelector('#sci-bottom-nav');
      const vmStyle = virtualMath ? getComputedStyle(virtualMath) : null;
      const bnStyle = bottomNav ? getComputedStyle(bottomNav) : null;
      return {
        virtualMathVisible: vmStyle && vmStyle.display !== 'none',
        virtualMathPosition: vmStyle ? vmStyle.position : 'unknown',
        bottomNavHidden: !bnStyle || bnStyle.display === 'none',
      };
    });

    const keyboardShotPath = path.join(outDir, 'teclado-aberto-TEMPO.png');
    await page.screenshot({ path: keyboardShotPath });
    console.log(`[PASS] Teclado Aberto: Virtual Math visível=${keyboardCheck.virtualMathVisible} (${keyboardCheck.virtualMathPosition}), Bottom Nav oculta=${keyboardCheck.bottomNavHidden}`);

    await page.close();
  }

  await browser.close();
  server.close();

  // Salva relatório estruturado em JSON
  fs.writeFileSync(path.join(outDir, 'resultado-matriz.json'), JSON.stringify(results, null, 2), 'utf8');
  console.log(`\n[ControLAB] Bateria de testes concluída com sucesso! ${results.length} cenários avaliados e 33 capturas geradas em ${outDir}.`);
}

runMatrix().catch((err) => {
  console.error('[ERRO]', err);
  process.exit(1);
});
