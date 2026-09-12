import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Alert, Box, CssBaseline, ThemeProvider, Typography } from '@mui/material';

import { apiCall } from './api';
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
import IdText from './components/IdText';
import ErrorBoundary from './components/ErrorBoundary';
import CommandPalette from './components/CommandPalette';
import { NAV } from './components/AppShell';
import { formatNumber, isNumeric } from './utils/format';
import { recordBalances, readHistory, sliceSince, RANGES } from './utils/balanceHistory';

const POLLING_INTERVAL = 3000;
const TICK_INTERVAL = 1000;
const TICK_OFFSET = 10;

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
  const [selectedAsset, setSelectedAsset] = useState(null);
  const [selectedId, setSelectedId] = useState('');

  // Settings & local preferences
  const [unlockTimer, setUnlockTimer] = useState('60000');
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
  const [password, setPassword] = useState('');
  const [invalidPassword, setInvalidPassword] = useState('');
  const [actionBusy, setActionBusy] = useState(false);
  const [confirm, setConfirm] = useState(null);
  const [unlockedUntil, setUnlockedUntil] = useState(0);
  const [send, setSend] = useState({ open: false, from: '' });
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

  const fetchOrderbook = useCallback(async (asset) => {
    if (!asset) return;
    const [ask, bid] = await Promise.all([
      apiCall(`qx/orderbook/${asset}/ASK/1000/0`),
      apiCall(`qx/orderbook/${asset}/BID/1000/0`),
    ]);
    setAskOrders(asList(ask));
    setBidOrders(asList(bid));
  }, []);

  const fetchPeerLimits = useCallback(async () => {
    const res = await apiCall('peers/limit');
    if (res.success && res.data && typeof res.data === 'object') {
      setPeerLimits({ min: Number(res.data.min), max: Number(res.data.max) });
    }
  }, []);

  useEffect(() => {
    const init = async () => {
      const encrypted = await apiCall('wallet/is_encrypted');
      setIsEncrypted(typeof encrypted.data === 'boolean');

      const assets = await apiCall('asset/issued');
      const pairs = asList(assets).reduce(
        (acc, val, idx, arr) => (idx % 2 === 0 ? [...acc, [val, arr[idx + 1]]] : acc),
        []
      );
      const map = new Map(pairs);
      setAssetsNIssuer(map);
      setSelectedAsset([...map.keys()].sort()[0] ?? null);

      await fetchPeerLimits();
    };
    init();
  }, [fetchPeerLimits]);

  useEffect(() => {
    let cancelled = false;
    const fetchPrice = async () => {
      try {
        const vs = currency.toLowerCase();
        const response = await fetch(
          `https://api.coingecko.com/api/v3/simple/price?ids=qubic-network&vs_currencies=${vs}`
        );
        const data = await response.json();
        if (!cancelled) setPrice(data['qubic-network']?.[vs] ?? 0);
      } catch {
        // Price is decorative; ignore failures.
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

  useEffect(() => {
    const intervalId = setInterval(async () => {
      const tick = await apiCall('tick');
      setConnected(tick.success);
      setLatestTick(tick.data);
      latestTickRef.current = tick.data;
      setShowProgress(orderTick >= tick.data);
      setNow(Date.now());
    }, TICK_INTERVAL);
    return () => clearInterval(intervalId);
  }, [orderTick]);

  // Keep the unlock indicator honest if the server locked earlier than expected.
  useEffect(() => {
    if (!unlockedUntil) return undefined;
    const intervalId = setInterval(async () => {
      const res = await apiCall('wallet/unlocked');
      if (res.success && res.data !== true) setUnlockedUntil(0);
    }, 5000);
    return () => clearInterval(intervalId);
  }, [unlockedUntil]);

  useEffect(() => {
    fetchOrderbook(selectedAsset);
  }, [selectedAsset, fetchOrderbook]);

  // Resting orders of our own identities across every known asset.
  useEffect(() => {
    const assets = [...assetsNIssuer.keys()];
    if (assets.length === 0 || ownIds.size === 0) return undefined;
    let cancelled = false;
    const load = async () => {
      const found = [];
      for (const asset of assets) {
        const [ask, bid] = await Promise.all([
          apiCall(`qx/orderbook/${asset}/ASK/1000/0`),
          apiCall(`qx/orderbook/${asset}/BID/1000/0`),
        ]);
        asList(ask).filter((o) => ownIds.has(o.entity)).forEach((o) => found.push({ ...o, asset, side: 'ASK' }));
        asList(bid).filter((o) => ownIds.has(o.entity)).forEach((o) => found.push({ ...o, asset, side: 'BID' }));
      }
      if (!cancelled) setOpenOrders(found);
    };
    load();
    const intervalId = setInterval(load, 10000);
    return () => {
      cancelled = true;
      clearInterval(intervalId);
    };
  }, [assetsNIssuer, ownIds, orderTick]);

  useEffect(() => {
    const poll = async () => {
      const peersRes = await apiCall('peers');
      if (Array.isArray(peersRes.data)) setPeers(peersRes.data);

      const ids = await apiCall('identities');
      if (!ids.success || !Array.isArray(ids.data)) return;
      const merged = [];
      for (let i = 0; i + 1 < ids.data.length; i += 2) {
        merged.push({ id: ids.data[i], encrypted: ids.data[i + 1] });
      }

      const reported = await Promise.all(
        merged.map(async (item) => {
          const [balanceRes, assetsRes] = await Promise.all([
            apiCall(`balance/${item.id}`),
            apiCall(`asset/balance/${item.id}`),
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
        apiCall('transfer/0/0/0'),
        apiCall('asset/transfer/0/0/0'),
        apiCall('qx/orders/1/1000/0'),
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
          else toast.error(`Failed: ${cur.title} — tick passed without confirmation`);
        });
      }
      txStatusRef.current = next;

      await fetchOrderbook(selectedAsset);
    };
    poll();
    const intervalId = setInterval(poll, POLLING_INTERVAL);
    return () => clearInterval(intervalId);
  }, [selectedAsset, fetchOrderbook]);

  // ---------- actions ----------

  const actionHandler = async (pendingAction, unlocked) => {
    const actionPassword = password || '0';
    let result;

    const tick = await apiCall('tick');
    setConnected(tick.success);
    setLatestTick(tick.data);
    const targetTick = Number(tick.data) + TICK_OFFSET;

    const isTx = pendingAction.includes('transfer') || pendingAction.startsWith('qx/order');
    if (pendingAction.startsWith('qx/order')) {
      result = await apiCall(`${pendingAction.replace('<tick>', targetTick)}/${actionPassword}`);
      setQxOrders(asList(await apiCall('qx/orders/1/1000/0')));
    } else if (pendingAction.includes('transfer')) {
      result = await apiCall(`${pendingAction}${targetTick}/${actionPassword}`);
    } else {
      result = await apiCall(`${pendingAction}${actionPassword}`);
    }

    const invalid = INVALID_PASSWORD_RESPONSES.includes(result.data);
    setPassword('');

    if (invalid) {
      setShowProgress(false);
      setInvalidPassword('Invalid password — try again');
      return;
    }

    // The server has answered: close the password modal now. The unlock call below
    // (argon2, seconds in debug builds) runs in the background.
    setAction('');
    setInvalidPassword('');
    if (!unlocked) {
      apiCall(`wallet/unlock/${actionPassword}/${unlockTimer}`).then((res) => {
        if (res.success && !looksLikeFailure(res.data)) setUnlockedUntil(Date.now() + Number(unlockTimer));
      });
    }

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

  const commitAction = async (pendingAction) => {
    const unlocked = await apiCall('wallet/unlocked');
    if (unlocked.success && unlocked.data === true) actionHandler(pendingAction, true);
    else setAction(pendingAction);
  };

  const submitPasswordAction = async () => {
    if (!password || actionBusy) return;
    setActionBusy(true);
    try {
      await actionHandler(action, false);
    } finally {
      setActionBusy(false);
    }
  };

  const cancelPasswordAction = () => {
    setPassword('');
    setAction('');
    setInvalidPassword('');
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
    const set = await apiCall(`wallet/set_master_password/${newPassword}`);
    if (!set.success || looksLikeFailure(set.data)) {
      toast.error(String(set.data || set.error || 'Could not set the master password'));
      return;
    }
    const encrypt = await apiCall(`wallet/encrypt/${newPassword}`);
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
  const ticksToGo = isNumeric(latestTick) ? Number(orderTick) - Number(latestTick) : null;
  const pending = showProgress
    ? {
        label: ticksToGo !== null && ticksToGo > 0 ? `Pending · ${ticksToGo} tick${ticksToGo === 1 ? '' : 's'}` : 'Pending',
        tooltip: `A transaction is waiting to be included at tick ${formatNumber(orderTick)}`,
      }
    : null;
  const pendingCount = [...transfers, ...assetTransfers, ...qxOrders].filter(
    (t) => txState(t.status, t.tick, latestTick) === 'pending'
  ).length;
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
      />
    ),
    exchange: (
      <QxPanel
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
        allowNonEncrypted={allowNonEncrypted}
        onAllowNonEncryptedChange={setAllowNonEncrypted}
        unencryptedCount={unencryptedCount}
        peerLimits={peerLimits}
        onSavePeerLimits={savePeerLimits}
        onDownloadWallet={() => commitAction('/wallet/download/')}
        latestTick={latestTick}
        currency={currency}
        onCurrencyChange={setCurrency}
        addressBook={addressBook}
        onAddressBookChange={setAddressBook}
        identities={identities}
        labels={labels}
      />
    ),
  }[nav] ?? null;

  const dimWhileLocked = { pointerEvents: action ? 'none' : 'all', opacity: action ? 0.4 : 1 };

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
          <LockScreen onSetPassword={setMasterPassword} />
        </Box>
      )}

      <PasswordDialog
        open={Boolean(action)}
        password={password}
        onPasswordChange={setPassword}
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
