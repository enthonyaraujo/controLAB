const path = require('path');
const { spawn } = require('child_process');

const androidDir = path.join(__dirname, '..', 'android');
const gradle = process.platform === 'win32' ? 'gradlew.bat' : './gradlew';
const tasks = process.argv.slice(2);

if (tasks.length === 0) {
  console.error('Uso: node scripts/run-gradle.js <tarefa>');
  process.exit(1);
}

const child = spawn(gradle, tasks, { cwd: androidDir, stdio: 'inherit', shell: false });
child.on('error', (error) => {
  console.error(`[LGR Studio] Falha ao iniciar Gradle: ${error.message}`);
  process.exit(1);
});
child.on('exit', (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
