const path = require('path');
const { spawn } = require('child_process');

const apk = path.join(__dirname, '..', 'android', 'app', 'build', 'outputs', 'apk', 'debug', 'app-debug.apk');
const child = spawn('adb', ['install', '-r', apk], { stdio: 'inherit' });

child.on('error', (error) => {
  const hint = error.code === 'ENOENT' ? ' Instale o Android Platform Tools e adicione adb ao PATH.' : '';
  console.error(`[LGR Studio] Falha ao executar adb: ${error.message}.${hint}`);
  process.exit(1);
});
child.on('exit', (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
