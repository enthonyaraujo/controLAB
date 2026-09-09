const path = require('path');

const pythonExecutable = process.platform === 'win32' ? 'lgr-bridge.exe' : 'lgr-bridge';

module.exports = {
  appId: 'br.com.enthony.lgrstudio',
  productName: 'ControLAB',
  publish: null,
  icon: 'build/icon.png',
  artifactName: '${productName}-${version}-${os}-${arch}.${ext}',
  directories: {
    output: 'release',
  },
  files: [
    'src/**/*',
    'scripts/start-electron.js',
    'package.json',
  ],
  extraResources: [
    {
      from: path.join('build', 'python', pythonExecutable),
      to: path.join('python', pythonExecutable),
    },
  ],
  asar: true,
  linux: {
    category: 'Education;Science',
    executableName: 'controlab',
    maintainer: 'Enthony Araujo <contato@enthony.com.br>',
    target: ['AppImage', 'deb', 'rpm'],
  },
  win: {
    executableName: 'ControLAB',
    target: ['nsis'],
  },
  nsis: {
    oneClick: false,
    perMachine: false,
    allowToChangeInstallationDirectory: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    shortcutName: 'ControLAB',
    uninstallDisplayName: 'ControLAB',
    runAfterFinish: true,
  },
};
