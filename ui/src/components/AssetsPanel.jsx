import React, { useEffect, useMemo, useState } from 'react';
import {
  Autocomplete,
  Box,
  Button,
  Chip,
  FormControl,
  InputAdornment,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import SendIcon from '@mui/icons-material/Send';
import IdText from './IdText';
import Identicon from './Identicon';
import { digitsOnly, formatNumber, shortenId, upperOnly } from '../utils/format';

const holdingOf = (identity, asset) => Number(identity?.assets?.find((a) => a?.name === asset)?.balance ?? 0);

const Step = ({ n, title, children }) => (
  <Box sx={{ display: 'flex', gap: 2 }}>
    <Box
      sx={{
        width: 26,
        height: 26,
        borderRadius: '50%',
        bgcolor: 'action.selected',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '0.8rem',
        fontWeight: 700,
        flexShrink: 0,
        mt: 0.25,
      }}
    >
      {n}
    </Box>
    <Box sx={{ flex: 1, minWidth: 0 }}>
      <Typography variant='subtitle2' sx={{ mb: 1.25 }}>
        {title}
      </Typography>
      {children}
    </Box>
  </Box>
);

export default function AssetsPanel({
  identities,
  assetsNIssuer,
  selectedAsset,
  onSelectAsset,
  labels,
  addressBook = [],
  onAction,
  requestConfirm,
  warningsFor,
}) {
  const [source, setSource] = useState('');
  const [destination, setDestination] = useState('');
  const [amount, setAmount] = useState('');

  const assets = useMemo(() => [...assetsNIssuer.keys()].sort(), [assetsNIssuer]);

  // Holdings per asset across all identities, derived from the polled identity data.
  const totals = useMemo(() => {
    const map = new Map();
    assets.forEach((a) => {
      let sum = 0;
      let holders = 0;
      identities.forEach((i) => {
        const h = holdingOf(i, a);
        if (h > 0) {
          sum += h;
          holders += 1;
        }
      });
      map.set(a, { sum, holders });
    });
    return map;
  }, [assets, identities]);

  const holders = useMemo(
    () => identities.filter((i) => holdingOf(i, selectedAsset) > 0),
    [identities, selectedAsset]
  );

  // Identities re-poll every few seconds; key on stable values so typing is never wiped.
  const holderIds = holders.map((i) => i.id).join(',');
  useEffect(() => {
    setAmount('');
  }, [selectedAsset]);
  useEffect(() => {
    if (!holderIds.split(',').includes(source)) setSource(holderIds.split(',')[0] || '');
  }, [holderIds, source]);

  const sourceIdentity = identities.find((i) => i.id === source);
  const available = holdingOf(sourceIdentity, selectedAsset);
  const amountNum = Number(amount);
  const dest = destination.trim();
  const destValid = /^[A-Z]{60}$/.test(dest) && dest !== source;
  const valid = selectedAsset && source && destValid && amountNum > 0 && amountNum <= available;
  const issuer =
    assetsNIssuer.get(selectedAsset) ||
    sourceIdentity?.assets?.find((a) => a?.name === selectedAsset)?.issuer;
  const nameOf = (id) => labels?.[id] || addressBook.find((e) => e.id === id)?.label || null;

  const recipientOptions = useMemo(() => {
    const seen = new Set();
    const out = [];
    const push = (id, label, group) => {
      if (!id || seen.has(id) || id === source) return;
      seen.add(id);
      out.push({ id, label, group });
    };
    addressBook.forEach((e) => push(e.id, e.label, 'Address book'));
    identities.forEach((i) => push(i.id, labels?.[i.id] || null, 'My identities'));
    return out;
  }, [addressBook, identities, labels, source]);

  const send = () =>
    requestConfirm({
      title: 'Send this asset transfer?',
      confirmLabel: `Send ${formatNumber(amountNum)} ${selectedAsset}`,
      body: (
        <>
          <Typography variant='body2' color='text.secondary'>From</Typography>
          {nameOf(source) && <Typography sx={{ fontWeight: 600 }}>{nameOf(source)}</Typography>}
          <IdText id={source} full copy={false} />
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>To</Typography>
          {nameOf(dest) && <Typography sx={{ fontWeight: 600 }}>{nameOf(dest)}</Typography>}
          <IdText id={dest} full copy={false} />
          {(warningsFor?.(source) || []).map((text) => (
            <Typography key={text} variant='body2' sx={{ mt: 1.5, color: 'warning.main' }}>
              {text}
            </Typography>
          ))}
          <Typography variant='body2' color='text.secondary' sx={{ mt: 2 }}>
            Asset transfers cannot be reversed once included in a tick.
          </Typography>
        </>
      ),
      onConfirm: () =>
        onAction(`asset/transfer/${selectedAsset}/${issuer}/${source}/${dest}/${amountNum}/`),
    });

  const selectedTotals = totals.get(selectedAsset);

  return (
    <Paper variant='outlined' sx={{ p: 3, maxWidth: 820 }}>
      <Stack spacing={3.5}>
        <Step n={1} title='Choose an asset'>
          {assets.length === 0 ? (
            <Typography variant='body2' color='text.secondary'>
              No issued assets are known yet — they appear once peers report them.
            </Typography>
          ) : (
            <Stack direction='row' spacing={2} useFlexGap sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
              <FormControl size='small' sx={{ minWidth: 300 }}>
                <InputLabel id='asset-select-label'>Asset</InputLabel>
                <Select
                  labelId='asset-select-label'
                  label='Asset'
                  value={selectedAsset || ''}
                  onChange={(e) => onSelectAsset(e.target.value)}
                  renderValue={(a) => (
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', gap: 2 }}>
                      <span>{a}</span>
                      <Box component='span' sx={{ color: 'text.secondary' }}>
                        {formatNumber(totals.get(a)?.sum ?? 0)}
                      </Box>
                    </Box>
                  )}
                >
                  {assets.map((a) => {
                    const t = totals.get(a);
                    return (
                      <MenuItem key={a} value={a}>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between', width: '100%', gap: 3 }}>
                          <Typography sx={{ fontWeight: 600 }}>{a}</Typography>
                          <Typography sx={{ color: t?.sum ? 'text.primary' : 'text.disabled' }}>
                            {formatNumber(t?.sum ?? 0)}
                            {t?.holders ? (
                              <Box component='span' sx={{ color: 'text.secondary', ml: 1 }}>
                                · {t.holders} identit{t.holders === 1 ? 'y' : 'ies'}
                              </Box>
                            ) : null}
                          </Typography>
                        </Box>
                      </MenuItem>
                    );
                  })}
                </Select>
              </FormControl>
              {selectedAsset && (
                <Box>
                  <Typography variant='body2'>
                    You hold <b>{formatNumber(selectedTotals?.sum ?? 0)} {selectedAsset}</b>
                    {selectedTotals?.holders
                      ? ` across ${selectedTotals.holders} identit${selectedTotals.holders === 1 ? 'y' : 'ies'}`
                      : ''}
                  </Typography>
                  {issuer && (
                    <Typography variant='caption' color='text.secondary' sx={{ display: 'flex', gap: 0.5, alignItems: 'center' }}>
                      Issuer <IdText id={issuer} head={6} tail={6} explorer='address' />
                    </Typography>
                  )}
                </Box>
              )}
            </Stack>
          )}
        </Step>

        <Step n={2} title='Fill in the transfer'>
          <Stack spacing={2}>
            <FormControl size='small' fullWidth disabled={!selectedAsset || holders.length === 0}>
              <InputLabel id='asset-source-label'>From</InputLabel>
              <Select
                labelId='asset-source-label'
                label='From'
                value={holders.some((i) => i.id === source) ? source : ''}
                onChange={(e) => setSource(e.target.value)}
                renderValue={(id) => (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <Identicon id={id} size={20} />
                    {nameOf(id) && <Typography sx={{ fontWeight: 600 }}>{nameOf(id)}</Typography>}
                    <Box component='span' className='mono' sx={{ color: 'text.secondary' }}>
                      {shortenId(id, 10, 10)}
                    </Box>
                  </Box>
                )}
              >
                {holders.map((i) => (
                  <MenuItem key={i.id} value={i.id}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', width: '100%', gap: 2 }}>
                      <Identicon id={i.id} size={28} />
                      <Box sx={{ minWidth: 0, flex: 1 }}>
                        {labels?.[i.id] && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{labels[i.id]}</Typography>}
                        <Box component='span' className='mono' sx={{ fontSize: '0.8rem', color: 'text.secondary' }}>
                          {shortenId(i.id, 12, 12)}
                        </Box>
                      </Box>
                      <Typography sx={{ whiteSpace: 'nowrap' }}>
                        {formatNumber(holdingOf(i, selectedAsset))} {selectedAsset}
                      </Typography>
                    </Box>
                  </MenuItem>
                ))}
              </Select>
              {selectedAsset && holders.length === 0 && (
                <Typography variant='caption' color='text.secondary' sx={{ mt: 0.5 }}>
                  None of your identities hold {selectedAsset}.
                </Typography>
              )}
            </FormControl>

            <Autocomplete
              freeSolo
              disabled={!source}
              options={recipientOptions}
              groupBy={(o) => o.group}
              getOptionLabel={(o) => (typeof o === 'string' ? o : o.id)}
              inputValue={destination}
              onInputChange={(_e, value) => {
                const v = value.toUpperCase();
                if (upperOnly(v)) setDestination(v.slice(0, 60));
              }}
              onChange={(_e, value) => {
                if (value && typeof value === 'object') setDestination(value.id);
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
                  size='small'
                  label='To'
                  placeholder='Recipient identity (60 letters)'
                  error={destination.length > 0 && !destValid}
                  helperText={
                    dest === source && dest
                      ? 'That is the sending identity'
                      : destination.length > 0 && !destValid
                      ? `${destination.length}/60`
                      : nameOf(dest) || ' '
                  }
                  slotProps={{ ...params.slotProps, htmlInput: { ...params.slotProps?.htmlInput, className: 'mono' } }}
                />
              )}
            />

            <TextField
              label='Shares'
              size='small'
              disabled={!source}
              value={amount}
              onChange={(e) => digitsOnly(e.target.value) && setAmount(e.target.value)}
              error={amount !== '' && amountNum > available}
              helperText={
                !source
                  ? ' '
                  : amountNum > available
                  ? `Only ${formatNumber(available)} ${selectedAsset} available`
                  : `${formatNumber(available)} ${selectedAsset} available`
              }
              sx={{ maxWidth: 320 }}
              slotProps={{
                input: {
                  endAdornment: (
                    <InputAdornment position='end'>
                      <Typography variant='body2' color='text.secondary' sx={{ mr: 1 }}>
                        {selectedAsset || ''}
                      </Typography>
                      {available > 0 && (
                        <Button size='small' onClick={() => setAmount(String(available))} sx={{ minWidth: 0 }}>
                          Max
                        </Button>
                      )}
                    </InputAdornment>
                  ),
                },
              }}
            />
          </Stack>
        </Step>

        <Step n={3} title='Send'>
          <Stack direction='row' spacing={2} sx={{ alignItems: 'center' }}>
            <Button variant='contained' startIcon={<SendIcon />} disabled={!valid} onClick={send}>
              Send {amount ? formatNumber(amount) : ''} {selectedAsset || 'asset'}
            </Button>
            {valid && (
              <Chip size='small' variant='outlined' label='You will review the transfer before it is signed' />
            )}
          </Stack>
        </Step>
      </Stack>
    </Paper>
  );
}
