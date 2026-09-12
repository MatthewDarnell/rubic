import React, { useEffect, useState } from 'react';
import {
  Box,
  Button,
  Chip,
  CircularProgress,
  FormControl,
  FormControlLabel,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Switch,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Tooltip,
  Typography,
} from '@mui/material';
import DownloadIcon from '@mui/icons-material/Download';
import SaveIcon from '@mui/icons-material/Save';
import WarningAmberIcon from '@mui/icons-material/WarningAmber';
import AddIcon from '@mui/icons-material/Add';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutline';
import DriveFileRenameOutlineIcon from '@mui/icons-material/DriveFileRenameOutline';
import Identicon from './Identicon';
import IdText from './IdText';
import { CURRENCIES, digitsOnly, formatNumber, isNumeric, upperOnly } from '../utils/format';
import { serverIp } from '../api_config';
import pkg from '../../package.json';

// The server rejects unlock timeouts above 99,999 ms.
const MAX_UNLOCK_SECONDS = 99;
const PRESETS = [15, 30, 60, 90];

const AddressBook = ({ entries, onChange, identities, labels }) => {
  const [label, setLabel] = useState('');
  const [id, setId] = useState('');
  const [editing, setEditing] = useState(null); // { id, label }

  const idValid = /^[A-Z]{60}$/.test(id);
  const duplicate = entries.some((e) => e.id === id);
  const ownIdentity = identities.find((i) => i.id === id);
  const canAdd = idValid && !duplicate && label.trim().length > 0;

  const add = () => {
    if (!canAdd) return;
    onChange([...entries, { id, label: label.trim() }]);
    setLabel('');
    setId('');
  };
  const remove = (entryId) => onChange(entries.filter((e) => e.id !== entryId));
  const saveEdit = () => {
    if (!editing) return;
    const next = editing.label.trim();
    onChange(next ? entries.map((e) => (e.id === editing.id ? { ...e, label: next } : e)) : entries.filter((e) => e.id !== editing.id));
    setEditing(null);
  };

  return (
    <Stack spacing={2}>
      <Stack direction='row' spacing={1} alignItems='flex-start' flexWrap='wrap' useFlexGap component='form' onSubmit={(e) => { e.preventDefault(); add(); }}>
        <TextField
          label='Name'
          size='small'
          value={label}
          onChange={(e) => setLabel(e.target.value.slice(0, 40))}
          sx={{ width: 200 }}
        />
        <TextField
          label='Identity (60 letters)'
          size='small'
          value={id}
          onChange={(e) => upperOnly(e.target.value.toUpperCase()) && setId(e.target.value.toUpperCase().slice(0, 60))}
          error={id.length > 0 && (!idValid || duplicate)}
          helperText={
            duplicate ? 'Already in the address book' : ownIdentity ? `This is your own identity${labels[id] ? ` (${labels[id]})` : ''}` : id.length > 0 && !idValid ? `${id.length}/60` : ' '
          }
          slotProps={{ htmlInput: { className: 'mono', spellCheck: false } }}
          sx={{ flex: 1, minWidth: 420 }}
        />
        <Button type='submit' variant='contained' startIcon={<AddIcon />} disabled={!canAdd} sx={{ height: 40 }}>
          Add
        </Button>
      </Stack>

      {entries.length === 0 ? (
        <Typography variant='body2' color='text.secondary'>
          No saved recipients yet. You can also save one from the Send dialog.
        </Typography>
      ) : (
        <Paper variant='outlined'>
          {entries.map((e) => (
            <Box
              key={e.id}
              sx={{ display: 'flex', alignItems: 'center', gap: 1.5, px: 2, py: 1, borderBottom: 1, borderColor: 'divider', '&:last-of-type': { borderBottom: 0 } }}
            >
              <Identicon id={e.id} size={28} />
              <Box sx={{ minWidth: 0, flex: 1 }}>
                {editing?.id === e.id ? (
                  <TextField
                    autoFocus
                    size='small'
                    value={editing.label}
                    onChange={(ev) => setEditing({ ...editing, label: ev.target.value.slice(0, 40) })}
                    onKeyDown={(ev) => {
                      if (ev.key === 'Enter') saveEdit();
                      if (ev.key === 'Escape') setEditing(null);
                    }}
                    onBlur={saveEdit}
                    sx={{ width: 240 }}
                  />
                ) : (
                  <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{e.label}</Typography>
                )}
                <IdText id={e.id} full copy explorer='address' sx={{ fontSize: '0.74rem', color: 'text.secondary' }} />
              </Box>
              <Tooltip title='Rename'>
                <IconButton size='small' onClick={() => setEditing({ id: e.id, label: e.label })}>
                  <DriveFileRenameOutlineIcon fontSize='small' />
                </IconButton>
              </Tooltip>
              <Tooltip title='Remove'>
                <IconButton size='small' onClick={() => remove(e.id)}>
                  <DeleteOutlineIcon fontSize='small' />
                </IconButton>
              </Tooltip>
            </Box>
          ))}
        </Paper>
      )}
    </Stack>
  );
};

