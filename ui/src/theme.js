import { createTheme } from '@mui/material';

export const FONT_SANS =
  'Inter, "Segoe UI Variable", "Segoe UI", -apple-system, BlinkMacSystemFont, Roboto, "Helvetica Neue", Arial, sans-serif';
export const FONT_MONO =
  '"JetBrains Mono", "Cascadia Code", "SF Mono", Menlo, Consolas, "Liberation Mono", monospace';

const shared = {
  shape: { borderRadius: 10 },
  typography: {
    fontFamily: FONT_SANS,
    fontSize: 13.5,
    h5: { fontWeight: 700, letterSpacing: '-0.01em' },
    h6: { fontSize: '1.05rem', fontWeight: 600, letterSpacing: '-0.01em' },
    subtitle1: { fontWeight: 600 },
    subtitle2: { fontWeight: 600 },
    body1: { fontSize: '0.9rem' },
    body2: { fontSize: '0.82rem' },
    caption: { fontSize: '0.74rem' },
    button: { textTransform: 'none', fontWeight: 600, fontSize: '0.85rem' },
  },
};

const components = (mode) => ({
  MuiCssBaseline: {
    styleOverrides: {
      body: { fontVariantNumeric: 'tabular-nums' },
      'code, pre, .mono': { fontFamily: FONT_MONO },
      '*::-webkit-scrollbar': { width: 10, height: 10 },
      '*::-webkit-scrollbar-thumb': {
        backgroundColor: mode === 'dark' ? 'rgba(255,255,255,0.14)' : 'rgba(0,0,0,0.18)',
        borderRadius: 8,
        border: '2px solid transparent',
        backgroundClip: 'content-box',
      },
    },
  },
  MuiPaper: {
    defaultProps: { elevation: 0 },
    styleOverrides: { root: { backgroundImage: 'none' } },
  },
  MuiTableCell: {
    styleOverrides: {
      root: { fontSize: '0.84rem', fontVariantNumeric: 'tabular-nums', borderColor: 'var(--rubic-border)' },
      head: {
        fontSize: '0.72rem',
        fontWeight: 600,
        textTransform: 'uppercase',
        letterSpacing: '0.04em',
        color: mode === 'dark' ? '#8a93a6' : '#5f6675',
        backgroundColor: mode === 'dark' ? '#161c2b' : '#f7f8fb',
      },
    },
  },
  MuiButton: {
    defaultProps: { disableElevation: true },
    styleOverrides: { root: { borderRadius: 8 } },
  },
  MuiChip: {
    styleOverrides: { root: { fontWeight: 500 } },
  },
  MuiTooltip: {
    defaultProps: { arrow: true },
  },
  MuiTextField: {
    defaultProps: { size: 'small' },
  },
  MuiDialog: {
    styleOverrides: { paper: { backgroundImage: 'none' } },
  },
});

export const getTheme = (mode) =>
  createTheme({
    ...shared,
    palette:
      mode === 'dark'
        ? {
            mode,
            primary: { main: '#5b8cff' },
            secondary: { main: '#9d7bff' },
            success: { main: '#3ecf8e' },
            error: { main: '#ff5c6c' },
            warning: { main: '#f5b33b' },
            background: { default: '#0f1420', paper: '#161c2b' },
            divider: 'rgba(255,255,255,0.08)',
            text: { primary: '#e8ecf4', secondary: '#8a93a6' },
          }
        : {
            mode,
            primary: { main: '#2f6bff' },
            secondary: { main: '#7a4dff' },
            success: { main: '#1f9d6b' },
            error: { main: '#d9384a' },
            warning: { main: '#c98a12' },
            background: { default: '#f3f5f9', paper: '#ffffff' },
            divider: 'rgba(15,20,32,0.1)',
            text: { primary: '#141a26', secondary: '#5f6675' },
          },
    components: components(mode),
  });
