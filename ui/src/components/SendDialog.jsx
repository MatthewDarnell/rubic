import React, { useEffect, useMemo, useState } from 'react';
import {
  Autocomplete,
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  FormControl,
  InputAdornment,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import SendIcon from '@mui/icons-material/Send';
import BookmarkAddOutlinedIcon from '@mui/icons-material/BookmarkAddOutlined';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import IdText from './IdText';
import Identicon from './Identicon';
import { digitsOnly, formatFiat, formatNumber, isNumeric, parseCreated, shortenId, upperOnly } from '../utils/format';

const Row = ({ label, children }) => (
  <Box sx={{ display: 'flex', gap: 1.5, py: 1 }}>
    <Typography variant='body2' color='text.secondary' sx={{ width: 56, flexShrink: 0, pt: 0.25 }}>
      {label}
    </Typography>
    <Box sx={{ minWidth: 0, flex: 1 }}>{children}</Box>
  </Box>
);

// Keeps a 60-character identity on one line inside the review step.
const idOnOneLine = { fontSize: '0.7rem', letterSpacing: '-0.01em' };

export default function SendDialog({
  open,
  onClose,
  identities,
  labels,
  transfers,
  price,
  currency,
  addressBook,
  onAddressBookChange,
  initialFrom,
  onAction,
  warningsFor,
}) {
  const usd = (qu) => formatFiat(qu, price, currency);
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [step, setStep] = useState('form');
  const [saveLabel, setSaveLabel] = useState(null); // null | string being edited
  const setAddressBook = onAddressBookChange;

  // Reset only when the dialog opens; identities re-poll every few seconds and must not wipe the form.
  useEffect(() => {
    if (!open) return;
    setFrom(initialFrom && identities.some((i) => i.id === initialFrom) ? initialFrom : identities[0]?.id ?? '');
    setTo('');
    setAmount('');
    setStep('form');
    setSaveLabel(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const ownIds = useMemo(() => new Set(identities.map((i) => i.id)), [identities]);
  const nameOf = (id) => labels[id] || addressBook.find((e) => e.id === id)?.label || null;

  const options = useMemo(() => {
    const seen = new Set();
    const out = [];
    const push = (id, label, group) => {
      if (!id || seen.has(id) || id === from) return;
      seen.add(id);
      out.push({ id, label, group });
    };
    addressBook.forEach((e) => push(e.id, e.label, 'Address book'));
    identities.forEach((i) => push(i.id, labels[i.id] || null, 'My identities'));
    [...transfers]
      .filter((t) => ownIds.has(t.source) && !ownIds.has(t.destination))
      .sort((a, b) => parseCreated(b.created) - parseCreated(a.created))
      .forEach((t) => push(t.destination, null, 'Recent recipients'));
    return out;
  }, [addressBook, identities, labels, transfers, ownIds, from]);

  const source = identities.find((i) => i.id === from);
  const balance = source && isNumeric(source.balance) ? Number(source.balance) : null;
  const amountNum = Number(amount);
  const overBalance = balance !== null && amountNum > balance;
  const toValid = /^[A-Z]{60}$/.test(to);
  const valid = Boolean(from) && toValid && amountNum > 0 && !overBalance && to !== from;
  const inBook = addressBook.some((e) => e.id === to);

  const saveToBook = () => {
    const label = (saveLabel || '').trim();
    if (!label) return;
    setAddressBook((book) => [...book.filter((e) => e.id !== to), { id: to, label }]);
    setSaveLabel(null);
  };

  const submit = () => {
    onAction(`transfer/${from}/${to}/${amountNum}/`);
    onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth='sm'
      fullWidth
      onKeyDown={(e) => {
        if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
          e.preventDefault();
          if (step === 'form' && valid) setStep('review');
          else if (step === 'review') submit();
        }
      }}
    >
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        {step === 'review' && (
          <Button size='small' startIcon={<ArrowBackIcon />} onClick={() => setStep('form')} sx={{ ml: -1 }}>
            Back
          </Button>
        )}
        {step === 'form' ? 'Send QU' : 'Review transfer'}
      </DialogTitle>

      {step === 'form' ? (
        <DialogContent>
          <Stack spacing={2.5} sx={{ pt: 0.5 }}>
            <FormControl fullWidth>
              <InputLabel id='send-from-label'>From</InputLabel>
              <Select
                labelId='send-from-label'
                label='From'
                value={from}
                onChange={(e) => setFrom(e.target.value)}
                renderValue={(id) => (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 0 }}>
                    <Identicon id={id} size={20} />
                    {nameOf(id) && <Typography sx={{ fontWeight: 600 }}>{nameOf(id)}</Typography>}
                    <Box component='span' className='mono' sx={{ color: 'text.secondary' }}>
                      {shortenId(id, 10, 10)}
                    </Box>
                  </Box>
                )}
              >
                {identities.map((i) => (
                  <MenuItem key={i.id} value={i.id}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', width: '100%', gap: 2 }}>
                      <Identicon id={i.id} size={28} />
                      <Box sx={{ minWidth: 0, flex: 1 }}>
                        {labels[i.id] && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{labels[i.id]}</Typography>}
                        <Box component='span' className='mono' sx={{ fontSize: '0.8rem', color: 'text.secondary' }}>
                          {shortenId(i.id, 12, 12)}
                        </Box>
                      </Box>
                      <Typography sx={{ whiteSpace: 'nowrap' }}>
                        {isNumeric(i.balance) ? `${formatNumber(i.balance)} QU` : '—'}
                      </Typography>
                    </Box>
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <Box>
              <Autocomplete
                freeSolo
                options={options}
                groupBy={(o) => o.group}
                getOptionLabel={(o) => (typeof o === 'string' ? o : o.id)}
                inputValue={to}
                onInputChange={(_e, value) => {
                  const v = value.toUpperCase();
                  if (upperOnly(v)) setTo(v.slice(0, 60));
                }}
                onChange={(_e, value) => {
                  if (value && typeof value === 'object') setTo(value.id);
                }}
                filterOptions={(opts, state) => {
                  const q = state.inputValue.toLowerCase();
                  return q
                    ? opts.filter((o) => o.id.toLowerCase().includes(q) || (o.label || '').toLowerCase().includes(q))
                    : opts;
                }}
                renderOption={(props, o) => (
                  <li {...props} key={o.id}>
                    <Identicon id={o.id} size={24} sx={{ mr: 1.5 }} />
                    <Box sx={{ minWidth: 0 }}>
                      {o.label && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{o.label}</Typography>}
                      <Box component='span' className='mono' sx={{ fontSize: '0.8rem', color: 'text.secondary' }}>
                        {o.id}
                      </Box>
                    </Box>
                  </li>
                )}
                renderInput={(params) => (
                  <TextField
                    {...params}
                    label='To'
                    placeholder='Recipient identity (60 letters)'
                    error={to.length > 0 && (!toValid || to === from)}
                    helperText={
                      to === from && to
                        ? 'That is the sending identity'
                        : to.length > 0 && !toValid
                        ? `${to.length}/60`
                        : nameOf(to)
                        ? nameOf(to)
                        : ' '
                    }
                    slotProps={{ htmlInput: { ...params.inputProps, className: 'mono' } }}
                  />
                )}
              />
              {toValid && !inBook && !ownIds.has(to) && saveLabel === null && (
                <Button size='small' startIcon={<BookmarkAddOutlinedIcon />} onClick={() => setSaveLabel('')} sx={{ mt: -0.5 }}>
                  Save to address book
                </Button>
              )}
              {saveLabel !== null && (
                <Stack direction='row' spacing={1} sx={{ mt: 0.5 }}>
                  <TextField
                    autoFocus
                    label='Name for this recipient'
                    value={saveLabel}
                    onChange={(e) => setSaveLabel(e.target.value.slice(0, 40))}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        saveToBook();
                      }
                    }}
                    sx={{ flex: 1 }}
                  />
                  <Button variant='outlined' disabled={!saveLabel.trim()} onClick={saveToBook}>
                    Save
                  </Button>
                  <Button onClick={() => setSaveLabel(null)}>Cancel</Button>
                </Stack>
              )}
            </Box>

            <TextField
              label='Amount'
              value={amount}
              onChange={(e) => digitsOnly(e.target.value) && setAmount(e.target.value)}
              error={overBalance}
              helperText={
                overBalance
                  ? `Only ${formatNumber(balance)} QU available`
                  : amount
                  ? [formatNumber(amount) + ' QU', usd(amount)].filter(Boolean).join(' · ')
                  : balance !== null
                  ? `${formatNumber(balance)} QU available`
                  : ' '
              }
              slotProps={{
                input: {
                  endAdornment: (
                    <InputAdornment position='end'>
                      <Typography variant='body2' color='text.secondary' sx={{ mr: 1 }}>
                        QU
                      </Typography>
                      {balance !== null && balance > 0 && (
                        <Button size='small' onClick={() => setAmount(String(balance))} sx={{ minWidth: 0 }}>
                          Max
                        </Button>
                      )}
                    </InputAdornment>
                  ),
                },
              }}
            />
          </Stack>
        </DialogContent>
      ) : (
        <DialogContent>
          <Row label='From'>
            <Box sx={{ display: 'flex', gap: 1.25, alignItems: 'center' }}>
              <Identicon id={from} size={28} />
              <Box sx={{ minWidth: 0 }}>
                {nameOf(from) && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{nameOf(from)}</Typography>}
                <IdText id={from} full nowrap copy={false} sx={idOnOneLine} />
              </Box>
            </Box>
          </Row>
          <Divider />
          <Row label='To'>
            <Box sx={{ display: 'flex', gap: 1.25, alignItems: 'center' }}>
              <Identicon id={to} size={28} />
              <Box sx={{ minWidth: 0 }}>
                {nameOf(to) && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{nameOf(to)}</Typography>}
                <IdText id={to} full nowrap copy={false} sx={idOnOneLine} />
              </Box>
            </Box>
          </Row>
          <Divider />
          <Row label='Amount'>
            <Typography variant='h6'>{formatNumber(amountNum)} QU</Typography>
            {usd(amountNum) && (
              <Typography variant='body2' color='text.secondary'>
                {usd(amountNum)}
              </Typography>
            )}
            {balance !== null && (
              <Typography variant='caption' color='text.secondary'>
                {formatNumber(balance - amountNum)} QU will remain
              </Typography>
            )}
          </Row>
          <Divider />
          {(warningsFor?.(from) || []).map((text) => (
            <Box
              key={text}
              sx={{ mt: 2, px: 1.5, py: 1, borderRadius: 1, border: 1, borderColor: 'warning.main', color: 'warning.main' }}
            >
              <Typography variant='body2'>{text}</Typography>
            </Box>
          ))}
          <Box sx={{ mt: 2 }}>
            <Chip size='small' color='warning' variant='outlined' label='Transfers cannot be reversed once included in a tick' />
          </Box>
        </DialogContent>
      )}

      <DialogActions
        sx={{ px: 3, pb: 2 }}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            if (step === 'form' && valid) setStep('review');
            else if (step === 'review') submit();
          }
        }}
      >
        <Button onClick={onClose}>Cancel</Button>
        {step === 'form' ? (
          <Button variant='contained' disabled={!valid} onClick={() => setStep('review')}>
            Review
          </Button>
        ) : (
          <Button variant='contained' startIcon={<SendIcon />} onClick={submit}>
            Send {formatNumber(amountNum)} QU
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
}
