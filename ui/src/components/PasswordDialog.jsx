import React, { useEffect, useState } from 'react';
import {
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
  Typography,
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';

// The typed password is local state so each keystroke re-renders only this
// dialog, not the whole app behind it. It is handed to `onSubmit(password)`.
export default function PasswordDialog({
  open,
  error,
  busy,
  unlockTimerMs,
  onSubmit,
  onCancel,
}) {
  const [password, setPassword] = useState('');

  useEffect(() => {
    if (!open) setPassword('');
  }, [open]);

  // A rejected password is cleared so the user retypes it.
  useEffect(() => {
    if (error) setPassword('');
  }, [error]);

  const submit = () => {
    if (password && !busy) onSubmit(password);
  };

  return (
    <Dialog open={open} onClose={busy ? undefined : onCancel} maxWidth='xs' fullWidth>
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <LockIcon fontSize='small' sx={{ color: 'orange' }} />
        Unlock wallet
      </DialogTitle>
      <DialogContent>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
          Enter your master password to continue. The wallet will stay unlocked
          for {Math.round(Number(unlockTimerMs) / 1000)}s.
        </Typography>
        <TextField
          autoFocus
          fullWidth
          type='password'
          label='Master password'
          value={password}
          error={Boolean(error)}
          helperText={error || ' '}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              submit();
            }
          }}
          autoComplete='current-password'
        />
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onCancel} disabled={busy}>
          Cancel
        </Button>
        <Button
          variant='contained'
          disabled={!password || busy}
          onClick={submit}
          startIcon={busy ? <CircularProgress size={16} color='inherit' /> : null}
        >
          Confirm
        </Button>
      </DialogActions>
    </Dialog>
  );
}
