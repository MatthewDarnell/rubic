import React from 'react';
import {
  Box,
  Button,
  Chip,
  Divider,
  Drawer,
  IconButton,
  Stack,
  Tooltip,
  Typography,
} from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import SendIcon from '@mui/icons-material/Send';
import DriveFileRenameOutlineIcon from '@mui/icons-material/DriveFileRenameOutline';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutlined';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import { ToggleButton, ToggleButtonGroup } from '@mui/material';
import Identicon from './Identicon';
import IdText from './IdText';
import Sparkline from './Sparkline';
import { TxStatusChip } from './TxStatus';
import { formatFiat, formatNumber, isNumeric } from '../utils/format';
import { RANGES, sliceSince } from '../utils/balanceHistory';

const Section = ({ title, children }) => (
  <Box sx={{ px: 3, py: 2 }}>
    <Typography
      variant='caption'
      sx={{ display: 'block', mb: 1, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'text.secondary' }}
    >
      {title}
    </Typography>
    {children}
  </Box>
);

export default function IdentityDrawer({
  identity,
  label,
  price,
  currency,
  history,
  activity,
  latestTick,
  onClose,
  onSend,
  onRename,
  onDelete,
}) {
  const [range, setRange] = React.useState('7d');
  const open = Boolean(identity);
  const numeric = identity && isNumeric(identity.balance);
  const fiat = numeric ? formatFiat(identity.balance, price, currency) : null;
  const points = history ? sliceSince(history, RANGES[range]) : [];
  const assets = (identity?.assets || []).filter((a) => Number(a.balance) > 0);
  const recent = activity.slice(0, 20);

  return (
    <Drawer anchor='right' open={open} onClose={onClose} slotProps={{ paper: { sx: { width: 460, maxWidth: '100vw' } } }}>
      {identity && (
        <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2, px: 3, pt: 3, pb: 2 }}>
            <Identicon id={identity.id} size={48} />
            <Box sx={{ minWidth: 0, flex: 1 }}>
              <Typography variant='h6' sx={{ lineHeight: 1.2 }}>
                {label || 'Identity'}
              </Typography>
              <IdText id={identity.id} full explorer='address' sx={{ fontSize: '0.78rem', color: 'text.secondary' }} />
              <Box sx={{ mt: 1 }}>
                {identity.encrypted === 'true' ? (
                  <Chip size='small' icon={<LockIcon />} label='Seed encrypted' variant='outlined' />
                ) : (
                  <Chip size='small' icon={<LockOpenIcon />} label='Seed not encrypted' color='warning' variant='outlined' />
                )}
              </Box>
            </Box>
            <IconButton size='small' onClick={onClose} aria-label='Close'>
              <CloseIcon fontSize='small' />
            </IconButton>
          </Box>

          <Box sx={{ px: 3, pb: 2 }}>
            <Typography variant='caption' color='text.secondary'>
              Balance
            </Typography>
            <Typography variant='h5' sx={{ color: numeric ? 'text.primary' : 'text.secondary' }}>
              {numeric
                ? `${formatNumber(identity.balance)} QU`
                : identity.balance === 'Not Yet Reported'
                ? 'Syncing…'
                : identity.balance === 'Peer Balance Mismatch'
                ? 'Peers disagree'
                : String(identity.balance)}
            </Typography>
            {fiat && (
              <Typography variant='body2' color='text.secondary'>
                {fiat}
              </Typography>
            )}
            {identity.stale && (
              <Typography variant='caption' sx={{ color: 'warning.main', display: 'block' }}>
                Last known balance — peers have not confirmed it in the latest poll.
              </Typography>
            )}
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mt: 1.5 }}>
              <Sparkline points={points} width={240} height={48} label={range} />
              <ToggleButtonGroup exclusive size='small' value={range} onChange={(_e, v) => v && setRange(v)}>
                {Object.keys(RANGES).map((r) => (
                  <ToggleButton key={r} value={r} sx={{ px: 1.25, py: 0.25, fontSize: '0.72rem' }}>
                    {r}
                  </ToggleButton>
                ))}
              </ToggleButtonGroup>
            </Box>
            <Stack direction='row' spacing={1} sx={{ mt: 2 }}>
              <Button variant='contained' startIcon={<SendIcon />} onClick={() => onSend(identity.id)}>
                Send
              </Button>
              <Button variant='outlined' startIcon={<DriveFileRenameOutlineIcon />} onClick={() => onRename(identity.id)}>
                {label ? 'Rename' : 'Nickname'}
              </Button>
              <Box sx={{ flex: 1 }} />
              <Tooltip title='Delete identity'>
                <IconButton color='error' onClick={() => onDelete(identity.id)} aria-label='Delete identity'>
                  <DeleteOutlineIcon />
                </IconButton>
              </Tooltip>
            </Stack>
          </Box>

          <Divider />
          <Section title='Assets'>
            {assets.length === 0 ? (
              <Typography variant='body2' color='text.secondary'>
                No assets held.
              </Typography>
            ) : (
              <Stack spacing={0.75}>
                {assets.map((a) => (
                  <Box key={a.name} sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography sx={{ fontWeight: 600 }}>{a.name}</Typography>
                    <Typography>{formatNumber(a.balance)}</Typography>
                  </Box>
                ))}
              </Stack>
            )}
          </Section>

          <Divider />
          <Section title={`Recent activity${activity.length ? ` · ${activity.length}` : ''}`}>
            {recent.length === 0 ? (
              <Typography variant='body2' color='text.secondary'>
                No activity for this identity yet.
              </Typography>
            ) : (
              <Stack spacing={1.25} sx={{ overflow: 'auto' }}>
                {recent.map((item) => (
                  <Box key={item.key} sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                    <Box sx={{ minWidth: 0, flex: 1 }}>
                      <Typography variant='body2' sx={{ fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {item.title}
                      </Typography>
                      <Typography variant='caption' color='text.secondary'>
                        {item.created}
                      </Typography>
                    </Box>
                    <TxStatusChip status={item.status} tick={item.tick} latestTick={latestTick} />
                  </Box>
                ))}
              </Stack>
            )}
          </Section>
        </Box>
      )}
    </Drawer>
  );
}
