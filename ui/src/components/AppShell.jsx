import React from 'react';
import {
  Badge,
  Box,
  Button,
  Chip,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
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

const StatusPill = ({ connected, tick, peersConnected, syncing }) => {
  const noPeers = connected && peersConnected === 0;
  const label = !connected
    ? 'Server unreachable'
    : noPeers
    ? `No peers connected${!syncing ? ` · tick ${formatNumber(tick)}` : ''}`
    : syncing
    ? 'Waiting for first tick'
    : `Tick ${formatNumber(tick)} · ${peersConnected} peer${peersConnected === 1 ? '' : 's'}`;
  const color = !connected ? 'error.main' : noPeers || syncing ? 'warning.main' : 'success.main';
  return (
    <Tooltip
      title={
        !connected
          ? 'Cannot reach the local Rubic server'
          : noPeers
          ? 'Rubic is not talking to any peer right now; balances and ticks will not update until it reconnects'
          : syncing
          ? 'Connected to the server, but no peer has reported a tick yet'
          : 'Latest tick reported by connected peers'
      }
    >
      <Box
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
        }}
      >
        <Box sx={{ width: 8, height: 8, borderRadius: '50%', bgcolor: color, boxShadow: `0 0 0 3px ${'rgba(0,0,0,0.0)'}` }} />
        <Typography variant='body2' sx={{ fontWeight: 500 }}>
          {label}
        </Typography>
      </Box>
    </Tooltip>
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
          <StatusPill connected={connected} tick={tick} peersConnected={peersConnected} syncing={!isNumeric(tick)} />
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
