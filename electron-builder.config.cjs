const path = require('path');

const pythonExecutable = process.platform === 'win32' ? 'lgr-bridge.exe' : 'lgr-bridge';

module.exports = {
  appId: 'br.com.enthony.lgrstudio',
  productName: 'LGR Studio',
  publish: null,
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
    executableName: 'lgr-studio',
    maintainer: 'Enthony Araujo <contato@enthony.com.br>',
    target: ['AppImage', 'deb', 'rpm'],
  },
  win: {
    executableName: 'LGR Studio',
    target: ['nsis', 'portable'],
  },
  nsis: {
    oneClick: false,
    allowToChangeInstallationDirectory: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
  },
};
