const { app, BrowserWindow, Menu, ipcMain, dialog, clipboard, nativeImage, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn, exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);

Menu.setApplicationMenu(null);

let mainWindow = null;

if (process.platform === 'linux') {
  app.commandLine.appendSwitch('no-sandbox');
  app.commandLine.appendSwitch('disable-gpu-sandbox');
}

function getPythonPath() {
  if (process.env.LGR_PYTHON) {
    return process.env.LGR_PYTHON;
  }

  const projectRoot = path.join(__dirname, '..', '..');
  const candidates = process.platform === 'win32'
    ? [path.join(projectRoot, '.venv', 'Scripts', 'python.exe')]
    : [path.join(projectRoot, '.venv', 'bin', 'python')];

  return candidates.find((candidate) => fs.existsSync(candidate))
    || (process.platform === 'win32' ? 'python' : 'python3');
}

function getPackagedBridgePath() {
  const executable = process.platform === 'win32' ? 'lgr-bridge.exe' : 'lgr-bridge';
  return path.join(process.resourcesPath, 'python', executable);
}

function runPythonBridge(payload) {
  return new Promise((resolve, reject) => {
    const command = app.isPackaged ? getPackagedBridgePath() : getPythonPath();
    const args = app.isPackaged
      ? []
      : [path.join(__dirname, '..', '..', 'python_bridge.py')];

    const pyProcess = spawn(command, args);

    let stdoutData = '';
    let stderrData = '';

    pyProcess.stdout.on('data', (data) => {
      stdoutData += data.toString();
    });

    pyProcess.stderr.on('data', (data) => {
      stderrData += data.toString();
    });

    pyProcess.on('close', (code) => {
      if (code !== 0 && !stdoutData) {
        return reject(new Error(stderrData || `Python encerrou com código de erro ${code}`));
      }
      try {
        const json = JSON.parse(stdoutData.trim());
        resolve(json);
      } catch (err) {
        reject(new Error(`Erro ao interpretar resposta do Python: ${stdoutData || stderrData}`));
      }
    });

    pyProcess.on('error', (err) => {
      reject(new Error(`Falha ao iniciar processo Python: ${err.message}`));
    });

    pyProcess.stdin.write(JSON.stringify(payload));
    pyProcess.stdin.end();
  });
}

