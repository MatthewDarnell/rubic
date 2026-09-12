import React, { useEffect, useState } from 'react';
import {
  Box,
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  TextField,
  Typography,
} from '@mui/material';

export default function ConfirmDialog({
  open,
  title,
  children,
  confirmLabel = 'Confirm',
  confirmColor = 'primary',
  busy = false,
  challenge, // optional: { expected, hint } — the user must type `expected` to enable Confirm
  onConfirm,
  onCancel,
}) {
  const [typed, setTyped] = useState('');
  useEffect(() => {
    if (open) setTyped('');
  }, [open]);

  const challengeOk = !challenge || typed.trim().toUpperCase() === String(challenge.expected).toUpperCase();
  const canConfirm = !busy && challengeOk;

  return (
    <Dialog
      open={open}
      onClose={busy ? undefined : onCancel}
      maxWidth='sm'
      fullWidth
      aria-labelledby='confirm-dialog-title'
      onKeyDown={(e) => {
        if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && canConfirm) {
          e.preventDefault();
          onConfirm();
        }
      }}
    >
      <DialogTitle id='confirm-dialog-title'>{title}</DialogTitle>
      <DialogContent>
        {typeof children === 'string' ? (
          <DialogContentText>{children}</DialogContentText>
        ) : (
          children
        )}
        {challenge && (
          <Box sx={{ mt: 2.5 }}>
            <Typography variant='body2' sx={{ mb: 1 }}>
              {challenge.hint || `Type ${challenge.expected} to confirm`}
            </Typography>
            <TextField
              autoFocus
              size='small'
              value={typed}
              onChange={(e) => setTyped(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && canConfirm) {
                  e.preventDefault();
                  onConfirm();
                }
              }}
              placeholder={String(challenge.expected)}
              slotProps={{ htmlInput: { className: 'mono', autoCapitalize: 'characters', spellCheck: false } }}
              sx={{ width: 220 }}
            />
          </Box>
        )}
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onCancel} disabled={busy}>
          Cancel
        </Button>
        <Button
          onClick={onConfirm}
          color={confirmColor}
          variant='contained'
          disabled={!canConfirm}
          autoFocus={!challenge}
          startIcon={busy ? <CircularProgress size={16} color='inherit' /> : null}
        >
          {confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
