import React from 'react';
import {
  Badge,
  Box,
  Button,
  Chip,
  Divider,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Popover,
  Tooltip,
  Typography,
} from '@mui/material';
import WalletIcon from '@mui/icons-material/AccountBalanceWalletOutlined';
import HistoryIcon from '@mui/icons-material/History';
import TokenIcon from '@mui/icons-material/TokenOutlined';
import SwapHorizIcon from '@mui/icons-material/SwapHoriz';
import HubIcon from '@mui/icons-material/HubOutlined';
import SettingsIcon from '@mui/icons-material/SettingsOutlined';
import LightModeIcon from '@mui/icons-material/LightMode';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import LockIcon from '@mui/icons-material/Lock';
import SendIcon from '@mui/icons-material/Send';
import HourglassTopIcon from '@mui/icons-material/HourglassTop';
import { Skeleton } from '@mui/material';
import Sparkline from './Sparkline';
import { formatAge, formatFiat, formatNumber, isNumeric } from '../utils/format';
import logo from '../assets/rubic.png';

export const NAV = [
  { key: 'wallet', label: 'Wallet', icon: WalletIcon },
  { key: 'activity', label: 'Activity', icon: HistoryIcon },
  { key: 'assets', label: 'Assets', icon: TokenIcon },
  { key: 'exchange', label: 'QX Exchange', icon: SwapHorizIcon },
  { key: 'network', label: 'Network', icon: HubIcon },
  { key: 'settings', label: 'Settings', icon: SettingsIcon },
];

const SIDEBAR_WIDTH = 216;

const BrandMark = () => (
  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.25, px: 2.5, py: 2.5 }}>
    <Box
      component='img'
      src={logo}
      alt=''
      aria-hidden
      sx={{ width: 34, height: 34, borderRadius: 2, display: 'block', flexShrink: 0 }}
    />
    <Box>
      <Typography sx={{ fontWeight: 700, lineHeight: 1.1, letterSpacing: '-0.01em' }}>Rubic</Typography>
      <Typography variant='caption' color='text.secondary' sx={{ lineHeight: 1 }}>
        Qubic wallet
      </Typography>
    </Box>
  </Box>
);

const Sidebar = ({ nav, onNav, badges }) => (
  <Box
    component='nav'
    sx={{
      width: SIDEBAR_WIDTH,
      flexShrink: 0,
      borderRight: 1,
      borderColor: 'divider',
      bgcolor: 'background.paper',
      display: 'flex',
      flexDirection: 'column',
    }}
  >
    <BrandMark />
    <List sx={{ px: 1.5, py: 0 }}>
      {NAV.map((item) => {
        const Icon = item.icon;
        const active = nav === item.key;
        const badge = badges?.[item.key];
        return (
          <ListItemButton
            key={item.key}
            selected={active}
            onClick={() => onNav(item.key)}
            sx={{
              borderRadius: 2,
              mb: 0.5,
              py: 0.9,
              '&.Mui-selected': { bgcolor: 'action.selected' },
            }}
          >
            <ListItemIcon sx={{ minWidth: 34, color: active ? 'primary.main' : 'text.secondary' }}>
              <Icon fontSize='small' />
            </ListItemIcon>
            <ListItemText
              primary={item.label}
              slotProps={{ primary: { sx: { fontWeight: active ? 600 : 500, fontSize: '0.9rem' } } }}
            />
            {badge ? <Chip size='small' label={badge} sx={{ height: 20, fontSize: '0.7rem' }} /> : null}
          </ListItemButton>
        );
      })}
    </List>
  </Box>
);

// Latency as the peer table stores it (9999 = never measured).
const latencyText = (ping) => {
  const n = Number(ping);
  return Number.isFinite(n) && n > 0 && n < 9999 ? `${n} ms` : '—';
};

