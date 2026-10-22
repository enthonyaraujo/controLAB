/* Script editor for the openMAT core vendored in ControLAB. */
(function registerStudio(global) {
  'use strict';
  let controller = null;
  const MAX_BYTES = 1024 * 1024;
  const EXAMPLES = {
    matrix: { name: 'multiplicacao_matrizes.m', source: '% ControLAB · multiplicação matricial\nA = [1 2; 3 4];\nB = [5 6; 7 8];\nC = A * B;\ndisp(C);\n' },
    linear: { name: 'sistema_linear.m', source: '% ControLAB · solução de A*x = b\nA = [4 -2; 1 1];\nb = [2; 3];\nx = A \\ b\n' },
    plot: { name: 'grafico_seno.m', source: '% ControLAB · gráfico de uma função\nx = 0:0.1:6;\nplot(x, sin(x));\n' },
    step: { name: 'degrau_segunda_ordem.m', source: '% ControLAB · resposta ao degrau de um sistema de 2ª ordem\nwn = 2;\nzeta = 0.3;\nG = tf([wn^2], [1 2*zeta*wn wn^2]);\nstep(G);\n' },
  };
  const escape = (text) => text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  function initialize({ showToast = () => {} } = {}) {
    if (controller) return controller;
    const element = (id) => document.getElementById(id);
    const source = element('studio-source');
    if (!source) return null;
    const home = element('page-home');
    const logs = element('home-console-logs');
    const session = global.crypto?.randomUUID?.() || `studio-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const documents = [];
    let activeId = null;
    let nextId = 1;
    let folder = '';
    let parentFolder = '';
    let folderHistory = [];
    let folderHistoryIndex = -1;
    let pickerFolder = '';
    let pickerParent = '';
    let pickerGeneration = 0;
    let files = [];
    let busy = false;
    let listGeneration = 0;
    let highlightTimer = null;
    let plots = [];
    let available = false;

    function status(text) { element('studio-status').textContent = text; }
    function log(text, type = 'output') {
      if (!text) return;
      const line = document.createElement('div');
      line.className = `home-console-log-line ${type}`;
      line.textContent = text;
      logs.appendChild(line);
      while (logs.children.length > 250) logs.firstElementChild.remove();
      logs.scrollTop = logs.scrollHeight;
    }
    async function request(operation, data = {}) {
      if (!global.api?.scriptRequest) throw new Error('O motor de scripts não está disponível nesta plataforma.');
      const result = await global.api.scriptRequest({ operation, session, ...data });
      if (!result || (result.success === false && !result.diagnostic && !result.exists)) {
        throw new Error(result?.error || 'Não foi possível concluir a operação.');
      }
      return result;
    }
    const current = () => documents.find((doc) => doc.id === activeId);
    const dirty = (doc) => doc.source !== doc.saved;
    function syncDocument() { const doc = current(); if (doc) doc.source = source.value; }
    function updateCursor() {
      const prefix = source.value.slice(0, source.selectionStart);
      const lines = prefix.split('\n');
      element('studio-cursor').textContent = `Ln ${lines.length}, Col ${lines.at(-1).length + 1}`;
    }
    function updateHighlight() {
      const text = source.value;
      // Match tokens before escaping, so source text is never interpreted as markup.
      const token = /(%[^\n]*|'(?:[^'\n]|'')*'|\b\d+(?:\.\d*)?(?:[eE][+-]?\d+)?\b)/g;
      let html = '', previous = 0;
      for (const match of text.matchAll(token)) {
        html += escape(text.slice(previous, match.index));
        const kind = match[0][0] === '%' ? 'comment' : match[0][0] === "'" ? 'string' : 'number';
        html += `<span class="studio-${kind}">${escape(match[0])}</span>`;
        previous = match.index + match[0].length;
      }
      element('studio-highlight').innerHTML = `${html}${escape(text.slice(previous))}\n`;
      element('studio-line-numbers').textContent = Array.from({ length: text.split('\n').length }, (_, i) => i + 1).join('\n');
      syncScroll();
      updateCursor();
    }
    function syncScroll() {
      const highlight = element('studio-highlight');
      highlight.scrollTop = source.scrollTop;
      highlight.scrollLeft = source.scrollLeft;
      element('studio-line-numbers').scrollTop = source.scrollTop;
    }
    function renderTabs() {
      const tabs = element('studio-tabs');
      tabs.replaceChildren();
      documents.forEach((doc) => {
        const tab = document.createElement('div');
        tab.className = 'studio-tab';
        tab.setAttribute('role', 'presentation');
        tab.setAttribute('aria-selected', String(doc.id === activeId));
        const button = document.createElement('button');
        button.type = 'button';
        button.setAttribute('role', 'tab');
        button.setAttribute('aria-selected', String(doc.id === activeId));
        button.setAttribute('aria-controls', 'studio-editor-pane');
        button.textContent = `${doc.name}${dirty(doc) ? ' •' : ''}`;
        button.title = doc.path || doc.name;
        button.addEventListener('click', () => activate(doc.id));
        const close = document.createElement('button');
        close.type = 'button'; close.className = 'studio-close-tab';
        close.textContent = '×'; close.setAttribute('aria-label', `Fechar ${doc.name}`);
        close.addEventListener('click', () => closeDocument(doc.id));
        tab.append(button, close); tabs.append(tab);
      });
      const doc = current();
      const empty = !documents.length;
      element('studio-welcome').hidden = !empty;
      element('home-workspace-central').classList.toggle('studio-welcome-active', empty);
      element('studio-editor-title').textContent = empty ? 'Início' : `Editor — ${doc?.name || ''}`;
      element('studio-document-status').textContent = `${doc?.path || doc?.name || ''}${doc && dirty(doc) ? ' · Não salvo' : ''}`;
    }
    function showView(view) {
      const figures = view === 'figures';
      element('studio-editor-pane').hidden = figures;
      element('studio-figures').hidden = !figures;
      element('studio-editor-view').setAttribute('aria-pressed', String(!figures));
      element('studio-figures-view').setAttribute('aria-pressed', String(figures));
    }
    function activate(id) {
      syncDocument();
      activeId = id;
      source.value = current().source;
      source.scrollTop = source.scrollLeft = 0;
      renderTabs(); updateHighlight(); showView('editor'); source.focus();
    }
    function newDocument(text = '', name = null, path = null, saved = null) {
      syncDocument();
      const id = nextId++;
      const initialSaved = saved !== null && saved !== undefined ? saved : text;
      documents.push({ id, name: name || `Sem título ${id}.m`, source: text, path, saved: initialSaved });
      activate(id);
    }
    function closeDocument(id) {
      syncDocument();
      const doc = documents.find((item) => item.id === id);
      if (dirty(doc) && !global.confirm(`Fechar ${doc.name} e descartar as alterações não salvas?`)) return;
      documents.splice(documents.indexOf(doc), 1);
      // The source still belongs to the closed tab; don't sync it into another document.
      if (activeId === id) {
        activeId = null;
        if (documents.length) activate(documents.at(-1).id);
        else { source.value = ''; renderTabs(); updateHighlight(); }
      } else renderTabs();
    }
    function openExample(key) {
      const example = EXAMPLES[key];
      if (!example) return;
      const existing = documents.find((doc) => !doc.path && doc.name === example.name);
      if (existing) activate(existing.id); else newDocument(example.source, example.name);
      status(`Exemplo: ${example.name} · F5 executa`);
    }
    function runCurrent() {
      syncDocument();
      const doc = current();
      if (!doc) { status('Abra ou crie um script para executar'); return; }
      return execute(doc.source, doc.name);
    }
    function addRecentScript(result) {
      global.ControLABRecents?.addRecent({ name: result.name, file: result.path, type: 'script' });
    }
    async function openPath(path) {
      try {
        const existing = documents.find((doc) => doc.path === path);
        if (existing) { activate(existing.id); return; }
        const result = await request('read', { path });
        newDocument(result.source, result.name, result.path, result.source);
        addRecentScript(result);
        home.classList.remove('studio-show-files');
        status(`Aberto: ${result.name}`);
      } catch (error) { log(error.message, 'error'); showToast(error.message, 'error'); }
    }
    async function openFile() {
      if (global.api?.openScriptDialog) {
        try {
          const result = await global.api.openScriptDialog();
          if (!result.canceled && result.path) await openPath(result.path);
        } catch (error) { log(error.message, 'error'); }
      } else element('studio-file-input').click();
    }
    function saveName(name) {
      return new Promise((resolve) => {
        const dialog = element('studio-save-dialog');
        element('studio-save-name').value = name.startsWith('Sem título') ? 'script.m' : name;
        dialog.returnValue = 'cancel';
        dialog.addEventListener('close', () => resolve(dialog.returnValue === 'save' ? element('studio-save-name').value.trim() : null), { once: true });
        dialog.showModal();
      });
    }
    async function save() {
      syncDocument();
      const doc = current();
      if (!doc) return;
      const text = doc.source;
      try {
        if (new TextEncoder().encode(text).length > MAX_BYTES) throw new Error('Script excede 1 MiB.');
        let path = doc.path;
        if (!path && global.api?.saveScriptDialog) {
          const result = await global.api.saveScriptDialog(doc.name);
          if (result.canceled || !result.path) return;
          path = /\.m$/i.test(result.path) ? result.path : `${result.path}.m`;
        }
        if (!path) {
          const name = await saveName(doc.name);
          if (!name) return;
          if (!/^[^/\\]+\.m$/i.test(name) || name.includes('\0')) throw new Error('Use um nome de arquivo com extensão .m.');
          if (!available) {
            const url = URL.createObjectURL(new Blob([text], { type: 'text/plain;charset=utf-8' }));
            const link = document.createElement('a'); link.href = url; link.download = name; link.click();
            setTimeout(() => URL.revokeObjectURL(url), 1000);
            doc.name = name; doc.saved = text; renderTabs(); status(`Exportado: ${name}`); return;
          }
          path = `${folder.replace(/[\\/]$/, '')}/${name}`;
        }
        let result = await request('save', { path, source: text, overwrite: path === doc.path });
        if (result.exists) {
          if (!global.confirm(`O arquivo ${path} já existe. Substituir?`)) return;
          result = await request('save', { path, source: text, overwrite: true });
        }
        if (!result.success) throw new Error(result.error || 'Não foi possível salvar.');
        doc.path = result.path; doc.name = result.name; doc.saved = text;
        addRecentScript(result);
        renderTabs(); status(`Salvo: ${result.name}`); await refreshFolder(folder);
      } catch (error) { log(error.message, 'error'); showToast(error.message, 'error'); }
    }
    function renderFiles() {
      const list = element('home-file-list');
      list.replaceChildren();
      const search = element('studio-file-search').value.toLocaleLowerCase();
      files.filter((file) => file.name.toLocaleLowerCase().includes(search)).forEach((file) => {
        const li = document.createElement('li');
        const button = document.createElement('button'); button.type = 'button'; button.className = 'studio-file-button';
        const icon = document.createElement('span'); icon.className = `studio-file-icon${file.directory ? ' folder' : ''}`; icon.setAttribute('aria-hidden', 'true');
        const name = document.createElement('span'); name.className = 'studio-file-name'; name.textContent = file.name;
        button.title = file.path; button.append(icon, name);
        button.addEventListener('click', () => file.directory ? refreshFolder(file.path) : openPath(file.path));
        li.append(button); list.append(li);
      });
      if (!list.children.length) {
        const hint = document.createElement('li'); hint.className = 'home-empty-hint';
        hint.textContent = search ? 'Nenhum arquivo encontrado.' : 'Nenhum script .m nesta pasta.';
        list.append(hint);
      }
    }
    function updateFolderButtons() {
      element('studio-back').disabled = folderHistoryIndex <= 0;
      element('studio-forward').disabled = folderHistoryIndex >= folderHistory.length - 1;
      element('studio-up').disabled = !parentFolder;
    }
    async function navigateHistory(offset) {
      const index = folderHistoryIndex + offset;
      if (index < 0 || index >= folderHistory.length) return;
      await refreshFolder(folderHistory[index], { historyIndex: index });
    }
    async function refreshFolder(path, options = {}) {
      const generation = ++listGeneration;
      try {
        const result = await request('list', { path: path || undefined });
        if (generation !== listGeneration) return;
        if (Number.isInteger(options.historyIndex)) {
          folderHistoryIndex = options.historyIndex;
        } else if (result.folder !== folderHistory[folderHistoryIndex]) {
          folderHistory = folderHistory.slice(0, folderHistoryIndex + 1);
          folderHistory.push(result.folder);
          if (folderHistory.length > 50) folderHistory.shift();
          folderHistoryIndex = folderHistory.length - 1;
        }
        folder = result.folder; parentFolder = result.parent; files = result.files;
        element('studio-folder-input').value = folder;
        element('home-folder-path-text').textContent = folder;
        updateFolderButtons();
        renderFiles();
        global.api?.setWorkspaceFolder?.(folder);
      } catch (error) {
        if (generation === listGeneration) { log(error.message, 'error'); status('Pasta indisponível'); }
      }
    }
    async function browsePicker(path) {
      const generation = ++pickerGeneration;
      const confirm = element('studio-picker-confirm');
      confirm.disabled = true;
      element('studio-picker-error').textContent = 'Carregando…';
      try {
        const result = await request('list', { path: path || undefined });
        if (generation !== pickerGeneration || !element('studio-folder-dialog').open) return;
        pickerFolder = result.folder; pickerParent = result.parent;
        element('studio-picker-path').value = pickerFolder;
        element('studio-picker-up').disabled = !pickerParent;
        const list = element('studio-picker-list'); list.replaceChildren();
        result.files.filter((entry) => entry.directory).forEach((entry) => {
          const li = document.createElement('li');
          const button = document.createElement('button'); button.type = 'button'; button.className = 'studio-file-button';
          const icon = document.createElement('span'); icon.className = 'studio-file-icon folder'; icon.setAttribute('aria-hidden', 'true');
          const name = document.createElement('span'); name.textContent = entry.name;
          button.append(icon, name); button.addEventListener('click', () => browsePicker(entry.path));
          li.append(button); list.append(li);
        });
        element('studio-picker-error').textContent = list.children.length ? '' : 'Nenhuma subpasta.';
        confirm.disabled = false;
      } catch (error) {
        if (generation === pickerGeneration) element('studio-picker-error').textContent = error.message;
      }
    }
    async function selectFolder() {
      if (global.api?.openScriptDialog && global.api?.selectWorkspaceFolder) {
        try {
          const result = await global.api.selectWorkspaceFolder();
          if (result.success && result.folder) await refreshFolder(result.folder);
        } catch (error) { log(error.message, 'error'); }
        return;
      }
      const dialog = element('studio-folder-dialog');
      if (dialog.open) return;
      dialog.showModal();
      await browsePicker(folder);
    }
    function renderWorkspace(variables) {
      const list = element('home-workspace-list'); list.replaceChildren();
      variables.forEach((variable) => {
        const li = document.createElement('li');
        const button = document.createElement('button'); button.type = 'button'; button.className = 'studio-variable';
        button.title = `${variable.name} · ${variable.type} · ${variable.dimensions}\n${variable.value}`;
        const name = document.createElement('span'); name.textContent = variable.name;
        const value = document.createElement('span'); value.textContent = variable.value;
        button.append(name, value); button.addEventListener('click', () => executeCommand(variable.name));
        li.append(button); list.append(li);
      });
      if (!variables.length) {
        const hint = document.createElement('li'); hint.className = 'home-empty-hint'; hint.textContent = 'Nenhuma variável no workspace'; list.append(hint);
      }
    }
    function renderPlots(newPlots) {
      plots.push(...newPlots);
      plots = plots.slice(-16);
      element('studio-figure-count').textContent = plots.length;
    }
    function showInlineFigures(figures) {
      global.ControLABFigures.render(element('studio-figures'), figures);
      showView('figures');
    }
    async function openFigures(figures) {
      if (!figures.length) { showToast('Execute um script para gerar uma figura.', 'info'); return; }
      // The Android WebView has no real pop-up windows: window.open replaces the app page.
      if (global.Capacitor?.isNativePlatform?.() && !global.api?.openScriptFigure) { showInlineFigures(figures); return; }
      for (const plot of figures) {
        const theme = document.documentElement.dataset.theme || 'dark';
        if (global.api?.openScriptFigure) {
          const result = await global.api.openScriptFigure({ plot, theme });
          if (!result?.success) log(result?.error || 'Não foi possível abrir a figura.', 'error');
          continue;
        }
        // Transfer data only after the local figure document announces readiness.
        const id = global.crypto?.randomUUID?.() || `${Date.now()}-${Math.random()}`;
        let popup;
        const origin = global.location.origin;
        const listener = (event) => {
          if (event.origin !== origin || event.source !== popup || event.data?.type !== 'controlab-figure-ready' || event.data.id !== id) return;
          popup.postMessage({ type: 'controlab-figure-data', id, plot, theme }, origin);
          global.removeEventListener('message', listener); clearTimeout(timer);
        };
        global.addEventListener('message', listener);
        const timer = setTimeout(() => global.removeEventListener('message', listener), 15000);
        popup = global.open(`figure.html?id=${encodeURIComponent(id)}`, `controlab-figure-${id}`, 'popup,width=900,height=640,resizable=yes,scrollbars=yes');
        if (!popup) {
          clearTimeout(timer); global.removeEventListener('message', listener);
          showInlineFigures(figures);
          showToast('Janelas pop-up bloqueadas: a figura foi aberta no editor.', 'info');
          return;
        }
      }
    }
    async function execute(text, label) {
      if (busy) { log('Aguarde a execução atual terminar.', 'error'); return; }
      if (new TextEncoder().encode(text).length > MAX_BYTES) { log('Script excede 1 MiB.', 'error'); return; }
      busy = true; element('studio-run').disabled = true; status('Executando…');
      log(`>> ${label}`, 'cmd');
      const start = performance.now();
      try {
        const result = await request('execute', { source: text });
        log(result.output);
        if (result.workspace) renderWorkspace(result.workspace);
        if (result.plots?.length) { renderPlots(result.plots); await openFigures(result.plots); }
        if (result.diagnostic) {
          const diagnostic = result.diagnostic;
          log(`Linha ${diagnostic.line}, coluna ${diagnostic.column}: ${diagnostic.message}`, 'error');
          status('Erro de execução');
          if (label !== text && text === source.value) {
            showView('editor');
            const lines = source.value.split('\n');
            const offset = lines.slice(0, diagnostic.line - 1).reduce((sum, line) => sum + line.length + 1, 0) + diagnostic.column - 1;
            source.focus(); source.setSelectionRange(offset, Math.min(source.value.length, offset + 1)); updateCursor();
          }
        } else status(`Concluído em ${((performance.now() - start) / 1000).toFixed(2)} s`);
      } catch (error) { log(error.message, 'error'); status('Execução indisponível'); }
      finally { busy = false; element('studio-run').disabled = false; }
    }
    async function executeCommand(raw) {
      const command = raw.trim();
      if (!command) return;
      if (command === 'clc') { logs.replaceChildren(); return; }
      if (command === 'clear') { await clear(); return; }
      if (command === 'help' || command === 'ajuda') { help(); return; }
      if (command === 'pwd') { log(folder); return; }
      if (command === 'dir' || command === 'ls') { log(files.map((file) => file.name).join('\n') || '(pasta vazia)'); return; }
      if (command === 'whos') {
        try { const result = await request('state'); log(result.workspace.map((v) => `${v.name}\t${v.type}\t${v.dimensions}\t${v.value}`).join('\n') || 'Workspace vazio.'); } catch (error) { log(error.message, 'error'); } return;
      }
      if (/^cd\s+/.test(command)) {
        const target = command.slice(3).trim();
        await refreshFolder(/^(?:[/\\]|[A-Za-z]:)/.test(target) ? target : `${folder}/${target}`); return;
      }
      const run = command.match(/^run\(\s*'([^']+)'\s*\)\s*;?$/);
      if (run) {
        try {
          const path = /^(?:[/\\]|[A-Za-z]:)/.test(run[1]) ? run[1] : `${folder}/${run[1]}`;
          const result = await request('read', { path }); await execute(result.source, result.name);
        } catch (error) { log(error.message, 'error'); } return;
      }
      await execute(command, command);
    }
    async function clear() {
      if (busy) return;
      try { const result = await request('clear'); renderWorkspace(result.workspace); log(result.output); status('Workspace limpo'); }
      catch (error) { log(error.message, 'error'); }
    }
    function help() {
      log('Scripts .m · openMAT\nF5: executar · Ctrl+S: salvar · Ctrl+O: abrir · Ctrl+N: novo script\nExemplo: A=[1 2;3 4]; disp(A*A)\nGráficos: x=0:0.1:6; plot(x,sin(x))\nControle: G=tf([1],[1 2 1]); step(G)\nConsole: clear, clc, whos, pwd, ls, cd <pasta>, run(\'arquivo.m\')\nO interpretador suporta um subconjunto de MATLAB; funções de usuário e controle de fluxo ainda não estão disponíveis.');
    }
    source.addEventListener('input', () => { syncDocument(); renderTabs(); clearTimeout(highlightTimer); highlightTimer = setTimeout(updateHighlight, 60); });
    source.addEventListener('scroll', syncScroll);
    source.addEventListener('click', updateCursor); source.addEventListener('keyup', updateCursor);
    source.addEventListener('keydown', (event) => {
      if (event.key === 'Tab') {
        event.preventDefault(); const start = source.selectionStart, end = source.selectionEnd;
        source.setRangeText('    ', start, end, 'end'); source.dispatchEvent(new Event('input'));
      }
    });
    element('studio-new').addEventListener('click', () => newDocument());
    element('studio-open').addEventListener('click', openFile);
    element('studio-save').addEventListener('click', save);
    element('studio-run').addEventListener('click', runCurrent);
    element('studio-welcome-examples').addEventListener('click', (event) => {
      const item = event.target.closest('[data-example]');
      if (item) openExample(item.dataset.example);
    });
    element('studio-clear').addEventListener('click', clear);
    element('studio-clear-output').addEventListener('click', () => logs.replaceChildren());
    element('studio-help').addEventListener('click', help);
    element('studio-back').addEventListener('click', () => navigateHistory(-1));
    element('studio-forward').addEventListener('click', () => navigateHistory(1));
    element('studio-select-folder').addEventListener('click', selectFolder);
    element('studio-picker-form').addEventListener('submit', (event) => { event.preventDefault(); browsePicker(element('studio-picker-path').value); });
    element('studio-picker-up').addEventListener('click', () => pickerParent && browsePicker(pickerParent));
    element('studio-picker-cancel').addEventListener('click', () => element('studio-folder-dialog').close());
    element('studio-picker-confirm').addEventListener('click', async () => {
      if (!pickerFolder || element('studio-picker-confirm').disabled) return;
      const selected = pickerFolder; element('studio-folder-dialog').close(); await refreshFolder(selected);
    });
    element('studio-refresh').addEventListener('click', () => refreshFolder(folder));
    element('studio-up').addEventListener('click', () => parentFolder && refreshFolder(parentFolder));
    element('studio-file-search').addEventListener('input', renderFiles);
    element('studio-folder-form').addEventListener('submit', (event) => { event.preventDefault(); refreshFolder(element('studio-folder-input').value); });
    element('studio-files').addEventListener('click', () => { home.classList.remove('studio-show-vars'); const shown = home.classList.toggle('studio-show-files'); element('studio-files').setAttribute('aria-pressed', String(shown)); });
    element('studio-variables').addEventListener('click', () => {
      home.classList.remove('studio-focus', 'studio-show-files');
      const shown = global.innerWidth < 768 ? home.classList.toggle('studio-show-vars') : !home.classList.toggle('studio-hide-vars');
      element('studio-variables').setAttribute('aria-pressed', String(shown));
    });
    element('studio-layout').addEventListener('click', () => { home.classList.remove('studio-show-vars', 'studio-show-files', 'studio-hide-vars'); const focus = home.classList.toggle('studio-focus'); element('studio-layout').setAttribute('aria-pressed', String(focus)); });
    element('studio-editor-view').addEventListener('click', () => showView('editor'));
    element('studio-figures-view').addEventListener('click', () => openFigures(plots));
    element('studio-file-input').addEventListener('change', async (event) => {
      const file = event.target.files[0];
      try {
        if (!file) return;
        if (!/\.m$/i.test(file.name)) throw new Error('Selecione um arquivo .m.');
        if (file.size > MAX_BYTES) throw new Error('Script excede 1 MiB.');
        const text = await file.text(); newDocument(text, file.name, null, text);
      } catch (error) { log(error.message, 'error'); }
      finally { event.target.value = ''; }
    });
    document.addEventListener('keydown', (event) => {
      if (!home.classList.contains('active') || document.querySelector('dialog[open], .modal-overlay.active')) return;
      if (event.altKey && ['ArrowLeft', 'ArrowRight', 'ArrowUp'].includes(event.key)) {
        event.preventDefault();
        if (event.key === 'ArrowUp') { if (parentFolder) refreshFolder(parentFolder); }
        else navigateHistory(event.key === 'ArrowLeft' ? -1 : 1);
        return;
      }
      if (event.key === 'F5') { event.preventDefault(); runCurrent(); }
      if ((event.ctrlKey || event.metaKey) && ['s', 'o', 'n'].includes(event.key.toLowerCase())) {
        event.preventDefault(); ({ s: save, o: openFile, n: newDocument })[event.key.toLowerCase()]();
      }
    });
    global.addEventListener('pagehide', () => { global.api?.releaseScriptSession?.(session); });
    global.addEventListener('beforeunload', (event) => {
      syncDocument(); if (documents.some(dirty)) { event.preventDefault(); event.returnValue = ''; }
    });
    renderTabs();
    log('Pronto. Crie ou abra um script, ou digite um comando no console.');
    request('state').then((result) => { available = true; renderWorkspace(result.workspace); }).catch((error) => { log(error.message, 'error'); status('Motor .m indisponível'); });
    controller = { newDocument, openFile, openPath, openExample, save, clear, executeCommand, refreshFolder, selectFolder, renderWorkspace, focus: () => { if (!documents.length) newDocument(); showView('editor'); source.focus(); } };
    return controller;
  }
  global.ControLABStudio = { initialize, get controller() { return controller; } };
})(window);
