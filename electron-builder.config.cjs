const path = require('path');

const pythonExecutable = process.platform === 'win32' ? 'lgr-bridge.exe' : 'lgr-bridge';

module.exports = {
  appId: 'br.com.enthony.lgrstudio',
  productName: 'ControLAB',
  publish: null,
  icon: 'build/icon.png',
  artifactName: '${productName}-v${version}-${os}-${arch}.${ext}',
  directories: {
    output: 'release',
    buildResources: 'build',
  },
  files: [
    'src/**/*',
    '!src/python/**/*',
    'scripts/start-electron.js',
    'package.json',
  ],
  extraResources: [
    { from: path.join('vendor', 'openmat', 'LICENSE'), to: path.join('licenses', 'openmat.txt') },
    { from: path.join('vendor', 'openmat', 'THIRD_PARTY_NOTICES.txt'), to: path.join('licenses', 'rust-runtime-notices.txt') },
    {
      from: path.join('build', 'runtime', process.platform === 'win32' ? 'controlab-runtime.exe' : 'controlab-runtime'),
      to: path.join('python', process.platform === 'win32' ? 'controlab-runtime.exe' : 'controlab-runtime'),
    },
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
    icon: 'build/icons',
    syncDesktopName: true,
  },
  win: {
    executableName: 'ControLAB',
    icon: 'build/icon.ico',
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
    installerIcon: 'build/icon.ico',
    uninstallerIcon: 'build/icon.ico',
    runAfterFinish: true,
  },
};
