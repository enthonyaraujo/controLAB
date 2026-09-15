const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

// Executa o renderer real com eventos, relógio e transporte controlados.
const elements = new Map();
function element(id) {
  if (!elements.has(id)) {
    const classes = new Set();
    elements.set(id, {
      value: '', style: {}, dataset: {}, handlers: {}, textContent: '',
      classList: {
        add: (name) => classes.add(name), remove: (name) => classes.delete(name),
        contains: (name) => classes.has(name),
        toggle: (name, enabled) => enabled ? classes.add(name) : classes.delete(name),
      },
      addEventListener(name, handler) { this.handlers[name] = handler; },
      setAttribute() {}, getAttribute() {}, querySelector() { return element('child'); },
    });
  }
  return elements.get(id);
}
let clock = 0;
let timerId = 0;
const timers = new Map();
const schedule = (fn, delay) => { const id = ++timerId; timers.set(id, {fn, at:clock+delay}); return id; };
async function tick(ms) {
  clock += ms;
  for (const [id, timer] of [...timers]) {
    if (timer.at <= clock) { timers.delete(id); timer.fn(); }
  }
  await Promise.resolve(); await Promise.resolve();
}
const calls = [];
function request(payload) {
  return new Promise(resolve => calls.push({payload, resolve}));
}
const docHandlers = {};
const document = {
  hidden: false, documentElement: element('html'), body: element('body'),
  getElementById: element, querySelectorAll: () => [],
  addEventListener: (name, handler) => { docHandlers[name] = handler; },
};
const window = {
  setTimeout: schedule, clearTimeout: id => timers.delete(id), addEventListener() {},
  api: {previewTransferFunction: request, calculateLGR: request},
};
const context = {document, window, localStorage: {getItem: () => null}, setTimeout:schedule, clearTimeout:window.clearTimeout, console};
vm.runInNewContext(fs.readFileSync(path.join(__dirname, '../src/renderer/app.js'), 'utf8'), context);
docHandlers.DOMContentLoaded();
const input = element('input-expr');
const calculate = element('btn-calculate').handlers.click;
function edit(value) { input.value = value; input.handlers.input(); }
const valid = {success:true, latex_fac:'valid', latex_exp:'expanded', image:'png', svg:'svg',
  detalhes:{P:1,Z:0,ramos:1,centroide:null}};

(async () => {
  edit('1/(s+1)');
  await tick(450);
  assert.equal(calls.length,1);
  edit('1/(s+2)');
  await tick(450);
  assert.equal(calls.length,1, 'A prévia seguinte aguarda a requisição atual');
  calls[0].resolve(valid);
  await tick(0); await tick(450);
  assert.equal(calls.length,2);
  assert.equal(calls[1].payload.expr,'1/(s+2)');
  calls[1].resolve(valid); await tick(0);
  edit('1/(s+2)'); await tick(450);
  assert.equal(calls.length,2, 'Entrada idêntica reutiliza a prévia');
  edit('invalid'); await tick(450);
  calls[2].resolve({success:false,error:'Inválida'}); await tick(0);
  edit('1/(s+2)'); await tick(450);
  assert.equal(calls.length,4, 'Voltar a uma entrada válida limpa o erro anterior');
  calls[3].resolve(valid); await tick(0);
  document.hidden = true;
  edit('1/(s+3)'); await tick(450);
  assert.equal(calls.length,4, 'Página oculta não inicia prévias');
  document.hidden = false;
  const first = calculate();
  await calculate();
  assert.equal(calls.length,5, 'Cálculos não se sobrepõem');
  calls[4].resolve(valid); await first;
  await calculate();
  assert.equal(calls.length,5, 'O mesmo gráfico não é recalculado');
  console.log('Prévias, concorrência e reutilização do renderer validadas.');
})().catch(error => { console.error(error); process.exitCode = 1; });
