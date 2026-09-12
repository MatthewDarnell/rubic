import React, { useMemo, useState } from 'react';
import { Box, Typography, useTheme } from '@mui/material';
import { formatNumber } from '../utils/format';

// levels: { bids: [{price, volume}] sorted desc, asks: [{price, volume}] sorted asc }
export default function DepthChart({ bids, asks, asset, width = 380, height = 300 }) {
  const theme = useTheme();
  const [hover, setHover] = useState(null);

  const chart = useMemo(() => {
    const cum = (levels) => {
      let total = 0;
      return levels.map((l) => {
        total += l.volume;
        return { ...l, cumulative: total };
      });
    };
    const bidCum = cum(bids);
    const askCum = cum(asks);
    if (bidCum.length === 0 && askCum.length === 0) return null;

    const prices = [...bidCum, ...askCum].map((l) => l.price);
    const minP = Math.min(...prices);
    const maxP = Math.max(...prices);
    const maxV = Math.max(1, ...bidCum.map((l) => l.cumulative), ...askCum.map((l) => l.cumulative));
    const bestBid = bidCum[0]?.price;
    const bestAsk = askCum[0]?.price;
    const mid =
      bestBid !== undefined && bestAsk !== undefined
        ? (bestBid + bestAsk) / 2
        : bestBid ?? bestAsk;

    const pad = { top: 12, right: 12, bottom: 28, left: 12 };
    const w = width - pad.left - pad.right;
    const h = height - pad.top - pad.bottom;
    const span = maxP - minP || 1;
    const x = (p) => pad.left + ((p - minP) / span) * w;
    const y = (v) => pad.top + h - (v / maxV) * h;

    // Step paths: bids extend leftwards from the mid, asks rightwards.
    const bidPath = () => {
      if (bidCum.length === 0) return '';
      let d = `M ${x(mid)} ${y(0)} L ${x(mid)} ${y(bidCum[0].cumulative)}`;
      bidCum.forEach((l, i) => {
        const next = bidCum[i + 1];
        d += ` L ${x(l.price)} ${y(l.cumulative)}`;
        if (next) d += ` L ${x(l.price)} ${y(next.cumulative)}`;
      });
      const last = bidCum[bidCum.length - 1];
      d += ` L ${x(last.price)} ${y(0)} Z`;
      return d;
    };
    const askPath = () => {
      if (askCum.length === 0) return '';
      let d = `M ${x(mid)} ${y(0)} L ${x(mid)} ${y(askCum[0].cumulative)}`;
      askCum.forEach((l, i) => {
        const next = askCum[i + 1];
        d += ` L ${x(l.price)} ${y(l.cumulative)}`;
        if (next) d += ` L ${x(l.price)} ${y(next.cumulative)}`;
      });
      const last = askCum[askCum.length - 1];
      d += ` L ${x(last.price)} ${y(0)} Z`;
      return d;
    };

    return { bidCum, askCum, minP, maxP, maxV, mid, x, y, pad, w, h, bidPath: bidPath(), askPath: askPath() };
  }, [bids, asks, width, height]);

  const bidColor = theme.palette.success.main;
  const askColor = theme.palette.error.main;

  if (!chart) {
    return (
      <Box sx={{ width, height, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Typography variant='body2' color='text.secondary'>
          No depth to show.
        </Typography>
      </Box>
    );
  }

  const onMove = (e) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const px = ((e.clientX - rect.left) / rect.width) * width;
    const price = chart.minP + ((px - chart.pad.left) / chart.w) * (chart.maxP - chart.minP);
    const side = price < chart.mid ? 'bid' : 'ask';
    const levels = side === 'bid' ? chart.bidCum : chart.askCum;
    // Cumulative depth at this price: every level at or better than it.
    const hit = levels.filter((l) => (side === 'bid' ? l.price >= price : l.price <= price));
    const level = hit[hit.length - 1];
    setHover(level ? { side, level, px: chart.x(level.price) } : null);
  };

  return (
    <Box sx={{ position: 'relative', width, userSelect: 'none' }}>
      <svg
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        onMouseMove={onMove}
        onMouseLeave={() => setHover(null)}
        style={{ display: 'block', cursor: 'crosshair' }}
      >
        <path d={chart.bidPath} fill={bidColor} fillOpacity={0.25} stroke={bidColor} strokeWidth={1.5} />
        <path d={chart.askPath} fill={askColor} fillOpacity={0.25} stroke={askColor} strokeWidth={1.5} />
        <line
          x1={chart.x(chart.mid)}
          x2={chart.x(chart.mid)}
          y1={chart.pad.top}
          y2={chart.pad.top + chart.h}
          stroke={theme.palette.divider}
          strokeDasharray='3 3'
        />
        <line
          x1={chart.pad.left}
          x2={chart.pad.left + chart.w}
          y1={chart.pad.top + chart.h}
          y2={chart.pad.top + chart.h}
          stroke={theme.palette.divider}
        />
        <text x={chart.pad.left} y={height - 10} fontSize={10} fill={theme.palette.text.secondary}>
          {formatNumber(chart.minP)}
        </text>
        <text
          x={chart.x(chart.mid)}
          y={height - 10}
          fontSize={10}
          textAnchor='middle'
          fill={theme.palette.text.secondary}
        >
          {formatNumber(Math.round(chart.mid))}
        </text>
        <text
          x={chart.pad.left + chart.w}
          y={height - 10}
          fontSize={10}
          textAnchor='end'
          fill={theme.palette.text.secondary}
        >
          {formatNumber(chart.maxP)}
        </text>
        <text x={chart.pad.left + 2} y={chart.pad.top + 10} fontSize={10} fill={theme.palette.text.secondary}>
          {formatNumber(chart.maxV)} {asset}
        </text>
        {hover && (
          <>
            <line
              x1={hover.px}
              x2={hover.px}
              y1={chart.pad.top}
              y2={chart.pad.top + chart.h}
              stroke={hover.side === 'bid' ? bidColor : askColor}
              strokeWidth={1}
            />
            <circle
              cx={hover.px}
              cy={chart.y(hover.level.cumulative)}
              r={3.5}
              fill={hover.side === 'bid' ? bidColor : askColor}
            />
          </>
        )}
      </svg>
      <Box sx={{ minHeight: 40, px: 0.5 }}>
        {hover ? (
          <Typography variant='caption' sx={{ color: hover.side === 'bid' ? bidColor : askColor }}>
            {hover.side === 'bid' ? 'Bids' : 'Asks'} to {formatNumber(hover.level.price)} QU:{' '}
            <b>{formatNumber(hover.level.cumulative)}</b> {asset} · {formatNumber(hover.level.volume)} at this price
          </Typography>
        ) : (
          <Typography variant='caption' color='text.secondary'>
            Cumulative volume by price — hover for details
          </Typography>
        )}
      </Box>
    </Box>
  );
}
