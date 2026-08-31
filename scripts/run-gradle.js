const path = require('path');
const fs = require('fs');
const { spawn, spawnSync } = require('child_process');

const androidDir = path.join(__dirname, '..', 'android');
const gradle = process.platform === 'win32' ? 'gradlew.bat' : './gradlew';
const tasks = process.argv.slice(2);

if (tasks.length === 0) {
  console.error('Uso: node scripts/run-gradle.js <tarefa>');
  process.exit(1);
}

function isPython310(executable) {
  const result = spawnSync(executable, [
    '-c',
    'import sys; raise SystemExit(0 if sys.version_info[:2] == (3, 10) else 1)',
  ]);
  return result.status === 0;
}

function findAndroidPython() {
  if (process.env.LGR_ANDROID_PYTHON && isPython310(process.env.LGR_ANDROID_PYTHON)) {
    return process.env.LGR_ANDROID_PYTHON;
  }

  const projectRoot = path.join(__dirname, '..');
  const localPython = process.platform === 'win32'
    ? path.join(projectRoot, '.venv-android', 'Scripts', 'python.exe')
    : path.join(projectRoot, '.venv-android', 'bin', 'python');

  for (const candidate of [localPython, 'python3.10']) {
    if ((!candidate.includes(path.sep) || fs.existsSync(candidate)) && isPython310(candidate)) {
      return candidate;
    }
  }

  let uvFind = spawnSync('uv', ['python', 'find', '3.10'], { encoding: 'utf8' });
  if (uvFind.status === 0 && isPython310(uvFind.stdout.trim())) {
    return uvFind.stdout.trim();
  }

  console.log('[LGR Studio] Python 3.10 não encontrado; instalando pelo uv para o Chaquopy...');
  const uvInstall = spawnSync('uv', ['python', 'install', '3.10'], { stdio: 'inherit' });
  if (uvInstall.status === 0) {
    uvFind = spawnSync('uv', ['python', 'find', '3.10'], { encoding: 'utf8' });
    if (uvFind.status === 0 && isPython310(uvFind.stdout.trim())) {
      return uvFind.stdout.trim();
    }
  }

  console.error(
    '[LGR Studio] Instale Python 3.10 ou o uv. Você também pode definir ' +
    'LGR_ANDROID_PYTHON com o caminho do Python 3.10.'
  );
  process.exit(1);
}

function findJdk() {
  if (process.env.JAVA_HOME && fs.existsSync(path.join(process.env.JAVA_HOME, 'bin', process.platform === 'win32' ? 'javac.exe' : 'javac'))) {
    return process.env.JAVA_HOME;
  }
  const candidates = [
    '/usr/lib/jvm/java-17-openjdk-amd64',
    '/usr/lib/jvm/java-1.17.0-openjdk-amd64',
    '/usr/lib/jvm/default-java',
    '/usr/lib/jvm/java-21-openjdk-amd64',
  ];
  for (const dir of candidates) {
    if (fs.existsSync(path.join(dir, 'bin', 'javac'))) {
      return dir;
    }
  }
  return process.env.JAVA_HOME;
}

const androidPython = findAndroidPython();
const jdk = findJdk();
console.log(`[LGR Studio] Python do build Android: ${androidPython}`);
if (jdk) {
  console.log(`[LGR Studio] JDK do build Android: ${jdk}`);
}

const child = spawn(gradle, tasks, {
  cwd: androidDir,
  stdio: 'inherit',
  shell: false,
  env: {
    ...process.env,
    LGR_ANDROID_PYTHON: androidPython,
    ...(jdk ? { JAVA_HOME: jdk } : {}),
  },
});
child.on('error', (error) => {
  console.error(`[LGR Studio] Falha ao iniciar Gradle: ${error.message}`);
  process.exit(1);
});
child.on('exit', (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
