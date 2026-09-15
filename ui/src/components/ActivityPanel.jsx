import React, { useEffect, useMemo, useState } from 'react';
import {
  Box,
  Button,
  Chip,
  FormControl,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Skeleton,
  Stack,
  Tooltip,
  Typography,
} from '@mui/material';
import DownloadIcon from '@mui/icons-material/Download';
import ArrowUpwardIcon from '@mui/icons-material/ArrowUpward';
import ArrowDownwardIcon from '@mui/icons-material/ArrowDownward';
import TokenIcon from '@mui/icons-material/TokenOutlined';
import SwapHorizIcon from '@mui/icons-material/SwapHoriz';
import {
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  TextField,
} from '@mui/material';
import ReplayIcon from '@mui/icons-material/Replay';
import NotesIcon from '@mui/icons-material/NotesOutlined';
import TimeFilterChips from './TimeFilterChips';
import IdText from './IdText';
import Identicon from './Identicon';
import TxStatus, { canRetry, txState, TxStatusChip, UNCONFIRMED_HINT } from './TxStatus';
import { useStoredValue, KEYS } from '../utils/localStore';
import { ALL_TIME, formatNumber, isNumeric, parseCreated, shortenId, withinLastMinutes } from '../utils/format';

const HIDDEN_DESTINATION = 'BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID';
const QX_SIDES = {
  5: { label: 'Sell', side: 'ASK' },
  6: { label: 'Buy', side: 'BID' },
  7: { label: 'Cancel sell', side: 'REMOVEASK' },
  8: { label: 'Cancel buy', side: 'REMOVEBID' },
};

const KINDS = [
  { key: 'all', label: 'All' },
  { key: 'transfer', label: 'Transfers' },
  { key: 'asset', label: 'Assets' },
  { key: 'qx', label: 'QX' },
];
const STATES = [
  { key: 'all', label: 'Any status' },
  { key: 'pending', label: 'Pending' },
  { key: 'confirmed', label: 'Confirmed' },
  { key: 'unconfirmed', label: 'Unconfirmed' },
  { key: 'failed', label: 'Failed' },
];

