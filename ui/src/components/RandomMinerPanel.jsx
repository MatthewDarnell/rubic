import React, { useEffect, useRef, useState } from 'react';
import {
  Box,
  Button,
  Checkbox,
  Chip,
  FormControl,
  FormControlLabel,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Tooltip,
  Typography,
} from '@mui/material';
import CasinoIcon from '@mui/icons-material/Casino';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import { apiCall, apiPost } from '../api';
import { useToast } from './ToastProvider';
import IdText from './IdText';
import Identicon from './Identicon';
import { formatNumber, isNumeric, shortenId } from '../utils/format';

const SESSIONS_INTERVAL = 3000;
// A session signs this many steps up front (the server's MAX_STEPS plus the first commit).
const MAX_STEPS_PER_SESSION = 1001;
const STREAM_TICKS = 3; // a provider acts every third tick
// The collateral tiers the RANDOM contract accepts. Each step refunds the
// previous stake and locks this amount again, so the identity needs twice the
// tier free when the session starts.
const TIERS = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000];

// Session status as stored by the server.
const SESSION_STATE = {
  0: { label: 'Mining', color: 'primary', running: true, hint: 'Steps are signed ahead and broadcast every 3 ticks; the collateral rolls over from step to step.' },
  1: { label: 'Stopping', color: 'warning', running: false, hint: 'The next step not yet on the wire goes out as a reveal-and-leave.' },
  2: { label: 'Stopping', color: 'warning', running: false, hint: 'The reveal-and-leave is on the wire; the collateral is returned when it is included.' },
  3: { label: 'Failed', color: 'error', running: false, hint: 'The contract stopped accepting this provider: a step was missed or rejected, and the stake was forfeited.' },
  4: { label: 'Stopped', color: 'success', running: false, hint: 'Left cleanly: the collateral was returned.' },
};

const asList = (res) => (res.success && Array.isArray(res.data) ? res.data : []);

/**
 * Random Miner: sustained entropy providing for the RANDOM smart contract.
 * Start signs a whole chain of reveal-and-commit steps on the server, one per
 * 3 ticks; the server broadcasts each step a few ticks ahead and Stop turns
 * the next unsent step into a reveal-and-leave that returns the collateral.
 */
