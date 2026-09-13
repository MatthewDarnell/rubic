import React, { useEffect, useMemo, useState } from 'react';
import {
  Box,
  Button,
  Checkbox,
  Chip,
  Collapse,
  IconButton,
  InputAdornment,
  LinearProgress,
  List,
  ListItem,
  ListItemText,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
  Tooltip,
  Typography,
  CircularProgress,
} from '@mui/material';
import WifiIcon from '@mui/icons-material/Wifi';
import WifiOffIcon from '@mui/icons-material/WifiOff';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutline';
import DeleteSweepIcon from '@mui/icons-material/DeleteSweep';
import AddIcon from '@mui/icons-material/Add';
import RestoreIcon from '@mui/icons-material/Restore';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import SearchIcon from '@mui/icons-material/Search';
import BlockIcon from '@mui/icons-material/Block';
import { apiCall } from '../api';
import { useToast } from './ToastProvider';
import ConfirmDialog from './ConfirmDialog';
import { formatAbsoluteTime, formatRelativeTime } from '../utils/format';

const IPV4 =
  /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}$/;

const SECTION_MAX_HEIGHT = 360;

// The peer loop stores 9999 for "never measured".
const Latency = ({ ms }) => {
  const n = Number(ms);
  if (!Number.isFinite(n) || n <= 0 || n >= 9999) return <Box component='span' sx={{ color: 'text.disabled' }}>—</Box>;
  const color = n < 200 ? 'success.main' : n < 600 ? 'warning.main' : 'error.main';
  return (
    <Tooltip title={n < 200 ? 'Fast' : n < 600 ? 'Slow' : 'Very slow'}>
      <Box component='span' sx={{ color, fontVariantNumeric: 'tabular-nums' }}>
        {n} ms
      </Box>
    </Tooltip>
  );
};

const isRemoved = (p) => p.whitelisted === '-1';
const isConnected = (p) => p.connected === '1' || p.connected === 'true';
const plural = (n, word) => `${n} ${word}${n === 1 ? '' : 's'}`;