const relative = (created) => {
  const diff = Math.max(0, Date.now() - parseCreated(created).getTime());
  const s = Math.floor(diff / 1000);
  if (s < 60) return 'just now';
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} min ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} h ago`;
  const d = Math.floor(h / 24);
  return `${d} d ago`;
};

// Group by the user's local calendar day (timestamps are stored in UTC, so a
// UTC day boundary would put an evening transaction under "tomorrow").
const localDayKey = (date) =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
const dayKey = (created) => localDayKey(parseCreated(created));
const dayLabel = (key) => {
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (key === localDayKey(now)) return 'Today';
  if (key === localDayKey(yesterday)) return 'Yesterday';
  const [y, m, d] = key.split('-').map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { weekday: 'short', year: 'numeric', month: 'short', day: 'numeric' });
};
const localTime = (created) => parseCreated(created).toLocaleString();

export const normalizeActivity = ({ transfers, assetTransfers, qxOrders, ownIds, assetsNIssuer }) => {
  const taken = new Set([...assetTransfers, ...qxOrders].map((t) => t.txid).filter(Boolean));
  const items = [];

  qxOrders.forEach((t) => {
    const side = QX_SIDES[Number(t.input_type)];
    items.push({
      key: `qx-${t.txid || t.created}`,
      kind: 'qx',
      status: t.status,
      created: t.created,
      tick: t.tick,
      txid: t.txid,
      from: t.source,
      title: `${side?.label ?? `QX ${t.input_type}`} ${formatNumber(t.num_shares)} ${t.name} @ ${formatNumber(t.price)} QU`,
      amountText: `${formatNumber(Number(t.price) * Number(t.num_shares))} QU`,
      counterparty: null,
      retry: side
        ? {
            path: `qx/order/<tick>/${t.issuer}/${t.name}/${side.side}/${t.source}/${t.price}/${t.num_shares}/`,
            label: `Resubmit ${side.label.toLowerCase()} order`,
          }
        : null,
    });
  });

  assetTransfers.forEach((t) => {
    const outgoing = ownIds.has(t.source);
    const issuer = t.issuer || assetsNIssuer.get(t.name);
    items.push({
      key: `asset-${t.txid || t.created}`,
      kind: 'asset',
      direction: outgoing ? 'out' : 'in',
      status: t.status,
      created: t.created,
      tick: t.tick,
      txid: t.txid,
      from: t.source,
      to: t.new_owner_and_possessor,
      title: `${outgoing ? 'Sent' : 'Received'} ${formatNumber(t.num_shares)} ${t.name}`,
      amountText: `${outgoing ? '−' : '+'}${formatNumber(t.num_shares)} ${t.name}`,
      counterparty: outgoing ? t.new_owner_and_possessor : t.source,
      retry: outgoing
        ? {
            path: `asset/transfer/${t.name}/${issuer}/${t.source}/${t.new_owner_and_possessor}/${t.num_shares}/`,
            label: 'Resend asset transfer',
          }
        : null,
    });
  });

  transfers
    .filter((t) => t.destination !== HIDDEN_DESTINATION && !(t.txid && taken.has(t.txid)))
    .forEach((t) => {
      const outgoing = ownIds.has(t.source);
      items.push({
        key: `tx-${t.txid || `${t.source}-${t.tick}-${t.created}`}`,
        kind: 'transfer',
        direction: outgoing ? 'out' : 'in',
        status: t.status,
        created: t.created,
        tick: t.tick,
        txid: t.txid,
        from: t.source,
        to: t.destination,
        title: `${outgoing ? 'Sent' : 'Received'} ${formatNumber(t.amount)} QU`,
        amountText: `${outgoing ? '−' : '+'}${formatNumber(t.amount)} QU`,
        counterparty: outgoing ? t.destination : t.source,
        retry: outgoing
          ? { path: `transfer/${t.source}/${t.destination}/${t.amount}/`, label: 'Resend transfer' }
          : null,
      });
    });

  return items.sort((a, b) => parseCreated(b.created) - parseCreated(a.created));
};

const KindIcon = ({ item }) => {
  if (item.kind === 'qx') return <SwapHorizIcon fontSize='small' />;
  if (item.kind === 'asset') return <TokenIcon fontSize='small' />;
  return item.direction === 'out' ? (
    <ArrowUpwardIcon fontSize='small' color='error' />
  ) : (
    <ArrowDownwardIcon fontSize='small' color='success' />
  );
};

const Field = ({ label, children }) => (
  <Box sx={{ display: 'flex', gap: 2, py: 1 }}>
    <Typography variant='body2' color='text.secondary' sx={{ width: 84, flexShrink: 0, pt: 0.25 }}>
      {label}
    </Typography>
    <Box sx={{ minWidth: 0, flex: 1 }}>{children}</Box>
  </Box>
);

const Party = ({ id, labels }) =>
  id ? (
    <Box sx={{ display: 'flex', gap: 1.25, alignItems: 'center' }}>
      <Identicon id={id} size={26} />
      <Box sx={{ minWidth: 0 }}>
        {labels[id] && <Typography sx={{ fontWeight: 600, lineHeight: 1.2 }}>{labels[id]}</Typography>}
        <IdText id={id} full nowrap explorer='address' sx={{ fontSize: '0.72rem' }} />
      </Box>
    </Box>
  ) : (
    <Typography color='text.secondary'>—</Typography>
  );

const TxDetailDialog = ({ item, labels, latestTick, note, onSaveNote, onRetry, onClose }) => {
  const [draft, setDraft] = useState(note || '');
  useEffect(() => {
    setDraft(note || '');
  }, [note, item?.key]);
  if (!item) return null;
  const state = txState(item.status, item.tick, latestTick);
  const dirty = draft.trim() !== (note || '');
  return (
    <Dialog open onClose={onClose} maxWidth='sm' fullWidth>
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
        <Box sx={{ width: 32, height: 32, borderRadius: '50%', bgcolor: 'action.hover', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'text.secondary' }}>
          <KindIcon item={item} />
        </Box>
        <Box sx={{ flex: 1, minWidth: 0 }}>
          <Typography variant='h6' sx={{ lineHeight: 1.2 }}>{item.title}</Typography>
          <Typography variant='caption' color='text.secondary'>{localTime(item.created)}</Typography>
        </Box>
        <TxStatusChip status={item.status} tick={item.tick} latestTick={latestTick} />
      </DialogTitle>
      <DialogContent>
        <Field label='Amount'>
          <Typography sx={{ fontWeight: 600 }}>{item.amountText}</Typography>
        </Field>
        <Divider />
        <Field label='From'><Party id={item.from} labels={labels} /></Field>
        {item.to && (
          <>
            <Divider />
            <Field label='To'><Party id={item.to} labels={labels} /></Field>
          </>
        )}
        <Divider />
        <Field label='Transaction'>
          {item.txid ? <IdText id={item.txid} full nowrap explorer='tx' sx={{ fontSize: '0.72rem' }} /> : <Typography color='text.secondary'>—</Typography>}
        </Field>
        <Divider />
        <Field label='Tick'>
          {isNumeric(item.tick) ? (
            <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
              <IdText id={String(item.tick)} full copy={false} explorer='tick' />
              {state === 'pending' && isNumeric(latestTick) && (
                <Typography variant='caption' sx={{ color: 'warning.main' }}>
                  {Math.max(0, Number(item.tick) - Number(latestTick))} ticks to go
                </Typography>
              )}
              {state === 'unconfirmed' && (
                <Typography variant='caption' sx={{ color: 'warning.main' }}>{UNCONFIRMED_HINT}</Typography>
              )}
            </Box>
          ) : (
            <Typography color='text.secondary'>—</Typography>
          )}
        </Field>
        <Divider />
        <Box sx={{ pt: 2 }}>
          <TextField
            fullWidth
            multiline
            minRows={2}
            size='small'
            label='Note'
            placeholder='Private note for your own records — stored on this machine only'
            value={draft}
            onChange={(e) => setDraft(e.target.value.slice(0, 500))}
          />
        </Box>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2 }}>
        {canRetry(state) && item.retry && (
          <Button color='warning' startIcon={<ReplayIcon />} onClick={() => { onClose(); onRetry(item); }} sx={{ mr: 'auto' }}>
            {item.retry.label}
          </Button>
        )}
        <Button onClick={onClose}>Close</Button>
        <Button variant='contained' disabled={!dirty} onClick={() => onSaveNote(item, draft.trim())}>
          Save note
        </Button>
      </DialogActions>
    </Dialog>
  );
};

const ActivityRow = ({ item, labels, latestTick, note, onRetry, onOpen }) => {
  const state = txState(item.status, item.tick, latestTick);
  const ticksLeft =
    state === 'pending' && isNumeric(latestTick) && isNumeric(item.tick)
      ? Number(item.tick) - Number(latestTick)
      : null;
  const name = (id) => labels[id];
  return (
    <Box
      onClick={() => onOpen(item)}
      sx={{
        display: 'grid',
        gridTemplateColumns: '36px 1fr auto',
        alignItems: 'center',
        gap: 2,
        px: 2,
        py: 1.25,
        borderBottom: 1,
        borderColor: 'divider',
        cursor: 'pointer',
        '&:hover': { bgcolor: 'action.hover' },
        '&:last-of-type': { borderBottom: 0 },
      }}
    >
      <Box
        sx={{
          width: 36,
          height: 36,
          borderRadius: '50%',
          bgcolor: 'action.hover',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'text.secondary',
        }}
      >
        <KindIcon item={item} />
      </Box>
      <Box sx={{ minWidth: 0 }}>
        <Typography sx={{ fontWeight: 600, lineHeight: 1.3 }}>
          {item.title}
          {item.counterparty && (
            <Box component='span' sx={{ fontWeight: 400, color: 'text.secondary' }}>
              {' '}
              {item.direction === 'out' ? 'to' : 'from'}{' '}
              <Identicon id={item.counterparty} size={14} sx={{ display: 'inline-block', verticalAlign: '-2px', mr: 0.5 }} />
              {name(item.counterparty) ? (
                <Tooltip title={item.counterparty}>
                  <Box component='span' sx={{ fontWeight: 600, color: 'text.primary' }}>{name(item.counterparty)}</Box>
                </Tooltip>
              ) : (
                <IdText id={item.counterparty} head={6} tail={6} explorer='address' />
              )}
            </Box>
          )}
          {item.kind === 'qx' && (
            <Box component='span' sx={{ fontWeight: 400, color: 'text.secondary' }}>
              {' '}from {name(item.from) || <IdText id={item.from} head={6} tail={6} explorer='address' />}
            </Box>
          )}
        </Typography>
        <Typography variant='caption' color='text.secondary' sx={{ display: 'flex', gap: 1.5, alignItems: 'center', flexWrap: 'wrap' }}>
          <Tooltip title={`${localTime(item.created)} (${item.created} UTC)`}>
            <span>{relative(item.created)}</span>
          </Tooltip>
          {item.txid && (
            <span>
              tx <IdText id={item.txid} head={6} tail={6} explorer='tx' />
            </span>
          )}
          {isNumeric(item.tick) && <span>tick {formatNumber(item.tick)}</span>}
          {ticksLeft !== null && (
            <Box component='span' sx={{ color: 'warning.main' }}>
              {ticksLeft > 0 ? `${ticksLeft} tick${ticksLeft === 1 ? '' : 's'} to go` : 'at tick — awaiting confirmation'}
            </Box>
          )}
          {state === 'unconfirmed' && (
            <Tooltip title={UNCONFIRMED_HINT}>
              <Box component='span' sx={{ color: 'warning.main' }}>
                not verified yet
              </Box>
            </Tooltip>
          )}
          {note && (
            <Box component='span' sx={{ display: 'inline-flex', alignItems: 'center', gap: 0.5, color: 'text.primary' }}>
              <NotesIcon sx={{ fontSize: 13 }} />
              <Box component='span' sx={{ maxWidth: 260, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{note}</Box>
            </Box>
          )}
        </Typography>
      </Box>
      <Box sx={{ textAlign: 'right', display: 'flex', alignItems: 'center', gap: 1.5 }}>
        <Typography
          sx={{
            fontWeight: 600,
            whiteSpace: 'nowrap',
            color: item.direction === 'in' ? 'success.main' : item.direction === 'out' ? 'text.primary' : 'text.secondary',
          }}
        >
          {item.amountText}
        </Typography>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          <TxStatusChip status={item.status} tick={item.tick} latestTick={latestTick} />
          {canRetry(state) && item.retry && (
            <TxStatus
              status={item.status}
              tick={item.tick}
              latestTick={latestTick}
              retryLabel={item.retry.label}
              onRetry={() => onRetry(item)}
            />
          )}
        </Box>
      </Box>
    </Box>
  );
};

const csvEscape = (v) => {
  const s = String(v ?? '');
  return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
};

const toCsv = (items, labels) => {
  const header = ['created_utc', 'type', 'status', 'title', 'from', 'from_nickname', 'to', 'to_nickname', 'amount', 'txid', 'tick'];
  const rows = items.map((i) => [
    i.created,
    i.kind,
    i.state,
    i.title,
    i.from || '',
    labels[i.from] || '',
    i.to || '',
    labels[i.to] || '',
    i.amountText,
    i.txid || '',
    i.tick || '',
  ]);
  return [header, ...rows].map((r) => r.map(csvEscape).join(',')).join('\n');
};

const downloadText = (name, text) => {
  const link = document.createElement('a');
  link.href = 'data:text/csv;charset=utf-8,' + encodeURIComponent(text);
  link.download = name;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
};

export default function ActivityPanel({
  transfers,
  assetTransfers,
  qxOrders,
  identities,
  labels,
  assetsNIssuer,
  latestTick,
  loading,
  onAction,
  requestConfirm,
}) {
  const [kind, setKind] = useState('all');
  const [state, setState] = useState('all');
  const [identity, setIdentity] = useState('all');
  const [minutes, setMinutes] = useState(ALL_TIME);
  const [detail, setDetail] = useState(null);
  const [notes, setNotes] = useStoredValue(KEYS.txNotes, {});
  const noteKey = (item) => item.txid || item.key;
  const saveNote = (item, text) => {
    setNotes((prev) => {
      const next = { ...prev };
      if (text) next[noteKey(item)] = text;
      else delete next[noteKey(item)];
      return next;
    });
  };

  const ownIds = useMemo(() => new Set(identities.map((i) => i.id)), [identities]);
  const items = useMemo(
    () => normalizeActivity({ transfers, assetTransfers, qxOrders, ownIds, assetsNIssuer }),
    [transfers, assetTransfers, qxOrders, ownIds, assetsNIssuer]
  );

  const stateOf = (i) => txState(i.status, i.tick, latestTick);
  const filtered = items.filter(
    (i) =>
      (kind === 'all' || i.kind === kind) &&
      (state === 'all' || stateOf(i) === state) &&
      (identity === 'all' || i.from === identity || i.to === identity) &&
      withinLastMinutes(i.created, minutes)
  );

  const pending = filtered.filter((i) => stateOf(i) === 'pending');
  const settled = filtered.filter((i) => stateOf(i) !== 'pending');
  const groups = settled.reduce((acc, item) => {
    const key = dayKey(item.created);
    (acc[key] ||= []).push(item);
    return acc;
  }, {});

  const retry = (item) =>
    requestConfirm({
      title: `${item.retry.label}?`,
      confirmLabel: item.retry.label,
      body: (
        <>
          <Typography sx={{ fontWeight: 600 }}>{item.title}</Typography>
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            A new transaction is created at the current tick.
          </Typography>
          <Typography variant='body2' sx={{ mt: 1, color: 'warning.main', fontWeight: 600 }}>
            Only resend if the balance shows the original did not go through — resending one that
            succeeded sends it twice.
          </Typography>
        </>
      ),
      onConfirm: () => onAction(item.retry.path),
    });

  const Group = ({ title, list, accent }) => (
    <Box sx={{ mb: 2.5 }}>
      <Typography
        variant='caption'
        sx={{ display: 'block', mb: 0.75, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em', color: accent || 'text.secondary' }}
      >
        {title}
      </Typography>
      <Paper variant='outlined'>
        {list.map((item) => (
          <ActivityRow
            key={item.key}
            item={item}
            labels={labels}
            latestTick={latestTick}
            note={notes[noteKey(item)]}
            onRetry={retry}
            onOpen={setDetail}
          />
        ))}
      </Paper>
    </Box>
  );

  return (
    <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      <Stack direction='row' spacing={2} useFlexGap sx={{ mb: 2, flexShrink: 0, alignItems: 'center', flexWrap: 'wrap' }}>
        <Stack direction='row' spacing={1}>
          {KINDS.map((k) => (
            <Chip
              key={k.key}
              size='small'
              label={k.label}
              clickable
              color={kind === k.key ? 'primary' : 'default'}
              variant={kind === k.key ? 'filled' : 'outlined'}
              onClick={() => setKind(k.key)}
            />
          ))}
        </Stack>
        <FormControl size='small' sx={{ minWidth: 150 }}>
          <InputLabel id='activity-state'>Status</InputLabel>
          <Select labelId='activity-state' label='Status' value={state} onChange={(e) => setState(e.target.value)}>
            {STATES.map((s) => (
              <MenuItem key={s.key} value={s.key}>{s.label}</MenuItem>
            ))}
          </Select>
        </FormControl>
        <FormControl size='small' sx={{ minWidth: 220 }}>
          <InputLabel id='activity-identity'>Identity</InputLabel>
          <Select labelId='activity-identity' label='Identity' value={identity} onChange={(e) => setIdentity(e.target.value)}>
            <MenuItem value='all'>All identities</MenuItem>
            {identities.map((i) => (
              <MenuItem key={i.id} value={i.id}>
                {labels[i.id] ? `${labels[i.id]} · ` : ''}
                <Box component='span' className='mono'>{shortenId(i.id, 8, 8)}</Box>
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <Box sx={{ flex: 1 }} />
        <Typography variant='body2' color='text.secondary'>
          {filtered.length} of {items.length}
        </Typography>
        <Button
          size='small'
          variant='outlined'
          startIcon={<DownloadIcon />}
          disabled={filtered.length === 0}
          onClick={() =>
            downloadText(
              `rubic-activity-${new Date().toISOString().slice(0, 10)}.csv`,
              toCsv(filtered.map((i) => ({ ...i, state: stateOf(i) })), labels)
            )
          }
        >
          Export CSV
        </Button>
      </Stack>
      <TimeFilterChips value={minutes} onChange={setMinutes} />

      {/* Filters stay put; the timeline scrolls on its own. */}
      <Box sx={{ flex: 1, minHeight: 0, overflow: 'auto', pr: 0.5 }}>
      {loading && (
        <Paper variant='outlined' sx={{ p: 2 }}>
          {[0, 1, 2, 3].map((i) => (
            <Box key={i} sx={{ display: 'flex', alignItems: 'center', gap: 2, py: 1.25 }}>
              <Skeleton variant='circular' width={36} height={36} />
              <Box sx={{ flex: 1 }}>
                <Skeleton variant='text' width='40%' />
                <Skeleton variant='text' width='60%' sx={{ fontSize: '0.7rem' }} />
              </Box>
              <Skeleton variant='text' width={90} />
              <Skeleton variant='rounded' width={72} height={22} />
            </Box>
          ))}
        </Paper>
      )}

      {!loading && filtered.length === 0 && (
        <Paper variant='outlined' sx={{ p: 5, textAlign: 'center' }}>
          <Typography variant='body1' sx={{ fontWeight: 600 }}>
            Nothing here yet
          </Typography>
          <Typography variant='body2' color='text.secondary'>
            {items.length === 0 ? 'Transfers, asset moves and QX orders will show up here.' : 'No activity matches these filters.'}
          </Typography>
        </Paper>
      )}

      {pending.length > 0 && <Group title={`Pending · ${pending.length}`} list={pending} accent='warning.main' />}
      {Object.keys(groups)
        .sort((a, b) => (a < b ? 1 : -1))
        .map((key) => (
          <Group key={key} title={dayLabel(key)} list={groups[key]} />
        ))}
      </Box>

      <TxDetailDialog
        item={detail}
        labels={labels}
        latestTick={latestTick}
        note={detail ? notes[noteKey(detail)] : ''}
        onSaveNote={(item, text) => {
          saveNote(item, text);
          setDetail(null);
        }}
        onRetry={retry}
        onClose={() => setDetail(null)}
      />
    </Box>
  );
}
