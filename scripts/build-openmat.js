const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');
const root = path.join(__dirname, '..');
const target = path.join(root, 'build', 'openmat');
const result = spawnSync('cargo', ['build', ...(process.argv.includes('--offline') ? ['--offline'] : []), '--locked', '--release',
  '--manifest-path', path.join(root, 'vendor', 'openmat', 'Cargo.toml'),
  '--target-dir', target, '-p', 'controlab-runtime'], { cwd: root, stdio: 'inherit' });
if (result.error) console.error('Rust/Cargo é necessário para compilar o motor .m:', result.error.message);
if (result.status === 0) {
  fs.mkdirSync(path.join(root, 'build', 'runtime'), { recursive: true });
  const name = process.platform === 'win32' ? 'controlab-runtime.exe' : 'controlab-runtime';
  const destination = path.join(root, 'build', 'runtime', name);
  const pending = `${destination}.${process.pid}.tmp`;
  try {
    fs.copyFileSync(path.join(target, 'release', name), pending);
    fs.renameSync(pending, destination);
  } catch (error) {
    fs.rmSync(pending, { force: true });
    console.error('Não foi possível atualizar o motor .m. Feche o ControLAB e tente novamente:', error.message);
    process.exit(1);
  }
}
process.exit(result.status ?? 1);
