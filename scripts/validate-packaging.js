const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const requiredFiles = [
  'capacitor.config.json',
  'electron-builder.config.cjs',
  'build/icon.png',
  'build/icon.ico',
  'build/icons/512x512.png',
  'android/gradlew',
  'android/app/build.gradle',
  'android/app/src/main/java/br/com/enthony/lgrstudio/LgrPythonPlugin.java',
  'android/app/src/main/python/lgr_engine.py',
  'android/app/src/main/python/control_utils.py',
  'android/app/src/main/python/plot_theme.py',
  'android/app/src/main/python/routh_hurwitz.py',
  'android/app/src/main/python/time_response.py',
  'android/app/src/main/python/frequency_response.py',
  'android/app/src/main/python/controller_design.py',
  'android/app/src/main/python/state_space.py',
  'android/app/src/main/python/python_bridge.py',
  'android/app/src/main/python/android_bridge.py',
];

for (const file of requiredFiles) {
  if (!fs.existsSync(path.join(root, file))) {
    throw new Error(`Arquivo de empacotamento ausente: ${file}`);
  }
}

for (const file of [
  'lgr_engine.py',
  'localization.py',
  'control_utils.py',
  'plot_theme.py',
  'routh_hurwitz.py',
  'time_response.py',
  'frequency_response.py',
  'controller_design.py',
  'state_space.py',
  'python_bridge.py',
  'android_bridge.py',
  'script_service.py',
]) {
  const canonical = fs.readFileSync(path.join(root, 'src', 'python', file));
  const android = fs.readFileSync(path.join(root, 'android', 'app', 'src', 'main', 'python', file));
  if (!canonical.equals(android)) {
    throw new Error(`Motor Android dessincronizado: execute npm run mobile:sync (${file})`);
  }
}

const gradle = fs.readFileSync(path.join(root, 'android', 'app', 'build.gradle'), 'utf8');
if (!/useLegacyPackaging\s*=\s*true/.test(gradle)) {
  throw new Error('O APK precisa extrair libcontrolab_runtime.so para executar o motor .m.');
}
const packageJson = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
if (!packageJson.scripts['mobile:sync'].includes('build:openmat:android')) {
  throw new Error('mobile:sync precisa compilar o motor .m do Android.');
}

const capacitor = JSON.parse(fs.readFileSync(path.join(root, 'capacitor.config.json'), 'utf8'));
if (capacitor.appId !== 'br.com.enthony.lgrstudio' || capacitor.webDir !== 'src/renderer') {
  throw new Error('Configuração Capacitor inválida.');
}

const desktop = require(path.join(root, 'electron-builder.config.cjs'));
if (!desktop.linux?.target?.includes('AppImage') || !desktop.win?.target?.includes('nsis')) {
  throw new Error('Alvos desktop obrigatórios não configurados.');
}

console.log('Configurações Android, Linux e Windows válidas.');

if (!fs.readFileSync(path.join(root, 'src/renderer/locales/en.json')).equals(fs.readFileSync(path.join(root, 'android/app/src/main/python/en.json')))) throw new Error('Catálogo Android dessincronizado.');
