const fs = require('fs');
const { spawn } = require('child_process');
const electron = require('electron');

const userArgs = process.argv.slice(2);
const electronOptions = userArgs.filter((arg) => arg.startsWith('-'));
const appArgs = userArgs.filter((arg) => !arg.startsWith('-'));

if (process.platform === 'linux' && !electronOptions.includes('--no-sandbox')) {
  const sandboxPath = `${electron.slice(0, electron.lastIndexOf('/'))}/chrome-sandbox`;

  try {
    const stat = fs.statSync(sandboxPath);
    const hasSetuidRootSandbox = stat.uid === 0 && (stat.mode & 0o4000) !== 0;

    if (!hasSetuidRootSandbox) {
      console.warn(
        '[ControLAB] chrome-sandbox sem permissão SUID; iniciando com --no-sandbox.\n' +
        'Para manter o sandbox completo, configure o arquivo como root:4755.'
      );
      electronOptions.push('--no-sandbox');
    }
  } catch {
    console.warn('[ControLAB] chrome-sandbox não encontrado; iniciando com --no-sandbox.');
    electronOptions.push('--no-sandbox');
  }
}

const child = spawn(electron, [...electronOptions, '.', ...appArgs], { stdio: 'inherit' });

child.on('error', (error) => {
  console.error(`[ControLAB] Não foi possível iniciar o Electron: ${error.message}`);
  process.exitCode = 1;
});

child.on('exit', (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
