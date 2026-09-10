const { app, BrowserWindow, Menu, ipcMain, dialog, clipboard, nativeImage, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

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
  mainWindow = new BrowserWindow({
    width: 1360,
    height: 880,
    minWidth: 360,
    minHeight: 640,
    autoHideMenuBar: true,
    title: 'ControLAB - Sistemas de Controle',
    backgroundColor: '#0f172a',
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
  const appVersion = app.getVersion() || '2.0.0';

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

app.whenReady().then(() => {
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
