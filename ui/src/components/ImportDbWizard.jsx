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
  LinearProgress,
  Step,
  StepLabel,
  Stepper,
  TextField,
  Typography,
} from '@mui/material';
import UploadFileIcon from '@mui/icons-material/UploadFile';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import DeleteForeverIcon from '@mui/icons-material/DeleteForever';
import { apiCall, apiPost } from '../api';
import IdText from './IdText';
import Identicon from './Identicon';

const MIN_PASSWORD_LENGTH = 5; // the server's MINPASSWORDLEN

const identities = (n) => `${n} identit${n === 1 ? 'y' : 'ies'}`;

// The CSV is what "Export wallet as decrypted CSV" writes: one line per identity,
// `identity,seed,salt,hash` (seed is the 55-letter plaintext seed; salt/hash may
// be empty). Seeds are never displayed here - only the identities they belong to.
const parseCsv = (text) => {
  const rows = [];
  const problems = [];
  const seen = new Set();
  text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .forEach((line, index) => {
      if (!line) return;
      const [identity = '', seed = '', salt = '', hash = ''] = line.split(',').map((f) => f.trim());
      if (index === 0 && /^identity$/i.test(identity)) return; // header line
      const lineNo = index + 1;
      if (!/^[A-Z]{60}$/.test(identity)) {
        problems.push(`Line ${lineNo}: not a 60-letter identity`);
        return;
      }
      if (seen.has(identity)) {
        problems.push(`Line ${lineNo}: ${identity.slice(0, 8)}… listed twice; keeping the first`);
        return;
      }
      if (!/^[a-z]{55}$/.test(seed)) {
        problems.push(
          /^[0-9a-f]{20,}$/i.test(seed)
            ? `Line ${lineNo}: ${identity.slice(0, 8)}… has an encrypted seed. Export the wallet as a decrypted CSV first.`
            : `Line ${lineNo}: ${identity.slice(0, 8)}… has no valid 55-letter seed`
        );
        return;
      }
      seen.add(identity);
      rows.push({ identity, seed, salt, hash });
    });
  return { rows, problems };
};

const STEP = { UNLOCK: 'unlock', RESET: 'reset', PASSWORD: 'password', UPLOAD: 'upload', IMPORT: 'import' };
const LABEL = { [STEP.UNLOCK]: 'Unlock', [STEP.RESET]: 'Reset Db', [STEP.PASSWORD]: 'New password', [STEP.UPLOAD]: 'Upload CSV', [STEP.IMPORT]: 'Import' };

/**
 * Multi-step "Import DB From CSV" wizard.
 *
 * Modes decided when the dialog opens:
 *  - hasMasterPassword && locked   -> Unlock, or Reset Db -> New password -> Upload CSV -> Import
 *  - hasMasterPassword && unlocked -> Upload CSV -> Import (encrypts with the unlocked password)
 *  - no master password (fresh db) -> New password -> Upload CSV -> Import
 *
 * Importing with a reset first POSTs `/wallet/reset` with the new master password
 * (which drops the old password and every identity encrypted with it), then each
 * CSV row goes to `/identity/add` with that password. Without a reset the rows go
 * to `/identity/add` with the master password entered at the Unlock step, or with
 * none while the wallet is unlocked (the server then uses the unlocked password).
 */
