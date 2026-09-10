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
assert(serviceWorker.includes("'/settings.js'"), 'settings.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/navigation.js'"), 'navigation.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/routh.js'"), 'routh.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/scientific-modules.js'"), 'scientific-modules.js precisa integrar o shell offline');
assert(scientificModules.includes("action: 'time_response'"), 'o módulo temporal precisa estar registrado');
assert(scientificModules.includes("action: 'frequency_response'"), 'o módulo de frequência precisa estar registrado');
assert(html.includes('id="card-module-time" class="module-card card-active"'), 'o card temporal precisa estar disponível');
assert(html.includes('id="card-module-frequency" class="module-card card-active"'), 'o card de frequência precisa estar disponível');
assert(html.includes('id="page-scientific"'), 'o workspace científico compartilhado precisa existir');
assert(serviceWorker.includes('mustBeFresh'), 'scripts do shell precisam de atualização network-first');
assert(styles.includes('@media (max-width: 1000px)'), 'o layout precisa adaptar os módulos em telas menores');
assert(styles.includes('#page-routh'), 'o módulo Routh precisa ter regras de layout próprias');
assert(styles.includes('.scientific-workspace'), 'os novos módulos precisam ter um layout responsivo próprio');
assert(styles.includes('prefers-reduced-motion: reduce'), 'a interface precisa respeitar redução de movimento');
assert(styles.includes('safe-area-inset-bottom'), 'os elementos fixos precisam respeitar a área segura móvel');
assert(styles.includes('.katex-display'), 'fórmulas extensas precisam permanecer navegáveis em telas estreitas');
assert(styles.includes('grid-template-columns: minmax(0, 1fr)'), 'cards do Routh não podem expandir além da tela móvel');

console.log('Integração, responsividade e navegação do renderer válidas.');
