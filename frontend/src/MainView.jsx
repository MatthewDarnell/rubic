import {
  Box,
  Button,
  TextField,
  Typography,
  Divider,
  InputAdornment,
  IconButton,
  LinearProgress,
  Select,
  MenuItem,
  InputLabel,
  FormControl,
  Paper,
  ThemeProvider,
  createTheme,
  CssBaseline,
  CircularProgress,
  Checkbox,
  Link,
  Autocomplete,
  Stack,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Tabs,
  Tab,
  Fade,
  List,
  ListItem,
  ListItemText,
  Card,
  CardHeader,
  CardContent,
  Chip,
  Avatar,
  Grid,
  Switch,
} from '@mui/material';
import TabContext from '@mui/lab/TabContext';
import TabPanel from '@mui/lab/TabPanel';
import TabList from '@mui/lab/TabList';
import LockIcon from '@mui/icons-material/Lock';
import FormatListBulletedIcon from '@mui/icons-material/FormatListBulleted';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import CheckIcon from '@mui/icons-material/Check';
import WifiIcon from '@mui/icons-material/Wifi';
import WifiOffIcon from '@mui/icons-material/WifiOff';
import WalletIcon from '@mui/icons-material/Wallet';
import ExploreIcon from '@mui/icons-material/Explore';
import NotificationsIcon from '@mui/icons-material/Notifications';
import CasinoIcon from '@mui/icons-material/Casino';
import FileUploadIcon from '@mui/icons-material/FileUpload';

import { indigo, teal, amber } from '@mui/material/colors';
import { styled, keyframes } from '@mui/system';

import CloseIcon from '@mui/icons-material/Close';
import OpenInFullIcon from '@mui/icons-material/OpenInFull';
import SendIcon from '@mui/icons-material/Send';
import DisabledByDefaultIcon from '@mui/icons-material/DisabledByDefault';
import ReplayIcon from '@mui/icons-material/Replay';
import DeleteIcon from '@mui/icons-material/Delete';
import ReceiptIcon from '@mui/icons-material/Receipt';
import ArrowDownwardIcon from '@mui/icons-material/ArrowDownward';
import ArrowUpwardIcon from '@mui/icons-material/ArrowUpward';
import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';
import CircleIcon from '@mui/icons-material/Circle';
import TokenIcon from '@mui/icons-material/Token';
import LightModeIcon from '@mui/icons-material/LightMode';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import UnfoldLessIcon from '@mui/icons-material/UnfoldLess';
import Badge from '@mui/material/Badge';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';
import SellIcon from '@mui/icons-material/Sell';
import ReportProblemIcon from '@mui/icons-material/ReportProblem';
import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { apiCall } from './api';
import { serverIp } from './api_config';
import { doArrayElementsAgree } from './api_helper';

const seedRegex = /^[a-z]{55}$/;

const POLLING_INTERVAL = 3000;
const TICK_INTERVAL = 1000;

const getTheme = (mode) =>
  createTheme({
    palette: {
      mode,
      ...(mode === 'dark' && {
        background: {
          default: '#121928',
          paper: '#1a2235',
        },
      }),
      ...(mode === 'light' && {
        background: {
          default: '#cccccc',
          paper: '#999999',
        },
      }),
    },
    typography: {
      fontFamily: 'Exo, sans-serif',
      h6: {
        fontSize: '1.1rem',
        fontWeight: 600,
      },
      body1: {
        fontSize: '0.95rem',
      },
      button: {
        textTransform: 'none',
        fontSize: '0.9rem',
      },
    },
    components: {
      MuiTableCell: {
        styleOverrides: {
          root: {
            fontSize: '0.9rem',
          },
        },
      },
    },
  });

