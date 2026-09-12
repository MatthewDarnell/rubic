import React from 'react';
import { Chip, IconButton, Tooltip } from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import HourglassTopIcon from '@mui/icons-material/HourglassTop';
import ErrorOutlineIcon from '@mui/icons-material/ErrorOutline';
import ReplayIcon from '@mui/icons-material/Replay';
import { isNumeric } from '../utils/format';

// DB status codes: '0' confirmed, '-1' pending, anything else failed.
// A pending transaction whose tick has already passed can no longer be included,
// so it is reported as failed (and can be retried) rather than pending forever.
export const txState = (status, tick, latestTick) => {
  if (status === '0') return 'confirmed';
  if (status === '-1') {
    if (isNumeric(tick) && isNumeric(latestTick) && Number(latestTick) > Number(tick)) return 'failed';
    return 'pending';
  }
  return 'failed';
};

export const isExpired = (status, tick, latestTick) =>
  status === '-1' && txState(status, tick, latestTick) === 'failed';

const failedTitle = (status, tick, latestTick) =>
  isExpired(status, tick, latestTick) ? 'Tick passed without confirmation' : 'Failed';

export default function TxStatus({ status, tick, latestTick, onRetry, retryLabel = 'Resend' }) {
  const state = txState(status, tick, latestTick);
  if (state === 'confirmed')
    return (
      <Tooltip title='Confirmed'>
        <CheckCircleIcon fontSize='small' color='success' />
      </Tooltip>
    );
  if (state === 'pending')
    return (
      <Tooltip title='Pending — waiting for the tick'>
        <HourglassTopIcon fontSize='small' color='warning' />
      </Tooltip>
    );
  return (
    <>
      <Tooltip title={failedTitle(status, tick, latestTick)}>
        <ErrorOutlineIcon fontSize='small' color='error' />
      </Tooltip>
      {onRetry && (
        <Tooltip title={retryLabel}>
          <IconButton
            size='small'
            aria-label={retryLabel}
            onClick={(e) => {
              e.stopPropagation();
              onRetry();
            }}
            sx={{ p: 0.25 }}
          >
            <ReplayIcon fontSize='small' color='warning' />
          </IconButton>
        </Tooltip>
      )}
    </>
  );
}

export const TxStatusChip = ({ status, tick, latestTick }) => {
  const state = txState(status, tick, latestTick);
  const props = {
    confirmed: { color: 'success', label: 'Confirmed' },
    pending: { color: 'warning', label: 'Pending' },
    failed: { color: 'error', label: 'Failed' },
  }[state];
  const chip = <Chip size='small' variant='outlined' {...props} />;
  return state === 'failed' ? <Tooltip title={failedTitle(status, tick, latestTick)}>{chip}</Tooltip> : chip;
};
