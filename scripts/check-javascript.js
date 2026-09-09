const path = require('path');
const { spawnSync } = require('child_process');

const projectRoot = path.join(__dirname, '..');
const javascriptFiles = [
  'scripts/start-electron.js',
  'scripts/run-python.js',
  'scripts/sync-mobile.js',
  'scripts/run-gradle.js',
  'scripts/install-apk.js',
  'scripts/build-python.js',
  'scripts/assert-platform.js',
  'scripts/validate-packaging.js',
  'scripts/check-javascript.js',
  'electron-builder.config.cjs',
  'src/main/main.js',
  'src/preload/preload.js',
  'src/renderer/app.js',
  'src/renderer/navigation.js',
  'src/renderer/web-api.js',
  'src/renderer/sw.js',
  'tests/renderer-smoke.js',
];

for (const file of javascriptFiles) {
  const result = spawnSync(process.execPath, ['--check', file], {
    cwd: projectRoot,
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    process.stderr.write(result.stderr || result.stdout);
    process.exit(result.status ?? 1);
  }
}

const packaging = spawnSync(process.execPath, ['scripts/validate-packaging.js'], {
  cwd: projectRoot,
  stdio: 'inherit',
});
process.exit(packaging.status ?? 1);