const MainView = () => {
  const [connected, setConnected] = useState(true);
  const [latestTick, setLatestTick] = useState(0);
  const [ip, setIp] = useState('');
  const [port, setPort] = useState(21841);
  const [peers, setPeers] = useState([]);
  const [seed, setSeed] = useState('');
  const [seedError, setSeedError] = useState('');
  const [showSeed, setShowSeed] = useState(false);
  const [id, setId] = useState('');
  const [destinationId, setDestinationId] = useState('');
  const [identities, setIdentities] = useState([]);
  const [transfers, setTransfers] = useState([]);
  const [assetTransfers, setAssetTransfers] = useState([]);
  const [QXtransfers, setQXTransfers] = useState([]);
  const [password, setPassword] = useState('');

  const [retypePassword, setRetypePassword] = useState('');
  const [encodedURI, setEncodedURI] = useState('');
  const [price, setPrice] = useState(0);
  const [error, setError] = useState('');
  const [qxPrice, setQxPrice] = useState(0);
  const [qxAmount, setQxAmount] = useState(0);

  const [isEncrypted, setIsEncrypted] = useState(false);
  const [action, setAction] = useState('');
  const [tickOffset, setTickOffset] = useState(0);
  const [unlockTimer, setUnlockTimer] = useState(60000); // 1 minute
  const [replay, setReplay] = useState(true);
  const [allowNonEncrypted, setAllowNonEncrypted] = useState(false);
  const [minPeers, setMinPeers] = useState(5);
  const [maxPeers, setMaxPeers] = useState(8);
  const [showProgress, setShowProgress] = useState(false);
  const [orderTick, setOrderTick] = useState(0);

  const [askOrders, setAskOrders] = useState([]);
  const [bidOrders, setBidOrders] = useState([]);

  const [totalBalance, setTotalBalance] = useState(0);
  const [amount, setAmount] = useState(0);
  const [assetAmount, setAssetAmount] = useState(0);
  const [assetsNIssuer, setAssetsNIssuer] = React.useState([]);
  const [assetsBalance, setAssetsBalance] = React.useState(new Map());
  const [displayTotalAssets, setDisplayTotalAssets] = React.useState(true);

  const [assetSource, setAssetSource] = useState('');
  const [assetDestination, setAssetDestination] = useState('');

  const [selectedAsset, setSelectedAsset] = React.useState(null);

  const [invalidPassword, setInvalidPassword] = useState('');

  const [tab, setTab] = React.useState('1');

  const [themeMode, setThemeMode] = useState('dark');
  const theme = useMemo(() => getTheme(themeMode), [themeMode]);

  const [selectedPeer, setSelectedPeer] = React.useState(null);
  const [selectedTxId, setSelectedTxId] = React.useState(null);
  const [timeOption, setTimeOption] = useState(60 * 24 * 7 * 31 * 12 * 10);

  const flash = keyframes`
  0%, 100% { opacity: 1; }
  50% { opacity: 0.2; }
`;

  const timeOptions = [
    { label: '10 min', minutes: 10 },
    { label: '1 hour', minutes: 60 },
    { label: '1 day', minutes: 60 * 24 },
    { label: '1 week', minutes: 60 * 24 * 7 },
    { label: '1 month', minutes: 60 * 24 * 7 * 31 },
    { label: '1 year', minutes: 60 * 24 * 7 * 31 * 12 },
    { label: 'all', minutes: 60 * 24 * 7 * 31 * 12 * 10 }, // 10 years
  ];

  const reverseSamePriceGroups = (arr) => {
    const result = [...arr]; // shallow copy
    let i = 0;

    while (i < result.length) {
      let j = i + 1;

      // Find the end index of the current group with same price
      while (j < result.length && result[j].price === result[i].price) {
        j++;
      }

      // Reverse the group [i, j)
      result.splice(i, j - i, ...result.slice(i, j).reverse());

      // Move to the next group
      i = j;
    }

    return result;
  };

  const commitAction = async (action) => {
    const unlocked = await apiCall(`wallet/unlocked`);
    console.log(unlocked.data);
    if (unlocked.data) {
      actionHandler(action, true);
    } else setAction(action);
  };

  const handleDelete = async (id) => {
    commitAction(`identity/delete/${id}/`);
  };

  const handleSelectPeer = (id, ip, port) => {
    setSelectedPeer((prevSelected) => (prevSelected === id ? null : id));
    setIp(ip);
    setPort(port);
  };

  const handleTxShow = (id) => {
    setSelectedTxId((prevSelected) => (prevSelected === id ? null : id));
  };

  const handleAssetUpdate = async (id) => {
    setId(id);
    const assetsById = await apiCall(`asset/balance/${id}`);
    setAssetsBalance(
      new Map(assetsById.data.map((item) => [item.name, item.balance]))
    );
  };

  const handleTabChange = async (event, newValue) => {
    // if entering peers tab, update to current settings for min/max peers
    if (newValue !== '8') {
      // not settings tab
      const setMinPeersResult = await apiCall(`/peers/limit/min/${minPeers}`);
      const setMaxPeersResult = await apiCall(`/peers/limit/max/${maxPeers}`);
      // console.log(setMinPeersResult, setMaxPeersResult);
    }
    setTab(newValue);
  };

  const [openPanel, setOpenPanel] = useState(null);

  const handleChange = (panel) => (event, isExpanded) => {
    setOpenPanel(isExpanded ? panel : null);
  };

  const getQubicPriceUSD = async () => {
    try {
      const response = await fetch(
        'https://api.coingecko.com/api/v3/simple/price?ids=qubic-network&vs_currencies=usd'
      );
      const data = await response.json();
      const price = data['qubic-network']?.usd;
      setPrice(price);

      console.log(`QUBIC price: $${price}`);
    } catch (error) {
      console.error('Error fetching QUBIC price:', error);
    }
  };

  useEffect(() => {
    // declare the async data fetching function
    const checkWalletEncryption = async () => {
      // get the data from the api
      const result = await apiCall('wallet/is_encrypted');
      // console.log('apiCall(wallet/is_encrypted', result);
      setIsEncrypted(typeof result.data === 'boolean');

      const assets = await apiCall('asset/issued');
      const newMap = new Map(
        assets.data.reduce(
          (acc, val, idx, arr) =>
            idx % 2 === 0 ? [...acc, [val, arr[idx + 1]]] : acc,
          []
        )
      );
      setAssetsNIssuer(newMap);
      setSelectedAsset([...newMap.keys()].sort()[0]);

      const limits = await apiCall('peers/limit');
      setMaxPeers(limits.data.max);
      setMinPeers(limits.data.min);
    };

    const getQubicData = async () => {
      // get the data from the api
      await getQubicPriceUSD();
    };

    getQubicData();
    checkWalletEncryption();
  }, []);

  useEffect(() => {
    const intervalId = setInterval(async () => {
      setConnected(true);
      const tick = await apiCall('tick');
      if (!tick.success) setConnected(false);
      setLatestTick(tick.data);
      // console.log('setLatestTick ', tick.data, ' orderTick ', orderTick);
      setShowProgress(orderTick >= tick.data);
    }, TICK_INTERVAL);
    return () => clearInterval(intervalId);
  }, [orderTick, isEncrypted]);

  useEffect(() => {
    const intervalId = setInterval(async () => {
      setConnected(true);
      const peers = await apiCall('peers');
      // console.log(peers);
      if (!peers.success) setConnected(false);
      const identities = await apiCall('identities');
      // console.log(identities.data);
      const ids = identities.data.filter((_, index) => index % 2 === 0);
      const encrypted = identities.data.filter((_, index) => index % 2 === 1);

      const mergedIds = ids.map((id, index) => ({
        id: id,
        encrypted: encrypted[index],
      }));

      let localTotalBalance = 0;

      const fullMergeIds = await Promise.all(
        mergedIds.map(async (item) => {
          let balance = '';
          const result = await apiCall(`balance/${item.id}`);
          const resultAssets = await apiCall(`asset/balance/${item.id}`);
          const res = result.data;

          if (res.length < 3) {
            balance = 'Not Yet Reported';
            return {
              ...item,
              balance,
            };
          }

          const balanceArray = [];
          for (let i = 0; i < res.length; i += 3) {
            balanceArray.push(res[i + 2]);
          }
          const isQuorumMet = doArrayElementsAgree(balanceArray, 50); // 1/2 of peers agree at this tick?

          let balanceResult = {
            ...item,
            assets: resultAssets?.data,
            balance:
              balanceArray.every((v) => v === res[0]) || isQuorumMet >= 0
                ? balanceArray[0]
                : 'Peer Balance Mismatch',
          };

          localTotalBalance += parseInt(balanceResult.balance);

          return balanceResult;
        })
      );

      const totalByAsset = fullMergeIds.reduce((acc, obj) => {
        obj.assets?.forEach((asset) => {
          const balance = parseFloat(asset.balance);
          if (!isNaN(balance)) {
            acc[asset.name] = (acc[asset.name] || 0) + balance;
          }
        });
        return acc;
      }, {});

      if (id) {
        const assetsById = await apiCall(`asset/balance/${id}`);
        setAssetsBalance(
          new Map(assetsById.data.map((item) => [item.name, item.balance]))
        );
      } else {
        setAssetsBalance(new Map(Object.entries(totalByAsset)));
      }

      const transfer = await apiCall('transfer/0/0/0');
      setTransfers(transfer.data);
      const assetTransfers = await apiCall('asset/transfer/0/0/0');
      setAssetTransfers(assetTransfers.data);
      console.log(assetTransfers.data);
      const qxOrders = await apiCall(`qx/orders/1/1000/0`);
      setQXTransfers(qxOrders.data);

      setTotalBalance(localTotalBalance);

      if (selectedAsset) {
        const orderbookAsk = await apiCall(
          `qx/orderbook/${selectedAsset}/ASK/1000/0`
        );
        const orderbookBid = await apiCall(
          `qx/orderbook/${selectedAsset}/BID/1000/0`
        );

        setAskOrders(reverseSamePriceGroups(orderbookAsk.data.reverse()) || []);
        setBidOrders(orderbookBid.data || []);
      }

      setIdentities(fullMergeIds);

      // console.log(fullMergeIds);
      // const peersArray = peers.data.map(
      //   ([id, ip, nickName, tag1, tag2, tag3, lastResponded, tag4]) => ({
      //     id,
      //     ip,
      //     nickName,
      //     enabled: tag1,
      //     tag2,
      //     tag3,
      //     lastResponded,
      //     tag4,
      //   })
      // );
      // console.log(peers.data);
      setPeers(peers.data);
      // qubic data
    }, POLLING_INTERVAL);

    return () => clearInterval(intervalId);
  }, [id, selectedAsset]);

  const handleInputChangeIp = useCallback(
    (setter) => (event) => {
      const newValue = event.target.value;
      if (/^[\d.]*$/.test(newValue)) {
        setter(newValue);
      }
    },
    []
  );

  const handleInputChange = useCallback(
    (setter) => (event) => {
      const newValue = event.target.value;
      if (/^[\d]*$/.test(newValue)) {
        setter(newValue);
      }
    },
    []
  );

  const handleInputChangeId = useCallback(
    (setter) => (event) => {
      const newValue = event.target.value;
      if (/^[A-Z]*$/.test(newValue)) {
        setter(newValue);
      }
    },
    []
  );

  const renderAddPeer = useCallback(
    () => (
      <>
        <TextField
          label='IP Address'
          variant='outlined'
          value={ip}
          onChange={handleInputChangeIp(setIp)}
          required
        />
        <TextField
          label='Port'
          variant='outlined'
          type='number'
          value={port}
          sx={{
            ml: 1,
          }}
          onChange={handleInputChange(setPort)}
          inputProps={{ min: 1, max: 65535 }}
          required
        />
        <Button
          onClick={async () => {
            // console.log(`peers/add/${ip}:${port}`);
            const result = await apiCall(`peers/add/${ip}:${port}`);
            // console.log(result);
            // console.log(result.data);
          }}
          sx={{
            ml: 1,
            mt: 1, // margin-right: theme.spacing(2)
          }}
          variant='contained'
          color='primary'
        >
          Connect to Peer
        </Button>
        <Button
          disabled={!!!selectedPeer}
          onClick={async () => {
            // console.log(`peers/delete/${selectedPeer}`);
            const result = await apiCall(`peers/delete/${selectedPeer}`);
            // console.log(result);
            // console.log(result.data);
          }}
          sx={{
            ml: 1,
            mt: 1, // margin-right: theme.spacing(2)
          }}
          variant='contained'
          color='primary'
        >
          Disconnect from Peer
        </Button>
      </>
    ),
    [ip, port, selectedPeer]
  );

  const renderPeersTable = useCallback(
    () => (
      <TableContainer component={Paper} sx={{ mt: 2, mb: 3 }}>
        <Table size='small'>
          <TableHead>
            <TableRow>
              <TableCell />
              <TableCell>IP</TableCell>
              <TableCell align='right'>NickName</TableCell>
              <TableCell align='right'>Last Responded</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {peers.map((item, index) => (
              <TableRow
                onClick={() =>
                  handleSelectPeer(
                    item.id,
                    item.ip.slice(0, item.ip.indexOf(':')),
                    item.ip.slice(item.ip.indexOf(':') + 1)
                  )
                }
                key={item.id}
                sx={{
                  backgroundColor:
                    item.id === selectedPeer
                      ? '#555765'
                      : item.enabled === '-1'
                      ? '#222432'
                      : 'inherit',
                  color: item.enabled === '-1' ? 'green' : 'inherit',
                  // pointerEvents: item.enabled === '-1' ? 'none' : 'auto', // Optional: disable interactions
                }}
              >
                <TableCell>
                  {item.connected === '1' ? (
                    <WifiIcon sx={{ color: 'lightgreen' }}></WifiIcon>
                  ) : (
                    <WifiOffIcon sx={{ color: 'red' }}></WifiOffIcon>
                  )}
                </TableCell>
                <TableCell>{item.ip}</TableCell>
                <TableCell align='right'>{item.nick}</TableCell>
                <TableCell align='right'>{item.last_responded}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    ),
    [peers, selectedPeer, minPeers, maxPeers]
  );

  const renderSendModal = useCallback(
    () => (
      <>
        <TextField
          size='small'
          sx={{
            mr: 1, // margin-right: theme.spacing(2)
            pr: 1, // padding-right (only affects outer container if applicable)
            width: '150px', // optional: custom width
          }}
          // label='Amount'
          placeholder='Amount'
          variant='outlined'
          value={amount}
          onChange={handleInputChange(setAmount)}
          required
        />
        <TextField
          size='small'
          sx={{
            mr: 1, // margin-right: theme.spacing(2)
            pr: 1, // padding-right (only affects outer container if applicable)
            width: '700px', // optional: custom width
          }}
          // label='Destination ID'
          placeholder='Destination ID'
          variant='outlined'
          value={destinationId}
          onChange={handleInputChangeId(setDestinationId)}
          required
        />

        <Button
          variant='contained'
          disabled={!id}
          sx={{
            mt: 1,
            mb: 1,
          }}
          onClick={async () => {
            commitAction(`transfer/${id}/${destinationId}/${amount}/`);
            // const result = await apiCall(
            //   `transfer/${selectedId}/${destinationId}/${amount}/${
            //     tick.data + tickOffset
            //   }/${password}`
            // );
          }}
        >
          SEND
        </Button>
      </>
    ),
    [id, destinationId, latestTick, amount]
  );

  const renderShowTxList = useCallback(
    () => (
      <>
        <Paper elevation={3} sx={{ height: 300, overflow: 'auto' }}>
          <List>
            {transfers.map((item, index) => {
              if (
                item.source === selectedTxId ||
                item.destination === selectedTxId
              ) {
                return (
                  <ListItem key={index}>
                    {item.status === '0' ? (
                      <CheckIcon sx={{ color: 'lightgreen' }}></CheckIcon>
                    ) : item.status === '-1' ? (
                      <CheckIcon sx={{ color: 'yellow' }}></CheckIcon>
                    ) : (
                      <CheckIcon sx={{ color: 'red' }}></CheckIcon>
                    )}
                    <ListItemText
                      secondary={`${item.created} - ${
                        item.source === selectedTxId
                          ? item.destination
                          : item.source
                      }`}
                    />
                    {item.source === selectedTxId ? (
                      <ArrowUpwardIcon
                        sx={{
                          color: 'red',
                        }}
                      ></ArrowUpwardIcon>
                    ) : (
                      <ArrowDownwardIcon
                        sx={{
                          color: 'green',
                        }}
                      ></ArrowDownwardIcon>
                    )}
                    <ListItemText secondary={`${item.amount} QU`} />
                  </ListItem>
                );
              }
            })}
          </List>
        </Paper>
      </>
    ),
    [transfers, selectedTxId]
  );

  const formatId = (id, start = 5, end = 1) => {
    if (!id || id.length <= start + end) return id;
    return `${id.slice(0, start)} ...   `;
  };

  const renderIdentitiyTable = useCallback(
    () => (
      <TableContainer component={Paper} sx={{ mt: 2, mb: 3 }}>
        <Table size='small'>
          <TableHead>
            <TableRow>
              <TableCell>
                {' '}
                <LockIcon></LockIcon>{' '}
              </TableCell>
              <TableCell></TableCell>
              <TableCell></TableCell>
              <TableCell>ID</TableCell>
              <TableCell>TX</TableCell>
              <TableCell align='right'>BALANCE</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {identities.map((item, index) => (
              <TableRow key={item.id}>
                <TableCell>
                  {item.encrypted === 'true' ? (
                    <LockIcon
                      onClick={() => setId(item.id)}
                      style={{ color: id === item.id ? 'orange' : 'white' }}
                    ></LockIcon>
                  ) : (
                    <></>
                  )}
                </TableCell>
                <TableCell>
                  <DeleteIcon
                    sx={{
                      ml: 2,
                      color: 'orange',
                    }}
                    onClick={async () => await handleDelete(item.id)}
                  />
                </TableCell>
                <TableCell>
                  <SendIcon
                    sx={{
                      color: id === item.id ? 'yellow' : 'white',
                    }}
                    onClick={() => setId(item.id)}
                  />
                </TableCell>
                <TableCell>
                  {id === item.id ? (
                    <>
                      <Typography
                        sx={{
                          display: 'flex',
                          mt: 1,
                          mb: 1,
                          alignItems: 'center',
                          gap: 1,
                        }}
                      >
                        {item.id}
                      </Typography>
                      {renderSendModal()}
                    </>
                  ) : (
                    <Typography
                      sx={{
                        display: 'flex',
                        mt: 1,
                        mb: 1,
                        alignItems: 'center',
                        gap: 1,
                      }}
                    >
                      {item.id}
                    </Typography>
                  )}
                  {selectedTxId === item.id ? renderShowTxList() : <></>}
                </TableCell>

                <TableCell align='left'>
                  <Box></Box>
                  <Badge
                    badgeContent={
                      transfers.filter(
                        (transfer) =>
                          transfer.source === item.id ||
                          transfer.destination === item.id
                      ).length
                    }
                    color='primary'
                  >
                    <FormatListBulletedIcon
                      sx={{
                        color: selectedTxId === item.id ? 'yellow' : 'white',
                      }}
                      onClick={() =>
                        transfers.filter(
                          (transfer) =>
                            transfer.source === item.id ||
                            transfer.destination === item.id
                        ).length > 0
                          ? handleTxShow(item.id)
                          : console.log('nothing')
                      }
                    ></FormatListBulletedIcon>
                  </Badge>
                </TableCell>
                <TableCell align='right'>
                  {item.balance.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    ),
    [identities, id, amount, destinationId, selectedTxId, id]
  );

  const actionHandler = async (action, unlocked) => {
    let result = '';
    let actionPassword = password ? password : '0';
    console.log('action', action, actionPassword);

    const tick = await apiCall('tick');
    if (!tick.success) setConnected(false);
    setLatestTick(tick.data);

    if (action.includes('transfer') || action.startsWith('qx/order')) {
      if (action.includes('transfer'))
        result = await apiCall(`${action}${tick.data + 10}/${actionPassword}`);

      if (action.startsWith('qx/order')) {
        result = await apiCall(
          `${action.replace('<tick>', tick.data + 10)}/${actionPassword}`
        );
        const orders = await apiCall(`qx/orders/1/1000/0`);
        setQXTransfers(orders.data);
      }
      if (
        result.data === 'Invalid Password' ||
        result.data === 'Invalid Password!' ||
        result.data === 'Must Enter A Password!'
      ) {
        setShowProgress(false);
        setInvalidPassword('Invalid Password');
      } else {
        setOrderTick(tick.data + 10);
        setShowProgress(true);
        setInvalidPassword('');
      }
    } else result = await apiCall(`${action}${actionPassword}`);
    // download special case
    if (action === '/wallet/download/') {
      let csvContent = '';
      if (result.data.split(',').length < 2) {
        console.log('Invalid Password!');
        csvContent += 'Invalid Password!';
      } else {
        csvContent += result.data;
      }
      // Create a temporary link element and trigger download
      const link = document.createElement('a');
      link.href = 'data:text/csv;charset=utf-8,' + encodeURI(csvContent);
      link.download = 'rubic-db-decrypted.csv';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }

    setPassword('');
    setAction('');

    if (!unlocked) {
      const unlock = await apiCall(
        `wallet/unlock/${actionPassword}/${unlockTimer}`
      );
      console.log(unlock.data);
    }

    // console.log(result);
    // console.log(result.data);
    if (
      result.data === 'Invalid Password' ||
      result.data === 'Invalid Password!' ||
      result.data === 'Must Enter A Password!'
    ) {
      setInvalidPassword('Invalid Password');
    } else {
      setInvalidPassword('');
    }
  };

  const renderPassword = useCallback(
    () => (
      <Box
        sx={{
          display: 'flex',
          mt: 2,
        }}
      >
        <Typography variant='h7' sx={{ mt: 2 }}>
          PLEASE ENTER PASSWORD
        </Typography>
        <LockIcon sx={{ mr: 1, mt: 2, ml: 1, color: 'orange' }}></LockIcon>
        <TextField
          size='10'
          sx={{
            mr: 1, // margin-right: theme.spacing(2)
            pr: 1, // padding-right (only affects outer container if applicable)
            width: '150px', // optional: custom width
          }}
          // label='Amount'
          placeholder='password'
          variant='outlined'
          value={password}
          type={'password'}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
        <Button
          variant='contained'
          sx={{
            mt: 0, // margin-right: theme.spacing(2)
          }}
          color='primary'
          onClick={async () => await actionHandler(action, false)}
        >
          CONFIRM
        </Button>
        <Button
          variant='contained'
          sx={{
            mt: 0,
            ml: 2, // margin-right: theme.spacing(2)
          }}
          color='primary'
          onClick={async () => {
            setPassword('');
            setAction('');
          }}
        >
          CANCEL
        </Button>
      </Box>
    ),
    [password, action, QXtransfers]
  );

  const renderProgress = useCallback(
    () => (
      <>
        <Typography variant='h7' sx={{ mb: 3 }}>
          {!invalidPassword
            ? `TICK : ${latestTick
                ?.toString()
                .replace(/\B(?=(\d{3})+(?!\d))/g, ',')} `
            : `${invalidPassword}`}
          {showProgress ? '   TRANSFER IN PROGRESS' : ''}
        </Typography>
        {showProgress ? (
          <LinearProgress sx={{ mb: 2 }} />
        ) : (
          <LinearProgress
            variant='determinate'
            value={100} // Static value, fully filled
            sx={
              //txSuccess || orderTick === 0
              !invalidPassword
                ? {
                    mb: 2,
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: 'success.main', // Set the progress bar color to green
                    },
                  }
                : {
                    mb: 2,
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: 'error.main', // Set the progress bar color to green
                    },
                  }
            }
          ></LinearProgress>
        )}
      </>
    ),
    [showProgress, latestTick]
  );

  const renderQXMenu = useCallback(
    () => (
      <Box sx={{ display: 'flex', gap: 2, mb: 3, flexWrap: 'wrap' }}>
        <FormControl fullWidth>
          <InputLabel id='id-select-label'>ID</InputLabel>
          <Select
            labelId='id-select-label'
            id='id-select'
            value={id}
            label='Id'
            onChange={(event) => {
              setId(event.target.value);
            }}
          >
            {identities.map((item, index) => (
              <MenuItem key={index} value={item.id}>
                {item.id}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <FormControl fullWidth>
          <InputLabel id='asset-select-label'>ASSET</InputLabel>
          <Select
            labelId='asset-select-label'
            id='asset-select'
            value={selectedAsset}
            label='Asset'
            onChange={async (event) => {
              setSelectedAsset(event.target.value);
              const orderbookAsk = await apiCall(
                `qx/orderbook/${event.target.value}/ASK/1000/0`
              );
              const orderbookBid = await apiCall(
                `qx/orderbook/${event.target.value}/BID/1000/0`
              );
              setAskOrders(
                reverseSamePriceGroups(orderbookAsk.data.reverse()) || []
              );
              setBidOrders(orderbookBid.data || []);
            }}
          >
            {[...assetsNIssuer.keys()].sort().map((asset, index) => (
              <MenuItem key={index} value={asset}>
                {asset}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <TextField
          label={`Amount ${qxAmount
            .toString()
            .replace(/\B(?=(\d{3})+(?!\d))/g, "'")}`}
          value={qxAmount}
          onChange={handleInputChange(setQxAmount)}
          variant='outlined'
          size='small'
          error={!Number(qxAmount) || error}
          helperText={error}
          sx={{ width: 190 }}
        />
        <TextField
          label={`Price ${qxPrice
            .toString()
            .replace(/\B(?=(\d{3})+(?!\d))/g, ',')}`}
          value={qxPrice}
          onChange={handleInputChange(setQxPrice)}
          variant='outlined'
          size='small'
          error={!Number(qxPrice) || error}
          helperText={error}
          sx={{ width: 200 }}
        />
        <Button
          style={{ color: themeMode === 'dark' ? 'white' : 'black' }}
          variant='contained'
          disabled={true}
          color='secondary'
        >
          {`Total: ${(qxAmount * qxPrice)
            .toString()
            .replace(/\B(?=(\d{3})+(?!\d))/g, ',')}  qu`}
        </Button>
        <Button
          variant='contained'
          disabled={showProgress}
          startIcon={<ShoppingCartIcon />}
          onClick={() =>
            commitAction(
              `qx/order/<tick>/${assetsNIssuer.get(
                selectedAsset
              )}/${selectedAsset}/BID/${id}/${qxPrice}/${qxAmount}/`
            )
          }
        >
          Buy
        </Button>
        <Button
          variant='contained'
          disabled={showProgress}
          color='secondary'
          startIcon={<SellIcon />}
          onClick={() =>
            commitAction(
              `qx/order/<tick>/${assetsNIssuer.get(
                selectedAsset
              )}/${selectedAsset}/ASK/${id}/${qxPrice}/${qxAmount}/`
            )
          }
        >
          Sell
        </Button>
      </Box>
    ),
    [
      id,
      qxPrice,
      qxAmount,
      selectedAsset,
      error,
      assetsNIssuer,
      askOrders,
      bidOrders,
      identities,
    ]
  );

  const renderQX = useCallback(
    (orders, type) => (
      <TableContainer component={Paper} sx={{ mt: 2, mb: 3 }}>
        <Table size='small'>
          <TableHead>
            <TableRow>
              <TableCell>Action</TableCell>
              <TableCell align='right'>Price (qu)</TableCell>
              <TableCell align='right'>Amount</TableCell>
              <TableCell align='right'>Total (qu)</TableCell>
              <TableCell>Entity ID</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {orders.map((item, index) => (
              <TableRow key={item.entity + index}>
                <TableCell>
                  {id === item.entity && (
                    <IconButton
                      size='small'
                      disabled={showProgress}
                      onClick={() =>
                        commitAction(
                          `qx/order/<tick>/${assetsNIssuer.get(
                            selectedAsset
                          )}/${selectedAsset}/${
                            type === 'ASK' ? 'REMOVEASK' : 'REMOVEBID'
                          }/${id}/${item.price}/${item.num_shares}/`
                        )
                      }
                    >
                      <DeleteIcon fontSize='small' />
                    </IconButton>
                  )}
                </TableCell>
                <TableCell align='right'>
                  {item.price.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
                </TableCell>
                <TableCell align='right'>
                  {item.num_shares
                    .toString()
                    .replace(/\B(?=(\d{3})+(?!\d))/g, "'")}
                </TableCell>
                <TableCell align='right'>
                  {(item.num_shares * item.price)
                    .toString()
                    .replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
                </TableCell>
                <TableCell>
                  <Typography
                    variant='body2'
                    sx={{
                      color:
                        id === item.entity ? 'primary.main' : 'text.primary',
                      fontSize: '0.85rem',
                    }}
                  >
                    {item.entity}
                  </Typography>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    ),
    [id, assetsNIssuer, showProgress, selectedAsset, askOrders, bidOrders]
  );

  const renderAssetList = useCallback(() => {
    return (
      <>
        <>
          <Box sx={{ display: 'flex', gap: 2, mb: 3, flexWrap: 'wrap' }}>
            <Switch
              checked={displayTotalAssets}
              onChange={() => {
                const newId = displayTotalAssets ? identities[0].id : '';
                setId(newId);
                setDisplayTotalAssets(!displayTotalAssets);
                handleAssetUpdate(newId);
              }}
              slotProps={{ input: { 'aria-label': 'controlled' } }}
            />
            <Typography
              sx={{
                display: 'flex',
                alignItems: 'center',
                mr: 2,
                ml: 2,
              }}
            >
              {displayTotalAssets ? 'ALL ID' : 'SINGLE ID'}
            </Typography>
            {!displayTotalAssets ? (
              <FormControl fullWidth>
                <InputLabel id='id-select-label'>ID</InputLabel>
                <Select
                  labelId='id-select-label'
                  id='id-select'
                  value={id}
                  label='Id'
                  onChange={async (event) => {
                    handleAssetUpdate(event.target.value);
                  }}
                >
                  {identities.map((item, index) => (
                    <MenuItem key={index} value={item.id}>
                      {item.id}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            ) : (
              <></>
            )}
          </Box>
        </>
        {displayTotalAssets ? null : (
          <>
            <Typography
              sx={{ display: 'flex', alignItems: 'center', mb: 3, mt: 0 }}
            >
              SELECT ASSET
            </Typography>
          </>
        )}
        <Grid sx={{ ml: 2 }} container spacing={2}>
          {[...assetsNIssuer.keys()].sort().map((option, index) => (
            <>
              <Grid item xs={12} sm={6} md={3} rowSpacing={2} key={index}>
                <Chip
                  key={option}
                  label={option}
                  clickable
                  color={selectedAsset === option ? 'warning' : 'default'}
                  // variant={timeOption === option.minutes ? 'filled' : 'outlined'}
                  onClick={() => setSelectedAsset(option)}
                />
                <Paper
                  sx={{
                    width: 120,
                    display: 'flex',
                    alignItems: 'center',
                    paddingLeft: 1,
                    mt: 1,
                  }}
                  square={false}
                >
                  <Typography
                    sx={{
                      color: selectedAsset === option ? 'warning' : 'default',
                    }}
                  >
                    {assetsBalance.get(option) || 0}
                  </Typography>
                </Paper>
              </Grid>
            </>
          ))}
        </Grid>

        <Box>
          <Typography
            sx={{ display: 'flex', alignItems: 'center', mb: 3, mt: 5 }}
          >
            TRANSFER {selectedAsset}
          </Typography>
          <Autocomplete
            value={assetSource}
            onInputChange={(event, newInputValue) => {
              setAssetSource(newInputValue);
            }}
            onChange={(event, newValue) => {
              setAssetSource(newValue);
              // if (!seedRegex.test(newValue)) {
              //   setReceiverError('ID must be exactly 60 uppercase A-Z letters');
              // } else {
              //   setReceiverError('');
              //   setReceiverAddress(newValue);
              // }
            }}
            options={identities.map(
              (identity) =>
                identity.id +
                ` # ${
                  identities
                    .find((el) => el.id === identity.id)
                    .assets?.find((asset) => asset?.name === selectedAsset)
                    ?.balance || 0
                } ${selectedAsset}`
            )}
            freeSolo // Allows input of custom text
            renderInput={(params) => (
              <TextField
                size='10'
                {...params}
                // error={!!receiverError}
                // helperText={receiverError}
                sx={{
                  // mt: 1, // margin-right: theme.spacing(2)
                  pr: 1, // padding-right (only affects outer container if applicable)
                  width: '950px', // optional: custom width
                  margin: 1,
                  '& .MuiInputBase-input': {
                    height: '20px', // Adjust the height as needed
                  },
                }}
                // InputProps={{
                //   ...params.InputProps,
                //   endAdornment: (
                //     <>
                //       <InputAdornment position='end'>
                //         <IconButton>
                //           <PlaylistRemove />
                //         </IconButton>
                //       </InputAdornment>
                //       {params.InputProps.endAdornment}
                //     </>
                //   ),
                // }}
                label='source'
                variant='outlined'
                required
              />
            )}
          />
          <SendIcon sx={{ mb: 2, mt: 2, mr: 3, ml: 2 }}></SendIcon>
          <TextField
            size='small'
            sx={{
              mr: 1, // margin-right: theme.spacing(2)
              mt: 1,
              pr: 1, // padding-right (only affects outer container if applicable)
              width: '150px', // optional: custom width
            }}
            // label='Amount'
            placeholder='Amount'
            variant='outlined'
            value={assetAmount}
            onChange={handleInputChange(setAssetAmount)}
            required
          />
          <Button
            variant='contained'
            sx={{
              mr: 1,
              mb: 6,
            }}
            onClick={async () => {
              // console.log(assetSource);
              commitAction(
                `asset/transfer/${selectedAsset}/${
                  identities
                    .find((el) => el.id === assetSource.substring(0, 60))
                    .assets?.find((asset) => asset?.name === selectedAsset)
                    .issuer
                }/${assetSource.substring(0, 60)}/${assetDestination.substring(
                  0,
                  60
                )}/${assetAmount}/`
              );
            }}
          >
            SEND {assetAmount} ASSET(S)
          </Button>
          <Autocomplete
            sx={{
              mt: -4,
            }}
            value={assetDestination}
            onInputChange={(event, newInputValue) => {
              setAssetDestination(newInputValue);
              // if (!seedRegex.test(newInputValue)) {
              //   setReceiverError('ID must be exactly 60 uppercase A-Z letters');
              // } else {
              //   setReceiverError('');
              //   setReceiverAddress(newInputValue);
              // }
            }}
            onChange={(event, newValue) => {
              setAssetDestination(newValue);
              // if (!seedRegex.test(newValue)) {
              //   setReceiverError('ID must be exactly 60 uppercase A-Z letters');
              // } else {
              //   setReceiverError('');
              //   setReceiverAddress(newValue);
              // }
            }}
            options={identities.map(
              (identity) =>
                identity.id +
                ` # ${
                  identities
                    .find((el) => el.id === identity.id)
                    .assets?.find((asset) => asset?.name === selectedAsset)
                    ?.balance || 0
                } ${selectedAsset}`
            )}
            freeSolo // Allows input of custom text
            renderInput={(params) => (
              <TextField
                size='10'
                {...params}
                // error={!!receiverError}
                // helperText={receiverError}
                sx={{
                  // mt: 1, // margin-right: theme.spacing(2)
                  pr: 1, // padding-right (only affects outer container if applicable)
                  width: '950px', // optional: custom width
                  margin: 1,
                  '& .MuiInputBase-input': {
                    height: '20px', // Adjust the height as needed
                  },
                }}
                // InputProps={{
                //   ...params.InputProps,
                //   endAdornment: (
                //     <>
                //       <InputAdornment position='end'>
                //         <IconButton>
                //           <PlaylistRemove />
                //         </IconButton>
                //       </InputAdornment>
                //       {params.InputProps.endAdornment}
                //     </>
                //   ),
                // }}
                label='destination'
                variant='outlined'
                required
              />
            )}
          />
        </Box>
      </>
    );
  }, [
    id,
    assetsNIssuer,
    identities,
    selectedAsset,
    assetsBalance,
    assetAmount,
    assetSource,
    assetDestination,
    displayTotalAssets,
  ]);

  const renderQXTxList = useCallback(() => {
    return (
      <>
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Stack
            sx={{
              mb: 3,
            }}
            direction='row'
            spacing={1}
          >
            {timeOptions.map((option) => (
              <Chip
                key={option.label}
                label={option.label}
                clickable
                color={timeOption === option.minutes ? 'primary' : 'default'}
                variant={timeOption === option.minutes ? 'filled' : 'outlined'}
                onClick={() => setTimeOption(option.minutes)}
              />
            ))}
          </Stack>
        </Box>
        <Box>
          {QXtransfers.filter((transfer) => {
            const transferDate = new Date(transfer.created.replace(' ', 'T')); // Convert to ISO 8601 for better parsing
            const currentDate = new Date();
            const currentDate_utc = new Date(
              currentDate.toUTCString().slice(0, -4)
            );
            return (
              currentDate_utc.setMinutes(
                currentDate_utc.getMinutes() - timeOption
              ) < transferDate
            );
          })
            .reverse()
            .map((item, index) => (
              <Accordion
                expanded={openPanel === `panel${index + 1}`}
                onChange={handleChange(`panel${index + 1}`)}
              >
                <AccordionSummary
                  expandIcon={<ExpandMoreIcon />}
                  aria-controls={`panel${index + 1}-content`}
                  id={`panel${index + 1}-header`}
                >
                  <ListItem key={index}>
                    {item.input_type === '6' ? (
                      <ShoppingCartIcon sx={{ mr: 1 }}></ShoppingCartIcon>
                    ) : (
                      <SellIcon sx={{ mr: 1 }}></SellIcon> // input_type 5
                    )}
                    <Typography sx={{ mr: 1 }}>{item.num_shares}</Typography>
                    <Typography sx={{ mr: 1 }}>{item.name}</Typography>
                    {item.status === '0' ? (
                      <>
                        <CheckIcon
                          sx={{ color: 'lightgreen', mr: 1 }}
                        ></CheckIcon>
                      </>
                    ) : item.status === '-1' ? (
                      <>
                        <CheckIcon sx={{ color: 'yellow' }}></CheckIcon>
                      </>
                    ) : (
                      <>
                        <ReplayIcon
                          sx={{
                            color: 'orange',
                            mr: 1,
                            animation: `${flash} 1s infinite`,
                          }}
                          onClick={() =>
                            commitAction(
                              `transfer/${item.source}/${item.destination}/${item.amount}/`
                            )
                          }
                        ></ReplayIcon>
                        <CheckIcon sx={{ color: 'red' }}></CheckIcon>
                      </>
                    )}

                    <ListItemText
                      secondary={`${item.created} - ${
                        item.source === selectedTxId
                          ? item.destination
                          : item.source
                      }`}
                    />
                    {item.source === selectedTxId ? (
                      <ArrowUpwardIcon
                        sx={{
                          color: 'red',
                          alignContent: 'end',
                        }}
                      ></ArrowUpwardIcon>
                    ) : (
                      <ArrowDownwardIcon
                        sx={{
                          color: 'green',
                          alignContent: 'end',
                        }}
                      ></ArrowDownwardIcon>
                    )}
                    <ListItemText secondary={`${item.amount} QU`} />
                  </ListItem>
                </AccordionSummary>
                <AccordionDetails>
                  <Card sx={{ bgcolor: '#121928' }}>
                    <List>
                      <ListItem key={index}>
                        <ListItemText secondary={`Amount`} />
                        {item.amount}
                      </ListItem>
                      <ListItem key={index + 1}>
                        <ListItemText secondary={`Tx`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tx/${item.txid}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.txid}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 2}>
                        <ListItemText secondary={`Source`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.source}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.source}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 3}>
                        <ListItemText secondary={`Dest`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.destination}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.destination}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 4}>
                        <ListItemText secondary={`Tick`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tick/${item.tick}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.tick}
                        </Link>
                      </ListItem>
                    </List>
                  </Card>
                </AccordionDetails>
              </Accordion>
            ))}
        </Box>
      </>
    );
  }, [openPanel, QXtransfers, password, timeOption]);

  const renderAssetTxList = useCallback(() => {
    return (
      <>
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Stack
            sx={{
              mb: 3,
            }}
            direction='row'
            spacing={1}
          >
            {timeOptions.map((option) => (
              <Chip
                key={option.label}
                label={option.label}
                clickable
                color={timeOption === option.minutes ? 'primary' : 'default'}
                variant={timeOption === option.minutes ? 'filled' : 'outlined'}
                onClick={() => setTimeOption(option.minutes)}
              />
            ))}
          </Stack>
        </Box>
        <Box>
          {assetTransfers
            .filter((transfer) => {
              const transferDate = new Date(transfer.created.replace(' ', 'T')); // Convert to ISO 8601 for better parsing
              const currentDate = new Date();
              const currentDate_utc = new Date(
                currentDate.toUTCString().slice(0, -4)
              );
              return (
                currentDate_utc.setMinutes(
                  currentDate_utc.getMinutes() - timeOption
                ) < transferDate
              );
            })
            .map((item, index) => (
              <Accordion
                expanded={openPanel === `panel${index + 1}`}
                onChange={handleChange(`panel${index + 1}`)}
              >
                <AccordionSummary
                  expandIcon={<ExpandMoreIcon />}
                  aria-controls={`panel${index + 1}-content`}
                  id={`panel${index + 1}-header`}
                >
                  <ListItem key={index}>
                    <Typography sx={{ mr: 1 }}>{item.num_shares}</Typography>
                    <Typography sx={{ mr: 1 }}>{item.name}</Typography>
                    {item.status === '0' ? (
                      <>
                        <CheckIcon
                          sx={{ color: 'lightgreen', mr: 1 }}
                        ></CheckIcon>
                      </>
                    ) : item.status === '-1' ? (
                      <>
                        <CheckIcon sx={{ color: 'yellow' }}></CheckIcon>
                      </>
                    ) : (
                      <>
                        <ReplayIcon
                          sx={{
                            color: 'orange',
                            mr: 1,
                            animation: `${flash} 1s infinite`,
                          }}
                          onClick={() =>
                            commitAction(
                              `asset/transfer/${item.name}/${
                                identities
                                  .find((el) => el.id === item.source)
                                  .assets?.find(
                                    (asset) => asset?.name === item.name
                                  ).issuer
                              }/${item.source}/${
                                item.new_owner_and_possessor
                              }/${item.num_shares}/`
                            )
                          }
                        ></ReplayIcon>
                        <CheckIcon sx={{ color: 'red' }}></CheckIcon>
                      </>
                    )}

                    <ListItemText
                      secondary={`${item.created} - ${
                        item.source === selectedTxId
                          ? item.new_owner_and_possessor
                          : item.source
                      }`}
                    />
                    {item.source === selectedTxId ? (
                      <ArrowUpwardIcon
                        sx={{
                          color: 'red',
                        }}
                      ></ArrowUpwardIcon>
                    ) : (
                      <ArrowDownwardIcon
                        sx={{
                          color: 'green',
                        }}
                      ></ArrowDownwardIcon>
                    )}
                    <ListItemText secondary={`${item.num_shares}`} />
                  </ListItem>
                </AccordionSummary>
                <AccordionDetails>
                  <Card sx={{ bgcolor: '#121928' }}>
                    <List>
                      <ListItem key={index}>
                        <ListItemText secondary={`Fee`} />
                        {item.amount} QU
                      </ListItem>
                      <ListItem key={index}>
                        <ListItemText secondary={`Amount`} />
                        {item.num_shares}
                      </ListItem>
                      <ListItem key={index + 1}>
                        <ListItemText secondary={`Tx`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tx/${item.txid}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.txid}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 2}>
                        <ListItemText secondary={`Source`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.source}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.source}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 3}>
                        <ListItemText secondary={`Dest`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.new_owner_and_possessor}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.new_owner_and_possessor}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 4}>
                        <ListItemText secondary={`Tick`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tick/${item.tick}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.tick}
                        </Link>
                      </ListItem>
                    </List>
                  </Card>
                </AccordionDetails>
              </Accordion>
            ))}
        </Box>
      </>
    );
  }, [openPanel, transfers, password, timeOption]);

  const renderTxList = useCallback(() => {
    return (
      <>
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Stack
            sx={{
              mb: 3,
            }}
            direction='row'
            spacing={1}
          >
            {timeOptions.map((option) => (
              <Chip
                key={option.label}
                label={option.label}
                clickable
                color={timeOption === option.minutes ? 'primary' : 'default'}
                variant={timeOption === option.minutes ? 'filled' : 'outlined'}
                onClick={() => setTimeOption(option.minutes)}
              />
            ))}
          </Stack>
        </Box>
        <Box>
          {transfers
            .filter((transfer) => {
              const transferDate = new Date(transfer.created.replace(' ', 'T')); // Convert to ISO 8601 for better parsing
              const currentDate = new Date();
              const currentDate_utc = new Date(
                currentDate.toUTCString().slice(0, -4)
              );
              return (
                currentDate_utc.setMinutes(
                  currentDate_utc.getMinutes() - timeOption
                ) < transferDate
              );
            })
            .filter(
              (transfer) =>
                transfer.destination !==
                'BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID'
            )
            .map((item, index) => (
              <Accordion
                expanded={openPanel === `panel${index + 1}`}
                onChange={handleChange(`panel${index + 1}`)}
              >
                <AccordionSummary
                  expandIcon={<ExpandMoreIcon />}
                  aria-controls={`panel${index + 1}-content`}
                  id={`panel${index + 1}-header`}
                >
                  <ListItem key={index}>
                    {item.status === '0' ? (
                      <>
                        <CheckIcon
                          sx={{ color: 'lightgreen', mr: 1 }}
                        ></CheckIcon>
                      </>
                    ) : item.status === '-1' ? (
                      <>
                        <CheckIcon sx={{ color: 'yellow' }}></CheckIcon>
                      </>
                    ) : (
                      <>
                        <ReplayIcon
                          sx={{
                            color: 'orange',
                            mr: 1,
                            animation: `${flash} 1s infinite`,
                          }}
                          onClick={() =>
                            commitAction(
                              `transfer/${item.source}/${item.destination}/${item.amount}/`
                            )
                          }
                        ></ReplayIcon>
                        <CheckIcon sx={{ color: 'red' }}></CheckIcon>
                      </>
                    )}

                    <ListItemText
                      secondary={`${item.created} - ${
                        item.source === selectedTxId
                          ? item.destination
                          : item.source
                      }`}
                    />
                    {item.source === selectedTxId ? (
                      <ArrowUpwardIcon
                        sx={{
                          color: 'red',
                        }}
                      ></ArrowUpwardIcon>
                    ) : (
                      <ArrowDownwardIcon
                        sx={{
                          color: 'green',
                        }}
                      ></ArrowDownwardIcon>
                    )}
                    <ListItemText secondary={`${item.amount} QU`} />
                  </ListItem>
                </AccordionSummary>
                <AccordionDetails>
                  <Card sx={{ bgcolor: '#121928' }}>
                    <List>
                      <ListItem key={index}>
                        <ListItemText secondary={`Amount`} />
                        {item.amount}
                      </ListItem>
                      <ListItem key={index + 1}>
                        <ListItemText secondary={`Tx`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tx/${item.txid}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.txid}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 2}>
                        <ListItemText secondary={`Source`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.source}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.source}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 3}>
                        <ListItemText secondary={`Dest`} />
                        <Link
                          href={`https://explorer.qubic.org/network/address/${item.destination}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.destination}
                        </Link>
                      </ListItem>
                      <ListItem key={index + 4}>
                        <ListItemText secondary={`Tick`} />
                        <Link
                          href={`https://explorer.qubic.org/network/tick/${item.tick}`}
                          target='_blank'
                          rel='noopener'
                        >
                          {item.tick}
                        </Link>
                      </ListItem>
                    </List>
                  </Card>
                </AccordionDetails>
              </Accordion>
            ))}
        </Box>
      </>
    );
  }, [openPanel, transfers, password, timeOption]);

  const CsvDownloadButton = () => {
    return (
      <Button
        variant='contained'
        sx={{
          width: 400,
        }}
        color='primary'
        onClick={async () => {
          commitAction(`/wallet/download/`);
          // console.log(result);
          // console.log(result.data);
        }}
      >
        Download WALLET
      </Button>
    );
  };

  const renderSettings = useCallback(
    () => (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          mt: 2,
        }}
      >
        <Box
          sx={{
            display: 'flex',
          }}
        >
          {' '}
          <Typography sx={{ display: 'flex', alignItems: 'center', mr: 2 }}>
            UNLOCK TIMER [ms]
          </Typography>
          <TextField
            size='10'
            sx={{
              // mt: 1, // margin-right: theme.spacing(2)
              pr: 1, // padding-right (only affects outer container if applicable)
              width: '100px', // optional: custom width
            }}
            // label='Amount'
            placeholder='unlock timer [ms]'
            variant='outlined'
            value={unlockTimer}
            inputProps={{ maxLength: 5 }}
            onChange={handleInputChange(setUnlockTimer)}
            required
          />
          <CsvDownloadButton></CsvDownloadButton>
        </Box>

        <Divider sx={{ borderColor: 'grey.300', my: 2, mt: 4, mb: 4 }} />
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Typography sx={{ display: 'flex', alignItems: 'center', mr: 2 }}>
            TICK OFFSET [s]
          </Typography>

          <TextField
            size='10'
            sx={{
              // mt: 1, // margin-right: theme.spacing(2)
              pr: 1, // padding-right (only affects outer container if applicable)
              width: '100px', // optional: custom width
            }}
            // label='Amount'
            placeholder='tick offset'
            variant='outlined'
            value={tickOffset}
            onChange={handleInputChange(setTickOffset)}
            required
          />

          <Typography
            sx={{
              display: 'flex',
              alignItems: 'center',
              color: 'grey',
              mr: 2,
              ml: 2,
            }}
          >
            REPLAY TRANSACTION
          </Typography>

          <Checkbox
            checked={replay}
            onChange={() => setReplay(!replay)}
          ></Checkbox>
          <Typography
            sx={{
              display: 'flex',
              alignItems: 'center',
              mr: 2,
              ml: 2,
            }}
          >
            ALLOW NON-ENCRYTED ID
          </Typography>

          <Checkbox
            checked={allowNonEncrypted}
            onChange={() => setAllowNonEncrypted(!allowNonEncrypted)}
          ></Checkbox>
        </Box>
        <Divider sx={{ borderColor: 'grey.300', my: 2, mt: 4, mb: 4 }} />
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Typography sx={{ display: 'flex', alignItems: 'center', mr: 2 }}>
            MIN PEERS
          </Typography>

          <TextField
            size='10'
            sx={{
              // mt: 1, // margin-right: theme.spacing(2)
              pr: 1, // padding-right (only affects outer container if applicable)
              width: '100px', // optional: custom width
            }}
            // label='Amount'
            placeholder='tick offset'
            variant='outlined'
            value={minPeers}
            onChange={handleInputChange(setMinPeers)}
            required
          />
          <Typography
            sx={{ display: 'flex', alignItems: 'center', mr: 2, ml: 2 }}
          >
            MAX PEERS
          </Typography>

          <TextField
            size='10'
            sx={{
              // mt: 1, // margin-right: theme.spacing(2)
              pr: 1, // padding-right (only affects outer container if applicable)
              width: '100px', // optional: custom width
            }}
            // label='Amount'
            placeholder='tick offset'
            variant='outlined'
            value={maxPeers}
            onChange={handleInputChange(setMaxPeers)}
            required
          />
        </Box>
      </Box>
    ),
    [
      password,
      tickOffset,
      isEncrypted,
      encodedURI,
      maxPeers,
      minPeers,
      replay,
      allowNonEncrypted,
      unlockTimer,
    ]
  );

  const renderIdentitiyEntry = useCallback(
    () => (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          columns: 2,
          gap: 2,
          // maxWidth: 800,
        }}
      >
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <TextField
            label='ENTER SEED'
            type={showSeed ? 'text' : 'password'}
            value={seed}
            error={!!seedError}
            helperText={seedError}
            onChange={(e) => {
              if (!seedRegex.test(e.target.value)) {
                setSeedError('seed must be exactly 55 lowercase a-z letters');
              } else {
                setSeedError('');
              }
              setSeed(e.target.value);
            }}
            sx={{ width: 350 }}
            variant='outlined'
            // fullWidth
            InputProps={{
              endAdornment: (
                <InputAdornment position='end'>
                  <IconButton onClick={() => setShowSeed(!showSeed)} edge='end'>
                    {showSeed ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
          <Button
            variant='contained'
            disabled={seedError}
            onClick={async () => {
              const result = await apiCall(`/identity/from_seed/${seed}`);
              if (
                result ===
                'AARQXIKNFIEZZEMOAVNVSUINZXAAXYBZZXVSWYOYIETZVPVKJPARMKTEKLKJ'
              ) {
                //invalid seed
                setId('invalid seed');
              }
              // console.log(result.data);
              setId(result.data);
            }}
            sx={{ ml: 2, mt: 1, width: 150, height: 'fit-content' }}
          >
            PREVIEW ID
          </Button>
        </Box>

        <Typography
          variant='h6'
          sx={{ display: 'flex', alignItems: 'center', gap: 1 }}
        >
          {id}
        </Typography>
        <Box
          sx={{
            display: 'flex',
          }}
        >
          {allowNonEncrypted ? (
            <Button
              startIcon={<FileUploadIcon />}
              disabled={seed.length !== 55}
              onClick={async () => {
                const result = await apiCall(`identity/add/${seed}/${''}`);
                // console.log(result);
                // console.log(result.data);
              }}
              variant='contained'
              color='primary'
              sx={{ mr: 1, width: 350 }}
            >
              IMPORT ID
            </Button>
          ) : (
            <></>
          )}
          <Button
            sx={{ width: 350 }}
            startIcon={<FileUploadIcon />}
            endIcon={<LockIcon />}
            disabled={seed.length !== 55}
            onClick={async () => {
              commitAction(`identity/add/${seed}/`);
              // const result = await apiCall(`identity/add/${seed}/${password}`);
              // console.log(result);
              // console.log(result.data);
            }}
            variant='contained'
            color='primary'
          >
            IMPORT ENCRYPTED ID
          </Button>
        </Box>
        <Box
          sx={{
            display: 'flex',
            width: 900,
          }}
        >
          {allowNonEncrypted ? (
            <Button
              startIcon={<CasinoIcon />}
              onClick={async () => {
                // commitAction('identity/new/');
                const result = await apiCall(`identity/new/${'0'}`);
                // console.log(result);
                // console.log(result.data);
              }}
              variant='contained'
              color='primary'
              sx={{ mr: 1, width: 350 }}
            >
              GENERATE RANDOM ID
            </Button>
          ) : (
            <></>
          )}
          <Button
            sx={{ width: 350 }}
            startIcon={<CasinoIcon />}
            endIcon={<LockIcon />}
            onClick={() => {
              commitAction('identity/new/');
              // const result = await apiCall(`identity/new/${password}`);
              // console.log(result);
              // console.log(result.data);
            }}
            variant='contained'
            color='primary'
          >
            GENERATE ENCRYPTED RANDOM ID
          </Button>
        </Box>
      </Box>
    ),
    [seed, showSeed, seedError, id, allowNonEncrypted]
  );

  const LockScreenButton = () => {
    const [open, setOpen] = useState(false);

    const toggle = async () => {
      if (password === retypePassword && password.length > 4) {
        setOpen((prev) => !prev);
        const result1 = await apiCall(
          `/wallet/set_master_password/${password}`
        );
        // console.log(result1.data);

        const result = await apiCall(`wallet/encrypt/${password}`);
        // console.log(result);
        // console.log(result.data);
        setIsEncrypted(true);
        setPassword('');
      } else {
        console.log('pw mismatch');
      }
    };

    return (
      <IconButton disabled={password.length < 5} onClick={toggle}>
        <Fade in={!open} timeout={300} unmountOnExit>
          <LockOpenIcon sx={{ fontSize: 200, color: 'orange' }} />
        </Fade>
        <Fade in={open} timeout={300} unmountOnExit>
          <LockIcon sx={{ fontSize: 200, color: 'orange' }} />
        </Fade>
      </IconButton>
    );
  };

  const renderLockScreen = useCallback(
    () => (
      <Box
        sx={{
          height: '80vh',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: 2, // space between icon and textfield
          }}
        >
          {/* <LockOpenIcon sx={{ fontSize: 400, color: 'orange' }} /> */}

          <WalletIcon sx={{ fontSize: 120 }}></WalletIcon>
          <TextField
            type='password'
            label='New password'
            variant='outlined'
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <LockScreenButton></LockScreenButton>
          <TextField
            type='password'
            label='Retype password'
            variant='outlined'
            value={retypePassword}
            helperText={
              retypePassword !== password
                ? 'Passwords do not match'
                : password.length < 5
                ? 'Password too short'
                : ''
            }
            onChange={(e) => setRetypePassword(e.target.value)}
          />
          {/* <Button
            disabled={password.length < 8 || retypePassword !== password}
            onClick={async () => {
              const result1 = await apiCall(
                `/wallet/set_master_password/${password}`
              );
              console.log(result1.data);

              const result = await apiCall(`wallet/encrypt/${password}`);
              console.log(result);
              console.log(result.data);
              setIsEncrypted(true);
            }}
            variant='contained'
            color='primary'
          >
            SET PASSWORD
          </Button> */}
        </Box>
      </Box>
    ),
    [password, retypePassword]
  );

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ p: 3, maxWidth: 1400, margin: '0 auto' }}>
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            mb: 3,
          }}
        >
          <Typography
            sx={{ display: 'flex', fontSize: 36, alignItems: 'center', gap: 1 }}
          >
            <CircleIcon
              sx={{ color: connected ? 'success.main' : 'error.main' }}
            />
            RUBIC{' '}
            <WalletIcon
              sx={{
                fontSize: 36,
              }}
            ></WalletIcon>
            {connected
              ? `${totalBalance
                  .toString()
                  .replace(/\B(?=(\d{3})+(?!\d))/g, ',')} QU`
              : 'comm error'}
            <br />
            <Typography
              sx={{
                fontStyle: 'italic',
                fontSize: 36,
                color: 'grey',
              }}
            >
              {'  /  '} {' $ '}
              {(price * totalBalance)
                .toFixed(2)
                .toString()
                .replace(/\B(?=(\d{3})+(?!\d))/g, ',')}{' '}
            </Typography>
          </Typography>

          <Box sx={{ justifyContent: 'flex-end' }}>
            <IconButton
              onClick={() =>
                setThemeMode((prev) => (prev === 'light' ? 'dark' : 'light'))
              }
            >
              {themeMode === 'light' ? <DarkModeIcon /> : <LightModeIcon />}
            </IconButton>
          </Box>
        </Box>

        {isEncrypted ? (
          <TabContext value={tab}>
            {action ? renderPassword() : <></>}
            <Box
              sx={{
                borderBottom: 1,
                borderColor: 'divider',
                marginBottom: 3,
                // pointerEvents: 'none',
                // opacity: 0.5,
              }}
            >
              <TabList
                onChange={handleTabChange}
                aria-label='lab API tabs example'
              >
                <Tab label='WALLET' value='1' />
                <Tab label='TX' value='2' />
                <Tab label='ASSETS' value='3' />
                <Tab label='ASSET TX' value='4' />
                <Tab label='QX' value='5' />
                <Tab label='QX TX' value='6' />
                <Tab label='PEERS' value='7' />
                <Tab label='SETTINGS' value='8' />
              </TabList>
            </Box>
            {renderProgress()}
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='1'
            >
              {renderIdentitiyEntry()}
              <Divider sx={{ borderColor: 'grey.300', my: 2, mt: 6 }} />
              {renderIdentitiyTable()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='2'
            >
              {renderTxList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='3'
            >
              {renderAssetList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='4'
            >
              {renderAssetTxList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='5'
            >
              {renderQXMenu()}

              <Typography variant='h6' sx={{ mb: 1, color: 'error.main' }}>
                ASK ORDERS
              </Typography>
              {renderQX(askOrders, 'Ask')}
              <Typography variant='h6' sx={{ mb: 1, color: 'success.main' }}>
                BID ORDERS
              </Typography>
              {renderQX(bidOrders, 'Bid')}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='6'
            >
              {renderQXTxList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='7'
            >
              {renderAddPeer()} {renderPeersTable()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='8'
            >
              {' '}
              {renderSettings()}
            </TabPanel>
          </TabContext>
        ) : (
          <>{renderLockScreen()}</>
        )}
      </Box>
    </ThemeProvider>
  );
};

export default MainView;
