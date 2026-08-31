const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const projectRoot = path.join(__dirname, '..');
const python = process.platform === 'win32'
  ? path.join(projectRoot, '.venv', 'Scripts', 'python.exe')
  : path.join(projectRoot, '.venv', 'bin', 'python');

if (!fs.existsSync(python)) {
  console.error('[LGR Studio] Crie a .venv e instale requirements.txt e requirements-build.txt.');
  process.exit(1);
}

const result = spawnSync(python, [
  '-m', 'PyInstaller',
  '--noconfirm',
  '--clean',
  '--onefile',
  '--name', 'lgr-bridge',
  '--distpath', path.join('build', 'python'),
  '--workpath', path.join('build', 'pyinstaller'),
  '--specpath', path.join('build', 'pyinstaller'),
  'python_bridge.py',
], { cwd: projectRoot, stdio: 'inherit' });

process.exit(result.status ?? 1);
