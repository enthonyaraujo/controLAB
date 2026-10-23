const assert = require('assert');
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'src', 'renderer', 'index.html'), 'utf8').replace(/\r\n/g, '\n');
const app = fs.readFileSync(path.join(root, 'src', 'renderer', 'app.js'), 'utf8').replace(/\r\n/g, '\n');
const serviceWorker = fs.readFileSync(path.join(root, 'src', 'renderer', 'sw.js'), 'utf8').replace(/\r\n/g, '\n');
const styles = fs.readFileSync(path.join(root, 'src', 'renderer', 'styles.css'), 'utf8').replace(/\r\n/g, '\n');
const scientificModules = fs.readFileSync(path.join(root, 'src', 'renderer', 'scientific-modules.js'), 'utf8').replace(/\r\n/g, '\n');
const mainProcess = fs.readFileSync(path.join(root, 'src', 'main', 'main.js'), 'utf8').replace(/\r\n/g, '\n');

const settingsPosition = html.indexOf('<script src="settings.js"></script>');
const navigationPosition = html.indexOf('<script src="navigation.js"></script>');
const routhPosition = html.indexOf('<script src="routh.js"></script>');
const scientificPosition = html.indexOf('<script src="scientific-modules.js"></script>');
const appPosition = html.indexOf('<script src="app.js"></script>');

assert(settingsPosition >= 0, 'settings.js precisa estar carregado pelo renderer');
assert(navigationPosition >= 0, 'navigation.js precisa estar carregado pelo renderer');
assert(routhPosition >= 0, 'routh.js precisa estar carregado pelo renderer');
assert(routhPosition > navigationPosition, 'routh.js precisa carregar depois de navigation.js');
assert(appPosition > routhPosition, 'routh.js precisa carregar antes de app.js');
assert(scientificPosition > routhPosition, 'scientific-modules.js precisa carregar depois dos controladores existentes');
assert(appPosition > scientificPosition, 'scientific-modules.js precisa carregar antes de app.js');
assert(app.includes('window.LGRNavigation.initialize'), 'app.js precisa inicializar a navegação');
assert(app.includes('window.ControLABSettings.initialize'), 'app.js precisa inicializar as configurações');
assert(app.includes('window.ControLABRouth.initialize'), 'app.js precisa inicializar o modulo routh');
assert(app.includes('window.ControLABScientific.initialize'), 'app.js precisa inicializar os módulos científicos');
assert(app.includes("getElementById('page-lgr')?.classList.contains('active')"), 'o atalho do LGR não pode executar em outros módulos');
assert(serviceWorker.includes("'/settings.js'"), 'settings.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/navigation.js'"), 'navigation.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/routh.js'"), 'routh.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/scientific-modules.js'"), 'scientific-modules.js precisa integrar o shell offline');
assert(scientificModules.includes("action: 'time_response'"), 'o módulo temporal precisa estar registrado');
assert(scientificModules.includes("action: 'frequency_response'"), 'o módulo de frequência precisa estar registrado');
assert(scientificModules.includes("action: 'controller_design'"), 'o módulo de controladores precisa estar registrado');
assert(scientificModules.includes("action: 'state_space'"), 'o módulo de espaço de estados precisa estar registrado');
assert(scientificModules.includes("input.step = 'any'"), 'campos científicos decimais não podem usar uma grade incompatível com o valor mínimo');
assert(scientificModules.includes("input.inputMode = 'decimal'"), 'campos científicos precisam abrir o teclado decimal em dispositivos móveis');
assert(html.includes('id="card-module-time" class="module-card card-active"'), 'o card temporal precisa estar disponível');
assert(html.includes('id="card-module-frequency" class="module-card card-active"'), 'o card de frequência precisa estar disponível');
assert(html.includes('id="card-module-controllers" class="module-card card-active"'), 'o card de controladores precisa estar disponível');
assert(html.includes('id="card-module-state-space" class="module-card card-active"'), 'o card de espaço de estados precisa estar disponível');
assert(html.includes('id="page-scientific"'), 'o workspace científico compartilhado precisa existir');
assert(serviceWorker.includes('mustBeFresh'), 'scripts do shell precisam de atualização network-first');
assert(styles.includes('@media (max-width: 1000px)'), 'o layout precisa adaptar os módulos em telas menores');
assert(styles.includes('#page-routh'), 'o módulo Routh precisa ter regras de layout próprias');
assert(styles.includes('.scientific-workspace'), 'os novos módulos precisam ter um layout responsivo próprio');
assert(styles.includes('prefers-reduced-motion: reduce'), 'a interface precisa respeitar redução de movimento');
assert(styles.includes('safe-area-inset-bottom'), 'os elementos fixos precisam respeitar a área segura móvel');
assert(styles.includes('.katex-display'), 'fórmulas extensas precisam permanecer navegáveis em telas estreitas');
assert(styles.includes('grid-template-columns: minmax(0, 1fr)'), 'cards do Routh não podem expandir além da tela móvel');
assert(html.includes('id="scientific-function-preview"'), 'a prévia matemática dos módulos científicos precisa existir no HTML');
assert(!html.includes('Ctrl + Enter'), 'o atalho visual Ctrl + Enter deve ser removido de todos os botões');
assert(!html.includes('id="btn-calculate" class="btn btn-primary btn-calc-action" type="button">\n          <svg'), 'o botão Traçar LGR não deve ter ícone SVG');
assert(!html.includes('id="btn-calculate-routh" class="btn btn-primary btn-calc-action" type="button">\n            <svg'), 'o botão Calcular Estabilidade não deve ter ícone SVG');
assert(scientificModules.includes('scheduleScientificPreview'), 'os módulos científicos precisam agendar e atualizar a prévia matemática');
assert(html.includes('<h2 class="module-title">Lugar Geométrico das Raízes</h2>'), 'o título do card LGR deve ser Lugar Geométrico das Raízes');
assert(html.includes('id="routh-empty-state" class="routh-empty-card" style="display: flex;"'), 'o empty state do Routh deve iniciar visível');
assert(html.includes('id="routh-results-container" class="routh-results-layout" style="display: none;"'), 'os resultados do Routh devem iniciar ocultos');
assert(html.includes('id="lgr-empty-state"'), 'o empty state do LGR deve existir');
assert(!app.includes('window.setTimeout(calculate, 0)'), 'o LGR não deve calcular automaticamente ao entrar no módulo');
assert(html.includes('<link rel="stylesheet" href="tokens.css">'), 'tokens.css precisa estar linkado no index.html');
assert(styles.includes("@import 'tokens.css';"), 'styles.css precisa importar tokens.css');
assert(serviceWorker.includes("'/tokens.css'"), 'tokens.css precisa integrar o shell offline');

