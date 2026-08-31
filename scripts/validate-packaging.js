const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const requiredFiles = [
  'capacitor.config.json',
  'electron-builder.config.cjs',
  'android/gradlew',
  'android/app/build.gradle',
  'android/app/src/main/java/br/com/enthony/lgrstudio/LgrPythonPlugin.java',
  'android/app/src/main/python/lgr_engine.py',
  'android/app/src/main/python/python_bridge.py',
  'android/app/src/main/python/android_bridge.py',
];

for (const file of requiredFiles) {
  if (!fs.existsSync(path.join(root, file))) {
    throw new Error(`Arquivo de empacotamento ausente: ${file}`);
  }
}

for (const file of ['lgr_engine.py', 'python_bridge.py', 'android_bridge.py']) {
  const canonical = fs.readFileSync(path.join(root, file));
  const android = fs.readFileSync(path.join(root, 'android', 'app', 'src', 'main', 'python', file));
  if (!canonical.equals(android)) {
    throw new Error(`Motor Android dessincronizado: execute npm run mobile:sync (${file})`);
  }
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
