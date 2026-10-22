/* Cross-compiles the .m runtime for the APK ABIs; Android only executes files from nativeLibraryDir. */
const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');
const root = path.join(__dirname, '..');
const target = path.join(root, 'build', 'openmat-android');
const jniLibs = path.join(root, 'android', 'app', 'src', 'main', 'jniLibs');
// Keep in sync with abiFilters and minSdkVersion in android/app/build.gradle and android/variables.gradle.
const API = 24;
const ABIS = [
  { abi: 'arm64-v8a', triple: 'aarch64-linux-android' },
  { abi: 'x86_64', triple: 'x86_64-linux-android' },
];

function newestChild(folder) {
  if (!fs.existsSync(folder)) return null;
  const versions = fs.readdirSync(folder).filter((name) => fs.statSync(path.join(folder, name)).isDirectory());
  versions.sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
  return versions.length ? path.join(folder, versions.at(-1)) : null;
}

function sdkFromLocalProperties() {
  try {
    const text = fs.readFileSync(path.join(root, 'android', 'local.properties'), 'utf8');
    const match = text.match(/^sdk\.dir=(.+)$/m);
    return match ? match[1].trim().replace(/\\:/g, ':').replace(/\\\\/g, '\\') : null;
  } catch {
    return null;
  }
}

function findNdk() {
  for (const variable of ['ANDROID_NDK_HOME', 'ANDROID_NDK_ROOT', 'ANDROID_NDK_LATEST_HOME', 'ANDROID_NDK']) {
    if (process.env[variable] && fs.existsSync(process.env[variable])) return process.env[variable];
  }
  for (const sdk of [process.env.ANDROID_HOME, process.env.ANDROID_SDK_ROOT, sdkFromLocalProperties()]) {
    const ndk = sdk && newestChild(path.join(sdk, 'ndk'));
    if (ndk) return ndk;
  }
  return null;
}

const ndk = findNdk();
if (!ndk) {
  console.error('Android NDK não encontrado. Instale o NDK pelo SDK Manager ou defina ANDROID_NDK_HOME para compilar o motor .m do Android.');
  process.exit(1);
}
const host = { linux: 'linux-x86_64', darwin: 'darwin-x86_64', win32: 'windows-x86_64' }[process.platform];
const bin = path.join(ndk, 'toolchains', 'llvm', 'prebuilt', host || '', 'bin');
if (!host || !fs.existsSync(bin)) {
  console.error(`Toolchain LLVM do NDK não encontrada em ${bin}.`);
  process.exit(1);
}

for (const { abi, triple } of ABIS) {
  const linker = path.join(bin, `${triple}${API}-clang${process.platform === 'win32' ? '.cmd' : ''}`);
  const env = { ...process.env, [`CARGO_TARGET_${triple.toUpperCase().replace(/-/g, '_')}_LINKER`]: linker };
  const result = spawnSync('cargo', ['build', ...(process.argv.includes('--offline') ? ['--offline'] : []), '--locked', '--release',
    '--manifest-path', path.join(root, 'vendor', 'openmat', 'Cargo.toml'),
    '--target-dir', target, '--target', triple, '-p', 'controlab-runtime'], { cwd: root, env, stdio: 'inherit' });
  if (result.error) console.error('Rust/Cargo é necessário para compilar o motor .m:', result.error.message);
  if (result.status !== 0) {
    console.error(`Falha ao compilar o motor .m para ${abi}. Instale o alvo com: rustup target add ${triple}`);
    process.exit(result.status ?? 1);
  }
  // The lib*.so name makes the packager ship the executable inside nativeLibraryDir.
  const destination = path.join(jniLibs, abi, 'libcontrolab_runtime.so');
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(path.join(target, triple, 'release', 'controlab-runtime'), destination);
  fs.chmodSync(destination, 0o755);
}
console.log('[ControLAB] Motor .m compilado para Android (arm64-v8a, x86_64).');
