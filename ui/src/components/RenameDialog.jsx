import React, { useEffect, useState } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField, Typography } from '@mui/material';
import IdText from './IdText';

export default function RenameDialog({ target, onClose, onSave }) {
  const [value, setValue] = useState('');
  useEffect(() => {
    setValue(target?.label ?? '');
  }, [target]);
  const save = () => {
    onSave(target.id, value.trim());
    onClose();
  };
  return (
    <Dialog open={Boolean(target)} onClose={onClose} maxWidth='xs' fullWidth>
      <DialogTitle>Nickname</DialogTitle>
      <DialogContent>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
          Shown next to the identity in this app only — it is never sent anywhere.
        </Typography>
        {target && <IdText id={target.id} full copy={false} sx={{ mb: 2, display: 'block' }} />}
        <TextField
          autoFocus
          fullWidth
          size='small'
          label='Nickname'
          value={value}
          onChange={(e) => setValue(e.target.value.slice(0, 40))}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              save();
            }
          }}
          helperText='Leave empty to remove'
        />
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onClose}>Cancel</Button>
        <Button variant='contained' onClick={save}>
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
}
