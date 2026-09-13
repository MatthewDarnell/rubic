import React, {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Snackbar, Alert } from '@mui/material';

const ToastContext = createContext(null);

export function ToastProvider({ children }) {
  const [queue, setQueue] = useState([]);
  const [current, setCurrent] = useState(null);
  const [open, setOpen] = useState(false);
  const nextId = useRef(0);

  const push = useCallback((severity, message) => {
    setQueue((q) => [...q, { id: nextId.current++, severity, message }]);
  }, []);

  React.useEffect(() => {
    if (!current && queue.length > 0) {
      setCurrent(queue[0]);
      setQueue((q) => q.slice(1));
      setOpen(true);
    }
  }, [queue, current]);

  const handleClose = (_event, reason) => {
    if (reason === 'clickaway') return;
    setOpen(false);
  };

  const handleExited = () => setCurrent(null);

  const api = useMemo(
    () => ({
      success: (m) => push('success', m),
      error: (m) => push('error', m),
      info: (m) => push('info', m),
      warning: (m) => push('warning', m),
    }),
    [push]
  );

  const viewport = (
    <Snackbar
      key={current?.id}
      open={open}
      autoHideDuration={current?.severity === 'error' ? 8000 : 4000}
      onClose={handleClose}
      slotProps={{ transition: { onExited: handleExited } }}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
    >
      {current ? (
        <Alert
          onClose={handleClose}
          severity={current.severity}
          variant='filled'
          sx={{ width: '100%', maxWidth: 560 }}
        >
          {current.message}
        </Alert>
      ) : undefined}
    </Snackbar>
  );

  return (
    <ToastContext.Provider value={{ ...api, viewport }}>
      {children}
    </ToastContext.Provider>
  );
}

export const useToast = () => {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast must be used inside <ToastProvider>');
  return ctx;
};

// Render this once, inside the ThemeProvider, so toasts pick up the app theme.
export const ToastViewport = () => useToast().viewport;
