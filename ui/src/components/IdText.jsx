import React from 'react';
import { Box, IconButton, Link, Tooltip } from '@mui/material';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import { useToast } from './ToastProvider';
import { EXPLORER, shortenId } from '../utils/format';
import { openExternal } from '../utils/openExternal';

export const useCopy = () => {
  const toast = useToast();
  return async (text, label = shortenId(text)) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`Copied ${label}`);
    } catch {
      toast.error('Could not copy to clipboard');
    }
  };
};

// Renders an identity / tx id / tick: truncated by default, full id in a tooltip,
// with copy and optional explorer link.
export default function IdText({
  id,
  full = false,
  head = 8,
  tail = 8,
  explorer, // 'address' | 'tx' | 'tick'
  copy = true,
  nowrap = false,
  sx,
}) {
  const copyToClipboard = useCopy();
  if (!id) return <Box component='span' sx={{ color: 'text.disabled' }}>—</Box>;
  const text = full ? id : shortenId(id, head, tail);
  return (
    <Box
      component='span'
      sx={{
        display: full ? 'inline' : 'inline-flex',
        alignItems: 'center',
        gap: 0.25,
        minWidth: 0,
        ...sx,
      }}
    >
      <Tooltip title={full ? '' : id} placement='top'>
        <Box
          component='span'
          sx={{
            fontFamily: 'monospace',
            wordBreak: full && !nowrap ? 'break-all' : 'normal',
            whiteSpace: nowrap ? 'nowrap' : 'normal',
          }}
        >
          {text}
        </Box>
      </Tooltip>
      {copy && (
        <Tooltip title='Copy'>
          <IconButton
            size='small'
            aria-label={`Copy ${shortenId(id)}`}
            onClick={(e) => {
              e.stopPropagation();
              copyToClipboard(id);
            }}
            sx={{ p: 0.25 }}
          >
            <ContentCopyIcon sx={{ fontSize: 14 }} />
          </IconButton>
        </Tooltip>
      )}
      {explorer && (
        <Tooltip title='Open in explorer'>
          <Link
            href={`${EXPLORER}/${explorer}/${id}`}
            target='_blank'
            rel='noopener'
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              openExternal(`${EXPLORER}/${explorer}/${id}`);
            }}
            sx={{ display: 'inline-flex', p: 0.25 }}
          >
            <OpenInNewIcon sx={{ fontSize: 14 }} />
          </Link>
        </Tooltip>
      )}
    </Box>
  );
}
