import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  Box,
  Button,
  Chip,
  CircularProgress,
  Divider,
  IconButton,
  InputAdornment,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TableSortLabel,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';
import AddIcon from '@mui/icons-material/Add';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import SendIcon from '@mui/icons-material/Send';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutline';
import DriveFileRenameOutlineIcon from '@mui/icons-material/DriveFileRenameOutline';
import MoreVertIcon from '@mui/icons-material/MoreVert';
import SearchIcon from '@mui/icons-material/Search';
import WalletIcon from '@mui/icons-material/Wallet';
import { apiCall } from '../api';
import { useToast } from './ToastProvider';
import IdText from './IdText';
import Identicon from './Identicon';
import { Skeleton } from '@mui/material';
import { formatAge, formatFiat, formatNumber, isNumeric } from '../utils/format';

const seedRegex = /^[a-z]{55}$/;
const INVALID_SEED_ID = 'AARQXIKNFIEZZEMOAVNVSUINZXAAXYBZZXVSWYOYIETZVPVKJPARMKTEKLKJ';

// ---------- Add identity ----------

const AddIdentity = ({ allowNonEncrypted, onAction, seedInputRef }) => {
  const toast = useToast();
  const [seed, setSeed] = useState('');
  const [showSeed, setShowSeed] = useState(false);
  const [derived, setDerived] = useState(null); // { id } | { invalid: true } | null
  const [busy, setBusy] = useState(false);

  const seedValid = seedRegex.test(seed);

  useEffect(() => {
    if (!seedValid) {
      setDerived(null);
      return undefined;
    }
    let cancelled = false;
    const timer = setTimeout(async () => {
      const res = await apiCall(`identity/from_seed/${seed}`);
      if (cancelled) return;
      const ok = res.success && res.data && res.data !== INVALID_SEED_ID;
      setDerived(ok ? { id: res.data } : { invalid: true });
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [seed, seedValid]);

  const reset = () => {
    setSeed('');
    setDerived(null);
  };

  const plain = async (path, successMessage) => {
    setBusy(true);
    try {
      const res = await apiCall(path);
      if (res.success && String(res.data) === '200') {
        toast.success(successMessage);
        reset();
      } else toast.error(String(res.data || res.error || 'Request failed'));
    } finally {
      setBusy(false);
    }
  };

  const canImport = seedValid && derived?.id && !busy;

  return (
    <Stack direction='row' spacing={2} sx={{ mb: 3 }} flexWrap='wrap' useFlexGap>
      <Paper variant='outlined' sx={{ p: 2, flex: '1 1 300px', display: 'flex', flexDirection: 'column' }}>
        <Typography variant='subtitle1' sx={{ fontWeight: 600 }}>
          Create a new identity
        </Typography>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2, flex: 1 }}>
          Rubic generates a fresh seed and stores it encrypted with your master password.
        </Typography>
        <Stack direction='row' spacing={1} flexWrap='wrap' useFlexGap>
          <Button variant='contained' startIcon={<AddIcon />} disabled={busy} onClick={() => onAction('identity/new/')}>
            Create new identity
          </Button>
          {allowNonEncrypted && (
            <Button
              variant='outlined'
              color='warning'
              startIcon={<LockOpenIcon />}
              disabled={busy}
              onClick={() => plain('identity/new/0', 'Identity created — seed stored without encryption')}
            >
              Create without encryption
            </Button>
          )}
        </Stack>
      </Paper>

      <Paper variant='outlined' sx={{ p: 2, flex: '2 1 480px' }}>
        <Typography variant='subtitle1' sx={{ fontWeight: 600 }}>
          Import an existing seed
        </Typography>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
          Paste the 55-letter seed of an identity you already own.
        </Typography>
        <TextField
          inputRef={seedInputRef}
          label='Seed'
          size='small'
          fullWidth
          type={showSeed ? 'text' : 'password'}
          value={seed}
          onChange={(e) => setSeed(e.target.value.trim().toLowerCase())}
          error={(seed.length > 0 && !seedValid) || Boolean(derived?.invalid)}
          helperText={
            seed.length > 0 && !seedValid
              ? `55 lowercase letters a–z (${seed.length}/55)`
              : derived?.invalid
              ? 'This seed does not produce a valid identity'
              : ' '
          }
          autoComplete='off'
          slotProps={{
            input: {
              className: 'mono',
              endAdornment: (
                <InputAdornment position='end'>
                  <Tooltip title={showSeed ? 'Hide seed' : 'Show seed'}>
                    <IconButton onClick={() => setShowSeed((v) => !v)} edge='end' size='small'>
                      {showSeed ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </Tooltip>
                </InputAdornment>
              ),
            },
          }}
        />
        <Box sx={{ minHeight: 28, mb: 1.5 }}>
          {seedValid && !derived && (
            <Typography variant='body2' color='text.secondary' sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
              <CircularProgress size={12} /> Deriving identity…
            </Typography>
          )}
          {derived?.id && (
            <Typography variant='body2' sx={{ display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
              <Identicon id={derived.id} size={20} />
              <Box component='span' sx={{ color: 'text.secondary' }}>This seed belongs to</Box>
              <IdText id={derived.id} full copy={false} />
            </Typography>
          )}
        </Box>
        <Stack direction='row' spacing={1} flexWrap='wrap' useFlexGap>
          <Button
            variant='contained'
            startIcon={<FileUploadIcon />}
            disabled={!canImport}
            onClick={() => {
              onAction(`identity/add/${seed}/`);
              reset();
            }}
          >
            Import identity
          </Button>
          {allowNonEncrypted && (
            <Button
              variant='outlined'
              color='warning'
              startIcon={<LockOpenIcon />}
              disabled={!canImport}
              onClick={() => plain(`identity/add/${seed}`, 'Identity imported — seed stored without encryption')}
            >
              Import without encryption
            </Button>
          )}
        </Stack>
      </Paper>
    </Stack>
  );
};

// ---------- Balance cell ----------

const BalanceCell = ({ balance, price, currency, stale, staleReason, staleSince }) => {
  if (isNumeric(balance)) {
    const fiat = formatFiat(balance, price, currency, { approx: false });
    const cell = (
      <Box sx={{ textAlign: 'right', opacity: stale ? 0.6 : 1 }}>
        <Typography sx={{ fontWeight: 600, whiteSpace: 'nowrap' }}>{formatNumber(balance)} QU</Typography>
        <Typography variant='caption' color={stale ? 'warning.main' : 'text.secondary'}>
          {stale ? `last known${staleSince ? ` · ${formatAge(Date.now() - staleSince)} ago` : ''}` : fiat}
        </Typography>
      </Box>
    );
    return stale ? (
      <Tooltip
        title={
          staleReason === 'Peer Balance Mismatch'
            ? 'Peers currently disagree about this balance; showing the last value they agreed on.'
            : 'No peer has reported this balance recently; showing the last value they agreed on.'
        }
      >
        {cell}
      </Tooltip>
    ) : (
      cell
    );
  }
  if (balance === 'Not Yet Reported') {
    return (
      <Tooltip title='Waiting for connected peers to report this balance'>
        <Chip size='small' variant='outlined' icon={<CircularProgress size={12} sx={{ ml: 1 }} />} label='Syncing…' />
      </Tooltip>
    );
  }
  if (balance === 'Peer Balance Mismatch') {
    return (
      <Tooltip title='Connected peers reported different balances for this identity. It usually settles within a few ticks.'>
        <Chip size='small' variant='outlined' color='warning' label='Peers disagree' />
      </Tooltip>
    );
  }
  return <Typography sx={{ color: 'warning.main' }}>{String(balance)}</Typography>;
};

// ---------- Empty state ----------

const EmptyState = ({ onCreate, onImport }) => (
  <Paper variant='outlined' sx={{ p: 5, textAlign: 'center' }}>
    <WalletIcon sx={{ fontSize: 56, color: 'text.disabled', mb: 1 }} />
    <Typography variant='h6' sx={{ mb: 0.5 }}>
      No identities yet
    </Typography>
    <Typography variant='body2' color='text.secondary' sx={{ mb: 3, maxWidth: 440, mx: 'auto' }}>
      An identity is a Qubic address you control. Create a new one, or import a seed you already have.
    </Typography>
    <Stack direction='row' spacing={1.5} justifyContent='center'>
      <Button variant='contained' startIcon={<AddIcon />} onClick={onCreate}>
        Create new identity
      </Button>
      <Button variant='outlined' startIcon={<FileUploadIcon />} onClick={onImport}>
        Import a seed
      </Button>
    </Stack>
  </Paper>
);

// ---------- Main panel ----------

const COLUMNS = [
  { key: 'id', label: 'Identity' },
  { key: 'tx', label: 'Transfers', align: 'right' },
  { key: 'balance', label: 'Balance', align: 'right' },
];

const SkeletonRows = ({ rows = 3 }) => (
  <Paper variant='outlined' sx={{ p: 2 }}>
    {Array.from({ length: rows }).map((_, i) => (
      <Box key={i} sx={{ display: 'flex', alignItems: 'center', gap: 2, py: 1.25 }}>
        <Skeleton variant='rounded' width={32} height={32} />
        <Skeleton variant='text' sx={{ flex: 1, fontSize: '0.9rem' }} />
        <Skeleton variant='text' width={60} />
        <Skeleton variant='text' width={110} />
      </Box>
    ))}
  </Paper>
);

export default function WalletPanel({
  identities,
  transfers,
  allowNonEncrypted,
  price,
  currency,
  loading,
  labels,
  onAction,
  onSend,
  onRename,
  onOpenIdentity,
  onDeleteIdentity,
}) {
  const [filter, setFilter] = useState('');
  const [orderBy, setOrderBy] = useState(null);
  const [order, setOrder] = useState('desc');
  const [menu, setMenu] = useState(null); // { anchor, identity }
  const seedInputRef = useRef(null);

  const txCounts = useMemo(() => {
    const counts = new Map();
    transfers.forEach((t) => {
      counts.set(t.source, (counts.get(t.source) || 0) + 1);
      counts.set(t.destination, (counts.get(t.destination) || 0) + 1);
    });
    return counts;
  }, [transfers]);

  const rows = useMemo(() => {
    const q = filter.trim().toLowerCase();
    let list = q
      ? identities.filter((i) => i.id.toLowerCase().includes(q) || (labels[i.id] || '').toLowerCase().includes(q))
      : [...identities];
    if (orderBy) {
      const dir = order === 'asc' ? 1 : -1;
      const value = (i) =>
        orderBy === 'id'
          ? (labels[i.id] || i.id).toLowerCase()
          : orderBy === 'tx'
          ? txCounts.get(i.id) || 0
          : isNumeric(i.balance)
          ? Number(i.balance)
          : -1;
      list.sort((a, b) => {
        const va = value(a);
        const vb = value(b);
        return (va < vb ? -1 : va > vb ? 1 : 0) * dir;
      });
    }
    return list;
  }, [identities, filter, orderBy, order, txCounts, labels]);

  const totals = useMemo(() => {
    let sum = 0;
    let syncing = 0;
    let disagree = 0;
    let stale = 0;
    identities.forEach((i) => {
      if (isNumeric(i.balance)) {
        sum += Number(i.balance);
        if (i.stale) stale += 1;
      } else if (i.balance === 'Peer Balance Mismatch') disagree += 1;
      else syncing += 1;
    });
    return { sum, syncing, disagree, stale };
  }, [identities]);

  const toggleSort = (key) => {
    if (orderBy === key) setOrder((o) => (o === 'asc' ? 'desc' : 'asc'));
    else {
      setOrderBy(key);
      setOrder(key === 'id' ? 'asc' : 'desc');
    }
  };

  const focusImport = () => {
    seedInputRef.current?.focus();
    seedInputRef.current?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  };

  const summaryParts = [
    rows.length === identities.length
      ? `${identities.length} identit${identities.length === 1 ? 'y' : 'ies'}`
      : `${rows.length} of ${identities.length} identities`,
  ];
  if (totals.syncing > 0) summaryParts.push(`${totals.syncing} syncing`);
  if (totals.disagree > 0) summaryParts.push(`${totals.disagree} with peer disagreement`);
  if (totals.stale > 0) summaryParts.push(`${totals.stale} showing last known balance`);

  return (
    <Box>
      <AddIdentity allowNonEncrypted={allowNonEncrypted} onAction={onAction} seedInputRef={seedInputRef} />

      {loading ? (
        <SkeletonRows />
      ) : identities.length === 0 ? (
        <EmptyState onCreate={() => onAction('identity/new/')} onImport={focusImport} />
      ) : (
        <>
          <Stack direction='row' spacing={2} alignItems='center' sx={{ mb: 1.5 }}>
            <TextField
              size='small'
              placeholder='Filter by ID or nickname  ( / )'
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              slotProps={{
                htmlInput: { 'data-filter-input': true },
                input: {
                  startAdornment: (
                    <InputAdornment position='start'>
                      <SearchIcon fontSize='small' />
                    </InputAdornment>
                  ),
                },
              }}
              sx={{ width: 320 }}
            />
            <Typography variant='body2' color='text.secondary'>
              {summaryParts.join(' · ')}
            </Typography>
          </Stack>

          <TableContainer component={Paper} variant='outlined'>
            <Table size='small'>
              <TableHead>
                <TableRow>
                  <TableCell sx={{ width: 56 }} />
                  {COLUMNS.map((col) => (
                    <TableCell key={col.key} align={col.align} sortDirection={orderBy === col.key ? order : false}>
                      <TableSortLabel
                        active={orderBy === col.key}
                        direction={orderBy === col.key ? order : 'asc'}
                        onClick={() => toggleSort(col.key)}
                      >
                        {col.label}
                      </TableSortLabel>
                    </TableCell>
                  ))}
                  <TableCell align='right' sx={{ width: 100 }} />
                </TableRow>
              </TableHead>
              <TableBody>
                {rows.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} align='center' sx={{ py: 4, color: 'text.secondary' }}>
                      No identities match the filter.
                    </TableCell>
                  </TableRow>
                )}
                {rows.map((item) => {
                  const count = txCounts.get(item.id) || 0;
                  const label = labels[item.id];
                  return (
                    <TableRow
                      key={item.id}
                      hover
                      onClick={() => onOpenIdentity(item.id)}
                      sx={{ cursor: 'pointer', '& td': { py: 1 } }}
                    >
                      <TableCell>
                        <Box sx={{ position: 'relative', width: 32, height: 32 }}>
                          <Identicon id={item.id} size={32} />
                          <Tooltip title={item.encrypted === 'true' ? 'Seed encrypted' : 'Seed stored unencrypted'}>
                            <Box
                              sx={{
                                position: 'absolute',
                                right: -6,
                                bottom: -6,
                                width: 16,
                                height: 16,
                                borderRadius: '50%',
                                bgcolor: 'background.paper',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                              }}
                            >
                              {item.encrypted === 'true' ? (
                                <LockIcon sx={{ fontSize: 11, color: 'text.secondary' }} />
                              ) : (
                                <LockOpenIcon sx={{ fontSize: 11, color: 'warning.main' }} />
                              )}
                            </Box>
                          </Tooltip>
                        </Box>
                      </TableCell>
                      <TableCell onClick={(e) => e.stopPropagation()} sx={{ cursor: 'default' }}>
                        {label && (
                          <Typography
                            sx={{ fontWeight: 600, lineHeight: 1.2, cursor: 'pointer' }}
                            onClick={() => onOpenIdentity(item.id)}
                          >
                            {label}
                          </Typography>
                        )}
                        <IdText
                          id={item.id}
                          full
                          explorer='address'
                          sx={label ? { fontSize: '0.78rem', color: 'text.secondary' } : undefined}
                        />
                      </TableCell>
                      <TableCell align='right' sx={{ color: count ? 'text.primary' : 'text.disabled' }}>
                        {count}
                      </TableCell>
                      <TableCell align='right'>
                        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                          <BalanceCell balance={item.balance} price={price} currency={currency} stale={item.stale} staleReason={item.staleReason} staleSince={item.staleSince} />
                        </Box>
                      </TableCell>
                      <TableCell align='right' sx={{ whiteSpace: 'nowrap' }} onClick={(e) => e.stopPropagation()}>
                        <Tooltip title='Send from this identity'>
                          <IconButton size='small' onClick={() => onSend(item.id)}>
                            <SendIcon fontSize='small' />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title='More'>
                          <IconButton
                            size='small'
                            aria-label={`More actions for ${label || item.id}`}
                            onClick={(e) => setMenu({ anchor: e.currentTarget, identity: item })}
                          >
                            <MoreVertIcon fontSize='small' />
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
              <TableBody>
                <TableRow>
                  <TableCell colSpan={3} sx={{ fontWeight: 600, borderBottom: 0 }}>
                    Total
                  </TableCell>
                  <TableCell align='right' sx={{ borderBottom: 0 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                      <BalanceCell balance={String(totals.sum)} price={price} currency={currency} />
                    </Box>
                  </TableCell>
                  <TableCell sx={{ borderBottom: 0 }} />
                </TableRow>
              </TableBody>
            </Table>
          </TableContainer>
        </>
      )}

      <Menu
        anchorEl={menu?.anchor}
        open={Boolean(menu)}
        onClose={() => setMenu(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
      >
        <MenuItem
          onClick={() => {
            const id = menu.identity.id;
            setMenu(null);
            onRename(id);
          }}
        >
          <ListItemIcon><DriveFileRenameOutlineIcon fontSize='small' /></ListItemIcon>
          <ListItemText>{labels[menu?.identity.id] ? 'Edit nickname…' : 'Add nickname…'}</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem
          onClick={() => {
            const id = menu.identity.id;
            setMenu(null);
            onDeleteIdentity(id);
          }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon><DeleteOutlineIcon fontSize='small' color='error' /></ListItemIcon>
          <ListItemText>Delete identity…</ListItemText>
        </MenuItem>
      </Menu>
    </Box>
  );
}
