const fs = require('fs');
const path = require('path');

const projectRoot = path.join(__dirname, '..');
const pythonTarget = path.join(projectRoot, 'android', 'app', 'src', 'main', 'python');
const sources = [
  'lgr_engine.py',
  'control_utils.py',
  'routh_hurwitz.py',
  'time_response.py',
  'python_bridge.py',
  'android_bridge.py',
];

fs.mkdirSync(pythonTarget, { recursive: true });
for (const filename of sources) {
  fs.copyFileSync(path.join(projectRoot, filename), path.join(pythonTarget, filename));
}

console.log('[ControLAB] Motor Python sincronizado com o projeto Android.');
