export const formatNumber = (value) =>
  String(value ?? '').replace(/\B(?=(\d{3})+(?!\d))/g, ',');

export const isNumeric = (value) =>
  value !== '' && value !== null && value !== undefined && !isNaN(Number(value));

export const formatRelativeTime = (unixSeconds) => {
  const ts = Number(unixSeconds);
  if (!ts) return 'never';
  const diff = Math.max(0, Math.floor(Date.now() / 1000) - ts);
  if (diff < 5) return 'just now';
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
};

export const formatAge = (ms) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  if (h < 48) return `${h} h`;
  return `${Math.floor(h / 24)} d`;
};

export const formatAbsoluteTime = (unixSeconds) => {
  const ts = Number(unixSeconds);
  if (!ts) return 'Never responded';
  return new Date(ts * 1000).toLocaleString();
};

export const shortenId = (id, head = 6, tail = 6) =>
  !id || id.length <= head + tail + 1
    ? id
    : `${id.slice(0, head)}…${id.slice(-tail)}`;

export const TIME_OPTIONS = [
  { label: '10 min', minutes: 10 },
  { label: '1 hour', minutes: 60 },
  { label: '1 day', minutes: 60 * 24 },
  { label: '1 week', minutes: 60 * 24 * 7 },
  { label: '1 month', minutes: 60 * 24 * 7 * 31 },
  { label: '1 year', minutes: 60 * 24 * 7 * 31 * 12 },
  { label: 'all', minutes: 60 * 24 * 7 * 31 * 12 * 10 },
];
export const ALL_TIME = TIME_OPTIONS[TIME_OPTIONS.length - 1].minutes;

// `created` comes from SQLite as "YYYY-MM-DD HH:MM:SS" in UTC.
export const parseCreated = (created) =>
  new Date(String(created ?? '').replace(' ', 'T') + 'Z');

export const withinLastMinutes = (created, minutes) =>
  Date.now() - parseCreated(created).getTime() < minutes * 60 * 1000;

export const digitsOnly = (value) => /^\d*$/.test(value);
export const upperOnly = (value) => /^[A-Z]*$/.test(value);

export const EXPLORER = 'https://explorer.qubic.org/network';

export const CURRENCIES = ['USD', 'EUR', 'GBP', 'CHF', 'JPY', 'AUD', 'CAD'];

// price = fiat per QU in `currency`. Returns null when no price is known.
export const formatFiat = (qu, price, currency = 'USD', { approx = true } = {}) => {
  if (!price || !isNumeric(qu)) return null;
  const value = Number(qu) * price;
  const text = new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
  return approx ? `≈ ${text}` : text;
};
