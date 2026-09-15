import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Alert, Box, CssBaseline, ThemeProvider, Typography } from '@mui/material';

import { apiCall, apiPost, backgroundCall } from './api';
import { serverIp } from './api_config';
import { doArrayElementsAgree } from './api_helper';
import { getTheme } from './theme';
import { useStoredValue, readStored, writeStored, KEYS } from './utils/localStore';
import { useToast, ToastViewport } from './components/ToastProvider';
import ConfirmDialog from './components/ConfirmDialog';
import PasswordDialog from './components/PasswordDialog';
import LockScreen from './components/LockScreen';
import AppShell from './components/AppShell';
import WalletPanel from './components/WalletPanel';
import ActivityPanel, { normalizeActivity } from './components/ActivityPanel';
import { txState } from './components/TxStatus';
import IdentityDrawer from './components/IdentityDrawer';
import RenameDialog from './components/RenameDialog';
import AssetsPanel from './components/AssetsPanel';
import QxPanel from './components/QxPanel';
import PeersPanel from './components/PeersPanel';
import SettingsPanel from './components/SettingsPanel';
import SendDialog from './components/SendDialog';
import ImportDbWizard from './components/ImportDbWizard';
import BulkImportDialog from './components/BulkImportDialog';
import IdText from './components/IdText';
import ErrorBoundary from './components/ErrorBoundary';
import CommandPalette from './components/CommandPalette';
import { NAV } from './components/AppShell';
import { formatNumber, isNumeric } from './utils/format';
import { recordBalances, readHistory, sliceSince, RANGES } from './utils/balanceHistory';

const POLLING_INTERVAL = 3000;
const TICK_INTERVAL = 1000;
// Transactions expire this many ticks after the current one unless the user
// overrides it in Settings ("Transfer Ticks Offset"). The wallet's own view of
// the current tick trails the network by 10-20 ticks (it learns a new tick only
// every few seconds), so anything much below 30 is already in the past by the
// time computors see it. The original UI used 30 for this reason.
const DEFAULT_TICK_OFFSET = 30;
const UNLOCK_CHECK_INTERVAL = 5000;
const OPEN_ORDERS_INTERVAL = 5000; // poll of the server's QX-reported open orders
const BOOK_VIEW_INTERVAL = 1000; // poll of the order book on screen in QX Exchange (a local read)
const HEALTH_INTERVAL = 5000;
const MIN_PEERS_FOR_SENDING = 3;
const ASSETS_RETRY_INTERVAL = 5000; // while no issued assets are known yet
const ASSETS_REFRESH_INTERVAL = 5 * 60 * 1000;

const INVALID_PASSWORD_RESPONSES = [
  'Invalid Password',
  'Invalid Password!',
  'Must Enter A Password!',
  'Must Supply Password To Delete Encrypted Identity',
];

const asList = (res) =>
  res && res.success && Array.isArray(res.data) ? res.data : [];

const looksLikeFailure = (data) =>
  /invalid|fail|error|must |too short|too long|not found|can't/i.test(String(data ?? ''));

const describeAction = (action) => {
  if (action.startsWith('identity/delete')) return 'Identity deleted';
  if (action.startsWith('identity/add')) return 'Identity imported';
  if (action.startsWith('identity/new')) return 'New identity created';
  if (action.startsWith('asset/transfer')) return 'Asset transfer sent';
  if (action.startsWith('transfer/')) return 'Transfer sent';
  if (action.startsWith('qx/order')) return 'QX order submitted';
  if (action.includes('wallet/download')) return 'Wallet exported';
  return 'Done';
};

