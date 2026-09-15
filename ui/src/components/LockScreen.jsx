import React, { useState } from 'react';
import { Box, Button, CircularProgress, TextField, Typography } from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import UploadFileIcon from '@mui/icons-material/UploadFile';
import logo from '../assets/rubic.png';

const MIN_LENGTH = 5;

// First-run screen: brand up top, then the master-password form.
export default function LockScreen({ onSetPassword, onImportDb }) {
  const [password, setPassword] = useState('');
  const [retype, setRetype] = useState('');
  const [busy, setBusy] = useState(false);

  const tooShort = password.length < MIN_LENGTH;
  const mismatch = retype !== password;
  const canSubmit = !tooShort && !mismatch && !busy;

  const submit = async (e) => {
    e.preventDefault();
    if (!canSubmit) return;
    setBusy(true);
    try {
      await onSetPassword(password);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Box
      component='form'
      onSubmit={submit}
      sx={{
        minHeight: '80vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center',
      }}
    >
      <Box
        component='img'
        src={logo}
        alt='Rubic'
        sx={{ width: 128, height: 128, borderRadius: 4, display: 'block', mb: 2.5 }}
      />
      <Typography variant='h3' component='h1' sx={{ fontWeight: 700, letterSpacing: '-0.02em', lineHeight: 1.1 }}>
        Rubic
      </Typography>
      <Typography variant='subtitle1' color='text.secondary' sx={{ mb: 5 }}>
        Qubic Wallet
      </Typography>

      <Typography variant='h6' sx={{ mb: 0.5 }}>
        Set a master password
      </Typography>
      <Typography variant='body2' color='text.secondary' sx={{ maxWidth: 420, mb: 3 }}>
        Seeds are encrypted with this password before they are stored. There is
        no recovery if you lose it.
      </Typography>
      <TextField
        type='password'
        label='New password'
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        helperText={password && tooShort ? `At least ${MIN_LENGTH} characters` : ' '}
        error={Boolean(password) && tooShort}
        autoComplete='new-password'
        sx={{ width: 320 }}
        autoFocus
      />
      <TextField
        type='password'
        label='Retype password'
        value={retype}
        onChange={(e) => setRetype(e.target.value)}
        helperText={retype && mismatch ? 'Passwords do not match' : ' '}
        error={Boolean(retype) && mismatch}
        autoComplete='new-password'
        sx={{ width: 320, mb: 1 }}
      />
      <Button
        type='submit'
        variant='contained'
        size='large'
        disabled={!canSubmit}
        startIcon={busy ? <CircularProgress size={18} color='inherit' /> : <LockIcon />}
      >
        Encrypt wallet
      </Button>
      {onImportDb && (
        <>
          <Typography variant='body2' color='text.secondary' sx={{ mt: 4, mb: 1 }}>
            Already have a wallet export?
          </Typography>
          <Button variant='outlined' startIcon={<UploadFileIcon />} disabled={busy} onClick={onImportDb}>
            Import DB From CSV
          </Button>
        </>
      )}
    </Box>
  );
}
