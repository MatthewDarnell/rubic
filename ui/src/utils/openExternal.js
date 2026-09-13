// Inside the Tauri window, links must go through the shell plugin to reach the
// system browser; in a plain browser (dev server) window.open is fine.
export const openExternal = async (url) => {
  const shellOpen = window.__TAURI__?.shell?.open;
  if (typeof shellOpen === 'function') {
    try {
      await shellOpen(url);
      return;
    } catch (err) {
      console.error('shell.open failed, falling back to window.open', err);
    }
  }
  window.open(url, '_blank', 'noopener');
};

export const isTauri = () => Boolean(window.__TAURI__);
