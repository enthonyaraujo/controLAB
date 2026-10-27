const fs = require('node:fs');
const path = require('node:path');
const directory = path.join(__dirname, '../src/renderer/locales');
const catalog = JSON.parse(fs.readFileSync(path.join(directory, 'en.json'), 'utf8'));
fs.writeFileSync(path.join(directory, 'en.js'), '// Generated from en.json by scripts/build-locales.js.\nwindow.ControLABEnglish = ' + JSON.stringify(catalog, null, 2) + ';\n');