const Section = ({ title, description, children, danger }) => (
  <Paper
    variant='outlined'
    sx={{ p: 2.5, mb: 2, ...(danger && { borderColor: 'error.main', bgcolor: 'rgba(255, 92, 108, 0.04)' }) }}
  >
    <Typography variant='subtitle1' sx={{ fontWeight: 600, display: 'flex', alignItems: 'center', gap: 1 }}>
      {danger && <WarningAmberIcon fontSize='small' color='error' />}
      {title}
    </Typography>
    {description && (
      <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
        {description}
      </Typography>
    )}
    {children}
  </Paper>
);

export default function SettingsPanel({
  unlockTimerMs,
  onUnlockTimerChange,
  allowNonEncrypted,
  onAllowNonEncryptedChange,
  unencryptedCount,
  peerLimits,
  onSavePeerLimits,
  onDownloadWallet,
  latestTick,
  currency,
  onCurrencyChange,
  addressBook = [],
  onAddressBookChange,
  identities = [],
  labels = {},
}) {
  const [min, setMin] = useState(String(peerLimits.min));
  const [max, setMax] = useState(String(peerLimits.max));
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setMin(String(peerLimits.min));
    setMax(String(peerLimits.max));
  }, [peerLimits.min, peerLimits.max]);

  const seconds = Math.round(Number(unlockTimerMs) / 1000);
  const setSeconds = (s) => {
    const clamped = Math.min(MAX_UNLOCK_SECONDS, Math.max(1, Number(s) || 1));
    onUnlockTimerChange(String(clamped * 1000));
  };

  const minNum = Number(min);
  const maxNum = Number(max);
  const minValid = min !== '' && minNum >= 1 && minNum <= 255;
  const maxValid = max !== '' && maxNum >= 1 && maxNum <= 255;
  const ordered = minValid && maxValid && minNum <= maxNum;
  const dirty = minNum !== Number(peerLimits.min) || maxNum !== Number(peerLimits.max);

  const save = async () => {
    setSaving(true);
    try {
      await onSavePeerLimits(minNum, maxNum);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Box sx={{ maxWidth: 820 }}>
      <Section
        title='Wallet unlock'
        description='After a correct password the wallet stays unlocked for this long, so follow-up actions do not prompt again.'
      >
        <Stack direction='row' spacing={2} alignItems='center' flexWrap='wrap' useFlexGap>
          <ToggleButtonGroup
            exclusive
            size='small'
            value={PRESETS.includes(seconds) ? seconds : null}
            onChange={(_e, v) => v !== null && setSeconds(v)}
          >
            {PRESETS.map((s) => (
              <ToggleButton key={s} value={s} sx={{ px: 2 }}>
                {s < 60 ? `${s} s` : s === 60 ? '1 min' : `${s} s`}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
          <TextField
            label='Custom (seconds)'
            size='small'
            value={String(seconds)}
            onChange={(e) => digitsOnly(e.target.value) && e.target.value !== '' && setSeconds(e.target.value)}
            inputProps={{ maxLength: 2 }}
            helperText={`1–${MAX_UNLOCK_SECONDS} seconds`}
            sx={{ width: 160 }}
          />
        </Stack>
      </Section>

      <Section title='Display' description='Fiat values are indicative only, from the CoinGecko public price feed.'>
        <FormControl size='small' sx={{ minWidth: 200 }}>
          <InputLabel id='currency-label'>Fiat currency</InputLabel>
          <Select labelId='currency-label' label='Fiat currency' value={currency} onChange={(e) => onCurrencyChange(e.target.value)}>
            {CURRENCIES.map((c) => (
              <MenuItem key={c} value={c}>{c}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Section>

      <Section
        title='Peer limits'
        description='Rubic keeps at least the minimum and at most the maximum number of peers connected. Changes apply after you save.'
      >
        <Stack direction='row' spacing={2} alignItems='flex-start'>
          <TextField
            label='Min peers'
            size='small'
            value={min}
            onChange={(e) => digitsOnly(e.target.value) && setMin(e.target.value)}
            error={!minValid || !ordered}
            helperText={!minValid ? '1–255' : !ordered ? 'Must be ≤ max' : ' '}
            sx={{ width: 140 }}
          />
          <TextField
            label='Max peers'
            size='small'
            value={max}
            onChange={(e) => digitsOnly(e.target.value) && setMax(e.target.value)}
            error={!maxValid}
            helperText={!maxValid ? '1–255' : ' '}
            sx={{ width: 140 }}
          />
          <Button
            variant='contained'
            onClick={save}
            disabled={!ordered || !dirty || saving}
            startIcon={saving ? <CircularProgress size={16} color='inherit' /> : <SaveIcon />}
            sx={{ height: 40 }}
          >
            Save
          </Button>
        </Stack>
      </Section>

      <Section
        title='Address book'
        description='Recipients you send to often. Names are stored in this app only and never leave your machine.'
      >
        <AddressBook entries={addressBook} onChange={onAddressBookChange} identities={identities} labels={labels} />
      </Section>

      <Section
        danger
        title='Danger zone'
        description='These settings weaken the protection of your seeds. Use them only if you understand the consequences.'
      >
        <Stack spacing={2}>
          <Box>
            <FormControlLabel
              control={
                <Switch
                  color='error'
                  checked={allowNonEncrypted}
                  onChange={(e) => onAllowNonEncryptedChange(e.target.checked)}
                />
              }
              label='Allow adding identities without encryption'
            />
            <Typography variant='caption' color='text.secondary' sx={{ display: 'block', ml: 6 }}>
              Unencrypted seeds are stored in plain text in the wallet database.
              {unencryptedCount > 0 && (
                <Chip
                  size='small'
                  color='warning'
                  variant='outlined'
                  label={`${unencryptedCount} unencrypted identit${unencryptedCount === 1 ? 'y' : 'ies'} in this wallet`}
                  sx={{ ml: 1 }}
                />
              )}
            </Typography>
          </Box>
          <Box>
            <Button variant='outlined' color='error' startIcon={<DownloadIcon />} onClick={onDownloadWallet}>
              Export wallet as decrypted CSV
            </Button>
            <Typography variant='caption' color='text.secondary' sx={{ display: 'block', mt: 0.5 }}>
              Contains every seed in plain text. Asks for the master password; treat the file like cash.
            </Typography>
          </Box>
        </Stack>
      </Section>

      <Section title='About'>
        <Stack spacing={0.5}>
          <Typography variant='body2'>
            Rubic UI <b>v{pkg.version}</b>
          </Typography>
          <Typography variant='body2' color='text.secondary'>
            Local server {serverIp}
            {isNumeric(latestTick) ? ` · tick ${formatNumber(latestTick)}` : ''}
          </Typography>
          <Typography variant='caption' color='text.secondary'>
            This software comes with no warranty. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.
          </Typography>
        </Stack>
      </Section>
    </Box>
  );
}