const MainView = () => {
  const toast = useToast();

  // Connection / chain state
  const [connected, setConnected] = useState(true);
  const [latestTick, setLatestTick] = useState(0);
  const [orderTick, setOrderTick] = useState(0);
  const [showProgress, setShowProgress] = useState(false);
  const [price, setPrice] = useState(0);
  const [priceStatus, setPriceStatus] = useState('loading'); // loading | ok | error
  const [now, setNow] = useState(() => Date.now());

  // Wallet data
  const [isEncrypted, setIsEncrypted] = useState(false);
  const [identities, setIdentities] = useState([]);
  const [totalBalance, setTotalBalance] = useState(0);
  const [transfers, setTransfers] = useState([]);
  const [assetTransfers, setAssetTransfers] = useState([]);
  const [qxOrders, setQxOrders] = useState([]);
  const [peers, setPeers] = useState([]);
  const [assetsNIssuer, setAssetsNIssuer] = useState(new Map());
  const [askOrders, setAskOrders] = useState([]);
  const [bidOrders, setBidOrders] = useState([]);
  const [bookAsset, setBookAsset] = useState(null); // asset the ask/bid arrays belong to
  const [bookAge, setBookAge] = useState({ ask: null, bid: null }); // seconds since each side was last received
  const [health, setHealth] = useState(null); // server request-queue health (see /health)
  const [selectedAsset, setSelectedAsset] = useState(null);
  const [selectedId, setSelectedId] = useState('');

  // Settings & local preferences
  const [unlockTimer, setUnlockTimer] = useState('60000');
  const [tickOffset, setTickOffset] = useStoredValue(KEYS.tickOffset, DEFAULT_TICK_OFFSET);
  const [allowNonEncrypted, setAllowNonEncrypted] = useState(false);
  const [peerLimits, setPeerLimits] = useState({ min: 3, max: 8 });
  const [labels, setLabels] = useStoredValue(KEYS.identityLabels, {});
  const [themeMode, setThemeMode] = useStoredValue(KEYS.theme, 'dark');
  const [nav, setNav] = useStoredValue(KEYS.nav, 'wallet');
  const [currency, setCurrency] = useStoredValue(KEYS.currency, 'USD');
  const [loading, setLoading] = useState(true);
  const [history, setHistory] = useState(() => readHistory());
  const [paletteOpen, setPaletteOpen] = useState(false);

  // Password-gated action flow
  const [action, setAction] = useState('');
  const [invalidPassword, setInvalidPassword] = useState('');
  const [actionBusy, setActionBusy] = useState(false);
  const [confirm, setConfirm] = useState(null);
  const [unlockedUntil, setUnlockedUntil] = useState(0);
  const [send, setSend] = useState({ open: false, from: '' });
  const [importOpen, setImportOpen] = useState(false);
  const [bulkOpen, setBulkOpen] = useState(false);
  const [detailId, setDetailId] = useState(null);
  const [renameTarget, setRenameTarget] = useState(null);
  const [openOrders, setOpenOrders] = useState([]);
  const txStatusRef = useRef(null); // txid -> derived state, null until the first poll
  const latestTickRef = useRef(0);
  // id -> { balance, at }: last value peers agreed on; persisted so a restart without peers is not "Syncing…".
  const lastGoodBalanceRef = useRef(new Map(Object.entries(readStored(KEYS.lastKnownBalances, {}))));
  const stableTotalRef = useRef({ total: null, count: 0 }); // consecutive identical fresh readings
  const [balanceStale, setBalanceStale] = useState(false);
  const [balanceStaleSince, setBalanceStaleSince] = useState(null);
  const [addressBook, setAddressBook] = useStoredValue(KEYS.addressBook, []);

  const theme = useMemo(() => getTheme(themeMode === 'light' ? 'light' : 'dark'), [themeMode]);
  const ownIds = useMemo(() => new Set(identities.map((i) => i.id)), [identities]);

  const setLabel = useCallback(
    (id, label) =>
      setLabels((prev) => {
        const next = { ...prev };
        if (label) next[id] = label;
        else delete next[id];
        return next;
      }),
    [setLabels]
  );

  // ---------- data fetching ----------

  const refreshPeers = useCallback(async () => {
    const res = await apiCall('peers');
    if (res.success && Array.isArray(res.data)) setPeers(res.data);
    return res;
  }, []);

  // The asset the user is looking at right now. Book responses are only applied
  // if they belong to it, so a slow reply for the previous asset can never
  // overwrite the book after a switch.
  const viewedAssetRef = useRef(null);
  const fetchOrderbook = useCallback(async (asset, { interactive = false } = {}) => {
    if (!asset) return;
    const call = interactive ? apiCall : backgroundCall;
    // `refresh=1`: this is the book on screen, so the server keeps re-fetching it
    // from peers once per tick, ahead of its background sweep.
    const [ask, bid] = await Promise.all([
      call(`qx/orderbook/${asset}/ASK/1000/0?refresh=1`),
      call(`qx/orderbook/${asset}/BID/1000/0?refresh=1`),
    ]);
    if (viewedAssetRef.current !== asset) return;
    setAskOrders(asList(ask));
    setBidOrders(asList(bid));
    setBookAsset(asset);
  }, []);

  const fetchPeerLimits = useCallback(async () => {
    const res = await apiCall('peers/limit');
    if (res.success && res.data && typeof res.data === 'object') {
      setPeerLimits({ min: Number(res.data.min), max: Number(res.data.max) });
    }
  }, []);

  // Whether a master password exists decides which screen is shown; re-checked
  // after an import that resets the database.
  const refreshEncrypted = useCallback(async () => {
    const encrypted = await apiCall('wallet/is_encrypted');
    setIsEncrypted(typeof encrypted.data === 'boolean');
  }, []);

  useEffect(() => {
    const init = async () => {
      await refreshEncrypted();
      await fetchPeerLimits();
    };
    init();
  }, [fetchPeerLimits, refreshEncrypted]);

  // Issued assets are only known once peers have reported them, which on a fresh
  // wallet can be a while after startup. Keep asking until the list arrives, then
  // refresh it slowly so newly issued assets show up without a restart.
  useEffect(() => {
    let cancelled = false;
    let timer;
    const load = async () => {
      const assets = await backgroundCall('asset/issued');
      if (cancelled) return;
      const pairs = asList(assets).reduce(
        (acc, val, idx, arr) => (idx % 2 === 0 ? [...acc, [val, arr[idx + 1]]] : acc),
        []
      );
      const map = new Map(pairs);
      if (map.size > 0) {
        setAssetsNIssuer((prev) =>
          prev.size === map.size && [...map].every(([k, v]) => prev.get(k) === v) ? prev : map
        );
        setSelectedAsset((current) => (current && map.has(current) ? current : [...map.keys()].sort()[0] ?? null));
      }
      timer = setTimeout(load, map.size > 0 ? ASSETS_REFRESH_INTERVAL : ASSETS_RETRY_INTERVAL);
    };
    load();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    const fetchPrice = async () => {
      try {
        const vs = currency.toLowerCase();
        const response = await fetch(
          `https://api.coingecko.com/api/v3/simple/price?ids=qubic-network&vs_currencies=${vs}`
        );
        const data = await response.json();
        const next = Number(data['qubic-network']?.[vs]) || 0;
        if (cancelled) return;
        setPrice(next);
        setPriceStatus(next > 0 ? 'ok' : 'error');
      } catch {
        // Price is decorative: fiat values simply stay hidden. Settings shows why.
        if (!cancelled) setPriceStatus('error');
      }
    };
    fetchPrice();
    const intervalId = setInterval(fetchPrice, 5 * 60 * 1000);
    return () => {
      cancelled = true;
      clearInterval(intervalId);
    };
  }, [currency]);

  // Keyboard shortcuts: Ctrl/Cmd+K palette, "/" focuses the section filter, 1–6 switch sections.
  useEffect(() => {
    const onKey = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((v) => !v);
        return;
      }
      const target = e.target;
      const typing =
        target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable);
      if (typing || e.ctrlKey || e.metaKey || e.altKey) return;
      if (document.querySelector('[role="dialog"]')) return;
      if (e.key === '/') {
        const input = document.querySelector('[data-filter-input]');
        if (input) {
          e.preventDefault();
          input.focus();
        }
      } else if (/^[1-6]$/.test(e.key)) {
        setNav(NAV[Number(e.key) - 1].key);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [setNav]);

  // Pollers reschedule themselves after each cycle completes, so a slow cycle
  // never overlaps the next one and requests cannot pile up.
  useEffect(() => {
    let cancelled = false;
    let timer;
    const run = async () => {
      // One request a second that drives the tick display: skip the background queue.
      const tick = await apiCall('tick');
      if (cancelled) return;
      setConnected(tick.success);
      setLatestTick(tick.data);
      latestTickRef.current = tick.data;
      setShowProgress(orderTick >= tick.data);
      setNow(Date.now());
      timer = setTimeout(run, TICK_INTERVAL);
    };
    run();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [orderTick]);

  // Keep the unlock indicator honest if the server locked earlier than expected.
  useEffect(() => {
    if (!unlockedUntil) return undefined;
    let cancelled = false;
    let timer;
    const run = async () => {
      const res = await backgroundCall('wallet/unlocked');
      if (cancelled) return;
      if (res.success && res.data !== true) setUnlockedUntil(0);
      else timer = setTimeout(run, UNLOCK_CHECK_INTERVAL);
    };
    timer = setTimeout(run, UNLOCK_CHECK_INTERVAL);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [unlockedUntil]);

  useEffect(() => {
    viewedAssetRef.current = selectedAsset;
    // Drop the previous asset's book at once and load the new one ahead of the polls.
    setAskOrders([]);
    setBidOrders([]);
    fetchOrderbook(selectedAsset, { interactive: true });
  }, [selectedAsset, fetchOrderbook]);

  // While the QX Exchange tab is open, keep the displayed book fresh: each poll
  // carries `refresh=1`, which keeps the server re-fetching this book from peers
  // once per tick; the poll itself only reads the server's local snapshot, so
  // once a second is cheap. Other tabs don't poll the book at all.
  useEffect(() => {
    if (nav !== 'exchange' || !selectedAsset) return undefined;
    let cancelled = false;
    let timer;
    setBookAge({ ask: null, bid: null });
    const run = async () => {
      const [, age] = await Promise.all([
        fetchOrderbook(selectedAsset, { interactive: true }),
        apiCall(`qx/book_age/${selectedAsset}`),
      ]);
      if (cancelled) return;
      if (age.success && age.data && typeof age.data === 'object') {
        setBookAge({ ask: age.data.ask ?? null, bid: age.data.bid ?? null });
      }
      timer = setTimeout(run, BOOK_VIEW_INTERVAL);
    };
    run();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [nav, selectedAsset, fetchOrderbook]);

  // Server health (request queue depth, peers) for the status pill's popover and
  // the pre-send warnings.
  useEffect(() => {
    let cancelled = false;
    let timer;
    const run = async () => {
      const res = await backgroundCall('health');
      if (cancelled) return;
      setHealth(res.success && res.data && typeof res.data === 'object' ? res.data : null);
      timer = setTimeout(run, HEALTH_INTERVAL);
    };
    run();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  // Open orders of our own identities, straight from the QX contract: the server
  // asks EntityAskOrders / EntityBidOrders for each identity every 10 s, so this
  // no longer depends on how fresh any asset's order book is.
  useEffect(() => {
    let cancelled = false;
    let timer;
    const load = async () => {
      const res = await backgroundCall('qx/open_orders');
      if (cancelled) return;
      if (res.success && Array.isArray(res.data)) {
        setOpenOrders(
          res.data.map((row) => ({
            asset: row.asset,
            issuer: row.issuer,
            side: row.side === 'A' ? 'ASK' : 'BID',
            entity: row.identity,
            price: row.price,
            num_shares: row.num_shares,
          }))
        );
      }
      timer = setTimeout(load, OPEN_ORDERS_INTERVAL);
    };
    load();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let timer;
    const poll = async () => {
      const peersRes = await backgroundCall('peers');
      if (Array.isArray(peersRes.data)) setPeers(peersRes.data);

      const ids = await backgroundCall('identities');
      if (!ids.success || !Array.isArray(ids.data)) return;
      const merged = [];
      for (let i = 0; i + 1 < ids.data.length; i += 2) {
        merged.push({ id: ids.data[i], encrypted: ids.data[i + 1] });
      }

      const reported = await Promise.all(
        merged.map(async (item) => {
          const [balanceRes, assetsRes] = await Promise.all([
            backgroundCall(`balance/${item.id}`),
            backgroundCall(`asset/balance/${item.id}`),
          ]);
          const res = asList(balanceRes);
          if (res.length < 3) return { ...item, balance: 'Not Yet Reported', assets: asList(assetsRes) };

          const balances = [];
          for (let i = 0; i < res.length; i += 3) balances.push(res[i + 2]);
          const quorum = doArrayElementsAgree(balances, 50);
          const agreed = balances.every((v) => v === res[0]) || quorum >= 0;
          const balance = agreed ? balances[0] : 'Peer Balance Mismatch';
          return { ...item, balance, assets: asList(assetsRes) };
        })
      );

      // Peers come and go; a poll with no fresh report must not read as "0 QU".
      // Fall back to the last balance peers agreed on and flag it as stale.
      const nowMs = Date.now();
      const lastGood = lastGoodBalanceRef.current;
      let sum = 0;
      let anyStale = false;
      let oldestStale = null;
      const full = reported.map((item) => {
        if (isNumeric(item.balance)) {
          lastGood.set(item.id, { balance: Number(item.balance), at: nowMs });
          sum += Number(item.balance);
          return { ...item, stale: false };
        }
        const known = lastGood.get(item.id);
        anyStale = true;
        if (!known) return { ...item, stale: false };
        sum += known.balance;
        oldestStale = oldestStale === null ? known.at : Math.min(oldestStale, known.at);
        return { ...item, balance: String(known.balance), stale: true, staleReason: item.balance, staleSince: known.at };
      });
      const knownIds = new Set(full.map((i) => i.id));
      [...lastGood.keys()].forEach((id) => !knownIds.has(id) && lastGood.delete(id));
      writeStored(KEYS.lastKnownBalances, Object.fromEntries(lastGood));
      setBalanceStale(anyStale);
      setBalanceStaleSince(oldestStale);

      // Only chart a total once every identity reported fresh and two polls in a row agree.
      const fresh = !anyStale && full.length > 0;
      const stable = stableTotalRef.current;
      if (fresh && stable.total === sum) stable.count += 1;
      else stableTotalRef.current = { total: fresh ? sum : null, count: fresh ? 1 : 0 };

      const [transferRes, assetTransferRes, qxRes] = await Promise.all([
        backgroundCall('transfer/0/0/0'),
        backgroundCall('asset/transfer/0/0/0'),
        backgroundCall('qx/orders/1/1000/0'),
      ]);
      const transfersNow = asList(transferRes);
      const assetTransfersNow = asList(assetTransferRes);
      const qxNow = asList(qxRes);
      setTransfers(transfersNow);
      setAssetTransfers(assetTransfersNow);
      setQxOrders(qxNow);
      setTotalBalance(sum);
      setIdentities(full);
      setLoading(false);
      if (fresh && stableTotalRef.current.count >= 2) setHistory(recordBalances(sum, full));

      // Notify when something that was pending settles, whichever section is open.
      const own = new Set(full.map((i) => i.id));
      const items = normalizeActivity({
        transfers: transfersNow,
        assetTransfers: assetTransfersNow,
        qxOrders: qxNow,
        ownIds: own,
        assetsNIssuer: new Map(),
      });
      const next = new Map();
      items.forEach(
        (it) =>
          it.txid &&
          next.set(it.txid, { state: txState(it.status, it.tick, latestTickRef.current), title: it.title })
      );
      const prev = txStatusRef.current;
      if (prev) {
        next.forEach((cur, txid) => {
          const was = prev.get(txid);
          if (!was || was.state !== 'pending' || cur.state === 'pending') return;
          if (cur.state === 'confirmed') toast.success(`Confirmed: ${cur.title}`);
          else if (cur.state === 'failed') toast.error(`Failed: ${cur.title}`);
          else toast.warning(`Tick passed, not verified yet: ${cur.title}`);
        });
      }
      txStatusRef.current = next;
    };
    const run = async () => {
      try {
        await poll();
      } finally {
        if (!cancelled) timer = setTimeout(run, POLLING_INTERVAL);
      }
    };
    run();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  // ---------- actions ----------

  // Locked, encrypted wallet: unlock once (a single argon2 verify on the server),
  // then run the action through the unlocked path, which skips the second verify.
  // Returns the password to use for the action, or null if the dialog should stay open.
  const unlockFirst = async (password) => {
    const res = await apiPost('wallet/unlock', { password, timeout_ms: Number(unlockTimer) });
    const text = String(res.data ?? '');
    if (!res.success) {
      toast.error(`Unlock failed: ${res.error || 'no response from server'}`);
      return null;
    }
    if (/already unlocked/i.test(text)) return '';
    if (looksLikeFailure(text)) {
      setInvalidPassword(/invalid|incorrect/i.test(text) ? 'Invalid password — try again' : text);
      return null;
    }
    setUnlockedUntil(Date.now() + Number(unlockTimer));
    return '';
  };

  // `password` is only supplied when the user typed one into the unlock dialog;
  // an empty password means "use the unlocked wallet / the seed is not encrypted".
  const actionHandler = async (pendingAction, unlocked, password = '') => {
    let actionPassword = password;
    let result;

    if (!unlocked && isEncrypted) {
      const unlockedPassword = await unlockFirst(password);
      if (unlockedPassword === null) return;
      // The export route decrypts with the password it is given, so keep the real one.
      if (pendingAction !== '/wallet/download/') actionPassword = unlockedPassword;
    }

    const tick = await apiCall('tick');
    setConnected(tick.success);
    setLatestTick(tick.data);
    const offset = Number.isInteger(Number(tickOffset)) && Number(tickOffset) > 0 ? Number(tickOffset) : DEFAULT_TICK_OFFSET;
    const targetTick = Number(tick.data) + offset;

    const isTx = pendingAction.includes('transfer') || pendingAction.startsWith('qx/order');
    if (isTx && !isNumeric(tick.data)) {
      // Without a tick there is nothing to target; the server would reject (or choke on) NaN.
      setAction('');
      setInvalidPassword('');
      toast.error('No tick from peers yet — wait until Rubic is connected, then try again');
      return;
    }
    if (pendingAction.startsWith('qx/order')) {
      // qx/order/<tick>/<issuer>/<asset>/<side>/<address>/<price>/<amount>/
      const [, , , issuer, asset, side, address, orderPrice, amount] = pendingAction.split('/');
      result = await apiPost('qx/order', {
        tick: targetTick,
        issuer,
        asset,
        side,
        address,
        price: Number(orderPrice),
        amount: Number(amount),
        password: actionPassword,
      });
      setQxOrders(asList(await apiCall('qx/orders/1/1000/0')));
    } else if (pendingAction.startsWith('asset/transfer/')) {
      // asset/transfer/<asset>/<issuer>/<source>/<dest>/<amount>/
      const [, , asset, issuer, source, dest, amount] = pendingAction.split('/');
      result = await apiPost('asset/transfer', {
        asset,
        issuer,
        source,
        dest,
        amount: Number(amount),
        expiration: targetTick,
        password: actionPassword,
      });
    } else if (pendingAction.startsWith('transfer/')) {
      // QU transfers go as JSON: POST /transfer {source, dest, amount, expiration, password}
      const [, source, dest, amount] = pendingAction.split('/');
      result = await apiPost('transfer', {
        source,
        dest,
        amount: Number(amount),
        expiration: targetTick,
        password: actionPassword,
      });
    } else if (pendingAction.startsWith('identity/delete/')) {
      result = await apiPost('identity/delete', { identity: pendingAction.split('/')[2], password: actionPassword });
    } else if (pendingAction.startsWith('identity/add/')) {
      result = await apiPost('identity/add', { seed: pendingAction.split('/')[2], password: actionPassword });
    } else if (pendingAction.startsWith('identity/new')) {
      result = await apiPost('identity/new', { password: actionPassword });
    } else if (pendingAction === '/wallet/download/') {
      result = await apiPost('wallet/download', { password: actionPassword });
    } else {
      result = await apiCall(`${pendingAction}${actionPassword}`);
    }

    const invalid = INVALID_PASSWORD_RESPONSES.includes(result.data);

    if (invalid) {
      setShowProgress(false);
      setInvalidPassword('Invalid password — try again');
      return;
    }

    // The server has answered: close the password modal now.
    setAction('');
    setInvalidPassword('');

    if (pendingAction === '/wallet/download/') {
      const csv = String(result.data ?? '');
      if (csv.split(',').length < 2) {
        toast.error('Export failed: the server did not return wallet data');
      } else {
        const link = document.createElement('a');
        link.href = 'data:text/csv;charset=utf-8,' + encodeURI(csv);
        link.download = 'rubic-db-decrypted.csv';
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        const rows = csv.split(/\r?\n/).filter((line) => line.includes(',') && !/^id(entity)?,/i.test(line)).length;
        toast.success(`Wallet exported: ${rows} identit${rows === 1 ? 'y' : 'ies'} with seeds in plain text, saved as rubic-db-decrypted.csv`);
      }
    }

    const failed = !result.success || looksLikeFailure(result.data);
    if (!result.success) {
      toast.error(`Request failed: ${result.error || 'no response from server'}`);
    } else if (failed) {
      toast.error(String(result.data));
    } else if (pendingAction !== '/wallet/download/') {
      toast.success(describeAction(pendingAction));
    }

    if (isTx && !failed) {
      setOrderTick(targetTick);
      setShowProgress(true);
    }
  };

  // The identity whose seed signs this action, if the action has one.
  const signerOf = (pendingAction) => {
    const parts = pendingAction.split('/');
    if (pendingAction.startsWith('transfer/')) return parts[1];
    if (pendingAction.startsWith('asset/transfer/')) return parts[4];
    if (pendingAction.startsWith('qx/order/')) return parts[6];
    if (pendingAction.startsWith('identity/delete/')) return parts[2];
    return null;
  };

  const commitAction = async (pendingAction) => {
    // A seed stored without encryption needs no master password to sign.
    const signer = identities.find((i) => i.id === signerOf(pendingAction));
    if (signer && signer.encrypted !== 'true') {
      actionHandler(pendingAction, true);
      return;
    }
    const unlocked = await apiCall('wallet/unlocked');
    if (unlocked.success && unlocked.data === true) actionHandler(pendingAction, true);
    else setAction(pendingAction);
  };

  const submitPasswordAction = async (password) => {
    if (!password || actionBusy) return;
    setActionBusy(true);
    setInvalidPassword('');
    try {
      await actionHandler(action, false, password);
    } finally {
      setActionBusy(false);
    }
  };

  const cancelPasswordAction = () => {
    setAction('');
    setInvalidPassword('');
  };

  // The export writes every seed in plain text, so it is confirmed like a
  // deletion is - before the master-password prompt, which only verifies.
  const exportWallet = () => {
    setConfirm({
      title: 'Export the wallet as a decrypted CSV?',
      confirmLabel: 'Export CSV',
      confirmColor: 'error',
      body: (
        <>
          <Typography variant='body2'>
            The file <b>rubic-db-decrypted.csv</b> will contain every identity in this wallet with its seed in
            plain text. Anyone who reads it can spend the funds.
          </Typography>
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            Keep it offline and delete it when you are done. {isEncrypted ? 'Your master password is asked for next.' : ''}
          </Typography>
        </>
      ),
      onConfirm: () => commitAction('/wallet/download/'),
    });
  };

  const deleteIdentity = (identityId) => {
    const target = identities.find((i) => i.id === identityId);
    const funded = target && isNumeric(target.balance) && Number(target.balance) > 0;
    setConfirm({
      title: 'Delete this identity?',
      confirmLabel: 'Delete identity',
      confirmColor: 'error',
      challenge: funded
        ? {
            expected: identityId.slice(0, 6),
            hint: `This identity still holds ${formatNumber(target.balance)} QU. Type the first 6 letters of its ID (${identityId.slice(0, 6)}) to confirm.`,
          }
        : undefined,
      body: (
        <>
          {labels[identityId] && <Typography sx={{ fontWeight: 600 }}>{labels[identityId]}</Typography>}
          <IdText id={identityId} full copy={false} />
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            This removes the identity and its seed from this wallet. Funds stay
            on the network, but without the seed you will not be able to access
            them again.
          </Typography>
        </>
      ),
      onConfirm: () => commitAction(`identity/delete/${identityId}/`),
    });
  };

  const setMasterPassword = async (newPassword) => {
    const set = await apiPost('wallet/set_master_password', { password: newPassword });
    if (!set.success || looksLikeFailure(set.data)) {
      toast.error(String(set.data || set.error || 'Could not set the master password'));
      return;
    }
    const encrypt = await apiPost('wallet/encrypt', { password: newPassword });
    if (!encrypt.success || looksLikeFailure(encrypt.data)) {
      toast.error(String(encrypt.data || encrypt.error || 'Could not encrypt the wallet'));
      return;
    }
    toast.success('Master password set — wallet encrypted');
    setIsEncrypted(true);
  };

  const savePeerLimits = async (min, max) => {
    // The server rejects min > current max and max < current min, so order the writes.
    const steps =
      max >= peerLimits.max
        ? [['max', max], ['min', min]]
        : [['min', min], ['max', max]];
    for (const [key, value] of steps) {
      const res = await apiCall(`peers/limit/${key}/${value}`);
      if (!res.success || res.data !== 'Ok') {
        toast.error(String(res.data || res.error || `Could not set ${key} peers`));
        await fetchPeerLimits();
        return;
      }
    }
    setPeerLimits({ min, max });
    toast.success(`Peer limits saved: ${min}–${max}`);
  };

  // ---------- derived view state ----------

  const connectedPeers = peers.filter(
    (p) => p.whitelisted !== '-1' && (p.connected === '1' || p.connected === 'true')
  ).length;
  const unlockSecondsLeft = Math.max(0, Math.ceil((unlockedUntil - now) / 1000));
  const pendingTxs = [...transfers, ...assetTransfers, ...qxOrders].filter(
    (t) => txState(t.status, t.tick, latestTick) === 'pending'
  );
  const pendingCount = pendingTxs.length;
  // Things worth knowing before signing from `sourceId`; shown in the review /
  // confirm step, never blocking.
  const warningsFor = (sourceId) => {
    const out = [];
    const pendingFromSource = pendingTxs.filter((t) => t.source === sourceId).length;
    if (pendingFromSource > 0) {
      out.push(
        `This identity already has ${pendingFromSource === 1 ? 'a transaction' : `${pendingFromSource} transactions`} waiting for its tick. A node keeps one pending transaction per identity, so sending now can replace the earlier one before it executes.`
      );
    }
    if (connectedPeers < MIN_PEERS_FOR_SENDING) {
      out.push(
        `Only ${connectedPeers} peer${connectedPeers === 1 ? '' : 's'} connected. Transactions are broadcast to every connected peer; with few peers they may not reach the network in time.`
      );
    }
    if (health && health.routine_limit && health.routine_backlog >= health.routine_limit / 2) {
      out.push('The wallet is behind on peer requests right now; balances and confirmations may lag.');
    }
    return out;
  };
  // The header pill counts down to the same tick Activity shows: the one stored
  // with the transaction. Until the first poll returns the new row, fall back to
  // the tick we asked for.
  const nextPendingTick =
    pendingTxs.length > 0
      ? Math.max(...pendingTxs.map((t) => Number(t.tick)))
      : showProgress && isNumeric(orderTick)
      ? Number(orderTick)
      : null;
  const ticksToGo = nextPendingTick !== null && isNumeric(latestTick) ? nextPendingTick - Number(latestTick) : null;
  const pending =
    nextPendingTick !== null
      ? {
          label: ticksToGo !== null && ticksToGo > 0 ? `Pending · ${ticksToGo} tick${ticksToGo === 1 ? '' : 's'}` : 'Pending',
          tooltip: `${pendingCount > 1 ? `${pendingCount} transactions are` : 'A transaction is'} waiting to be included${pendingCount > 1 ? '; the latest' : ''} at tick ${formatNumber(nextPendingTick)}`,
        }
      : null;
  const unencryptedCount = identities.filter((i) => i.encrypted !== 'true').length;
  const detailIdentity = identities.find((i) => i.id === detailId) || null;
  const detailActivity = useMemo(
    () =>
      detailId
        ? normalizeActivity({ transfers, assetTransfers, qxOrders, ownIds, assetsNIssuer }).filter(
            (it) => it.from === detailId || it.to === detailId
          )
        : [],
    [detailId, transfers, assetTransfers, qxOrders, ownIds, assetsNIssuer]
  );
  const openRename = (id) => setRenameTarget({ id, label: labels[id] || '' });
  const totalHistory24h = useMemo(() => sliceSince(history.total || [], RANGES['24h']), [history]);
  const section = (name, node) => (
    <ErrorBoundary name={name} resetKey={nav}>
      {node}
    </ErrorBoundary>
  );

  const panel = {
    wallet: (
      <WalletPanel
        identities={identities}
        transfers={transfers}
        allowNonEncrypted={allowNonEncrypted}
        price={price}
        currency={currency}
        loading={loading}
        labels={labels}
        onAction={commitAction}
        onBulkImport={() => setBulkOpen(true)}
        onSend={(from) => setSend({ open: true, from })}
        onRename={openRename}
        onOpenIdentity={setDetailId}
        onDeleteIdentity={deleteIdentity}
      />
    ),
    activity: (
      <ActivityPanel
        transfers={transfers}
        assetTransfers={assetTransfers}
        qxOrders={qxOrders}
        identities={identities}
        labels={labels}
        assetsNIssuer={assetsNIssuer}
        latestTick={latestTick}
        loading={loading}
        onAction={commitAction}
        requestConfirm={setConfirm}
      />
    ),
    assets: (
      <AssetsPanel
        identities={identities}
        assetsNIssuer={assetsNIssuer}
        selectedAsset={selectedAsset}
        onSelectAsset={setSelectedAsset}
        labels={labels}
        addressBook={addressBook}
        onAction={commitAction}
        requestConfirm={setConfirm}
        warningsFor={warningsFor}
      />
    ),
    exchange: (
      <QxPanel
        bookAge={bookAge}
        warningsFor={warningsFor}
        identities={identities}
        labels={labels}
        openOrders={openOrders}
        assetsNIssuer={assetsNIssuer}
        selectedAsset={selectedAsset}
        onSelectAsset={setSelectedAsset}
        selectedId={selectedId}
        onSelectId={setSelectedId}
        askOrders={askOrders}
        bidOrders={bidOrders}
        bookLoading={Boolean(selectedAsset) && bookAsset !== selectedAsset}
        busy={showProgress}
        onAction={commitAction}
        requestConfirm={setConfirm}
      />
    ),
    network: <PeersPanel peers={peers} onChanged={refreshPeers} />,
    settings: (
      <SettingsPanel
        unlockTimerMs={unlockTimer}
        onUnlockTimerChange={setUnlockTimer}
        tickOffset={tickOffset}
        onTickOffsetChange={setTickOffset}
        allowNonEncrypted={allowNonEncrypted}
        onAllowNonEncryptedChange={setAllowNonEncrypted}
        unencryptedCount={unencryptedCount}
        peerLimits={peerLimits}
        onSavePeerLimits={savePeerLimits}
        onDownloadWallet={exportWallet}
        onImportDb={() => setImportOpen(true)}
        latestTick={latestTick}
        currency={currency}
        onCurrencyChange={setCurrency}
        price={price}
        priceStatus={priceStatus}
        addressBook={addressBook}
        onAddressBookChange={setAddressBook}
        identities={identities}
        labels={labels}
      />
    ),
  }[nav] ?? null;

  const dimWhileLocked = {
    pointerEvents: action ? 'none' : 'all',
    opacity: action ? 0.4 : 1,
    // Let the section fill the window so its tables can scroll internally.
    flex: 1,
    minHeight: 0,
    display: 'flex',
    flexDirection: 'column',
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {isEncrypted ? (
        <AppShell
          nav={nav}
          onNav={setNav}
          badges={{ network: connectedPeers || undefined, activity: pendingCount || undefined }}
          totalBalance={totalBalance}
          price={price}
          connected={connected}
          tick={latestTick}
          peersConnected={connectedPeers}
          unlockSecondsLeft={unlockSecondsLeft}
          unencryptedCount={unencryptedCount}
          pending={pending}
          themeMode={themeMode}
          onToggleTheme={() => setThemeMode((m) => (m === 'light' ? 'dark' : 'light'))}
          onSend={() => setSend({ open: true, from: '' })}
          canSend={identities.length > 0}
          currency={currency}
          loading={loading}
          history={totalHistory24h}
          balanceStale={balanceStale}
          balanceStaleSince={balanceStaleSince}
          now={now}
          health={health}
          peers={peers}
        >
          {!connected && (
            <Alert severity='error' sx={{ mb: 2 }}>
              Cannot reach the Rubic server at {serverIp}. Retrying…
            </Alert>
          )}
          <Box sx={dimWhileLocked}>{section(NAV.find((n) => n.key === nav)?.label || 'this section', panel)}</Box>
        </AppShell>
      ) : (
        <Box sx={{ minHeight: '100vh', bgcolor: 'background.default', p: 3 }}>
          {!connected && (
            <Alert severity='error' sx={{ mb: 2 }}>
              Cannot reach the Rubic server at {serverIp}. Retrying…
            </Alert>
          )}
          <LockScreen onSetPassword={setMasterPassword} onImportDb={() => setImportOpen(true)} />
        </Box>
      )}

      <ImportDbWizard
        open={importOpen}
        onClose={() => setImportOpen(false)}
        hasMasterPassword={isEncrypted}
        unlockTimerMs={unlockTimer}
        existing={identities.map((i) => i.id)}
        // Unlocking inside the wizard is a normal unlock: start the header's countdown
        // too - unless one is already running (the server keeps its original timer).
        onUnlocked={() => setUnlockedUntil((current) => (current > Date.now() ? current : Date.now() + Number(unlockTimer)))}
        // The wizard replaced the master password (and dropped the identities
        // encrypted with the old one): any unlock is over, and a fresh wallet now
        // has a password, so the first-run screen gives way to the wallet.
        onReset={async () => {
          setUnlockedUntil(0);
          await refreshEncrypted();
        }}
        onImported={({ count, skipped }) => {
          const detail = skipped > 0 ? ` (${skipped} already in the wallet)` : '';
          toast.success(`Imported ${count} identit${count === 1 ? 'y' : 'ies'} from CSV${detail}`);
        }}
      />
      <BulkImportDialog
        open={bulkOpen}
        onClose={() => setBulkOpen(false)}
        isEncrypted={isEncrypted}
        allowNonEncrypted={allowNonEncrypted}
        unlockTimerMs={unlockTimer}
        existing={identities.map((i) => i.id)}
        onUnlocked={() => setUnlockedUntil((current) => (current > Date.now() ? current : Date.now() + Number(unlockTimer)))}
        onImported={({ count, skipped, plain }) => {
          const detail = [skipped > 0 && `${skipped} already in the wallet`, plain && 'seeds stored without encryption'].filter(Boolean).join('; ');
          toast.success(`Imported ${count} identit${count === 1 ? 'y' : 'ies'}${detail ? ` (${detail})` : ''}`);
        }}
      />
      <PasswordDialog
        open={Boolean(action)}
        error={invalidPassword}
        busy={actionBusy}
        unlockTimerMs={unlockTimer}
        onSubmit={submitPasswordAction}
        onCancel={cancelPasswordAction}
      />
      <CommandPalette
        open={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        identities={identities}
        labels={labels}
        onNav={setNav}
        onOpenIdentity={setDetailId}
        onSend={() => setSend({ open: true, from: '' })}
        onCreate={() => commitAction('identity/new/')}
        onToggleTheme={() => setThemeMode((m) => (m === 'light' ? 'dark' : 'light'))}
      />
      <IdentityDrawer
        identity={detailIdentity}
        label={detailId ? labels[detailId] : ''}
        price={price}
        currency={currency}
        history={detailId ? history.byId?.[detailId] || [] : []}
        activity={detailActivity}
        latestTick={latestTick}
        onClose={() => setDetailId(null)}
        onSend={(from) => setSend({ open: true, from })}
        onRename={openRename}
        onDelete={(id) => {
          setDetailId(null);
          deleteIdentity(id);
        }}
      />
      <RenameDialog target={renameTarget} onClose={() => setRenameTarget(null)} onSave={setLabel} />
      <SendDialog
        open={send.open}
        onClose={() => setSend({ open: false, from: '' })}
        identities={identities}
        labels={labels}
        transfers={transfers}
        price={price}
        currency={currency}
        addressBook={addressBook}
        onAddressBookChange={setAddressBook}
        initialFrom={send.from}
        onAction={commitAction}
        warningsFor={warningsFor}
      />
      <ConfirmDialog
        open={Boolean(confirm)}
        title={confirm?.title}
        confirmLabel={confirm?.confirmLabel}
        confirmColor={confirm?.confirmColor}
        challenge={confirm?.challenge}
        onCancel={() => setConfirm(null)}
        onConfirm={() => {
          const run = confirm?.onConfirm;
          setConfirm(null);
          run?.();
        }}
      >
        {confirm?.body}
      </ConfirmDialog>
      <ToastViewport />
    </ThemeProvider>
  );
};

export default MainView;
