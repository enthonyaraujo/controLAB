const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const projectRoot = path.join(__dirname, '..');
const commandArgs = process.argv.slice(2);

if (commandArgs.length === 0) {
  console.error('Uso: node scripts/run-python.js <script.py> [argumentos]');
  process.exit(1);
}

const candidates = process.platform === 'win32'
  ? [path.join(projectRoot, '.venv', 'Scripts', 'python.exe'), 'py', 'python']
  : [path.join(projectRoot, '.venv', 'bin', 'python'), 'python3', 'python'];

function tryCandidate(index) {
  if (index >= candidates.length) {
    console.error('[ControLAB] Python não encontrado. Crie a .venv ou instale Python 3.12+.');
    process.exit(1);
  }

  const executable = candidates[index];
  if (executable.includes(path.sep) && !fs.existsSync(executable)) {
    tryCandidate(index + 1);
    return;
  }

  const args = executable === 'py' ? ['-3', ...commandArgs] : commandArgs;
  const child = spawn(executable, args, { cwd: projectRoot, stdio: 'inherit' });

  child.on('error', (error) => {
    if (error.code === 'ENOENT') {
      tryCandidate(index + 1);
      return;
    }
    console.error(`[ControLAB] Falha ao iniciar Python: ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    process.exitCode = code ?? (signal ? 1 : 0);
  });
}

tryCandidate(0);
