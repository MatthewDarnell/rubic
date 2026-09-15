import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Checkbox,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControlLabel,
  IconButton,
  InputAdornment,
  LinearProgress,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';
import { apiCall, apiPost } from '../api';
import IdText from './IdText';
import Identicon from './Identicon';

const seedRegex = /^[a-z]{55}$/;
// What the server derives for a seed that is not a valid Qubic seed.
const INVALID_SEED_ID = 'AARQXIKNFIEZZEMOAVNVSUINZXAAXYBZZXVSWYOYIETZVPVKJPARMKTEKLKJ';

// One seed per line; blank lines ignored, case and surrounding spaces forgiven.
const parseSeeds = (text) => {
  const seeds = [];
  const seen = new Set();
  let invalid = 0;
  let duplicates = 0;
  text.split(/\r?\n/).forEach((line) => {
    const seed = line.trim().toLowerCase();
    if (!seed) return;
    if (!seedRegex.test(seed)) {
      invalid += 1;
    } else if (seen.has(seed)) {
      duplicates += 1;
    } else {
      seen.add(seed);
      seeds.push(seed);
    }
  });
  return { seeds, invalid, duplicates };
};

/**
 * "Import Bulk Identities": a textarea of seeds, one per line, each added
 * through `/identity/add`.
 *
 * How the seeds are stored follows the wallet's state:
 *  - unlocked                      -> the server encrypts each with the unlocked password
 *  - locked, unencrypted allowed   -> stored in plain text, no password asked
 *                                     (unless "Encrypt instead" is ticked)
 *  - locked otherwise              -> the master password is asked for here; the
 *                                     wallet is unlocked with it once, and it is
 *                                     also sent with every row so a row is still
 *                                     encrypted if the unlock timer runs out
 *  - no master password yet        -> stored as-is (nothing to encrypt with)
 */
