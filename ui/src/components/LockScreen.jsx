import React, { useState } from 'react';
import { Box, Button, CircularProgress, TextField, Typography } from '@mui/material';
import WalletIcon from '@mui/icons-material/Wallet';
import LockIcon from '@mui/icons-material/Lock';

const MIN_LENGTH = 5;

export default function LockScreen({ onSetPassword }) {
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
        minHeight: '70vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
      }}
    >
      <WalletIcon sx={{ fontSize: 96 }} />
      <Typography variant='h6'>Set a master password</Typography>
      <Typography variant='body2' color='text.secondary' sx={{ maxWidth: 420, textAlign: 'center' }}>
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
        sx={{ width: 320 }}
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
    </Box>
  );
}