const StatusPill = ({ connected, tick, peersConnected, syncing, health, peers, now }) => {
  const [anchor, setAnchor] = React.useState(null);
  const noPeers = connected && peersConnected === 0;
  const behind = Boolean(health && health.routine_limit && health.routine_backlog >= health.routine_limit / 2);
  const label = !connected
    ? 'Server unreachable'
    : noPeers
    ? `No peers connected${!syncing ? ` · tick ${formatNumber(tick)}` : ''}`
    : syncing
    ? 'Waiting for first tick'
    : `Tick ${formatNumber(tick)} · ${peersConnected} peer${peersConnected === 1 ? '' : 's'}${behind ? ' · catching up' : ''}`;
  const color = !connected ? 'error.main' : noPeers || syncing || behind ? 'warning.main' : 'success.main';
  const connectedPeers = (peers || []).filter((p) => p.whitelisted !== '-1' && (p.connected === '1' || p.connected === 'true'));
  const nowSec = Math.floor((now || Date.now()) / 1000);
  return (
    <>
      <Tooltip title='Network health — click for details'>
        <Box
          onClick={(e) => setAnchor(e.currentTarget)}
          sx={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 1,
            px: 1.5,
            py: 0.5,
            borderRadius: 999,
            border: 1,
            borderColor: 'divider',
            bgcolor: 'background.paper',
            cursor: 'pointer',
            '&:hover': { bgcolor: 'action.hover' },
          }}
        >
          <Box sx={{ width: 8, height: 8, borderRadius: '50%', bgcolor: color }} />
          <Typography variant='body2' sx={{ fontWeight: 500 }}>
            {label}
          </Typography>
        </Box>
      </Tooltip>
      <Popover
        open={Boolean(anchor)}
        anchorEl={anchor}
        onClose={() => setAnchor(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
        slotProps={{ paper: { sx: { p: 2, width: 380 } } }}
      >
        <Typography variant='subtitle2' sx={{ fontWeight: 600, mb: 1 }}>
          Network health
        </Typography>
        {!connected ? (
          <Typography variant='body2' color='error.main'>Cannot reach the local Rubic server.</Typography>
        ) : (
          <>
            <Box sx={{ display: 'grid', gridTemplateColumns: 'auto 1fr', columnGap: 2, rowGap: 0.5, mb: 1.5 }}>
              <Typography variant='body2' color='text.secondary'>Latest tick</Typography>
              <Typography variant='body2'>{isNumeric(tick) ? formatNumber(tick) : '—'}</Typography>
              <Typography variant='body2' color='text.secondary'>Request queue</Typography>
              <Typography variant='body2' sx={{ color: behind ? 'warning.main' : 'inherit' }}>
                {health
                  ? `${health.routine_backlog} waiting (limit ${health.routine_limit}) · ${health.tick_backlog} tick polls`
                  : '—'}
                {behind ? ' — behind; balances and confirmations may lag' : ''}
              </Typography>
              <Typography variant='body2' color='text.secondary'>Peers</Typography>
              <Typography variant='body2'>{peersConnected} connected</Typography>
            </Box>
            <Divider sx={{ mb: 1 }} />
            {connectedPeers.length === 0 ? (
              <Typography variant='body2' color='warning.main'>No peers connected — nothing updates until Rubic reconnects.</Typography>
            ) : (
              <Box sx={{ display: 'grid', gridTemplateColumns: '1fr auto auto', columnGap: 2, rowGap: 0.25 }}>
                {connectedPeers.slice(0, 10).map((p) => {
                  const age = nowSec - Number(p.last_responded || 0);
                  return (
                    <React.Fragment key={p.id || p.ip}>
                      <Typography variant='caption' sx={{ fontFamily: 'monospace' }}>{p.ip}</Typography>
                      <Typography variant='caption' color='text.secondary'>{latencyText(p.ping)}</Typography>
                      <Typography variant='caption' sx={{ color: age > 60 ? 'warning.main' : 'text.secondary' }}>
                        {age < 0 || !p.last_responded ? '—' : age < 60 ? `${age} s ago` : formatAge(age * 1000) + ' ago'}
                      </Typography>
                    </React.Fragment>
                  );
                })}
              </Box>
            )}
          </>
        )}
      </Popover>
    </>
  );
};

const UnlockPill = ({ secondsLeft }) =>
  secondsLeft > 0 ? (
    <Tooltip title='Actions will not ask for the master password until the timer runs out'>
      <Chip
        size='small'
        icon={<LockOpenIcon />}
        color='warning'
        variant='outlined'
        label={`Unlocked · ${secondsLeft}s`}
      />
    </Tooltip>
  ) : (
    <Tooltip title='Signing actions will ask for the master password'>
      <Chip size='small' icon={<LockIcon />} variant='outlined' label='Locked' />
    </Tooltip>
  );

const PendingPill = ({ pending }) =>
  pending ? (
    <Tooltip title={pending.tooltip}>
      <Chip size='small' color='primary' variant='outlined' icon={<HourglassTopIcon />} label={pending.label} />
    </Tooltip>
  ) : null;

const UnencryptedPill = ({ count, onClick }) =>
  count > 0 ? (
    <Tooltip title='Some seeds are stored without encryption. Open Wallet to review them.'>
      <Chip
        size='small'
        color='warning'
        icon={<LockOpenIcon />}
        label={`${count} unencrypted`}
        onClick={onClick}
        clickable
      />
    </Tooltip>
  ) : null;

export default function AppShell({
  nav,
  onNav,
  badges,
  totalBalance,
  price,
  connected,
  tick,
  peersConnected,
  unlockSecondsLeft,
  unencryptedCount,
  pending,
  themeMode,
  onToggleTheme,
  onSend,
  canSend,
  currency,
  loading,
  history,
  balanceStale,
  balanceStaleSince,
  now,
  health,
  peers,
  children,
}) {
  const fiat = formatFiat(totalBalance, price, currency);
  const staleAge = balanceStaleSince ? formatAge((now || Date.now()) - balanceStaleSince) : null;
  return (
    // The window never scrolls as a whole: sidebar and header stay put and each
    // section scrolls its own long content (tables, ladders, timelines).
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden', bgcolor: 'background.default' }}>
      <Sidebar nav={nav} onNav={onNav} badges={badges} />
      <Box sx={{ flex: 1, minWidth: 0, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
        <Box
          component='header'
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: 2,
            px: 3,
            py: 1.5,
            borderBottom: 1,
            borderColor: 'divider',
            bgcolor: 'background.paper',
            flexShrink: 0,
          }}
        >
          <Box sx={{ minWidth: 0 }}>
            <Typography variant='caption' color='text.secondary' sx={{ lineHeight: 1 }}>
              Total balance
            </Typography>
            {loading ? (
              <Skeleton variant='text' width={220} sx={{ fontSize: '1.6rem' }} />
            ) : (
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                <Box sx={{ display: 'flex', alignItems: 'baseline', gap: 1.25 }}>
                  <Typography variant='h5' sx={{ lineHeight: 1.2 }}>
                    {formatNumber(totalBalance)}{' '}
                    <Box component='span' sx={{ fontSize: '0.7em', color: 'text.secondary', fontWeight: 600 }}>QU</Box>
                  </Typography>
                  {fiat && (
                    <Typography variant='body2' color='text.secondary'>
                      {fiat}
                    </Typography>
                  )}
                  {balanceStale && (
                    <Tooltip title='Peers have not confirmed every balance in the last poll. Showing the last values they agreed on; the chart pauses until they do.'>
                      <Chip
                        size='small'
                        color='warning'
                        variant='outlined'
                        label={staleAge ? `last known · ${staleAge} ago` : 'last known'}
                        sx={{ height: 20, fontSize: '0.68rem' }}
                      />
                    </Tooltip>
                  )}
                </Box>
                {history && history.length > 1 && <Sparkline points={history} width={110} height={30} label='24 h' />}
              </Box>
            )}
          </Box>
          <Box sx={{ flex: 1 }} />
          <UnencryptedPill count={unencryptedCount} onClick={() => onNav('wallet')} />
          <PendingPill pending={pending} />
          <UnlockPill secondsLeft={unlockSecondsLeft} />
          <StatusPill
            connected={connected}
            tick={tick}
            peersConnected={peersConnected}
            syncing={!isNumeric(tick)}
            health={health}
            peers={peers}
            now={now}
          />
          <Button variant='contained' startIcon={<SendIcon />} disabled={!canSend} onClick={onSend}>
            Send
          </Button>
          <Tooltip title={themeMode === 'light' ? 'Switch to dark theme' : 'Switch to light theme'}>
            <IconButton onClick={onToggleTheme} size='small'>
              {themeMode === 'light' ? <DarkModeIcon fontSize='small' /> : <LightModeIcon fontSize='small' />}
            </IconButton>
          </Tooltip>
        </Box>
        <Box
          component='main'
          sx={{
            flex: 1,
            minHeight: 0,
            display: 'flex',
            flexDirection: 'column',
            p: 3,
            maxWidth: 1400,
            width: '100%',
            mx: 'auto',
            // Sections size themselves to the window; this only kicks in when a
            // section's fixed part alone is taller than the window.
            overflow: 'auto',
          }}
        >
          {children}
        </Box>
      </Box>
    </Box>
  );
}
