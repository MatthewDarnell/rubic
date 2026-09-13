import React from 'react';
import { Chip, Stack } from '@mui/material';
import { TIME_OPTIONS } from '../utils/format';

export default function TimeFilterChips({ value, onChange }) {
  return (
    <Stack direction='row' spacing={1} sx={{ mb: 2, flexShrink: 0 }} flexWrap='wrap' useFlexGap>
      {TIME_OPTIONS.map((option) => (
        <Chip
          key={option.label}
          label={option.label}
          clickable
          size='small'
          color={value === option.minutes ? 'primary' : 'default'}
          variant={value === option.minutes ? 'filled' : 'outlined'}
          onClick={() => onChange(option.minutes)}
        />
      ))}
    </Stack>
  );
}
