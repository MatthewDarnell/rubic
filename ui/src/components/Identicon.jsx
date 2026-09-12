import React, { useMemo } from 'react';
import { Box } from '@mui/material';

// FNV-1a: cheap, deterministic, spreads similar IDs apart.
const hash = (s) => {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h;
};

// GitHub-style 5x5 symmetric identicon so identities are recognisable at a glance.
export default function Identicon({ id, size = 28, sx }) {
  const { cells, hue } = useMemo(() => {
    const h = hash(id || '');
    let bits = h;
    const cells = [];
    for (let x = 0; x < 3; x++) {
      for (let y = 0; y < 5; y++) {
        bits = Math.imul(bits ^ (bits >>> 13), 0x5bd1e995) >>> 0;
        if (bits & 1) {
          cells.push([x, y]);
          if (x < 2) cells.push([4 - x, y]);
        }
      }
    }
    return { cells, hue: h % 360 };
  }, [id]);

  const fg = `hsl(${hue} 70% 60%)`;
  const bg = `hsl(${hue} 40% 18%)`;
  return (
    <Box
      component='svg'
      viewBox='0 0 5 5'
      width={size}
      height={size}
      shapeRendering='crispEdges'
      aria-hidden
      sx={{ borderRadius: '22%', flexShrink: 0, display: 'block', ...sx }}
    >
      <rect width='5' height='5' fill={bg} />
      {cells.map(([x, y]) => (
        <rect key={`${x}-${y}`} x={x} y={y} width='1' height='1' fill={fg} />
      ))}
    </Box>
  );
}