export default function RandomMinerPanel({ identities, labels, latestTick, tickOffset, busy, onAction, requestConfirm, warningsFor }) {
  const toast = useToast();
  const [selectedId, setSelectedId] = useState('');
  const [tier, setTier] = useState(1);
  const [sessions, setSessions] = useState([]);
  const [stopping, setStopping] = useState(null);
  // Read when the start dialog is confirmed: the dialog body is built once, so
  // the checkbox is uncontrolled and reports into this ref.
  const autoRestartRef = useRef(false);

  // Sessions from the server, refreshed while the tab is open.
  useEffect(() => {
    let cancelled = false;
    let timer;
    const load = async () => {
      const res = await apiCall('miner/random');
      if (cancelled) return;
      setSessions(asList(res));
      timer = setTimeout(load, SESSIONS_INTERVAL);
    };
    load();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  const signer = identities.find((i) => i.id === selectedId);
  const balance = isNumeric(signer?.balance) ? Number(signer.balance) : null;
  const needed = tier * 2;
  const cannotAfford = balance !== null && balance < needed;
  const alreadyRunning = sessions.some((s) => s.identity === selectedId && SESSION_STATE[Number(s.status)]?.running);
  const offset = Number.isInteger(Number(tickOffset)) && Number(tickOffset) > 0 ? Number(tickOffset) : 30;
  const firstTick = isNumeric(latestTick) ? Number(latestTick) + offset : null;
  const canStart = Boolean(selectedId) && !busy && !cannotAfford && !alreadyRunning && firstTick !== null;
  const live = isNumeric(latestTick) ? Number(latestTick) : null;

  const start = () => {
    const warnings = warningsFor?.(selectedId) || [];
    autoRestartRef.current = false;
    requestConfirm({
      title: 'Start providing entropy?',
      confirmLabel: 'Start',
      confirmColor: warnings.length > 0 ? 'warning' : 'primary',
      body: (
        <>
          {warnings.length > 0 && (
            <Box sx={{ mb: 1.5, p: 1.5, border: 1, borderColor: 'warning.main', borderRadius: 1 }}>
              <Typography variant='body2' sx={{ color: 'warning.main', fontWeight: 600 }}>
                Your connection looks unstable right now. A step that does not reach the network in time forfeits the
                stake, so consider waiting until this clears.
              </Typography>
              {warnings.map((text) => (
                <Typography key={text} variant='body2' sx={{ mt: 0.5, color: 'warning.main' }}>
                  • {text}
                </Typography>
              ))}
            </Box>
          )}
          <Typography variant='body2'>
            First commit at tick <b>{formatNumber(firstTick)}</b>, then a reveal-and-commit every {STREAM_TICKS} ticks until
            you stop, at the <b>{formatNumber(tier)} QU</b> tier.
          </Typography>
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            Each step returns the previous stake and locks {formatNumber(tier)} QU again. Stopping takes a few steps:
            the last one reveals without committing, which returns the stake.
          </Typography>
          <Typography variant='body2' sx={{ mt: 1.5, color: 'error.main', fontWeight: 600 }}>
            Warning: Random Miner involves risk of losing your committed Qus. If you lose network connection or have
            failed transactions, your funds will not be refunded.
          </Typography>
          <FormControlLabel
            sx={{ mt: 1, alignItems: 'flex-start' }}
            control={<Checkbox size='small' defaultChecked={false} onChange={(e) => { autoRestartRef.current = e.target.checked; }} sx={{ mt: -0.5 }} />}
            label={
              <Typography variant='body2'>
                Keep mining if a transaction fails: start a new session automatically. Each new session locks a fresh
                stake, and this identity's seed stays in memory while it mines. It stops after 3 failures in a row
                without an accepted step.
              </Typography>
            }
          />
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1.5 }}>
            From
          </Typography>
          <IdText id={selectedId} full copy={false} />
        </>
      ),
      onConfirm: () => onAction(`miner/random/start/${selectedId}/${tier}/${autoRestartRef.current ? 1 : 0}/`),
    });
  };

  const stop = async (session) => {
    setStopping(session.id);
    try {
      const res = await apiPost('miner/random/stop', { session_id: Number(session.id) });
      const text = String(res.data ?? '');
      if (res.success && /stop requested/i.test(text)) toast.success('Stop requested: the next step reveals and leaves');
      else toast.error(text || res.error || 'Could not stop the session');
    } finally {
      setStopping(null);
    }
  };

  // The session acting on the network right now, if any: providing, stopping or leaving.
  const current = sessions.find((s) => [0, 1, 2].includes(Number(s.status)));
  const currentState = current ? SESSION_STATE[Number(current.status)] || SESSION_STATE[0] : null;

  const totals = sessions.reduce(
    (acc, s) => ({
      accepted: acc.accepted + (Number(s.accepted) || 0),
      lost: acc.lost + (s.stake === 'lost' ? Number(s.tier) || 0 : 0),
    }),
    { accepted: 0, lost: 0 }
  );

  // What became of a session's stake, from the server's `stake` field.
  const STAKE = {
    locked: { color: 'primary.main', text: (s) => `${formatNumber(s.tier)} QU locked`, hint: 'The stake rolls over from step to step while providing.' },
    pending: { color: 'text.secondary', text: () => 'not locked yet', hint: 'The first commit has not been reported by the contract yet.' },
    returned: { color: 'success.main', text: () => 'returned', hint: 'The reveal-and-leave went through: the stake came back.' },
    lost: { color: 'error.main', text: (s) => `${formatNumber(s.tier)} QU lost`, hint: 'The contract evicted the provider and burned the stake.' },
    none: { color: 'text.secondary', text: () => 'nothing lost', hint: 'The identity never got in: a rejected first commit is refunded, an unsent one costs nothing.' },
    unknown: { color: 'text.disabled', text: () => '\u2014', hint: 'No report from the contract for this session.' },
  };

  const progress = (s) => {
    const first = Number(s.first_tick);
    const total = Number(s.steps) || 0;
    if (live === null || !Number.isFinite(first) || total === 0) return null;
    const done = Math.min(total, Math.max(0, Math.floor((live - first) / STREAM_TICKS) + 1));
    return { done, total, through: Number(s.broadcast_through) };
  };

  return (
    <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      <Paper variant='outlined' sx={{ px: 2, py: 1.5, mb: 2, flexShrink: 0 }}>
        <Stack direction='row' spacing={1.5} useFlexGap sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
          <CasinoIcon color='primary' />
          <FormControl size='small' sx={{ minWidth: 240, flex: 1 }}>
            <InputLabel id='random-id-label'>Identity</InputLabel>
            <Select
              labelId='random-id-label'
              value={identities.some((i) => i.id === selectedId) ? selectedId : ''}
              label='Identity'
              onChange={(e) => setSelectedId(e.target.value)}
            >
              {identities.map((item) => (
                <MenuItem key={item.id} value={item.id}>
                  <Identicon id={item.id} size={18} sx={{ mr: 1 }} />
                  {labels?.[item.id] && <Box component='span' sx={{ fontWeight: 600, mr: 1 }}>{labels[item.id]}</Box>}
                  <Box component='span' sx={{ fontFamily: 'monospace' }}>{shortenId(item.id, 6, 6)}</Box>
                  <Box component='span' sx={{ ml: 1, color: 'text.secondary' }}>
                    {formatNumber(item.balance)} QU
                  </Box>
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <FormControl size='small' sx={{ width: 190, flexShrink: 0 }}>
            <InputLabel id='random-tier-label'>Collateral stake</InputLabel>
            <Select labelId='random-tier-label' value={tier} label='Collateral stake' onChange={(e) => setTier(Number(e.target.value))}>
              {TIERS.map((t) => (
                <MenuItem key={t} value={t}>
                  {formatNumber(t)} QU
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <Box sx={{ flexShrink: 0, lineHeight: 1.1 }}>
            <Typography variant='caption' color='text.secondary' sx={{ display: 'block' }}>First commit</Typography>
            <Typography variant='body2' sx={{ whiteSpace: 'nowrap' }}>{firstTick === null ? '—' : `tick ${formatNumber(firstTick)}`}</Typography>
          </Box>
          <Tooltip
            title={
              busy
                ? 'A transaction is waiting for its tick — a session can start once it lands.'
                : alreadyRunning
                ? 'This identity already has a running session'
                : cannotAfford
                ? `This identity has ${formatNumber(balance)} QU; the ${formatNumber(tier)} QU tier needs ${formatNumber(needed)} QU free`
                : ''
            }
            placement='top'
          >
            <span>
              <Button variant='contained' startIcon={<PlayArrowIcon />} disabled={!canStart} onClick={start} sx={{ height: 40 }}>
                Start
              </Button>
            </span>
          </Tooltip>
        </Stack>
        <Box component='ul' sx={{ m: 0, mt: 1.5, pl: 2.5, '& li': { mt: 0.25 } }}>
          <Typography component='li' variant='body2'>
            Random Mining provides entropy to the Qubic network, enabling other Smart Contracts to have a source of random
            numbers.
          </Typography>
          <Typography component='li' variant='body2' color='text.secondary'>
            Your chosen Collateral stake rolls over every step and, if successful, is refunded when you stop.
          </Typography>
          <Typography component='li' variant='body2' color='text.secondary'>
            Maximum {formatNumber(MAX_STEPS_PER_SESSION)} steps per session ({formatNumber(MAX_STEPS_PER_SESSION * STREAM_TICKS)} ticks),
            after which the session ends.
          </Typography>
          <Typography component='li' variant='body2' color='text.secondary'>
            Keep Rubic running to avoid halting the mining and losing funds.
          </Typography>
        </Box>
      </Paper>

      <Paper variant='outlined' sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', gap: 1.5, flexShrink: 0 }}>
          <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
            Sessions
          </Typography>
          <Chip size='small' label={sessions.length} />
          {current ? (
            <Tooltip title={`${currentState.hint}${current.auto_restart === '1' ? ' Keep mining is on: a failed session is replaced automatically.' : ''}`}>
              <Stack direction='row' spacing={0.75} sx={{ alignItems: 'center' }}>
                <Chip size='small' color={currentState.color} label={currentState.label} />
                <Typography variant='caption' color='text.secondary'>
                  {shortenId(current.identity, 6, 6)} · {formatNumber(current.tier)} QU · step {Number(current.sent) || 0} / {Number(current.steps) || MAX_STEPS_PER_SESSION}
                  {current.auto_restart === '1' ? ' · auto' : ''}
                </Typography>
              </Stack>
            </Tooltip>
          ) : (
            <Chip size='small' variant='outlined' label='Not mining' />
          )}
          <Box sx={{ flex: 1 }} />
          <Tooltip title='Steps the contract accepted across all sessions'>
            <Chip size='small' color='success' variant='outlined' label={`${formatNumber(totals.accepted)} steps accepted`} />
          </Tooltip>
          <Tooltip title='Stakes burned by the contract across all sessions'>
            <Chip size='small' color={totals.lost > 0 ? 'error' : 'default'} variant='outlined' label={`${formatNumber(totals.lost)} QU lost`} />
          </Tooltip>
        </Box>
        <TableContainer sx={{ borderTop: 1, borderColor: 'divider', flex: 1, minHeight: 0, overflow: 'auto' }}>
          <Table size='small' stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell>Identity</TableCell>
                <TableCell align='right'>Tier</TableCell>
                <TableCell>First commit</TableCell>
                <TableCell>Step</TableCell>
                <TableCell>Stake</TableCell>
                <TableCell sx={{ width: 110 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {sessions.length === 0 && (
                <TableRow>
                  <TableCell colSpan={6} align='center' sx={{ py: 3, color: 'text.secondary' }}>
                    No sessions yet
                  </TableCell>
                </TableRow>
              )}
              {sessions.map((s) => {
                const state = SESSION_STATE[Number(s.status)] || SESSION_STATE[0];
                const p = progress(s);
                const finished = Number(s.status) >= 3;
                return (
                  <TableRow key={s.id} hover sx={{ opacity: finished ? 0.75 : 1 }}>
                    <TableCell>
                      <Stack direction='row' spacing={1} sx={{ alignItems: 'center' }}>
                        <Identicon id={s.identity} size={20} />
                        <IdText id={s.identity} head={6} tail={6} explorer='address' />
                      </Stack>
                    </TableCell>
                    <TableCell align='right' className='mono'>{formatNumber(s.tier)} QU</TableCell>
                    <TableCell className='mono'>tick {formatNumber(s.first_tick)}</TableCell>
                    <TableCell>
                      <Tooltip title={`${formatNumber(Number(s.sent) || 0)} steps sent by the wallet · ${formatNumber(Number(s.accepted) || 0)} accepted by the contract · ${formatNumber(p ? p.total : MAX_STEPS_PER_SESSION)} signed`}>
                        <Box component='span' className='mono' sx={{ whiteSpace: 'nowrap' }}>
                          {Number(s.sent) || 0} / {p ? p.total : MAX_STEPS_PER_SESSION}
                        </Box>
                      </Tooltip>
                    </TableCell>
                    <TableCell>
                      {(() => {
                        const stake = STAKE[s.stake] || STAKE.unknown;
                        const failedAt = Number(s.status) === 3 && Number(s.failed_tick) > 0 ? ` Failed at tick ${formatNumber(s.failed_tick)}.` : '';
                        return (
                          <Tooltip title={`${stake.hint}${failedAt}${s.reason ? ` ${s.reason}` : ''}`}>
                            <Box component='span' sx={{ color: stake.color, whiteSpace: 'nowrap' }}>{stake.text(s)}</Box>
                          </Tooltip>
                        );
                      })()}
                    </TableCell>
                    <TableCell>
                      {state.running && (
                        <Tooltip title='Reveal and leave at the next unsent step; no new session is started afterwards'>
                          <span>
                            <Button size='small' color='warning' startIcon={<StopIcon />} disabled={stopping === s.id} onClick={() => stop(s)}>
                              Stop
                            </Button>
                          </span>
                        </Tooltip>
                      )}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    </Box>
  );
}
