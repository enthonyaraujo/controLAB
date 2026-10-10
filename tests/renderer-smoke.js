const assert = require('assert');
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'src', 'renderer', 'index.html'), 'utf8');
const app = fs.readFileSync(path.join(root, 'src', 'renderer', 'app.js'), 'utf8');
const serviceWorker = fs.readFileSync(path.join(root, 'src', 'renderer', 'sw.js'), 'utf8');
const styles = fs.readFileSync(path.join(root, 'src', 'renderer', 'styles.css'), 'utf8');
const scientificModules = fs.readFileSync(path.join(root, 'src', 'renderer', 'scientific-modules.js'), 'utf8');

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

console.log('Integração, responsividade e navegação do renderer válidas.');