function createWindow() {
  const iconPath = path.join(__dirname, '..', 'renderer', 'app-icon.png');

  mainWindow = new BrowserWindow({
    width: 1360,
    height: 880,
    minWidth: 360,
    minHeight: 640,
    autoHideMenuBar: true,
    title: 'ControLAB - Sistemas de Controle',
    backgroundColor: '#0f172a',
    icon: iconPath,
    webPreferences: {
      preload: path.join(__dirname, '..', 'preload', 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  mainWindow.setMenuBarVisibility(false);
  mainWindow.loadFile(path.join(__dirname, '..', 'renderer', 'index.html'));

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('https://') || url.startsWith('http://')) {
      shell.openExternal(url);
      return { action: 'deny' };
    }
    return { action: 'allow' };
  });

  mainWindow.webContents.on('will-navigate', (event, url) => {
    if (url !== mainWindow.webContents.getURL() && (url.startsWith('http://') || url.startsWith('https://'))) {
      event.preventDefault();
      shell.openExternal(url);
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// ==========================================
// IPC HANDLERS
// ==========================================
ipcMain.handle('calculate-lgr', async (event, payload) => {
  try {
    const data = await runPythonBridge({ action: 'calculate', ...payload });
    return data;
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('preview-transfer-function', async (event, payload) => {
  try {
    return await runPythonBridge({ action: 'preview', ...payload });
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('get-presets', async () => {
  try {
    const data = await runPythonBridge({ action: 'presets' });
    return data;
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('calculate-routh', async (event, payload) => {
  try {
    return await runPythonBridge({ action: 'routh_hurwitz', ...payload });
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('get-routh-presets', async () => {
  try {
    return await runPythonBridge({ action: 'routh_presets' });
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('evaluate-routh-k', async (event, payload) => {
  try {
    return await runPythonBridge({ action: 'routh_evaluate_k', ...payload });
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('run-scientific-analysis', async (event, action, payload) => {
  try {
    return await runPythonBridge({ action, ...(payload || {}) });
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('save-image', async (event, { base64, defaultName }) => {
  const { filePath } = await dialog.showSaveDialog(mainWindow, {
    title: 'Salvar Gráfico do LGR',
    defaultPath: defaultName || 'lgr_grafico.png',
    filters: [{ name: 'Imagens PNG', extensions: ['png'] }],
  });

  if (filePath) {
    const cleanBase64 = base64.replace(/^data:image\/png;base64,/, '');
    await fs.promises.writeFile(filePath, Buffer.from(cleanBase64, 'base64'));
    return { success: true, filePath };
  }
  return { success: false, canceled: true };
});

ipcMain.handle('save-svg', async (event, { svg, defaultName }) => {
  const { filePath } = await dialog.showSaveDialog(mainWindow, {
    title: 'Salvar Gráfico Vetorial SVG',
    defaultPath: defaultName || 'lgr_grafico.svg',
    filters: [{ name: 'Vetorial SVG', extensions: ['svg'] }],
  });

  if (filePath) {
    await fs.promises.writeFile(filePath, svg, 'utf-8');
    return { success: true, filePath };
  }
  return { success: false, canceled: true };
});

ipcMain.handle('copy-image', async (event, base64) => {
  try {
    const img = nativeImage.createFromDataURL(base64);
    clipboard.writeImage(img);
    return { success: true };
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('open-external', async (event, url) => {
  if (url && (url.startsWith('https://') || url.startsWith('http://'))) {
    await shell.openExternal(url);
    return { success: true };
  }
  return { success: false };
});

function getSystemInfo() {
  const platform = process.platform;
  const arch = process.arch;
  const appVersion = app.getVersion() || '3.0.2';

  if (platform === 'win32') {
    return {
      os: 'windows',
      osName: 'Windows',
      packageType: 'exe',
      packageLabel: 'Instalador Windows (.exe)',
      arch,
      appVersion,
    };
  }

  if (platform === 'darwin') {
    return {
      os: 'mac',
      osName: 'macOS',
      packageType: 'dmg',
      packageLabel: 'macOS (.dmg)',
      arch,
      appVersion,
    };
  }

  if (platform === 'linux') {
    if (process.env.APPIMAGE || (process.execPath && process.execPath.includes('.mount_'))) {
      return {
        os: 'linux',
        osName: 'Linux Universal',
        packageType: 'appimage',
        packageLabel: 'Linux Universal (.AppImage)',
        arch,
        appVersion,
      };
    }

    let distroName = 'Linux';
    let isDebianLike = false;
    let isRpmLike = false;

    try {
      if (fs.existsSync('/etc/os-release')) {
        const osRelease = fs.readFileSync('/etc/os-release', 'utf-8');
        const lines = osRelease.split('\n');
        for (const line of lines) {
          if (line.startsWith('PRETTY_NAME=')) {
            distroName = line.replace(/^PRETTY_NAME=["']?/, '').replace(/["']?$/, '');
          }
          if (line.startsWith('ID=') || line.startsWith('ID_LIKE=')) {
            const val = line.toLowerCase();
            if (val.includes('debian') || val.includes('ubuntu') || val.includes('mint') || val.includes('pop')) {
              isDebianLike = true;
            }
            if (val.includes('fedora') || val.includes('rhel') || val.includes('centos') || val.includes('suse') || val.includes('redhat')) {
              isRpmLike = true;
            }
          }
        }
      }
    } catch {}

    if (isDebianLike || fs.existsSync('/usr/bin/dpkg')) {
      return {
        os: 'linux',
        osName: distroName || 'Debian / Ubuntu',
        packageType: 'deb',
        packageLabel: 'Debian / Ubuntu (.deb)',
        arch,
        appVersion,
      };
    }

    if (isRpmLike || fs.existsSync('/usr/bin/rpm')) {
      return {
        os: 'linux',
        osName: distroName || 'Fedora / RedHat',
        packageType: 'rpm',
        packageLabel: 'Fedora / RedHat (.rpm)',
        arch,
        appVersion,
      };
    }

    return {
      os: 'linux',
      osName: distroName || 'Linux Universal',
      packageType: 'appimage',
      packageLabel: 'Linux Universal (.AppImage)',
      arch,
      appVersion,
    };
  }

  return {
    os: 'unknown',
    osName: 'Desconhecido',
    packageType: 'web',
    packageLabel: 'Web',
    arch,
    appVersion,
  };
}

ipcMain.handle('get-system-info', async () => {
  return getSystemInfo();
});

ipcMain.handle('check-github-updates', async (event, token) => {
  try {
    const headers = {
      'User-Agent': 'ControLAB-Desktop',
      'Accept': 'application/vnd.github.v3+json',
    };
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }
    const response = await fetch('https://api.github.com/repos/enthonyaraujo/controlab/releases/latest', { headers });
    if (!response.ok) {
      return { success: false, status: response.status, error: `GitHub retornou status HTTP ${response.status}` };
    }
    const release = await response.json();
    return { success: true, release, systemInfo: getSystemInfo() };
  } catch (err) {
    return { success: false, error: err.message };
  }
});

let currentDownloadAbortController = null;

ipcMain.handle('download-update-package', async (event, { url, filename, expectedSize, token }) => {
  try {
    if (currentDownloadAbortController) {
      currentDownloadAbortController.abort();
      currentDownloadAbortController = null;
    }

    const updatesDir = path.join(app.getPath('temp'), 'controlab-updates');
    await fs.promises.mkdir(updatesDir, { recursive: true });
    const targetFilePath = path.join(updatesDir, filename);

    if (fs.existsSync(targetFilePath)) {
      try {
        await fs.promises.unlink(targetFilePath);
      } catch (_) {}
    }

    currentDownloadAbortController = new AbortController();
    const headers = {
      'User-Agent': 'ControLAB-Desktop',
      'Accept': 'application/octet-stream',
    };
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    const response = await fetch(url, {
      headers,
      redirect: 'follow',
      signal: currentDownloadAbortController.signal,
    });

    if (!response.ok) {
      throw new Error(`Falha no download da atualizacao (HTTP ${response.status})`);
    }

    const totalBytes = parseInt(response.headers.get('content-length') || expectedSize || '0', 10);
    const fileStream = fs.createWriteStream(targetFilePath);

    let transferred = 0;
    let lastProgressTime = Date.now();
    let lastBytes = 0;
    let bytesPerSecond = 0;

    for await (const chunk of response.body) {
      fileStream.write(chunk);
      transferred += chunk.length;
      const now = Date.now();
      if (now - lastProgressTime >= 150) {
        bytesPerSecond = Math.round(((transferred - lastBytes) / (now - lastProgressTime)) * 1000);
        lastProgressTime = now;
        lastBytes = transferred;
        const percent = totalBytes > 0 ? Math.min(100, Math.round((transferred / totalBytes) * 100)) : null;

        mainWindow?.webContents.send('on-update-download-progress', {
          percent,
          transferred,
          total: totalBytes,
          bytesPerSecond,
        });
      }
    }

    await new Promise((resolve, reject) => {
      fileStream.end((err) => (err ? reject(err) : resolve()));
    });

    currentDownloadAbortController = null;

    mainWindow?.webContents.send('on-update-download-progress', {
      percent: 100,
      transferred,
      total: totalBytes || transferred,
      bytesPerSecond: 0,
      completed: true,
      filePath: targetFilePath,
    });

    return { success: true, filePath: targetFilePath };
  } catch (err) {
    currentDownloadAbortController = null;
    if (err.name === 'AbortError') {
      return { success: false, canceled: true };
    }
    return { success: false, error: err.message };
  }
});

ipcMain.handle('cancel-update-download', async () => {
  if (currentDownloadAbortController) {
    currentDownloadAbortController.abort();
    currentDownloadAbortController = null;
    return { success: true };
  }
  return { success: false };
});

ipcMain.handle('open-update-folder', async (event, filePath) => {
  if (filePath && fs.existsSync(filePath)) {
    shell.showItemInFolder(filePath);
    return { success: true };
  }
  return { success: false, error: 'Arquivo nao encontrado.' };
});

ipcMain.handle('install-update-package', async (event, { filePath }) => {
  try {
    if (!filePath || !fs.existsSync(filePath)) {
      return { success: false, error: 'Arquivo do instalador nao encontrado.' };
    }

    const ext = path.extname(filePath).toLowerCase();
    const platform = process.platform;

    if (platform === 'win32' || ext === '.exe') {
      try {
        const child = spawn(filePath, [], { detached: true, stdio: 'ignore', shell: true });
        child.unref();
        setTimeout(() => {
          app.quit();
        }, 800);
        return { success: true, method: 'spawn-exe' };
      } catch (spawnErr) {
        await shell.openPath(filePath);
        setTimeout(() => {
          app.quit();
        }, 800);
        return { success: true, method: 'open-path' };
      }
    }

    if (platform === 'linux') {
      if (ext === '.deb') {
        try {
          const cmd = `pkexec apt-get install -y --allow-downgrades "${filePath}"`;
          await execPromise(cmd);
          spawn('controlab', [], { detached: true, stdio: 'ignore' }).unref();
          setTimeout(() => {
            app.quit();
          }, 600);
          return { success: true, method: 'pkexec-apt' };
        } catch (pkErr) {
          const openResult = await shell.openPath(filePath);
          if (!openResult) {
            return { success: true, method: 'open-path', fallbackNote: 'Instalador grafico do sistema iniciado.' };
          }
          throw pkErr;
        }
      } else if (ext === '.rpm') {
        try {
          const cmd = `pkexec rpm -Uvh --replacepkgs "${filePath}"`;
          await execPromise(cmd);
          spawn('controlab', [], { detached: true, stdio: 'ignore' }).unref();
          setTimeout(() => {
            app.quit();
          }, 600);
          return { success: true, method: 'pkexec-rpm' };
        } catch (pkErr) {
          await shell.openPath(filePath);
          return { success: true, method: 'open-path' };
        }
      } else if (ext === '.appimage') {
        fs.chmodSync(filePath, 0o755);
        const child = spawn(filePath, [], { detached: true, stdio: 'ignore' });
        child.unref();
        setTimeout(() => {
          app.quit();
        }, 600);
        return { success: true, method: 'spawn-appimage' };
      }
    }

    const openErr = await shell.openPath(filePath);
    if (openErr) {
      shell.showItemInFolder(filePath);
    }
    return { success: true, method: 'open-path' };
  } catch (err) {
    return { success: false, error: err.message };
  }
});


app.whenReady().then(() => {
  if (process.platform === 'win32') {
    app.setAppUserModelId('br.com.enthony.lgrstudio');
  }

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