// A table of active peers with row selection. Scrolls inside `maxHeight` when set.
const PeerTable = ({
  rows,
  selected,
  onToggle,
  onToggleAll,
  locked,
  removingId,
  onRemove,
  emptyText,
  maxHeight,
  selectAllLabel,
}) => {
  const selectedCount = rows.filter((p) => selected.has(p.id)).length;
  const allSelected = rows.length > 0 && selectedCount === rows.length;
  const someSelected = selectedCount > 0 && !allSelected;
  return (
    <TableContainer sx={maxHeight ? { maxHeight, overflow: 'auto' } : undefined}>
      <Table size='small' stickyHeader={Boolean(maxHeight)}>
        <TableHead>
          <TableRow>
            <TableCell padding='checkbox'>
              <Tooltip title={allSelected ? 'Clear selection' : selectAllLabel}>
                <span>
                  <Checkbox
                    size='small'
                    checked={allSelected}
                    indeterminate={someSelected}
                    disabled={rows.length === 0 || locked}
                    onChange={() => onToggleAll(rows, allSelected)}
                    inputProps={{ 'aria-label': selectAllLabel }}
                  />
                </span>
              </Tooltip>
            </TableCell>
            <TableCell>Address</TableCell>
            <TableCell align='right'>Latency</TableCell>
            <TableCell>Last responded</TableCell>
            <TableCell align='right' sx={{ width: 64 }} />
          </TableRow>
        </TableHead>
        <TableBody>
          {rows.length === 0 && (
            <TableRow>
              <TableCell colSpan={5} align='center' sx={{ py: 3, color: 'text.secondary' }}>
                {emptyText}
              </TableCell>
            </TableRow>
          )}
          {rows.map((p) => {
            const busy = removingId === p.id;
            const checked = selected.has(p.id);
            return (
              <TableRow
                key={p.id}
                hover
                selected={checked}
                onClick={() => !locked && onToggle(p.id)}
                sx={{ opacity: busy ? 0.5 : 1, cursor: locked ? 'default' : 'pointer' }}
              >
                <TableCell padding='checkbox' onClick={(e) => e.stopPropagation()}>
                  <Checkbox
                    size='small'
                    checked={checked}
                    disabled={locked}
                    onChange={() => onToggle(p.id)}
                    inputProps={{ 'aria-label': `Select peer ${p.ip}` }}
                  />
                </TableCell>
                <TableCell sx={{ fontFamily: 'monospace' }}>{p.ip}</TableCell>
                <TableCell align='right'>
                  <Latency ms={p.ping} />
                </TableCell>
                <TableCell>
                  <Tooltip title={formatAbsoluteTime(p.last_responded)}>
                    <span>{formatRelativeTime(p.last_responded)}</span>
                  </Tooltip>
                </TableCell>
                <TableCell align='right' onClick={(e) => e.stopPropagation()}>
                  <Tooltip title='Remove peer'>
                    <span>
                      <IconButton
                        size='small'
                        aria-label={`Remove peer ${p.ip}`}
                        disabled={locked}
                        onClick={() => onRemove(p)}
                      >
                        <DeleteOutlineIcon fontSize='small' />
                      </IconButton>
                    </span>
                  </Tooltip>
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

// Card with a clickable header that expands to reveal its body.
const SectionCard = ({ icon, title, count, subtitle, expanded, onToggle, actions, children, sx }) => (
  <Paper variant='outlined' sx={{ mb: 2, overflow: 'hidden', ...sx }}>
    <Box
      role='button'
      tabIndex={0}
      aria-expanded={expanded}
      onClick={onToggle}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onToggle();
        }
      }}
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: 1.5,
        px: 2,
        py: 1.25,
        cursor: 'pointer',
        userSelect: 'none',
        '&:hover': { bgcolor: 'action.hover' },
      }}
    >
      {icon}
      <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
        {title}
      </Typography>
      <Chip size='small' label={count} />
      {subtitle && (
        <Typography variant='body2' color='text.secondary' sx={{ ml: 0.5 }}>
          {subtitle}
        </Typography>
      )}
      <Box sx={{ flex: 1 }} />
      <Box onClick={(e) => e.stopPropagation()} sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
        {actions}
      </Box>
      <IconButton
        size='small'
        aria-label={expanded ? 'Collapse' : 'Expand'}
        sx={{ transform: expanded ? 'rotate(180deg)' : 'none', transition: 'transform 150ms' }}
      >
        <ExpandMoreIcon />
      </IconButton>
    </Box>
    <Collapse in={expanded} unmountOnExit>
      <Box sx={{ borderTop: 1, borderColor: 'divider' }}>{children}</Box>
    </Collapse>
  </Paper>
);

export default function PeersPanel({ peers, onChanged }) {
  const toast = useToast();

  const [ip, setIp] = useState('');
  const [port, setPort] = useState('21841');
  const [adding, setAdding] = useState(false);
  const [confirmPeer, setConfirmPeer] = useState(null);
  const [removingId, setRemovingId] = useState(null);
  const [restoringIp, setRestoringIp] = useState(null);
  const [justRemoved, setJustRemoved] = useState(() => new Set());
  const [showDisconnected, setShowDisconnected] = useState(false);
  const [showRemoved, setShowRemoved] = useState(false);
  const [disconnectedFilter, setDisconnectedFilter] = useState('');
  const [selected, setSelected] = useState(() => new Set());
  const [bulkConfirm, setBulkConfirm] = useState(false);
  const [bulkProgress, setBulkProgress] = useState(null); // { done, total }

  useEffect(() => {
    if (justRemoved.size === 0) return;
    const stillActive = new Set(peers.filter((p) => !isRemoved(p)).map((p) => p.id));
    const next = new Set([...justRemoved].filter((id) => stillActive.has(id)));
    if (next.size !== justRemoved.size) setJustRemoved(next);
  }, [peers, justRemoved]);

  const { connected, disconnected, removed } = useMemo(() => {
    const byRecency = (a, b) => Number(b.last_responded || 0) - Number(a.last_responded || 0);
    const byLatency = (a, b) => {
      const la = Number(a.ping) > 0 && Number(a.ping) < 9999 ? Number(a.ping) : Infinity;
      const lb = Number(b.ping) > 0 && Number(b.ping) < 9999 ? Number(b.ping) : Infinity;
      return la - lb || byRecency(a, b);
    };
    const active = peers.filter((p) => !isRemoved(p) && !justRemoved.has(p.id));
    return {
      connected: active.filter(isConnected).sort(byLatency),
      disconnected: active.filter((p) => !isConnected(p)).sort(byRecency),
      removed: peers.filter(isRemoved).sort(byRecency),
    };
  }, [peers, justRemoved]);

  const active = useMemo(() => [...connected, ...disconnected], [connected, disconnected]);

  const filteredDisconnected = useMemo(() => {
    const q = disconnectedFilter.trim().toLowerCase();
    if (!q) return disconnected;
    return disconnected.filter((p) => p.ip.toLowerCase().includes(q));
  }, [disconnected, disconnectedFilter]);

  useEffect(() => {
    if (selected.size === 0) return;
    const activeIds = new Set(active.map((p) => p.id));
    const next = new Set([...selected].filter((id) => activeIds.has(id)));
    if (next.size !== selected.size) setSelected(next);
  }, [active, selected]);

  const selectedPeers = active.filter((p) => selected.has(p.id));
  const bulkBusy = bulkProgress !== null;
  const rowsLocked = removingId !== null || bulkBusy;

  const toggleOne = (id) =>
    setSelected((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const toggleRows = (rows, allSelected) =>
    setSelected((s) => {
      const next = new Set(s);
      rows.forEach((p) => (allSelected ? next.delete(p.id) : next.add(p.id)));
      return next;
    });

  const selectAllDisconnected = () => {
    setSelected((s) => new Set([...s, ...disconnected.map((p) => p.id)]));
    setShowDisconnected(true);
  };

  const removeSelected = async () => {
    const targets = selectedPeers;
    if (targets.length === 0) return;
    setBulkProgress({ done: 0, total: targets.length });
    const failed = [];
    try {
      for (const [index, peer] of targets.entries()) {
        const res = await apiCall(`peers/delete/${peer.id}`);
        if (res.success && res.data === 'Peer Deleted') {
          setJustRemoved((s) => new Set(s).add(peer.id));
        } else {
          failed.push(peer.ip);
        }
        setBulkProgress({ done: index + 1, total: targets.length });
      }
    } finally {
      setBulkProgress(null);
      setBulkConfirm(false);
      setSelected(new Set());
    }
    const removedCount = targets.length - failed.length;
    if (failed.length === 0) {
      toast.success(`Removed ${plural(removedCount, 'peer')}`);
    } else if (removedCount === 0) {
      toast.error(`Could not remove ${plural(failed.length, 'peer')}: ${failed.join(', ')}`);
    } else {
      toast.warning(`Removed ${removedCount}, but ${failed.length} failed: ${failed.join(', ')}`);
    }
    await onChanged?.();
  };

  const portNum = Number(port);
  const ipValid = IPV4.test(ip);
  const portValid = Number.isInteger(portNum) && portNum >= 1 && portNum <= 65535;
  const address = `${ip}:${port}`;
  const alreadyActive = active.some((p) => p.ip === address);

  const addPeer = async (addr) => {
    setAdding(true);
    try {
      const res = await apiCall(`peers/add/${addr}`);
      if (res.success && res.data && res.data !== 'Failed To Add Peer') {
        toast.success(`Added peer ${addr}`);
        setIp('');
        await onChanged?.();
        return true;
      }
      toast.error(`Could not add ${addr}: ${res.data || res.error || 'unknown error'}`);
      return false;
    } finally {
      setAdding(false);
    }
  };

  const restorePeer = async (peer) => {
    setRestoringIp(peer.ip);
    try {
      const res = await apiCall(`peers/add/${peer.ip}`);
      if (res.success && res.data && res.data !== 'Failed To Add Peer') {
        toast.success(`Restored peer ${peer.ip}`);
        await onChanged?.();
      } else {
        toast.error(`Could not restore ${peer.ip}: ${res.data || res.error || 'unknown error'}`);
      }
    } finally {
      setRestoringIp(null);
    }
  };

  const removePeer = async () => {
    const peer = confirmPeer;
    if (!peer) return;
    setRemovingId(peer.id);
    try {
      const res = await apiCall(`peers/delete/${peer.id}`);
      if (res.success && res.data === 'Peer Deleted') {
        setJustRemoved((s) => new Set(s).add(peer.id));
        toast.success(`Removed peer ${peer.ip}`);
        setConfirmPeer(null);
        await onChanged?.();
      } else {
        toast.error(`Could not remove ${peer.ip}: ${res.data || res.error || 'unknown error'}`);
      }
    } finally {
      setRemovingId(null);
    }
  };

  const tableProps = {
    selected,
    onToggle: toggleOne,
    onToggleAll: toggleRows,
    locked: rowsLocked,
    removingId,
    onRemove: setConfirmPeer,
  };

  return (
    <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      <Paper variant='outlined' sx={{ p: 2, mb: 2, flexShrink: 0 }}>
        <Typography variant='subtitle2' sx={{ mb: 1.5 }}>
          Add a peer
        </Typography>
        <Stack
          direction='row'
          spacing={1}
          alignItems='flex-start'
          component='form'
          onSubmit={(e) => {
            e.preventDefault();
            if (ipValid && portValid && !alreadyActive && !adding) addPeer(address);
          }}
        >
          <TextField
            label='IP address'
            size='small'
            value={ip}
            onChange={(e) => /^[\d.]*$/.test(e.target.value) && setIp(e.target.value)}
            error={ip.length > 0 && !ipValid}
            helperText={
              ip.length > 0 && !ipValid
                ? 'Enter a valid IPv4 address'
                : alreadyActive
                ? 'This peer is already in your list'
                : ' '
            }
            sx={{ width: 220 }}
            autoComplete='off'
          />
          <TextField
            label='Port'
            size='small'
            value={port}
            onChange={(e) => /^\d{0,5}$/.test(e.target.value) && setPort(e.target.value)}
            error={!portValid}
            helperText={portValid ? ' ' : '1–65535'}
            sx={{ width: 110 }}
            autoComplete='off'
          />
          <Button
            type='submit'
            variant='contained'
            startIcon={adding ? <CircularProgress size={16} color='inherit' /> : <AddIcon />}
            disabled={!ipValid || !portValid || alreadyActive || adding}
            sx={{ height: 40 }}
          >
            Add peer
          </Button>
        </Stack>
      </Paper>

      {selectedPeers.length > 0 && (
        <Paper
          variant='outlined'
          sx={{
            px: 2,
            py: 1,
            mb: 2,
            display: 'flex',
            alignItems: 'center',
            gap: 2,
            flexWrap: 'wrap',
            bgcolor: 'action.selected',
            flexShrink: 0,
          }}
        >
          <Typography variant='body2' sx={{ fontWeight: 600 }}>
            {selectedPeers.length} of {active.length} selected
          </Typography>
          <Button
            size='small'
            color='error'
            variant='contained'
            startIcon={<DeleteSweepIcon />}
            disabled={rowsLocked}
            onClick={() => setBulkConfirm(true)}
          >
            Remove selected
          </Button>
          <Button size='small' disabled={rowsLocked} onClick={() => setSelected(new Set())}>
            Clear selection
          </Button>
        </Paper>
      )}

      {/* Peer lists scroll here; the add form and selection bar above stay put. */}
      <Box sx={{ flex: 1, minHeight: 0, overflow: 'auto', pr: 0.5 }}>
      <Paper variant='outlined' sx={{ mb: 2, overflow: 'hidden' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, px: 2, py: 1.25 }}>
          <WifiIcon fontSize='small' color={connected.length > 0 ? 'success' : 'disabled'} />
          <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
            Connected
          </Typography>
          <Chip size='small' color={connected.length > 0 ? 'success' : 'default'} label={connected.length} />
        </Box>
        <Box sx={{ borderTop: 1, borderColor: 'divider' }}>
          <PeerTable
            {...tableProps}
            rows={connected}
            maxHeight={connected.length > 8 ? SECTION_MAX_HEIGHT : undefined}
            emptyText={
              active.length === 0
                ? 'No peers yet — add one above to start syncing.'
                : 'Not connected to any peer right now.'
            }
            selectAllLabel='Select all connected peers'
          />
        </Box>
      </Paper>

      <SectionCard
        icon={<WifiOffIcon fontSize='small' sx={{ color: 'text.disabled' }} />}
        title='Not connected'
        count={disconnected.length}
        subtitle={
          disconnected.length > 0 && !showDisconnected
            ? 'Known peers Rubic is not currently talking to'
            : undefined
        }
        expanded={showDisconnected}
        onToggle={() => setShowDisconnected((v) => !v)}
        actions={
          disconnected.length > 0 && (
            <>
              {showDisconnected && (
                <TextField
                  size='small'
                  placeholder='Filter  ( / )'
                  value={disconnectedFilter}
                  onChange={(e) => setDisconnectedFilter(e.target.value)}
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
                  sx={{ width: 200 }}
                />
              )}
              <Button
                size='small'
                startIcon={<DeleteSweepIcon />}
                disabled={rowsLocked}
                onClick={selectAllDisconnected}
              >
                Select all {disconnected.length}
              </Button>
            </>
          )
        }
      >
        <PeerTable
          {...tableProps}
          rows={filteredDisconnected}
          maxHeight={SECTION_MAX_HEIGHT}
          emptyText={
            disconnected.length === 0
              ? 'Every known peer is connected.'
              : 'No peers match the filter.'
          }
          selectAllLabel='Select all listed disconnected peers'
        />
      </SectionCard>

      {removed.length > 0 && (
        <SectionCard
          icon={<BlockIcon fontSize='small' sx={{ color: 'text.disabled' }} />}
          title='Removed'
          count={removed.length}
          subtitle={!showRemoved ? 'Rubic will not use these until you re-add them' : undefined}
          expanded={showRemoved}
          onToggle={() => setShowRemoved((v) => !v)}
        >
          <TableContainer sx={{ maxHeight: SECTION_MAX_HEIGHT, overflow: 'auto' }}>
            <Table size='small'>
              <TableBody>
                {removed.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>{p.ip}</TableCell>
                    <TableCell align='right'>
                      <Button
                        size='small'
                        startIcon={
                          restoringIp === p.ip ? (
                            <CircularProgress size={14} color='inherit' />
                          ) : (
                            <RestoreIcon />
                          )
                        }
                        disabled={restoringIp !== null}
                        onClick={() => restorePeer(p)}
                      >
                        Re-add
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </SectionCard>
      )}
      </Box>

      <ConfirmDialog
        open={bulkConfirm}
        title={`Remove ${plural(selectedPeers.length, 'peer')}?`}
        confirmLabel={bulkBusy ? `Removing ${bulkProgress.done}/${bulkProgress.total}…` : 'Remove all selected'}
        confirmColor='error'
        busy={bulkBusy}
        onConfirm={removeSelected}
        onCancel={() => setBulkConfirm(false)}
      >
        <Typography sx={{ mb: 1 }}>
          Rubic will disconnect from these peers and stop using them. You can
          bring them back later from the “removed” list.
        </Typography>
        <List dense sx={{ maxHeight: 240, overflow: 'auto', bgcolor: 'background.default', borderRadius: 1 }}>
          {selectedPeers.map((p) => (
            <ListItem key={p.id}>
              <ListItemText
                primary={p.ip}
                secondary={isConnected(p) ? 'connected' : 'not connected'}
                slotProps={{ primary: { sx: { fontFamily: 'monospace' } } }}
              />
            </ListItem>
          ))}
        </List>
        {bulkBusy && (
          <LinearProgress
            variant='determinate'
            value={(bulkProgress.done / bulkProgress.total) * 100}
            sx={{ mt: 2 }}
          />
        )}
      </ConfirmDialog>

      <ConfirmDialog
        open={Boolean(confirmPeer)}
        title='Remove this peer?'
        confirmLabel='Remove'
        confirmColor='error'
        busy={removingId !== null}
        onConfirm={removePeer}
        onCancel={() => setConfirmPeer(null)}
      >
        <Typography sx={{ mb: 1 }}>
          Rubic will disconnect from{' '}
          <Box component='span' sx={{ fontFamily: 'monospace', fontWeight: 600 }}>
            {confirmPeer?.ip}
          </Box>{' '}
          and stop using it.
        </Typography>
        <Typography variant='body2' color='text.secondary'>
          You can bring it back later from the “removed” list.
        </Typography>
      </ConfirmDialog>
    </Box>
  );
}
