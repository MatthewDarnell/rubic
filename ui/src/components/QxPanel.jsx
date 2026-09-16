import React, { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import {
  Box,
  Button,
  Chip,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
  Tooltip,
  Typography,
  useTheme,
} from '@mui/material';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';
import SellIcon from '@mui/icons-material/Sell';
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutlined';
import IdText from './IdText';
import Identicon from './Identicon';
import DepthChart from './DepthChart';
import { digitsOnly, formatNumber, isNumeric, shortenId } from '../utils/format';

const CHART_LEVELS = 40; // price levels per side shown in the depth chart
const SIDE_WIDTH = 380; // depth chart + open orders column

const toNum = (v) => Number(v) || 0;

// Rows of one side of the book. Each row carries a background bar proportional
// to cumulative depth, like a classic exchange ladder. Each side takes half of
// the book's height and scrolls on its own.
const Ladder = ({ side, orders, ownId, busy, onCancel, onFill, containerRef, emptyText }) => {
  const theme = useTheme();
  const color = side === 'ASK' ? theme.palette.error.main : theme.palette.success.main;
  const maxCum = orders.length ? Math.max(...orders.map((o) => o.cumulative)) : 1;
  return (
    <TableContainer ref={containerRef} sx={{ flex: '1 1 0', minHeight: 0, overflow: 'auto' }}>
      <Table size='small' stickyHeader sx={{ '& td, & th': { py: 0.4 } }}>
        <TableHead>
          <TableRow>
            <TableCell align='right' sx={{ width: 140 }}>Price (QU)</TableCell>
            <TableCell align='right' sx={{ width: 130 }}>Shares</TableCell>
            <TableCell align='right' sx={{ width: 150 }}>Total (QU)</TableCell>
            <TableCell>Entity</TableCell>
            <TableCell sx={{ width: 40 }} />
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.length === 0 && (
            <TableRow>
              <TableCell colSpan={5} align='center' sx={{ py: 3, color: 'text.secondary' }}>
                {emptyText}
              </TableCell>
            </TableRow>
          )}
          {orders.map((o, index) => {
            const mine = ownId && o.entity === ownId;
            const pct = Math.min(100, (o.cumulative / maxCum) * 100);
            return (
              <TableRow
                key={`${o.entity}-${o.price}-${index}`}
                hover
                onClick={() => onFill?.(o)}
                title={
                  o.fromContract
                    ? 'Your order, as reported by QX; the book snapshot has not caught up yet'
                    : `Fill the form: ${formatNumber(o.cumulative)} shares at ${formatNumber(o.price)} QU`
                }
                sx={{
                  cursor: onFill ? 'pointer' : 'default',
                  background: `linear-gradient(to left, ${color}22 ${pct}%, transparent ${pct}%)`,
                  '& td': mine ? { fontWeight: 700 } : undefined,
                  ...(o.fromContract && { '& td': { fontWeight: 700, fontStyle: 'italic', opacity: 0.8 } }),
                }}
              >
                <TableCell align='right' sx={{ color, fontFamily: 'monospace' }}>
                  {formatNumber(o.price)}
                </TableCell>
                <TableCell align='right' sx={{ fontFamily: 'monospace' }}>{formatNumber(o.num_shares)}</TableCell>
                <TableCell align='right' sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
                  {formatNumber(o.total)}
                </TableCell>
                <TableCell onClick={(e) => e.stopPropagation()}>
                  <IdText id={o.entity} head={6} tail={6} explorer='address' sx={mine ? { color: 'primary.main' } : undefined} />
                </TableCell>
                <TableCell onClick={(e) => e.stopPropagation()}>
                  {mine && (
                    <Tooltip title='Cancel order'>
                      <span>
                        <IconButton size='small' disabled={busy} onClick={() => onCancel(o, side)}>
                          <DeleteOutlineIcon fontSize='small' />
                        </IconButton>
                      </span>
                    </Tooltip>
                  )}
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

// How long ago the server last received each side of this book from a peer.
const BookAge = ({ age }) => {
  if (!age) return null;
  const sides = [age.ask, age.bid];
  if (sides.some((s) => s === null || s === undefined)) {
    return (
      <Tooltip title='Asked peers for this book; no reply stored yet since the wallet started'>
        <Typography variant='caption' sx={{ color: 'warning.main' }}>
          waiting for peers…
        </Typography>
      </Tooltip>
    );
  }
  const oldest = Math.max(...sides);
  const stale = oldest > 20;
  return (
    <Tooltip title={`Asks updated ${age.ask} s ago · bids ${age.bid} s ago. The book on screen is re-fetched from all peers every couple of seconds.`}>
      <Typography variant='caption' sx={{ color: stale ? 'warning.main' : 'text.secondary' }}>
        updated {oldest} s ago
      </Typography>
    </Tooltip>
  );
};

const OpenOrders = ({ orders, labels, busy, onCancel }) => (
  <Paper variant='outlined' sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
    <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', gap: 1.5, flexShrink: 0 }}>
      <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
        My open orders
      </Typography>
      <Chip size='small' label={orders.length} />
    </Box>
    <TableContainer sx={{ borderTop: 1, borderColor: 'divider', flex: 1, minHeight: 0, overflow: 'auto' }}>
      <Table size='small' stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>Asset</TableCell>
            <TableCell>Side</TableCell>
            <TableCell align='right'>Price</TableCell>
            <TableCell align='right'>Shares</TableCell>
            <TableCell sx={{ width: 40 }} />
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.length === 0 && (
            <TableRow>
              <TableCell colSpan={5} align='center' sx={{ py: 2.5, color: 'text.secondary' }}>
                No Open Orders
              </TableCell>
            </TableRow>
          )}
          {orders.map((o, index) => (
            <TableRow
              key={`${o.asset}-${o.side}-${o.entity}-${o.price}-${index}`}
              hover
              title={`${labels?.[o.entity] ? `${labels[o.entity]} · ` : ''}${o.entity} · total ${formatNumber(Number(o.price) * Number(o.num_shares))} QU`}
            >
              <TableCell sx={{ fontWeight: 600 }}>{o.asset}</TableCell>
              <TableCell>
                <Chip
                  size='small'
                  variant='outlined'
                  color={o.side === 'ASK' ? 'error' : 'success'}
                  label={o.side === 'ASK' ? 'Sell' : 'Buy'}
                />
              </TableCell>
              <TableCell align='right' className='mono'>{formatNumber(o.price)}</TableCell>
              <TableCell align='right' className='mono'>{formatNumber(o.num_shares)}</TableCell>
              <TableCell>
                <Tooltip title='Cancel order'>
                  <span>
                    <IconButton size='small' disabled={busy} onClick={() => onCancel(o)}>
                      <DeleteOutlineIcon fontSize='small' />
                    </IconButton>
                  </span>
                </Tooltip>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  </Paper>
);

export default function QxPanel({
  miningActive,
  identities,
  labels,
  assetsNIssuer,
  selectedAsset,
  onSelectAsset,
  selectedId,
  onSelectId,
  askOrders,
  bidOrders,
  bookLoading,
  bookAge,
  openOrders,
  busy,
  onAction,
  requestConfirm,
  warningsFor,
}) {
  const [price, setPrice] = useState('');
  const [amount, setAmount] = useState('');
  const asksRef = useRef(null);
  // Whether the ask ladder should stay pinned to its bottom (the best ask, next
  // to the spread). True until the user scrolls up; true again when they scroll
  // back down or switch asset.
  const stickAsksRef = useRef(true);

  const assets = [...assetsNIssuer.keys()].sort();
  const priceNum = Number(price);
  const amountNum = Number(amount);
  const valid = selectedId && selectedAsset && priceNum > 0 && amountNum > 0;
  const total = priceNum * amountNum;

  // What the signing identity can back the order with. A buy locks its total in
  // QU with the contract; a sell hands the shares over. Neither is offered when
  // the identity is known to fall short; an unreported balance is not a "no".
  const signer = identities.find((i) => i.id === selectedId);
  const quBalance = isNumeric(signer?.balance) ? Number(signer.balance) : null;
  const holding = signer?.assets ? Number(signer.assets.find((a) => a?.name === selectedAsset)?.balance ?? 0) : null;
  const cannotAfford = quBalance !== null && total > quBalance;
  const notEnoughShares = holding !== null && amountNum > holding;
  const buyBlocked = valid && cannotAfford
    ? `Costs ${formatNumber(total)} QU, but this identity has ${formatNumber(quBalance)} QU`
    : '';
  const sellBlocked = valid && notEnoughShares
    ? `This identity holds ${formatNumber(holding)} ${selectedAsset}, not ${formatNumber(amountNum)}`
    : '';

  // Asks: highest price at the top, best (lowest) ask at the bottom next to the spread.
  // Bids: best (highest) bid at the top. Cumulative depth accumulates away from the spread.
  const book = useMemo(() => {
    const norm = (o) => ({
      ...o,
      price: toNum(o.price),
      num_shares: toNum(o.num_shares),
      total: toNum(o.price) * toNum(o.num_shares),
    });
    // Our own resting orders come from the QX contract directly and can be
    // ahead of the last book snapshot; show them in the ladder right away,
    // marked, until the book catches up.
    const same = (a, b) =>
      a.entity === b.entity && toNum(a.price) === toNum(b.price) && toNum(a.num_shares) === toNum(b.num_shares);
    const mine = (openOrders || []).filter((o) => o.asset === selectedAsset);
    const extra = (side, list) =>
      mine.filter((o) => o.side === side && !list.some((x) => same(x, o))).map((o) => ({ ...o, fromContract: true }));
    const asksAsc = [...askOrders, ...extra('ASK', askOrders)].map(norm).sort((a, b) => a.price - b.price);
    const bidsDesc = [...bidOrders, ...extra('BID', bidOrders)].map(norm).sort((a, b) => b.price - a.price);
    let cum = 0;
    const asksCum = asksAsc.map((o) => ({ ...o, cumulative: (cum += o.num_shares) }));
    cum = 0;
    const bidsCum = bidsDesc.map((o) => ({ ...o, cumulative: (cum += o.num_shares) }));

    const levels = (orders) => {
      const map = new Map();
      orders.forEach((o) => map.set(o.price, (map.get(o.price) || 0) + o.num_shares));
      return [...map.entries()].map(([p, v]) => ({ price: p, volume: v }));
    };
    const askLevels = levels(asksAsc).sort((a, b) => a.price - b.price).slice(0, CHART_LEVELS);
    const bidLevels = levels(bidsDesc).sort((a, b) => b.price - a.price).slice(0, CHART_LEVELS);

    const bestAsk = asksAsc[0]?.price;
    const bestBid = bidsDesc[0]?.price;
    const spread = bestAsk !== undefined && bestBid !== undefined ? bestAsk - bestBid : null;
    const spreadPct = spread !== null && bestAsk > 0 ? (spread / bestAsk) * 100 : null;

    return {
      asks: [...asksCum].reverse(),
      bids: bidsCum,
      askLevels,
      bidLevels,
      bestAsk,
      bestBid,
      spread,
      spreadPct,
      askVolume: asksAsc.reduce((s, o) => s + o.num_shares, 0),
      bidVolume: bidsDesc.reduce((s, o) => s + o.num_shares, 0),
    };
  }, [askOrders, bidOrders, openOrders, selectedAsset]);

  // Keep the best ask in view. The ask ladder is pinned to its bottom whenever
  // its content or size changes, unless the user has scrolled up to browse
  // deeper asks; scrolling back to the bottom (or switching asset) re-pins it.
  useEffect(() => {
    stickAsksRef.current = true;
  }, [selectedAsset]);

  useLayoutEffect(() => {
    const el = asksRef.current;
    if (!el) return undefined;
    const pin = () => {
      if (stickAsksRef.current) el.scrollTop = el.scrollHeight;
    };
    const onScroll = () => {
      stickAsksRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 4;
    };
    pin();
    el.addEventListener('scroll', onScroll, { passive: true });
    // The container gets its height from the flex layout, which can settle after
    // the rows render; re-pin whenever it or its content is resized.
    const observer = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(pin) : null;
    observer?.observe(el);
    const table = el.firstElementChild;
    if (table) observer?.observe(table);
    return () => {
      el.removeEventListener('scroll', onScroll);
      observer?.disconnect();
    };
  }, [selectedAsset, book.asks]);

  const orderPath = (side, id, p, n) =>
    `qx/order/<tick>/${assetsNIssuer.get(selectedAsset)}/${selectedAsset}/${side}/${id}/${p}/${n}/`;

  const place = (side) =>
    requestConfirm({
      title: side === 'BID' ? 'Place buy order?' : 'Place sell order?',
      confirmLabel: side === 'BID' ? 'Buy' : 'Sell',
      confirmColor: side === 'BID' ? 'primary' : 'secondary',
      body: (
        <>
          <Typography sx={{ fontWeight: 600 }}>
            {side === 'BID' ? 'Buy' : 'Sell'} {formatNumber(amountNum)} {selectedAsset} @{' '}
            {formatNumber(priceNum)} QU
          </Typography>
          <Typography variant='body2' color='text.secondary'>
            Total {formatNumber(total)} QU
          </Typography>
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            From
          </Typography>
          <IdText id={selectedId} full copy={false} />
          {(warningsFor?.(selectedId) || []).map((text) => (
            <Typography key={text} variant='body2' sx={{ mt: 1.5, color: 'warning.main' }}>
              {text}
            </Typography>
          ))}
        </>
      ),
      onConfirm: () => onAction(orderPath(side, selectedId, priceNum, amountNum)),
    });

  // The ladder's "fill the form" click (onFill) prefills price and shares; the
  // checks above then decide whether the resulting order is affordable.

  const cancel = (order, side) =>
    requestConfirm({
      title: 'Cancel this order?',
      confirmLabel: 'Cancel order',
      confirmColor: 'error',
      body: (
        <Typography>
          {side === 'ASK' ? 'Sell' : 'Buy'} {formatNumber(order.num_shares)} {selectedAsset} @{' '}
          {formatNumber(order.price)} QU
        </Typography>
      ),
      onConfirm: () =>
        onAction(
          orderPath(side === 'ASK' ? 'REMOVEASK' : 'REMOVEBID', selectedId, order.price, order.num_shares)
        ),
    });

  const fillPrice = (p) => setPrice(String(p));
  const fillFromLadder = (o) => {
    setPrice(String(o.price));
    setAmount(String(o.cumulative));
  };

  const cancelOpen = (o) =>
    requestConfirm({
      title: 'Cancel this order?',
      confirmLabel: 'Cancel order',
      confirmColor: 'error',
      body: (
        <Typography>
          {o.side === 'ASK' ? 'Sell' : 'Buy'} {formatNumber(o.num_shares)} {o.asset} @ {formatNumber(o.price)} QU
        </Typography>
      ),
      onConfirm: () =>
        onAction(
          `qx/order/<tick>/${o.issuer || assetsNIssuer.get(o.asset)}/${o.asset}/${o.side === 'ASK' ? 'REMOVEASK' : 'REMOVEBID'}/${o.entity}/${o.price}/${o.num_shares}/`
        ),
    });

  return (
    <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      {miningActive && (
        <Typography variant='body2' sx={{ color: 'error.main', fontWeight: 600, mb: 1, flexShrink: 0 }}>
          Orderbook is not updating while Random Mining
        </Typography>
      )}
      {/* Order ticket on one line where it fits; the book below gets the rest of the window. */}
      <Paper variant='outlined' sx={{ px: 2, py: 1.5, mb: 2, flexShrink: 0 }}>
        <Stack direction='row' spacing={1.5} useFlexGap sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
          <FormControl size='small' sx={{ minWidth: 200, flex: 1 }}>
            <InputLabel id='qx-id-label'>Identity</InputLabel>
            <Select
              labelId='qx-id-label'
              value={identities.some((i) => i.id === selectedId) ? selectedId : ''}
              label='Identity'
              onChange={(e) => onSelectId(e.target.value)}
            >
              {identities.map((item) => (
                <MenuItem key={item.id} value={item.id}>
                  <Identicon id={item.id} size={18} sx={{ mr: 1 }} />
                  {labels?.[item.id] && <Box component='span' sx={{ fontWeight: 600, mr: 1 }}>{labels[item.id]}</Box>}
                  <Box component='span' sx={{ fontFamily: 'monospace' }}>{shortenId(item.id, 6, 6)}</Box>
                  <Box component='span' sx={{ ml: 1, color: 'text.secondary' }}>
                    {formatNumber(item.balance)} QU
                  </Box>
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <FormControl size='small' sx={{ width: 130, flexShrink: 0 }}>
            <InputLabel id='qx-asset-label'>Asset</InputLabel>
            <Select
              labelId='qx-asset-label'
              value={selectedAsset || ''}
              label='Asset'
              onChange={(e) => onSelectAsset(e.target.value)}
            >
              {assets.map((asset) => (
                <MenuItem key={asset} value={asset}>{asset}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <Tooltip title={amount ? formatNumber(amount) : ''} placement='top'>
            <TextField
              label='Shares'
              size='small'
              value={amount}
              onChange={(e) => digitsOnly(e.target.value) && setAmount(e.target.value)}
              sx={{ width: 110, flexShrink: 0 }}
            />
          </Tooltip>
          <Tooltip title={price ? formatNumber(price) : ''} placement='top'>
            <TextField
              label='Price (QU)'
              size='small'
              value={price}
              onChange={(e) => digitsOnly(e.target.value) && setPrice(e.target.value)}
              sx={{ width: 130, flexShrink: 0 }}
            />
          </Tooltip>
          <Box sx={{ flexShrink: 0, minWidth: 90, lineHeight: 1.1 }}>
            <Typography variant='caption' color='text.secondary' sx={{ display: 'block' }}>Total</Typography>
            <Typography variant='body2' sx={{ fontWeight: 600, whiteSpace: 'nowrap', color: cannotAfford ? 'error.main' : 'inherit' }}>
              {formatNumber(total)} QU
            </Typography>
          </Box>
          {signer && (
            <Tooltip title='What the selected identity can spend or sell' placement='top'>
              <Box sx={{ flexShrink: 0, minWidth: 110, lineHeight: 1.1 }}>
                <Typography variant='caption' color='text.secondary' sx={{ display: 'block' }}>Available</Typography>
                <Typography variant='body2' sx={{ whiteSpace: 'nowrap' }}>
                  <Box component='span' sx={{ color: cannotAfford ? 'error.main' : 'inherit' }}>
                    {quBalance === null ? '? ' : formatNumber(quBalance)} QU
                  </Box>
                  {selectedAsset && (
                    <Box component='span' sx={{ color: notEnoughShares ? 'error.main' : 'inherit' }}>
                      {' · '}{holding === null ? '?' : formatNumber(holding)} {selectedAsset}
                    </Box>
                  )}
                </Typography>
              </Box>
            </Tooltip>
          )}
          <Stack direction='row' spacing={1} sx={{ flexShrink: 0 }}>
            <Tooltip title={busy ? 'An order is waiting for its tick — new orders are enabled once it lands.' : buyBlocked} placement='top'>
              <span>
                <Button
                  variant='contained'
                  disabled={!valid || busy || cannotAfford}
                  startIcon={<ShoppingCartIcon />}
                  onClick={() => place('BID')}
                  sx={{ height: 40 }}
                >
                  Buy
                </Button>
              </span>
            </Tooltip>
            <Tooltip title={busy ? 'An order is waiting for its tick — new orders are enabled once it lands.' : sellBlocked} placement='top'>
              <span>
                <Button
                  variant='contained'
                  color='secondary'
                  disabled={!valid || busy || notEnoughShares}
                  startIcon={<SellIcon />}
                  onClick={() => place('ASK')}
                  sx={{ height: 40 }}
                >
                  Sell
                </Button>
              </span>
            </Tooltip>
          </Stack>
        </Stack>
      </Paper>

      {/* Book on the left fills the height; depth chart and open orders share the right column. */}
      <Stack direction='row' spacing={2} sx={{ flex: 1, minHeight: 0 }}>
        <Paper
          variant='outlined'
          sx={{ flex: 1, minWidth: 0, minHeight: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}
        >
          <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', gap: 2, flexShrink: 0 }}>
            <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
              Order book{selectedAsset ? ` · ${selectedAsset}` : ''}
            </Typography>
            <Typography variant='caption' color='text.secondary'>
              {formatNumber(book.askVolume)} for sale · {formatNumber(book.bidVolume)} wanted
            </Typography>
            <Box sx={{ flex: 1 }} />
            <BookAge age={bookAge} />
          </Box>

          <Box sx={{ borderTop: 1, borderColor: 'divider', flex: '1 1 0', minHeight: 0, display: 'flex', flexDirection: 'column' }}>
            <Ladder
              side='ASK'
              orders={book.asks}
              ownId={selectedId}
              busy={busy}
              onCancel={cancel}
              onFill={fillFromLadder}
              containerRef={asksRef}
              emptyText={bookLoading ? 'Loading order book…' : 'No sell orders.'}
            />
          </Box>

          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              px: 2,
              py: 0.75,
              borderTop: 1,
              borderBottom: 1,
              borderColor: 'divider',
              bgcolor: 'action.hover',
              flexShrink: 0,
            }}
          >
            <Typography variant='body2' sx={{ display: 'flex', gap: 2 }}>
              <Box component='span' sx={{ color: 'error.main' }}>
                Ask{' '}
                {book.bestAsk !== undefined ? (
                  <Box component='b' onClick={() => fillPrice(book.bestAsk)} sx={{ cursor: 'pointer' }}>
                    {formatNumber(book.bestAsk)}
                  </Box>
                ) : (
                  '—'
                )}
              </Box>
              <Box component='span' sx={{ color: 'success.main' }}>
                Bid{' '}
                {book.bestBid !== undefined ? (
                  <Box component='b' onClick={() => fillPrice(book.bestBid)} sx={{ cursor: 'pointer' }}>
                    {formatNumber(book.bestBid)}
                  </Box>
                ) : (
                  '—'
                )}
              </Box>
            </Typography>
            <Typography variant='body2' color='text.secondary'>
              {book.spread !== null
                ? `Spread ${formatNumber(book.spread)} QU (${book.spreadPct.toFixed(2)}%)`
                : 'No spread — one side is empty'}
            </Typography>
          </Box>

          <Box sx={{ flex: '1 1 0', minHeight: 0, display: 'flex', flexDirection: 'column' }}>
            <Ladder
              side='BID'
              orders={book.bids}
              ownId={selectedId}
              busy={busy}
              onCancel={cancel}
              onFill={fillFromLadder}
              emptyText={bookLoading ? 'Loading order book…' : 'No buy orders.'}
            />
          </Box>
        </Paper>

        <Box sx={{ width: SIDE_WIDTH, flexShrink: 0, minHeight: 0, display: 'flex', flexDirection: 'column', gap: 2 }}>
          <Paper variant='outlined' sx={{ p: 1.5, flexShrink: 0 }}>
            <Typography variant='subtitle2' sx={{ fontWeight: 600, mb: 1 }}>
              Depth
            </Typography>
            <DepthChart bids={book.bidLevels} asks={book.askLevels} asset={selectedAsset || ''} width={SIDE_WIDTH - 24} height={200} />
          </Paper>
          <OpenOrders orders={openOrders} labels={labels} busy={busy} onCancel={cancelOpen} />
        </Box>
      </Stack>
    </Box>
  );
}
