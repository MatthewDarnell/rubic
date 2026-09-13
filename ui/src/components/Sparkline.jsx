import React, { useMemo } from 'react';
import { Box, Tooltip, useTheme } from '@mui/material';
import { formatNumber } from '../utils/format';

// points: [[unixMs, value], ...] ascending. Draws a tiny line + area with the last point highlighted.
export default function Sparkline({ points, width = 120, height = 32, label }) {
  const theme = useTheme();
  const geo = useMemo(() => {
    if (!points || points.length < 2) return null;
    const xs = points.map((p) => p[0]);
    const ys = points.map((p) => p[1]);
    const minX = xs[0];
    const maxX = xs[xs.length - 1];
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const spanX = maxX - minX || 1;
    const spanY = maxY - minY || 1;
    const pad = 2;
    const x = (t) => pad + ((t - minX) / spanX) * (width - pad * 2);
    const y = (v) => pad + (1 - (v - minY) / spanY) * (height - pad * 2);
    const line = points.map((p, i) => `${i ? 'L' : 'M'} ${x(p[0]).toFixed(1)} ${y(p[1]).toFixed(1)}`).join(' ');
    const area = `${line} L ${x(maxX).toFixed(1)} ${height - pad} L ${x(minX).toFixed(1)} ${height - pad} Z`;
    const first = ys[0];
    const last = ys[ys.length - 1];
    return { line, area, lastX: x(maxX), lastY: y(last), up: last >= first, first, last, minY, maxY };
  }, [points, width, height]);

  if (!geo) {
    return (
      <Box sx={{ width, height, display: 'flex', alignItems: 'center', color: 'text.disabled', fontSize: '0.7rem' }}>
        {label ? `${label}: not enough history yet` : 'not enough history yet'}
      </Box>
    );
  }

  const color = geo.up ? theme.palette.success.main : theme.palette.error.main;
  const delta = geo.last - geo.first;
  const title = `${label ? label + ': ' : ''}${formatNumber(geo.first)} → ${formatNumber(geo.last)} QU (${delta >= 0 ? '+' : '−'}${formatNumber(Math.abs(delta))})`;

  return (
    <Tooltip title={title}>
      <Box component='svg' width={width} height={height} viewBox={`0 0 ${width} ${height}`} sx={{ display: 'block' }}>
        <path d={geo.area} fill={color} fillOpacity={0.15} />
        <path d={geo.line} fill='none' stroke={color} strokeWidth={1.5} strokeLinejoin='round' />
        <circle cx={geo.lastX} cy={geo.lastY} r={2.5} fill={color} />
      </Box>
    </Tooltip>
  );
}
