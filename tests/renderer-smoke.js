const assert = require('assert');
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'src', 'renderer', 'index.html'), 'utf8');
const app = fs.readFileSync(path.join(root, 'src', 'renderer', 'app.js'), 'utf8');
const serviceWorker = fs.readFileSync(path.join(root, 'src', 'renderer', 'sw.js'), 'utf8');

const settingsPosition = html.indexOf('<script src="settings.js"></script>');
const navigationPosition = html.indexOf('<script src="navigation.js"></script>');
const appPosition = html.indexOf('<script src="app.js"></script>');

assert(settingsPosition >= 0, 'settings.js precisa estar carregado pelo renderer');
assert(navigationPosition >= 0, 'navigation.js precisa estar carregado pelo renderer');
assert(appPosition > navigationPosition, 'navigation.js precisa carregar antes de app.js');
assert(app.includes('window.LGRNavigation.initialize'), 'app.js precisa inicializar a navegação');
assert(app.includes('window.ControLABSettings.initialize'), 'app.js precisa inicializar as configurações');
assert(serviceWorker.includes("'/settings.js'"), 'settings.js precisa integrar o shell offline');
assert(serviceWorker.includes("'/navigation.js'"), 'navigation.js precisa integrar o shell offline');
assert(serviceWorker.includes('mustBeFresh'), 'scripts do shell precisam de atualização network-first');

console.log('Integração do renderer, configurações e navegação válida.');
