const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const projectRoot = path.join(__dirname, '..');

function findPython() {
  if (process.env.PYTHON) return process.env.PYTHON;

  const venvPython = process.platform === 'win32'
    ? path.join(projectRoot, '.venv', 'Scripts', 'python.exe')
    : path.join(projectRoot, '.venv', 'bin', 'python');

  if (fs.existsSync(venvPython)) return venvPython;

  const candidates = process.platform === 'win32'
    ? ['python.exe', 'python', 'py.exe', 'py']
    : ['python3', 'python'];

  for (const cmd of candidates) {
    const test = spawnSync(cmd, ['-c', 'import sys; print(sys.executable)'], { encoding: 'utf8' });
    if (test.status === 0 && test.stdout.trim()) {
      return cmd;
    }
  }

  console.error('[LGR Studio] Python não encontrado. Crie a .venv ou instale Python 3.10+.');
  process.exit(1);
}

const python = findPython();
console.log(`[LGR Studio] Usando Python para build: ${python}`);

const checkPyinstaller = spawnSync(python, ['-c', 'import PyInstaller']);
if (checkPyinstaller.status !== 0) {
  console.log('[LGR Studio] PyInstaller não encontrado. Instalando requirements-build.txt...');
  const uvInstall = spawnSync('uv', ['pip', 'install', '-r', 'requirements-build.txt'], { cwd: projectRoot, stdio: 'inherit' });
  if (uvInstall.status !== 0) {
    spawnSync(python, ['-m', 'pip', 'install', '-r', 'requirements-build.txt'], { cwd: projectRoot, stdio: 'inherit' });
  }
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
  '--hidden-import', 'matplotlib.backends.backend_svg',
  '--hidden-import', 'matplotlib.backends.backend_agg',
  '--hidden-import', 'matplotlib.backends.backend_pdf',
  '--collect-all', 'matplotlib',
  '--collect-all', 'control',
  'python_bridge.py',
], { cwd: projectRoot, stdio: 'inherit' });

process.exit(result.status ?? 1);

