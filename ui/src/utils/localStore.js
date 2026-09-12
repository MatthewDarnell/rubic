import { useCallback, useState } from 'react';

const read = (key, fallback) => {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch {
    return fallback;
  }
};

const write = (key, value) => {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Storage unavailable — value lives for this session only.
  }
};

// Small JSON-backed state in localStorage; only for local conveniences.
export const useStoredValue = (key, fallback) => {
  const [value, setValue] = useState(() => read(key, fallback));
  const update = useCallback(
    (next) => {
      setValue((prev) => {
        const resolved = typeof next === 'function' ? next(prev) : next;
        write(key, resolved);
        return resolved;
      });
    },
    [key]
  );
  return [value, update];
};

export const KEYS = {
  identityLabels: 'rubic.identityLabels',
  addressBook: 'rubic.addressBook',
  theme: 'rubic.theme',
  nav: 'rubic.nav',
  currency: 'rubic.currency',
  balanceHistory: 'rubic.balanceHistory',
  lastKnownBalances: 'rubic.lastKnownBalances',
  txNotes: 'rubic.txNotes',
};

export const readStored = (key, fallback) => read(key, fallback);
export const writeStored = (key, value) => write(key, value);
