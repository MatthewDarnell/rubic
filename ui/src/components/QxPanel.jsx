import React, { useEffect, useMemo, useRef, useState } from 'react';
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
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutline';
import IdText from './IdText';
import Identicon from './Identicon';
import DepthChart from './DepthChart';
import { digitsOnly, formatNumber, shortenId } from '../utils/format';

const LADDER_HEIGHT = 240;
const CHART_LEVELS = 40; // price levels per side shown in the depth chart

const toNum = (v) => Number(v) || 0;

// Rows of one side of the book. Each row carries a background bar proportional
// to cumulative depth, like a classic exchange ladder.
const Ladder = ({ side, orders, ownId, busy, onCancel, onFill, containerRef, emptyText }) => {
  const theme = useTheme();
  const color = side === 'ASK' ? theme.palette.error.main : theme.palette.success.main;
  const maxCum = orders.length ? Math.max(...orders.map((o) => o.cumulative)) : 1;
  return (
    <TableContainer ref={containerRef} sx={{ maxHeight: LADDER_HEIGHT, overflow: 'auto' }}>
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
                title={`Fill the form: ${formatNumber(o.cumulative)} shares at ${formatNumber(o.price)} QU`}
                sx={{
                  cursor: onFill ? 'pointer' : 'default',
                  background: `linear-gradient(to left, ${color}22 ${pct}%, transparent ${pct}%)`,
                  '& td': mine ? { fontWeight: 700 } : undefined,
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

const OpenOrders = ({ orders, labels, busy, onCancel }) => (
  <Paper variant='outlined' sx={{ mb: 3, overflow: 'hidden' }}>
    <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', gap: 1.5 }}>
      <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
        My open orders
      </Typography>
      <Chip size='small' label={orders.length} />
    </Box>
    <TableContainer sx={{ borderTop: 1, borderColor: 'divider', maxHeight: 220, overflow: 'auto' }}>
      <Table size='small' stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>Asset</TableCell>
            <TableCell>Side</TableCell>
            <TableCell align='right'>Price (QU)</TableCell>
            <TableCell align='right'>Shares</TableCell>
            <TableCell align='right'>Total (QU)</TableCell>
            <TableCell>Identity</TableCell>
            <TableCell sx={{ width: 40 }} />
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.length === 0 && (
            <TableRow>
              <TableCell colSpan={7} align='center' sx={{ py: 2.5, color: 'text.secondary' }}>
                You have no resting orders on QX.
              </TableCell>
            </TableRow>
          )}
          {orders.map((o, index) => (
            <TableRow key={`${o.asset}-${o.side}-${o.entity}-${o.price}-${index}`} hover>
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
              <TableCell align='right' className='mono' sx={{ color: 'text.secondary' }}>
                {formatNumber(Number(o.price) * Number(o.num_shares))}
              </TableCell>
              <TableCell>
                {labels?.[o.entity] ? (
                  <Tooltip title={o.entity}>
                    <Typography variant='body2' sx={{ fontWeight: 600 }}>{labels[o.entity]}</Typography>
                  </Tooltip>
                ) : (
                  <IdText id={o.entity} head={6} tail={6} />
                )}
              </TableCell>
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
  identities,
  labels,
  assetsNIssuer,
  selectedAsset,
  onSelectAsset,
  selectedId,
  onSelectId,
  askOrders,
  bidOrders,
  openOrders,
  busy,
  onAction,
  requestConfirm,
}) {
  const [price, setPrice] = useState('');
  const [amount, setAmount] = useState('');
  const asksRef = useRef(null);
  const lastAnchoredAsset = useRef(null);

  const assets = [...assetsNIssuer.keys()].sort();
  const priceNum = Number(price);
  const amountNum = Number(amount);
  const valid = selectedId && selectedAsset && priceNum > 0 && amountNum > 0;
  const total = priceNum * amountNum;

  // Asks: highest price at the top, best (lowest) ask at the bottom next to the spread.
  // Bids: best (highest) bid at the top. Cumulative depth accumulates away from the spread.
  const book = useMemo(() => {
    const norm = (o) => ({
      ...o,
      price: toNum(o.price),
      num_shares: toNum(o.num_shares),
      total: toNum(o.price) * toNum(o.num_shares),
    });
    const asksAsc = askOrders.map(norm).sort((a, b) => a.price - b.price);
    const bidsDesc = bidOrders.map(norm).sort((a, b) => b.price - a.price);
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
  }, [askOrders, bidOrders]);

  // Keep the best ask in view: scroll the ask ladder to the bottom when the asset changes
  // or when data first arrives, but leave it alone while the user is scrolling.
  useEffect(() => {
    const el = asksRef.current;
    if (!el) return;
    if (lastAnchoredAsset.current !== selectedAsset || book.asks.length > 0) {
      if (lastAnchoredAsset.current !== selectedAsset || el.dataset.anchored !== '1') {
        el.scrollTop = el.scrollHeight;
        el.dataset.anchored = book.asks.length > 0 ? '1' : '0';
        lastAnchoredAsset.current = selectedAsset;
      }
    }
  }, [selectedAsset, book.asks.length]);

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
        </>
      ),
      onConfirm: () => onAction(orderPath(side, selectedId, priceNum, amountNum)),
    });

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
          `qx/order/<tick>/${assetsNIssuer.get(o.asset)}/${o.asset}/${o.side === 'ASK' ? 'REMOVEASK' : 'REMOVEBID'}/${o.entity}/${o.price}/${o.num_shares}/`
        ),
    });

  return (
    <Box>
      <Paper variant='outlined' sx={{ p: 2, mb: 3 }}>
        <Stack direction='row' spacing={2} flexWrap='wrap' useFlexGap alignItems='flex-start'>
          <FormControl size='small' sx={{ minWidth: 260, flex: 2 }}>
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
                  <Box component='span' sx={{ fontFamily: 'monospace' }}>{shortenId(item.id, 8, 8)}</Box>
                  <Box component='span' sx={{ ml: 1, color: 'text.secondary' }}>
                    {formatNumber(item.balance)} QU
                  </Box>
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <FormControl size='small' sx={{ minWidth: 160, flex: 1 }}>
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
          <TextField
            label='Shares'
            size='small'
            value={amount}
            onChange={(e) => digitsOnly(e.target.value) && setAmount(e.target.value)}
            helperText={amount ? formatNumber(amount) : ' '}
            sx={{ width: 160 }}
          />
          <TextField
            label='Price (QU)'
            size='small'
            value={price}
            onChange={(e) => digitsOnly(e.target.value) && setPrice(e.target.value)}
            helperText={price ? formatNumber(price) : ' '}
            sx={{ width: 180 }}
          />
          <Box sx={{ alignSelf: 'center', minWidth: 160 }}>
            <Typography variant='body2' color='text.secondary'>Total</Typography>
            <Typography sx={{ fontWeight: 600 }}>{formatNumber(total)} QU</Typography>
          </Box>
          <Button
            variant='contained'
            disabled={!valid || busy}
            startIcon={<ShoppingCartIcon />}
            onClick={() => place('BID')}
            sx={{ height: 40 }}
          >
            Buy
          </Button>
          <Button
            variant='contained'
            color='secondary'
            disabled={!valid || busy}
            startIcon={<SellIcon />}
            onClick={() => place('ASK')}
            sx={{ height: 40 }}
          >
            Sell
          </Button>
        </Stack>
        {busy && (
          <Typography variant='body2' color='text.secondary' sx={{ mt: 1 }}>
            An order is waiting for its tick — new orders are enabled once it lands.
          </Typography>
        )}
      </Paper>

      <OpenOrders orders={openOrders} labels={labels} busy={busy} onCancel={cancelOpen} />

      <Stack direction='row' spacing={2} alignItems='stretch' flexWrap='wrap' useFlexGap>
        <Paper variant='outlined' sx={{ flex: '1 1 560px', minWidth: 0, overflow: 'hidden' }}>
          <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant='subtitle2' sx={{ fontWeight: 600 }}>
              Order book{selectedAsset ? ` · ${selectedAsset}` : ''}
            </Typography>
            <Typography variant='caption' color='text.secondary'>
              {formatNumber(book.askVolume)} for sale · {formatNumber(book.bidVolume)} wanted
            </Typography>
          </Box>

          <Box sx={{ borderTop: 1, borderColor: 'divider' }}>
            <Ladder
              side='ASK'
              orders={book.asks}
              ownId={selectedId}
              busy={busy}
              onCancel={cancel}
              onFill={fillFromLadder}
              containerRef={asksRef}
              emptyText='No sell orders.'
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

          <Ladder
            side='BID'
            orders={book.bids}
            ownId={selectedId}
            busy={busy}
            onCancel={cancel}
            onFill={fillFromLadder}
            emptyText='No buy orders.'
          />
        </Paper>

        <Paper variant='outlined' sx={{ flex: '0 0 auto', p: 1.5 }}>
          <Typography variant='subtitle2' sx={{ fontWeight: 600, mb: 1 }}>
            Depth
          </Typography>
          <DepthChart bids={book.bidLevels} asks={book.askLevels} asset={selectedAsset || ''} width={380} height={440} />
        </Paper>
      </Stack>
    </Box>
  );
}