export default function ImportDbWizard({ open, onClose, hasMasterPassword, unlockTimerMs, onImported, onUnlocked, onReset }) {
  const [step, setStep] = useState(STEP.UPLOAD);
  const [unlocked, setUnlocked] = useState(false);
  const [checking, setChecking] = useState(false);
  const [password, setPassword] = useState(''); // the existing master password (Unlock step)
  const [unlockError, setUnlockError] = useState('');
  const [unlocking, setUnlocking] = useState(false);
  const [resetConfirmed, setResetConfirmed] = useState(false);
  const [newPassword, setNewPassword] = useState(''); // the master password after a reset
  const [retype, setRetype] = useState('');
  const [file, setFile] = useState(null); // { name, rows, problems }
  const [importing, setImporting] = useState(false);
  const [progress, setProgress] = useState(null); // { done, total }
  const [importError, setImportError] = useState('');
  const fileInputRef = useRef(null);

  // Work out where to start each time the wizard opens.
  useEffect(() => {
    if (!open) return undefined;
    let cancelled = false;
    setPassword('');
    setUnlockError('');
    setResetConfirmed(false);
    setNewPassword('');
    setRetype('');
    setFile(null);
    setImportError('');
    setProgress(null);
    setUnlocked(false);
    if (!hasMasterPassword) {
      setStep(STEP.PASSWORD);
      return undefined;
    }
    setChecking(true);
    apiCall('wallet/unlocked').then((res) => {
      if (cancelled) return;
      const isUnlocked = res.success && res.data === true;
      setUnlocked(isUnlocked);
      if (isUnlocked) onUnlocked?.();
      setStep(isUnlocked ? STEP.UPLOAD : STEP.UNLOCK);
      setChecking(false);
    });
    return () => {
      cancelled = true;
    };
  }, [open, hasMasterPassword]);

  // "Reset" means the import replaces the master password first: chosen
  // explicitly, or implied when there is no master password yet.
  const reset = !hasMasterPassword || (!unlocked && resetConfirmed);

  // The steps shown, in order, for the mode we are in. The first step reads
  // "Unlock" until the reset is confirmed, then "Reset Db".
  const order = useMemo(() => {
    if (!hasMasterPassword) return [STEP.PASSWORD, STEP.UPLOAD, STEP.IMPORT];
    if (unlocked) return [STEP.UPLOAD, STEP.IMPORT];
    return resetConfirmed ? [STEP.RESET, STEP.PASSWORD, STEP.UPLOAD, STEP.IMPORT] : [STEP.UNLOCK, STEP.UPLOAD, STEP.IMPORT];
  }, [hasMasterPassword, unlocked, resetConfirmed]);
  const activeIndex = Math.max(0, order.indexOf(step === STEP.RESET ? order[0] : step));

  const tooShort = newPassword.length < MIN_PASSWORD_LENGTH;
  const mismatch = retype !== newPassword;
  const newPasswordOk = !tooShort && !mismatch;

  const unlock = async () => {
    if (!password || unlocking) return;
    setUnlocking(true);
    setUnlockError('');
    try {
      const res = await apiPost('wallet/unlock', { password, timeout_ms: Number(unlockTimerMs) || 60000 });
      const text = String(res.data ?? '');
      if (!res.success) {
        setUnlockError(`Unlock failed: ${res.error || 'no response from server'}`);
      } else if (/already unlocked/i.test(text) || /wallet unlocked/i.test(text)) {
        // Keep the password: it is sent with each imported row, so the rows are
        // encrypted even if the unlock timer runs out during the import.
        setUnlocked(true);
        onUnlocked?.();
        setStep(STEP.UPLOAD);
      } else {
        setUnlockError(/invalid|incorrect/i.test(text) ? 'Invalid password — try again, or reset the database.' : text);
      }
    } finally {
      setUnlocking(false);
    }
  };

  const chooseFile = (event) => {
    const picked = event.target.files?.[0];
    event.target.value = ''; // allow picking the same file again
    if (!picked) return;
    const reader = new FileReader();
    reader.onload = () => setFile({ name: picked.name, ...parseCsv(String(reader.result || '')) });
    reader.onerror = () => setFile({ name: picked.name, rows: [], problems: ['Could not read the file'] });
    reader.readAsText(picked);
  };

  const runImport = async () => {
    if (!file || file.rows.length === 0 || importing) return;
    setImporting(true);
    setImportError('');
    const total = file.rows.length;
    setProgress({ done: 0, total });
    try {
      if (reset) {
        const res = await apiPost('wallet/reset', { password: newPassword });
        const text = String(res.data ?? '');
        if (!res.success || !/master password set/i.test(text)) {
          setImportError(`Could not reset the wallet: ${text || res.error || 'no response from server'}`);
          return;
        }
        // From here on the wallet has the new master password and is locked.
        onReset?.();
      }
      // With a password each row is verified and encrypted by the server whether
      // or not the wallet is unlocked. Without one (the wallet was already
      // unlocked when the wizard opened) the server encrypts with the unlocked
      // password - so make sure it is still unlocked before every row, or the
      // seed would be stored in the clear.
      const rowPassword = reset ? newPassword : password;
      let added = 0;
      let skipped = 0;
      for (const [index, row] of file.rows.entries()) {
        const short = `${row.identity.slice(0, 8)}…`;
        if (!rowPassword) {
          const state = await apiCall('wallet/unlocked');
          if (!(state.success && state.data === true)) {
            setImportError(`The wallet locked itself before ${short} was added (${added} added so far, and kept). Unlock it and import the file again.`);
            return;
          }
        }
        const res = await apiPost('identity/add', { seed: row.seed, password: rowPassword });
        const text = String(res.data ?? '').trim();
        if (res.success && text === '200') {
          added += 1;
        } else if (/failed to insert/i.test(text)) {
          skipped += 1; // already in the wallet
        } else {
          setImportError(`Stopped at ${short} (row ${index + 1} of ${total}): ${text || res.error || 'no response from server'}. ${added} added so far, and kept.`);
          return;
        }
        setProgress({ done: index + 1, total });
      }
      onImported?.({ reset, count: added, skipped });
      onClose();
    } finally {
      setImporting(false);
    }
  };

  const busy = unlocking || importing || checking;

  return (
    <Dialog open={open} onClose={busy ? undefined : onClose} maxWidth='sm' fullWidth>
      <DialogTitle>Import DB From CSV</DialogTitle>
      <DialogContent>
        <Stepper activeStep={activeIndex} sx={{ mb: 3 }}>
          {order.map((key) => (
            <Step key={key}>
              <StepLabel>{LABEL[key]}</StepLabel>
            </Step>
          ))}
        </Stepper>

        {checking && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, py: 2 }}>
            <CircularProgress size={18} />
            <Typography variant='body2' color='text.secondary'>Checking whether the wallet is unlocked…</Typography>
          </Box>
        )}

        {!checking && step === STEP.UNLOCK && (
          <Box component='form' onSubmit={(e) => { e.preventDefault(); unlock(); }}>
            <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
              The wallet is locked. Unlock it to add the identities from the CSV to your existing database, encrypted
              with your master password. If you no longer know the password, reset the database instead.
            </Typography>
            <TextField
              autoFocus
              fullWidth
              type='password'
              label='Master password'
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              error={Boolean(unlockError)}
              helperText={unlockError || ' '}
              autoComplete='current-password'
              disabled={unlocking}
            />
            <Box sx={{ display: 'flex', gap: 1, mt: 1 }}>
              <Button
                type='submit'
                variant='contained'
                disabled={!password || unlocking}
                startIcon={unlocking ? <CircularProgress size={16} color='inherit' /> : <LockOpenIcon />}
              >
                Unlock
              </Button>
              <Button color='error' startIcon={<DeleteForeverIcon />} disabled={unlocking} onClick={() => setStep(STEP.RESET)}>
                Reset db
              </Button>
            </Box>
          </Box>
        )}

        {!checking && step === STEP.RESET && (
          <Box>
            <Alert severity='error' sx={{ mb: 2 }}>
              Resetting removes the master password and every identity and seed encrypted with it from this
              wallet's database. Anything not in the CSV you are about to import is lost for good.
            </Alert>
            <FormControlLabel
              control={<Checkbox color='error' checked={resetConfirmed} onChange={(e) => setResetConfirmed(e.target.checked)} />}
              label={<Typography sx={{ fontWeight: 600 }}>WARNING: This will destroy all Identities and Seeds in your Database</Typography>}
            />
          </Box>
        )}

        {!checking && step === STEP.PASSWORD && (
          <Box component='form' onSubmit={(e) => { e.preventDefault(); if (newPasswordOk) setStep(STEP.UPLOAD); }}>
            <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
              {hasMasterPassword
                ? 'Choose the new master password. It replaces the current one during the import, and the imported seeds are encrypted with it. There is no recovery if you lose it.'
                : 'Choose a master password. The imported seeds are encrypted with it before they are stored. There is no recovery if you lose it.'}
            </Typography>
            <TextField
              autoFocus
              fullWidth
              type='password'
              label='New master password'
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              helperText={newPassword && tooShort ? `At least ${MIN_PASSWORD_LENGTH} characters` : ' '}
              error={Boolean(newPassword) && tooShort}
              autoComplete='new-password'
            />
            <TextField
              fullWidth
              type='password'
              label='Retype password'
              value={retype}
              onChange={(e) => setRetype(e.target.value)}
              helperText={retype && mismatch ? 'Passwords do not match' : ' '}
              error={Boolean(retype) && mismatch}
              autoComplete='new-password'
            />
          </Box>
        )}

        {!checking && step === STEP.UPLOAD && (
          <Box>
            <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
              Choose a CSV written by “Export wallet as decrypted CSV” (one identity per line:
              identity, seed, salt, hash). Seeds are read from the file but never shown here.
            </Typography>
            <input ref={fileInputRef} type='file' accept='.csv,text/csv' hidden onChange={chooseFile} />
            <Button variant='contained' startIcon={<UploadFileIcon />} onClick={() => fileInputRef.current?.click()}>
              Upload CSV
            </Button>
            {file && (
              <Box sx={{ mt: 2 }}>
                <Typography variant='subtitle2'>
                  {file.name}: {identities(file.rows.length)} ready to import
                </Typography>
                {file.rows.length > 0 && (
                  <Box sx={{ maxHeight: 180, overflow: 'auto', mt: 1, pr: 1 }}>
                    {file.rows.map((row) => (
                      <Box key={row.identity} sx={{ display: 'flex', alignItems: 'center', gap: 1, py: 0.5 }}>
                        <Identicon id={row.identity} size={20} />
                        <IdText id={row.identity} full nowrap copy={false} sx={{ fontSize: '0.74rem' }} />
                      </Box>
                    ))}
                  </Box>
                )}
                {file.problems.length > 0 && (
                  <Alert severity='warning' sx={{ mt: 1.5 }}>
                    {file.problems.length} line{file.problems.length === 1 ? '' : 's'} skipped:
                    <Box component='ul' sx={{ m: 0, pl: 2.5, mt: 0.5 }}>
                      {file.problems.slice(0, 6).map((p) => (
                        <li key={p}>
                          <Typography variant='caption'>{p}</Typography>
                        </li>
                      ))}
                      {file.problems.length > 6 && (
                        <li>
                          <Typography variant='caption'>…and {file.problems.length - 6} more</Typography>
                        </li>
                      )}
                    </Box>
                  </Alert>
                )}
              </Box>
            )}
          </Box>
        )}

        {!checking && step === STEP.IMPORT && file && (
          <Box>
            <Typography sx={{ fontWeight: 600 }}>
              Import {identities(file.rows.length)} from {file.name}
            </Typography>
            <Typography variant='body2' sx={{ mt: 1, color: reset ? 'error.main' : 'success.main', fontWeight: 600 }}>
              {reset
                ? 'This will import the Identities from the csv and delete your database'
                : 'This will import the identities from the csv and automatically encrypt them with your master password'}
            </Typography>
            {reset && (
              <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
                {hasMasterPassword
                  ? 'Your current master password, and every identity encrypted with it, are removed first. The new master password is set, and the imported identities are encrypted with it.'
                  : 'The master password you chose is set first, and the imported identities are encrypted with it.'}
              </Typography>
            )}
            {importing && progress && (
              <Box sx={{ mt: 2 }}>
                <LinearProgress variant='determinate' value={(progress.done / progress.total) * 100} />
                <Typography variant='caption' color='text.secondary'>
                  {progress.done} of {progress.total} added
                </Typography>
              </Box>
            )}
            {importError && (
              <Alert severity='error' sx={{ mt: 2 }}>
                {importError}
              </Alert>
            )}
          </Box>
        )}
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onClose} disabled={busy}>
          Cancel
        </Button>
        {step === STEP.RESET && (
          <>
            <Button onClick={() => setStep(STEP.UNLOCK)} disabled={busy}>
              Back
            </Button>
            <Button variant='contained' color='error' disabled={!resetConfirmed || busy} onClick={() => setStep(STEP.PASSWORD)}>
              Next
            </Button>
          </>
        )}
        {step === STEP.PASSWORD && (
          <>
            {hasMasterPassword && (
              <Button onClick={() => setStep(STEP.RESET)} disabled={busy}>
                Back
              </Button>
            )}
            <Button variant='contained' disabled={!newPasswordOk || busy} onClick={() => setStep(STEP.UPLOAD)}>
              Next
            </Button>
          </>
        )}
        {step === STEP.UPLOAD && (
          <>
            {reset && (
              <Button onClick={() => setStep(STEP.PASSWORD)} disabled={busy}>
                Back
              </Button>
            )}
            <Button variant='contained' disabled={!file || file.rows.length === 0 || busy} onClick={() => setStep(STEP.IMPORT)}>
              Next
            </Button>
          </>
        )}
        {step === STEP.IMPORT && (
          <>
            <Button onClick={() => setStep(STEP.UPLOAD)} disabled={busy}>
              Back
            </Button>
            <Button
              variant='contained'
              color={reset ? 'error' : 'primary'}
              disabled={busy}
              onClick={runImport}
              startIcon={importing ? <CircularProgress size={16} color='inherit' /> : null}
            >
              Import
            </Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
}
