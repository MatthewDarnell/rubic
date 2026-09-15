import React from 'react';
import { Chip, IconButton, Tooltip } from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import HourglassTopIcon from '@mui/icons-material/HourglassTop';
import ErrorOutlineIcon from '@mui/icons-material/ErrorOutlined';
import HelpOutlineIcon from '@mui/icons-material/HelpOutlined';
import ReplayIcon from '@mui/icons-material/Replay';
import { isNumeric } from '../utils/format';

// DB status codes: '0' confirmed, '-1' pending, anything else failed.
// A pending transaction whose tick has passed is "unconfirmed": the wallet has
// not verified the outcome yet. It may well have executed, so it is not called
// failed - only the server decides that - but it can be resent if it did not.
export const txState = (status, tick, latestTick) => {
  if (status === '0') return 'confirmed';
  if (status === '-1') {
    if (isNumeric(tick) && isNumeric(latestTick) && Number(latestTick) > Number(tick)) return 'unconfirmed';
    return 'pending';
  }
  return 'failed';
};

export const canRetry = (state) => state === 'failed' || state === 'unconfirmed';

export const UNCONFIRMED_HINT =
  'The tick has passed but the wallet could not verify the result yet. Check the balance before resending.';

const TITLES = {
  confirmed: 'Confirmed',
  pending: 'Pending — waiting for the tick',
  unconfirmed: `Unconfirmed — ${UNCONFIRMED_HINT}`,
  failed: 'Failed',
};

export default function TxStatus({ status, tick, latestTick, onRetry, retryLabel = 'Resend' }) {
  const state = txState(status, tick, latestTick);
  if (state === 'confirmed')
    return (
      <Tooltip title={TITLES.confirmed}>
        <CheckCircleIcon fontSize='small' color='success' />
      </Tooltip>
    );
  if (state === 'pending')
    return (
      <Tooltip title={TITLES.pending}>
        <HourglassTopIcon fontSize='small' color='warning' />
      </Tooltip>
    );
  return (
    <>
      <Tooltip title={TITLES[state]}>
        {state === 'unconfirmed' ? (
          <HelpOutlineIcon fontSize='small' color='warning' />
        ) : (
          <ErrorOutlineIcon fontSize='small' color='error' />
        )}
      </Tooltip>
      {onRetry && (
        <Tooltip title={state === 'unconfirmed' ? `${retryLabel} (only if it did not go through)` : retryLabel}>
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
    unconfirmed: { color: 'warning', label: 'Unconfirmed' },
    failed: { color: 'error', label: 'Failed' },
  }[state];
  const chip = <Chip size='small' variant='outlined' {...props} />;
  return state === 'unconfirmed' ? <Tooltip title={UNCONFIRMED_HINT}>{chip}</Tooltip> : chip;
};
