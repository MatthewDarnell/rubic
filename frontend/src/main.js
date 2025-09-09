const { app, BrowserWindow, ipcMain, WebContentsView } = require('electron');
const path = require('path');

function createWindow() {
  const isDev = !app.isPackaged;
  const win = new BrowserWindow({
    width: 1400,
    height: 1000,
    title: 'RUBIC UI',
    autoHideMenuBar: true,
    resizable: false, // Prevent resizing
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      enableRemoteModule: false,
      webSecurity: true,
      //   sandbox: false,
    },
  });

  const entitiesView = new WebContentsView();

  if (isDev) {
    // Load Vite dev server in development
    win.loadURL('http://localhost:5173');
    // win.webContents.openDevTools();
  } else {
    // Production mode: Load the built index.html
    const indexPath = path.join(__dirname, '../dist/index.html');
    win.loadFile(indexPath);
  }
}

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  app.quit();
  // if (process.platform !== "darwin") {
  //   app.quit();
  // }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
