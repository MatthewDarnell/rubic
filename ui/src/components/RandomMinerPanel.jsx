import React, { useEffect, useRef, useState } from 'react';
import {
  Box,
  Button,
  Checkbox,
  Chip,
  CircularProgress,
  Collapse,
  FormControl,
  FormControlLabel,
  IconButton,
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
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import { apiCall, apiPost } from '../api';
import { useToast } from './ToastProvider';
import IdText from './IdText';
import Identicon from './Identicon';
import { formatNumber, isNumeric, shortenId } from '../utils/format';

const SESSIONS_INTERVAL = 3000;
// Reveal steps a session is signed for, chosen at Start; the first commit
// comes on top, so a session of N steps sends N + 1 transactions.
const SESSION_LENGTHS = [500, 1000, 2500, 5000, 10000];
const DEFAULT_STEPS = 1000;
const MAX_STEPS = 10000; // the server's MAX_STEPS
const STREAM_TICKS = 3; // a provider acts every third tick
// Step rows fetched when a session is expanded (newest first).
const HISTORY_LIMIT = 200;
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
  const [steps, setSteps] = useState(DEFAULT_STEPS);
  const [sessions, setSessions] = useState([]);
  const [stopping, setStopping] = useState(null);
  // The session whose step history is open, and what was loaded for it.
  const [expanded, setExpanded] = useState(null);
  const [history, setHistory] = useState({ id: null, rows: null, error: '' });
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

  // Step history of the expanded session: loaded when it opens, and again on
  // every sessions poll while that session is still on the network.
  const expandedActive = expanded !== null && sessions.some((s) => s.id === expanded && Number(s.status) < 3);
  useEffect(() => {
    if (expanded === null) return undefined;
    let cancelled = false;
    (async () => {
      const res = await apiCall(`miner/random/${expanded}/steps?limit=${HISTORY_LIMIT}`);
      if (cancelled) return;
      if (res.success && Array.isArray(res.data)) setHistory({ id: expanded, rows: res.data, error: '' });
      else setHistory({ id: expanded, rows: [], error: String(res.data || res.error || 'Could not load the steps') });
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [expanded, expandedActive ? sessions : null]);

  const toggle = (id) => {
    setExpanded((open) => (open === id ? null : id));
    setHistory({ id: null, rows: null, error: '' });
  };

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
            First commit at tick <b>{formatNumber(firstTick)}</b>, then a reveal-and-commit every {STREAM_TICKS} ticks at the{' '}
            <b>{formatNumber(tier)} QU</b> tier, for up to <b>{formatNumber(steps)} steps</b> (about {formatNumber(steps * STREAM_TICKS)} ticks)
            or until you stop.
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
      onConfirm: () => onAction(`miner/random/start/${selectedId}/${tier}/${autoRestartRef.current ? 1 : 0}/${steps}/`),
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
          <FormControl size='small' sx={{ width: 225, flexShrink: 0 }}>
            <InputLabel id='random-steps-label'>Session length</InputLabel>
            <Select labelId='random-steps-label' value={steps} label='Session length' onChange={(e) => setSteps(Number(e.target.value))}>
              {SESSION_LENGTHS.map((n) => (
                <MenuItem key={n} value={n}>
                  {formatNumber(n)} steps
                  <Box component='span' sx={{ ml: 1, color: 'text.secondary' }}>
                    ≈ {formatNumber(n * STREAM_TICKS)} ticks
                  </Box>
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
            Session length is chosen at Start, up to {formatNumber(MAX_STEPS + 1)} steps ({formatNumber((MAX_STEPS + 1) * STREAM_TICKS)} ticks)
            including the first commit, after which the session ends.
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
                  {shortenId(current.identity, 6, 6)} · {formatNumber(current.tier)} QU · step {Number(current.sent) || 0} / {Number(current.steps) || DEFAULT_STEPS + 1}
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
                <TableCell sx={{ width: 40 }} />
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
                  <TableCell colSpan={7} align='center' sx={{ py: 3, color: 'text.secondary' }}>
                    No sessions yet
                  </TableCell>
                </TableRow>
              )}
              {sessions.map((s) => {
                const state = SESSION_STATE[Number(s.status)] || SESSION_STATE[0];
                const finished = Number(s.status) >= 3;
                const open = expanded === s.id;
                return (
                  <React.Fragment key={s.id}>
                  <TableRow hover onClick={() => toggle(s.id)} sx={{ opacity: finished ? 0.75 : 1, cursor: 'pointer', '& > td': { borderBottom: open ? 'none' : undefined } }}>
                    <TableCell sx={{ pr: 0 }}>
                      <IconButton size='small' aria-label={open ? 'Hide steps' : 'Show steps'} onClick={(e) => { e.stopPropagation(); toggle(s.id); }}>
                        <ExpandMoreIcon fontSize='small' sx={{ transform: open ? 'rotate(180deg)' : 'none', transition: 'transform 150ms' }} />
                      </IconButton>
                    </TableCell>
                    <TableCell>
                      <Stack direction='row' spacing={1} sx={{ alignItems: 'center' }}>
                        <Identicon id={s.identity} size={20} />
                        <IdText id={s.identity} head={6} tail={6} explorer='address' />
                      </Stack>
                    </TableCell>
                    <TableCell align='right' className='mono'>{formatNumber(s.tier)} QU</TableCell>
                    <TableCell className='mono'>tick {formatNumber(s.first_tick)}</TableCell>
                    <TableCell>
                      <Tooltip title={`${formatNumber(Number(s.sent) || 0)} steps sent by the wallet · ${formatNumber(Number(s.accepted) || 0)} accepted by the contract · ${formatNumber(Number(s.steps) || 0)} signed`}>
                        <Box component='span' className='mono' sx={{ whiteSpace: 'nowrap' }}>
                          {Number(s.sent) || 0} / {Number(s.steps) || DEFAULT_STEPS + 1}
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
                    <TableCell onClick={(e) => e.stopPropagation()}>
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
                  <TableRow>
                    <TableCell colSpan={7} sx={{ py: 0, borderBottom: open ? undefined : 'none' }}>
                      <Collapse in={open} unmountOnExit>
                        <StepHistory session={s} live={live} history={history.id === s.id ? history : null} />
                      </Collapse>
                    </TableCell>
                  </TableRow>
                  </React.Fragment>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    </Box>
  );
}

// What a step's transaction went through: the wallet sent it, the network
// included it in its tick, the contract accepted it (or not).
const STEP_KIND = { commit: 'First commit', reveal: 'Reveal and commit', leave: 'Reveal and leave' };

function stepOutcome(row, session, live) {
  const tick = Number(row.tick);
  const included = String(row.included);
  if (row.accepted === '1') return { label: 'accepted', color: 'success.main', hint: 'The contract recorded this step: the previous stake came back and this one is locked.' };
  if (included === '0') return { label: 'included', color: 'primary.main', hint: 'Included in its tick; the contract has not reported on it yet.' };
  if (included === '1') return { label: 'not included', color: 'error.main', hint: 'The tick\'s transaction list does not contain this transaction.' };
  if (included === '2') return { label: 'expired', color: 'error.main', hint: 'Its tick passed without the transaction being included.' };
  if (Number(session.status) === 3 && Number(session.failed_tick) === tick) return { label: 'not accepted', color: 'error.main', hint: session.reason || 'The contract did not record this step; the transaction may still have been included and rejected.' };
  if (live !== null && tick < live - STREAM_TICKS) return { label: 'not reported', color: 'text.secondary', hint: 'Its tick has passed; no report from the contract yet.' };
  return { label: 'pending', color: 'text.secondary', hint: 'Sent; its tick has not come yet.' };
}

function StepHistory({ session, live, history }) {
  if (!history || history.rows === null) {
    return (
      <Box sx={{ py: 1.5, display: 'flex', alignItems: 'center', gap: 1, color: 'text.secondary' }}>
        <CircularProgress size={14} />
        <Typography variant='caption'>Loading steps…</Typography>
      </Box>
    );
  }
  if (history.error) {
    return (
      <Typography variant='caption' color='error.main' sx={{ display: 'block', py: 1.5 }}>
        {history.error}
      </Typography>
    );
  }
  if (history.rows.length === 0) {
    return (
      <Typography variant='caption' color='text.secondary' sx={{ display: 'block', py: 1.5 }}>
        {Number(session.sent) > 0 ? 'The steps of this session have been pruned from the history.' : 'No step has been sent yet.'}
      </Typography>
    );
  }
  const sent = Number(session.sent) || 0;
  return (
    <Box sx={{ py: 1, pl: 5 }}>
      <Typography variant='caption' color='text.secondary' sx={{ display: 'block', mb: 0.5 }}>
        {history.rows.length < sent ? `Latest ${formatNumber(history.rows.length)} of ${formatNumber(sent)} steps sent` : `${formatNumber(history.rows.length)} steps sent`}, newest first
      </Typography>
      <Table size='small' sx={{ '& td, & th': { py: 0.25 } }}>
        <TableHead>
          <TableRow>
            <TableCell>Step</TableCell>
            <TableCell>Tick</TableCell>
            <TableCell>Kind</TableCell>
            <TableCell>Transaction</TableCell>
            <TableCell>Sent</TableCell>
            <TableCell>Outcome</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {history.rows.map((row) => {
            const outcome = stepOutcome(row, session, live);
            return (
              <TableRow key={row.step}>
                <TableCell className='mono'>{row.step}</TableCell>
                <TableCell className='mono'>{formatNumber(row.tick)}</TableCell>
                <TableCell>{STEP_KIND[row.kind] || row.kind}</TableCell>
                <TableCell>{row.txid ? <IdText id={row.txid} head={6} tail={6} explorer='tx' /> : '—'}</TableCell>
                <TableCell className='mono' sx={{ whiteSpace: 'nowrap' }}>{row.sent ? `${row.sent} UTC` : '—'}</TableCell>
                <TableCell>
                  <Tooltip title={outcome.hint}>
                    <Box component='span' sx={{ color: outcome.color, whiteSpace: 'nowrap' }}>{outcome.label}</Box>
                  </Tooltip>
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </Box>
  );
}
