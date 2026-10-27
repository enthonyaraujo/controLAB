/* PyInstaller resolves --add-data sources relative to the generated spec file. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { test } = require('node:test');
const script = fs.readFileSync(path.join(__dirname, '../scripts/build-python.js'), 'utf8');

for (const platform of ['linux', 'win32']) {
  test(`Python build includes the translation catalog on ${platform}`, () => {
    const paths = platform === 'win32' ? path.win32 : path.posix;
    const root = platform === 'win32' ? 'C:\\workspace\\controlab' : '/workspace/controlab';
    const catalog = paths.join(root, 'src', 'renderer', 'locales', 'en.json');
    const python = paths.join(root, '.venv', platform === 'win32' ? 'Scripts' : 'bin', platform === 'win32' ? 'python.exe' : 'python');
    let build;
    vm.runInNewContext(script, {
      __dirname: paths.join(root, 'scripts'),
      require(name) {
        if (name === 'path') return paths;
        if (name === 'fs') return { existsSync: (file) => file === python || file === catalog };
        if (name === 'child_process') return { spawnSync(command, args, options) {
          if (args.includes('PyInstaller') && !args.includes('-c')) build = { command, args, options };
          return { status: 0, stdout: '' };
        } };
        throw new Error(`Unexpected module ${name}`);
      },
      console: { log() {}, error() {} },
      process: { platform, env: {}, exit(code) { assert.equal(code, 0); } },
    }, { filename: 'build-python.js' });
    assert.ok(build, 'PyInstaller must be invoked');
    const source = build.args[build.args.indexOf('--add-data') + 1].slice(0, -(paths.delimiter.length + 1));
    const specDir = paths.resolve(root, build.args[build.args.indexOf('--specpath') + 1]);
    assert.equal(paths.resolve(specDir, source), catalog,
      'The catalog must resolve from build/pyinstaller, not only from the working directory');
    assert.equal(build.options.cwd, root);
  });
}
