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
import UploadFileIcon from '@mui/icons-material/UploadFile';
import SaveIcon from '@mui/icons-material/Save';
import WarningAmberIcon from '@mui/icons-material/WarningAmber';
import AddIcon from '@mui/icons-material/Add';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutlined';
import DriveFileRenameOutlineIcon from '@mui/icons-material/DriveFileRenameOutline';
import Identicon from './Identicon';
import IdText from './IdText';
import { CURRENCIES, currencyLabel, digitsOnly, formatFiat, formatNumber, isNumeric, upperOnly } from '../utils/format';
import { serverIp } from '../api_config';
import pkg from '../../package.json';

// The server rejects unlock timeouts above 99,999 ms.
const MAX_UNLOCK_SECONDS = 99;
const PRESETS = [15, 30, 60, 90];
const DEFAULT_TICK_OFFSET = 30;
const MAX_TICK_OFFSET = 1000;

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
          sx={{ flex: 1, minWidth: 260 }}
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

// `grow` lets the last card of a column stretch so all column bottoms line up.
const Section = ({ title, description, children, danger, grow }) => (
  <Paper
    variant='outlined'
    sx={{
      p: 2,
      ...(grow && { flex: 1 }),
      ...(danger && { borderColor: 'error.main', bgcolor: 'rgba(255, 92, 108, 0.04)' }),
    }}
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
  tickOffset = DEFAULT_TICK_OFFSET,
  onTickOffsetChange,
  allowNonEncrypted,
  onAllowNonEncryptedChange,
  unencryptedCount,
  peerLimits,
  onSavePeerLimits,
  onDownloadWallet,
  onImportDb,
  latestTick,
  currency,
  onCurrencyChange,
  price = 0,
  priceStatus = 'loading',
  addressBook = [],
  onAddressBookChange,
  identities = [],
  labels = {},
}) {
  const [min, setMin] = useState(String(peerLimits.min));
  const [max, setMax] = useState(String(peerLimits.max));
  const [saving, setSaving] = useState(false);
  // Typed offset is kept as text so the field can be emptied while editing; the
  // saved value only changes when the text is a valid number.
  const [offsetText, setOffsetText] = useState(String(tickOffset));
  useEffect(() => {
    setOffsetText(String(tickOffset));
  }, [tickOffset]);
  const offsetNum = Number(offsetText);
  const offsetValid = offsetText !== '' && Number.isInteger(offsetNum) && offsetNum >= 1 && offsetNum <= MAX_TICK_OFFSET;
  const changeOffset = (text) => {
    if (!digitsOnly(text)) return;
    setOffsetText(text);
    const n = Number(text);
    if (text !== '' && Number.isInteger(n) && n >= 1 && n <= MAX_TICK_OFFSET) onTickOffsetChange(n);
  };

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
    // Three columns on a normal desktop window (two on medium, one on narrow) so
    // the whole page fits without scrolling.
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))', lg: 'repeat(3, minmax(0, 1fr))' },
        gap: 1.5,
        // Columns stretch to the tallest one; each column's last card grows to fill.
        alignItems: 'stretch',
        '& > div': { minWidth: 0, display: 'flex', flexDirection: 'column', gap: 1.5 },
      }}
    >
      <Box>
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

      <Section title='Transactions'>
        <Stack direction='row' spacing={2} alignItems='flex-start' flexWrap='wrap' useFlexGap>
          <TextField
            label='Transfer Ticks Offset'
            size='small'
            value={offsetText}
            onChange={(e) => changeOffset(e.target.value)}
            onBlur={() => !offsetValid && setOffsetText(String(tickOffset))}
            error={!offsetValid}
            helperText={
              !offsetValid
                ? `1–${MAX_TICK_OFFSET} ticks`
                : offsetNum === DEFAULT_TICK_OFFSET
                ? 'Default'
                : `Default is ${DEFAULT_TICK_OFFSET}`
            }
            inputProps={{ maxLength: 4, inputMode: 'numeric' }}
            sx={{ width: 200 }}
          />
          {tickOffset !== DEFAULT_TICK_OFFSET && (
            <Button size='small' onClick={() => onTickOffsetChange(DEFAULT_TICK_OFFSET)} sx={{ height: 40 }}>
              Reset to default
            </Button>
          )}
        </Stack>
      </Section>

      <Section grow title='Display' description='Fiat values are indicative only, from the CoinGecko public price feed.'>
        <Stack direction='row' spacing={2} alignItems='center' flexWrap='wrap' useFlexGap>
          <FormControl size='small' sx={{ minWidth: 160 }}>
            <InputLabel id='currency-label'>Fiat currency</InputLabel>
            <Select
              labelId='currency-label'
              label='Fiat currency'
              value={currency}
              onChange={(e) => onCurrencyChange(e.target.value)}
              renderValue={(c) => currencyLabel(c)}
            >
              {CURRENCIES.map((c) => (
                <MenuItem key={c} value={c}>
                  {currencyLabel(c)}
                  {currencyLabel(c) !== c && (
                    <Box component='span' sx={{ ml: 1, color: 'text.secondary' }}>{c}</Box>
                  )}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          {priceStatus === 'ok' && price > 0 ? (
            <Typography variant='body2' color='text.secondary'>
              1,000,000 QU {formatFiat(1000000, price, currency)}
            </Typography>
          ) : priceStatus === 'error' ? (
            <Typography variant='body2' sx={{ color: 'warning.main' }}>
              Price feed unreachable — fiat values are hidden until it answers.
            </Typography>
          ) : (
            <Typography variant='body2' color='text.secondary'>
              Fetching price…
            </Typography>
          )}
        </Stack>
      </Section>
      </Box>

      <Box>
      <Section
        title='Address book'
        description='Recipients you send to often. Names are stored in this app only and never leave your machine.'
      >
        <AddressBook entries={addressBook} onChange={onAddressBookChange} identities={identities} labels={labels} />
      </Section>

      <Section
        grow
        title='Peer limits'
        description='Rubic keeps at least the minimum and at most the maximum number of peers connected. Changes apply after you save.'
      >
        <Stack direction='row' spacing={2} alignItems='flex-start' flexWrap='wrap' useFlexGap>
          <TextField
            label='Min peers'
            size='small'
            value={min}
            onChange={(e) => digitsOnly(e.target.value) && setMin(e.target.value)}
            error={!minValid || !ordered}
            helperText={!minValid ? '1–255' : !ordered ? 'Must be ≤ max' : ' '}
            sx={{ width: 120 }}
          />
          <TextField
            label='Max peers'
            size='small'
            value={max}
            onChange={(e) => digitsOnly(e.target.value) && setMax(e.target.value)}
            error={!maxValid}
            helperText={!maxValid ? '1–255' : ' '}
            sx={{ width: 120 }}
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
      </Box>

      <Box>
      <Section
        danger
        title='Danger zone'
        description='These settings can weaken the protection of your seeds. Use them only if you understand the consequences.'
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
            <Button variant='outlined' color='error' startIcon={<UploadFileIcon />} onClick={onImportDb}>
              Import DB From CSV
            </Button>
            <Typography variant='caption' color='text.secondary' sx={{ display: 'block', mt: 0.5 }}>
              Restore identities from a decrypted CSV export. Unlock to add them to this database, or reset the
              database and start from the file.
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

      <Section grow title='About'>
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
    </Box>
  );
}
