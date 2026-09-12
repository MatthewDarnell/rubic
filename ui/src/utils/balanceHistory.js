import { KEYS } from './localStore';

const MAX_TOTAL_POINTS = 4000;
const MAX_ID_POINTS = 1000;
const MIN_INTERVAL_MS = 30 * 1000;

const read = () => {
  try {
    return JSON.parse(localStorage.getItem(KEYS.balanceHistory)) || { total: [], byId: {} };
  } catch {
    return { total: [], byId: {} };
  }
};

const write = (h) => {
  try {
    localStorage.setItem(KEYS.balanceHistory, JSON.stringify(h));
  } catch {
    // Storage full or unavailable — history is a convenience only.
  }
};

const append = (series, t, v, cap) => {
  const last = series[series.length - 1];
  // Only store a point when the value changes or enough time has passed, so the
  // series stays small but always has a recent anchor.
  if (last && last[1] === v && t - last[0] < MIN_INTERVAL_MS * 10) return series;
  if (last && last[1] !== v && t - last[0] < 1000) return series;
  const next = [...series, [t, v]];
  return next.length > cap ? next.slice(next.length - cap) : next;
};

// Records one snapshot: total balance plus each identity's numeric balance.
export const recordBalances = (total, identities) => {
  const t = Date.now();
  const h = read();
  h.total = append(h.total, t, total, MAX_TOTAL_POINTS);
  const known = new Set(identities.map((i) => i.id));
  identities.forEach((i) => {
    if (!Number.isFinite(Number(i.balance)) || i.balance === '') return;
    h.byId[i.id] = append(h.byId[i.id] || [], t, Number(i.balance), MAX_ID_POINTS);
  });
  Object.keys(h.byId).forEach((id) => {
    if (!known.has(id)) delete h.byId[id];
  });
  write(h);
  return h;
};

export const readHistory = read;

export const sliceSince = (series, ms) => {
  const cutoff = Date.now() - ms;
  const idx = series.findIndex((p) => p[0] >= cutoff);
  if (idx <= 0) return series;
  // Keep one point before the window so the line starts at the edge.
  return series.slice(idx - 1);
};

export const RANGES = {
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
};