// Valida que o bloco do diálogo não possui cores hexadecimais fixas
const modalStart = styles.indexOf('.modal-overlay {');
const modalEnd = styles.indexOf('/* ==========================================================================\n   MÓDULO 2:');
assert(modalStart >= 0 && modalEnd > modalStart, 'bloco do modal precisa existir em styles.css');
const modalCss = styles.slice(modalStart, modalEnd);
const fixedHexMatches = modalCss.match(/#[0-9a-fA-F]{3,8}/g);
assert(!fixedHexMatches || fixedHexMatches.length === 0, `o diálogo não pode conter cores hex fixas: ${fixedHexMatches}`);

// Valida que o botão de verificar atualizações não usa azul (#0369a1 / #0284c7) e usa token teal
assert(!styles.includes('#0369a1'), 'o azul #0369a1 deve ser removido de btn-action-primary');
assert(!styles.includes('#0284c7'), 'o azul #0284c7 deve ser removido de btn-action-primary');

// Valida alinhamento entre a barra de abas superior (.sci-top-bar) e a ribbon (.sci-ribbon)
assert(styles.includes('padding: 0 0.5rem;'), '.sci-top-bar precisa ter padding horizontal de 0.5rem alinhado com .sci-ribbon');

// Valida que a seção de Exemplos foi removida do Menu Início
assert(!html.includes('home-examples-list'), 'a seção Exemplos deve ser removida do Menu Início');

// Valida que o Workspace inicia limpo e os botões de limpar existem
assert(html.includes('id="btn-clear-recent"'), 'o botão de limpar recentes precisa existir');
assert(html.includes('id="btn-clear-workspace"'), 'o botão de limpar workspace precisa existir');
assert(html.includes('Nenhuma variável no workspace'), 'o workspace precisa iniciar limpo');
assert(styles.includes('.home-btn-clear'), 'os estilos do botão de limpar precisam existir');

// Valida split panes e responsividade do Menu Início
const navigation = fs.readFileSync(path.join(root, 'src', 'renderer', 'navigation.js'), 'utf8');
const tokens = fs.readFileSync(path.join(root, 'src', 'renderer', 'tokens.css'), 'utf8');
assert(html.includes('id="home-splitter-console"'), 'o splitter do console precisa existir no HTML');
assert(html.includes('id="home-splitter-left"'), 'o splitter esquerdo precisa existir no HTML');
assert(html.includes('id="home-splitter-right"'), 'o splitter direito precisa existir no HTML');
assert(tokens.includes('--home-console-height'), 'o token de altura do console precisa existir');
assert(styles.includes('.home-splitter-h') && styles.includes('.home-splitter-v'), 'os estilos dos splitters horizontal e vertical precisam existir');
assert(styles.includes('height: 1px;') && styles.includes('width: 1px;'), 'os splitters devem ter linhas finas de 1px');
assert(!html.includes('splitter-grip'), 'os grips grossos dos splitters devem ser removidos');
assert(styles.includes('.home-console-area {\n    border-top: none !important;'), 'o console não deve duplicar borda no desktop');
assert(styles.includes('cursor: row-resize') && styles.includes('cursor: col-resize'), 'os cursores de redimensionamento precisam estar definidos');
assert(styles.includes('.home-splitter {\n    display: none !important;'), 'os splitters devem ser ocultados em resoluções menores');
// Valida que todos os controladores do renderer inicializam sem exceções de runtime
{
  const vm = require('vm');
  const sandboxWindow = {
    innerWidth: 1200,
    innerHeight: 800,
    addEventListener: () => {},
    removeEventListener: () => {},
    location: { hash: '', search: '' },
    localStorage: { getItem: () => null, setItem: () => {}, removeItem: () => {} },
    MutationObserver: class { observe() {} disconnect() {} },
    api: {},
  };
  sandboxWindow.window = sandboxWindow;
  const makeElem = (id = '') => ({
    id,
    value: '',
    textContent: '',
    innerHTML: '',
    dataset: {},
    style: {},
    addEventListener: () => {},
    removeEventListener: () => {},
    classList: { add: () => {}, remove: () => {}, toggle: () => {}, contains: () => false },
    querySelectorAll: () => [],
    querySelector: () => null,
    appendChild: () => {},
    removeChild: () => {},
    replaceChildren: () => {},
    elements: { namedItem: () => null },
    getBoundingClientRect: () => ({ top: 0, bottom: 0, left: 0, right: 0, width: 300, height: 200 }),
  });
  sandboxWindow.document = {
    documentElement: {
      getAttribute: () => 'dark',
      setAttribute: () => {},
      style: { setProperty: () => {}, getPropertyValue: () => '' },
    },
    body: {
      classList: { add: () => {}, remove: () => {}, toggle: () => {}, contains: () => false },
    },
    getElementById: (id) => makeElem(id),
    querySelectorAll: () => [],
    querySelector: () => makeElem(),
    createElement: (tag) => makeElem(tag),
    addEventListener: (ev, fn) => { if (ev === 'DOMContentLoaded') fn(); },
  };

  const ctx = vm.createContext(sandboxWindow);
  vm.runInContext(fs.readFileSync(path.join(root, 'src', 'renderer', 'settings.js'), 'utf8'), ctx);
  vm.runInContext(fs.readFileSync(path.join(root, 'src', 'renderer', 'navigation.js'), 'utf8'), ctx);
  vm.runInContext(fs.readFileSync(path.join(root, 'src', 'renderer', 'routh.js'), 'utf8'), ctx);
  vm.runInContext(fs.readFileSync(path.join(root, 'src', 'renderer', 'scientific-modules.js'), 'utf8'), ctx);
  vm.runInContext(fs.readFileSync(path.join(root, 'src', 'renderer', 'app.js'), 'utf8'), ctx);

  assert(sandboxWindow.ControLABScientific, 'ControLABScientific precisa estar definido após carregar scripts');
  assert(sandboxWindow.LGRNavigation, 'LGRNavigation precisa estar definido após carregar scripts');
}

// Teste de escalabilidade da tela de início (UI scale via rem)
assert(tokens.includes('--home-console-height: 10rem'), 'A altura do console na tela de início precisa usar rem para responder à escala');
assert(tokens.includes('--home-left-col-width: 15rem'), 'A largura da coluna esquerda precisa usar rem para responder à escala');
assert(tokens.includes('--home-right-col-width: 18.75rem'), 'A largura da coluna direita precisa usar rem para responder à escala');
assert(styles.includes('.home-top-bar {\n  height: 2.75rem;'), 'O header da tela de início precisa usar rem para responder ao tamanho de interface');
assert(styles.includes('.home-panel-header {\n  height: 2rem;'), 'Os cabeçalhos de painel na tela de início precisam usar rem para responder ao tamanho de interface');
assert(styles.includes('.btn-home-primary {\n  height: 2.25rem;'), 'Os botões de ação na tela de início precisam usar rem para responder ao tamanho de interface');

// Validação do módulo de Projeto de Controladores em Série (LGR)
assert(scientificModules.includes("CONTROLLER_FORMULAS"), 'As fórmulas dos tipos de controladores precisam existir');
assert(scientificModules.includes("recomputeControllerTransients"), 'A conversão automática entre ts, Mp, xi e wn precisa existir na interface');
assert(scientificModules.includes("['parabola', 'Parábola 1/s³']"), 'A entrada de parábola unitária precisa estar disponível no projeto de controladores');
assert(html.includes('id="scientific-steps-panel"'), 'O painel expansível de memória de cálculo passo a passo precisa existir no HTML');

const figureLoad = mainProcess.indexOf("await figureWindow.loadFile(");
const figureLock = mainProcess.indexOf("figureWindow.webContents.on('will-navigate'");
assert(figureLoad >= 0 && figureLock > figureLoad, 'a trava de navegação da janela de figura precisa vir depois do loadFile, ou o Electron bloqueia o figure.html');
console.log('Integração, responsividade e navegação do renderer válidas.');