export default function BulkImportDialog({ open, onClose, isEncrypted, allowNonEncrypted, unlockTimerMs, existing = [], onUnlocked, onImported }) {
  const [text, setText] = useState('');
  const [showSeeds, setShowSeeds] = useState(false);
  const [unlocked, setUnlocked] = useState(false);
  const [checking, setChecking] = useState(false);
  const [encryptAnyway, setEncryptAnyway] = useState(false);
  const [password, setPassword] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [importing, setImporting] = useState(false);
  const [progress, setProgress] = useState(null); // { done, total }
  const [error, setError] = useState('');
  // seed -> { id } | { invalid: true }, as derived by the server; kept across
  // edits. The ref mirrors the state so the derivation loop can skip seeds
  // that are done without restarting every time one finishes.
  const [derived, setDerived] = useState({});
  const derivedRef = useRef({});
  const remember = (seed, value) => {
    derivedRef.current = { ...derivedRef.current, [seed]: value };
    setDerived(derivedRef.current);
  };

  useEffect(() => {
    if (!open) return undefined;
    let cancelled = false;
    setText('');
    derivedRef.current = {};
    setDerived({});
    setShowSeeds(false);
    setEncryptAnyway(false);
    setPassword('');
    setPasswordError('');
    setError('');
    setProgress(null);
    setUnlocked(false);
    if (!isEncrypted) return undefined;
    setChecking(true);
    apiCall('wallet/unlocked').then((res) => {
      if (cancelled) return;
      setUnlocked(res.success && res.data === true);
      setChecking(false);
    });
    return () => {
      cancelled = true;
    };
  }, [open, isEncrypted]);

  const parsed = useMemo(() => parseSeeds(text), [text]);

  // Derive the identity of every new seed (a short pause after typing stops),
  // one request at a time so a long paste does not flood the server.
  useEffect(() => {
    if (!parsed.seeds.some((seed) => !derivedRef.current[seed])) return undefined;
    let cancelled = false;
    const timer = setTimeout(async () => {
      for (const seed of parsed.seeds) {
        if (cancelled) return;
        if (derivedRef.current[seed]) continue;
        const res = await apiPost('identity/from_seed', { seed });
        if (cancelled) return;
        const ok = res.success && res.data && res.data !== INVALID_SEED_ID;
        remember(seed, ok ? { id: String(res.data) } : { invalid: true });
      }
    }, 300);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [parsed.seeds]);

  const existingSet = useMemo(() => new Set(existing), [existing]);
  const rows = useMemo(
    () => parsed.seeds.map((seed) => {
      const d = derived[seed];
      return { seed, id: d?.id, invalid: Boolean(d?.invalid), pending: !d, present: Boolean(d?.id && existingSet.has(d.id)) };
    }),
    [parsed.seeds, derived, existingSet]
  );
  const deriving = rows.some((r) => r.pending);
  const ready = rows.filter((r) => r.id && !r.present);
  const invalidSeeds = rows.filter((r) => r.invalid).length;
  const alreadyPresent = rows.filter((r) => r.present).length;

  // What will happen to the seeds, given the wallet's state right now.
  const mode = !isEncrypted ? 'plain-no-password' : unlocked ? 'unlocked' : allowNonEncrypted && !encryptAnyway ? 'plain' : 'password';
  const needsPassword = mode === 'password';
  const canImport = ready.length > 0 && !deriving && !importing && !checking && (!needsPassword || password.length > 0);

  const runImport = async () => {
    if (!canImport) return;
    setImporting(true);
    setError('');
    setPasswordError('');
    const total = ready.length;
    setProgress({ done: 0, total });
    try {
      let rowPassword = '';
      if (needsPassword) {
        // One unlock up front: the server then skips the password check per
        // row. The password still travels with every row (see above).
        const res = await apiPost('wallet/unlock', { password, timeout_ms: Number(unlockTimerMs) || 60000 });
        const reply = String(res.data ?? '');
        if (!res.success) {
          setError(`Unlock failed: ${res.error || 'no response from server'}`);
          return;
        }
        if (!/already unlocked|wallet unlocked/i.test(reply)) {
          setPasswordError(/invalid|incorrect/i.test(reply) ? 'Invalid password — try again' : reply);
          return;
        }
        onUnlocked?.();
        rowPassword = password;
      }
      let added = 0;
      let skipped = 0;
      for (const [index, { seed, id }] of ready.entries()) {
        const res = await apiPost('identity/add', { seed, password: rowPassword });
        const reply = String(res.data ?? '').trim();
        if (res.success && reply === '200') {
          added += 1;
        } else if (/failed to insert/i.test(reply)) {
          skipped += 1; // already in the wallet
        } else {
          setError(`Stopped at ${id.slice(0, 8)}… (${index + 1} of ${total}): ${reply || res.error || 'no response from server'}. ${added} added so far, and kept.`);
          return;
        }
        setProgress({ done: index + 1, total });
      }
      onImported?.({ count: added, skipped, plain: mode === 'plain' || mode === 'plain-no-password' });
      onClose();
    } finally {
      setImporting(false);
    }
  };

  const busy = importing || checking;

  return (
    <Dialog open={open} onClose={busy ? undefined : onClose} maxWidth='sm' fullWidth>
      <DialogTitle>Import Bulk Identities</DialogTitle>
      <DialogContent>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
          Paste one 55-letter seed per line. Each becomes an identity in this wallet.
        </Typography>
        <TextField
          autoFocus
          fullWidth
          multiline
          minRows={6}
          maxRows={12}
          label='Seeds, one per line'
          value={text}
          onChange={(e) => setText(e.target.value)}
          disabled={importing}
          autoComplete='off'
          spellCheck={false}
          slotProps={{
            input: {
              className: 'mono',
              // Hidden like a password field unless revealed; a textarea has no
              // password type, so the glyphs are masked instead.
              sx: showSeeds ? undefined : { '& textarea': { WebkitTextSecurity: 'disc' } },
              endAdornment: (
                <InputAdornment position='end' sx={{ alignSelf: 'flex-start', mt: 1 }}>
                  <Tooltip title={showSeeds ? 'Hide seeds' : 'Show seeds'}>
                    <IconButton onClick={() => setShowSeeds((v) => !v)} edge='end' size='small'>
                      {showSeeds ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </Tooltip>
                </InputAdornment>
              ),
            },
          }}
        />
        <Typography variant='caption' color='text.secondary' sx={{ display: 'block', mt: 0.5, minHeight: 20 }}>
          {text.trim()
            ? [
                deriving ? `deriving ${rows.filter((r) => r.pending).length}…` : `${ready.length} identit${ready.length === 1 ? 'y' : 'ies'} ready`,
                invalidSeeds > 0 && `${invalidSeeds} seed${invalidSeeds === 1 ? '' : 's'} invalid`,
                alreadyPresent > 0 && `${alreadyPresent} already in the wallet`,
                parsed.invalid > 0 && `${parsed.invalid} line${parsed.invalid === 1 ? '' : 's'} skipped (not 55 lowercase letters a–z)`,
                parsed.duplicates > 0 && `${parsed.duplicates} duplicate${parsed.duplicates === 1 ? '' : 's'} ignored`,
              ]
                .filter(Boolean)
                .join(' · ')
            : ' '}
        </Typography>
        {rows.length > 0 && (
          <Box sx={{ maxHeight: 180, overflow: 'auto', mt: 0.5, pr: 1 }}>
            {rows.map((row, index) => (
              <Box key={row.seed} sx={{ display: 'flex', alignItems: 'center', gap: 1, py: 0.5, opacity: row.present ? 0.6 : 1 }}>
                <Typography variant='caption' color='text.secondary' sx={{ width: 24, textAlign: 'right', flexShrink: 0 }}>
                  {index + 1}
                </Typography>
                {row.pending && <CircularProgress size={14} />}
                {row.pending && <Typography variant='caption' color='text.secondary'>Deriving identity…</Typography>}
                {row.invalid && (
                  <Typography variant='caption' color='error.main'>This seed does not produce a valid identity</Typography>
                )}
                {row.id && (
                  <>
                    <Identicon id={row.id} size={20} />
                    <IdText id={row.id} full nowrap copy={false} sx={{ fontSize: '0.74rem' }} />
                    {row.present && (
                      <Typography variant='caption' color='text.secondary' sx={{ whiteSpace: 'nowrap' }}>already in the wallet</Typography>
                    )}
                  </>
                )}
              </Box>
            ))}
          </Box>
        )}

        {checking && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mt: 1.5 }}>
            <CircularProgress size={16} />
            <Typography variant='body2' color='text.secondary'>Checking whether the wallet is unlocked…</Typography>
          </Box>
        )}
        {!checking && mode === 'unlocked' && (
          <Alert severity='success' sx={{ mt: 1.5 }}>
            The wallet is unlocked: each seed is encrypted with your master password as it is stored.
          </Alert>
        )}
        {!checking && mode === 'plain-no-password' && (
          <Alert severity='warning' sx={{ mt: 1.5 }}>
            No master password is set yet, so the seeds are stored without encryption.
          </Alert>
        )}
        {!checking && (mode === 'plain' || (mode === 'password' && allowNonEncrypted)) && (
          <Alert severity={mode === 'plain' ? 'warning' : 'info'} sx={{ mt: 1.5 }}>
            {mode === 'plain'
              ? 'Adding identities without encryption is enabled in Settings: the seeds are stored in plain text and no password is asked for.'
              : 'The seeds are encrypted with your master password.'}
            <FormControlLabel
              sx={{ display: 'flex', mt: 0.5 }}
              control={<Checkbox size='small' checked={encryptAnyway} onChange={(e) => setEncryptAnyway(e.target.checked)} disabled={importing} />}
              label={<Typography variant='body2'>Encrypt with my master password instead</Typography>}
            />
          </Alert>
        )}
        {!checking && needsPassword && (
          <Box component='form' onSubmit={(e) => { e.preventDefault(); runImport(); }} sx={{ mt: 1.5 }}>
            {!allowNonEncrypted && (
              <Typography variant='body2' color='text.secondary' sx={{ mb: 1 }}>
                The wallet is locked. Enter your master password to encrypt the seeds as they are stored.
              </Typography>
            )}
            <TextField
              fullWidth
              type='password'
              label='Master password'
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              error={Boolean(passwordError)}
              helperText={passwordError || ' '}
              autoComplete='current-password'
              disabled={importing}
            />
          </Box>
        )}

        {importing && progress && (
          <Box sx={{ mt: 1.5 }}>
            <LinearProgress variant='determinate' value={(progress.done / progress.total) * 100} />
            <Typography variant='caption' color='text.secondary'>
              {progress.done} of {progress.total} added
            </Typography>
          </Box>
        )}
        {error && (
          <Alert severity='error' sx={{ mt: 1.5 }}>
            {error}
          </Alert>
        )}
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onClose} disabled={busy}>
          Cancel
        </Button>
        <Button
          variant='contained'
          color={mode === 'plain' || mode === 'plain-no-password' ? 'warning' : 'primary'}
          disabled={!canImport}
          onClick={runImport}
          startIcon={importing ? <CircularProgress size={16} color='inherit' /> : <FileUploadIcon />}
        >
          Import Identities
        </Button>
      </DialogActions>
    </Dialog>
  );
}
