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

const ISSUER = new Map([
  ['QX', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QTRY', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['RANDOM', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QUTIL', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QPOOL', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['MLM', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QVAULT', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QEARN', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QFT', 'TFUYVBXYIYBVTEMJHAJGEJOOZHJBQFVQLTBBKMEHPEVIZFXZRPEYFUWGTIWG'],
  ['CFB', 'CFBMEMZOIDEXQAUXYYSZIURADQLAPWPMNJXQSNVQZAHYVOPYUKKJBJUCTVJL'],
  ['QWALLET', 'QWALLETSGQVAGBHUCVVXWZXMBKQBPQQSHRYKZGEJWFVNUFCEDDPRMKTAUVHA'],
  ['QCAP', 'QCAPWMYRSHLBJHSTTZQVCIBARVOASKDENASAKNOBRGPFWWKRCUVUAXYEZVOG'],
  ['VSTB001', 'VALISTURNWYFQAMVLAKJVOKJQKKBXZZFEASEYCAGNCFMZARJEMMFSESEFOWM'],
  ['MSVAULT', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['QBAY', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB'],
  ['MATILDA', 'ZWQHZOEAWYKSGDPWWAQBAOKECCSASFCMLYZOJGBXNABXVZJZXKXOWRTFIQHC'],
  ['QXTRADE', 'QXTRMABNAJWNQBKYYNUNVYAJAQMDLIKOFXNGTRVYRDQMNZNNMZDGBDNGYMRM'],
  ['QXMR', 'QXMRTKAIIGLUREPIQPCMHCKWSIPDTUYFCFNYXQLTECSUJVYEMMDELBMDOEYB'],
  ['CODED', 'CODEDBUUDDYHECBVSUONSSWTOJRCLZSWHFHZIUWVFGNWVCKIWJCSDSWGQAAI'],
]);
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
  const [password, setPassword] = useState('');
  const [retypePassword, setRetypePassword] = useState('');
  const [encodedURI, setEncodedURI] = useState('');
  const [price, setPrice] = useState(0);

  const [isEncrypted, setIsEncrypted] = useState(false);
  const [action, setAction] = useState('');
  const [tickOffset, setTickOffset] = useState(0);
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
  const [assets, setAssets] = React.useState([]);
  const [assetsBalance, setAssetsBalance] = React.useState(new Map());

  const [assetSource, setAssetSource] = useState('');
  const [assetDestination, setAssetDestination] = useState('');

  const [selectedAsset, setSelectedAsset] = React.useState(null);

  const [invalidPassword, setInvalidPassword] = useState('');

  const [tab, setTab] = React.useState('1');

  const [themeMode, setThemeMode] = useState('dark');
  const theme = useMemo(() => getTheme(themeMode), [themeMode]);

  const [selectedId, setSelectedId] = React.useState(null);
  const [selectedPeer, setSelectedPeer] = React.useState(null);
  const [selectedTxId, setSelectedTxId] = React.useState(null);
  const [timeOption, setTimeOption] = useState(60 * 24 * 7 * 31 * 12 * 10);

  const tabLabels = useMemo(() => [...ISSUER.keys()], []);

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

  const handleSelect = (id) => {
    setSelectedId((prevSelected) => (prevSelected === id ? null : id));
  };

  const handleDelete = async (id) => {
    setAction(`identity/delete/${id}/`);
  };

  const handleSelectPeer = (id, ip, port) => {
    setSelectedPeer((prevSelected) => (prevSelected === id ? null : id));
    setIp(ip);
    setPort(port);
  };

  const handleTxShow = (id) => {
    setSelectedTxId((prevSelected) => (prevSelected === id ? null : id));
  };

  const handleTabChange = async (event, newValue) => {
    // if entering peers tab, update to current settings for min/max peers
    if (newValue !== '6') {
      // not settings tab
      const setMinPeersResult = await apiCall(`/peers/limit/min/${minPeers}`);
      const setMaxPeersResult = await apiCall(`/peers/limit/max/${maxPeers}`);
      console.log(setMinPeersResult, setMaxPeersResult);
    }
    setTab(newValue);
  };

  const [openPanel, setOpenPanel] = useState(null);

  const handleChange = (panel) => (event, isExpanded) => {
    setOpenPanel(isExpanded ? panel : null);
  };

  const getDetailedQubicData = async () => {
    const apiUrl = `https://rpc.qubic.org/v1/latest-stats`;

    try {
      const response = await fetch(apiUrl);

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      setPrice(data.data.price);
    } catch (error) {
      console.error('Error fetching detailed Qubic data:', error);
      return null;
    }
  };

  const putBid = async () => {
    const result = await apiCall('qx/orders/0/0/0');
    // setLatestTick(tick.data);
    ///qx/order/<tick>/<issuer>/<asset>/<ask_bid>/<address>/<price>/<amount>/<password>
    // const result = await apiCall(
    //   `qx/order/${tick.data}/CFBMEMZOIDEXQAUXYYSZIURADQLAPWPMNJXQSNVQZAHYVOPYUKKJBJUCTVJL/CFB/REMOVEASK/XYZYOLNZNIGGKAKTHBYTFTKAQMBBKDSGYPWZNJVMFHAJYTEDBZWNMODHRIJD/10/3/12345`
    // );
    console.log('ooo', result.data);
  };

  useEffect(() => {
    // declare the async data fetching function
    const checkWalletEncryption = async () => {
      // get the data from the api
      const result = await apiCall('wallet/is_encrypted');
      console.log('apiCall(wallet/is_encrypted', result);
      setIsEncrypted(typeof result.data === 'boolean');
      const assets = await apiCall('asset/issued');
      setAssets(assets.data);
      setSelectedAsset(assets.data[0]);

      const limits = await apiCall('peers/limit');
      setMaxPeers(limits.data.max);
      setMinPeers(limits.data.min);

      // putBid();
    };

    const getQubicData = async () => {
      // get the data from the api
      await getDetailedQubicData();
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
        obj.assets.forEach((asset) => {
          const balance = parseFloat(asset.balance);
          if (!isNaN(balance)) {
            acc[asset.name] = (acc[asset.name] || 0) + balance;
          }
        });
        return acc;
      }, {});

      setAssetsBalance(new Map(Object.entries(totalByAsset)));

      const transfer = await apiCall('transfer/0/0/0');
      setTransfers(transfer.data);
      console.log(transfer.data);
      // putBid();

      setTotalBalance(localTotalBalance);

      setIdentities(fullMergeIds);

      const orderbook = await apiCall('qx/orderbook/CFB/ASK/1000/0');
      console.log('orderbook', orderbook);
      orderbook.data.reverse();

      setAskOrders(orderbook.data || []);
      // setBidOrders(bidData || []);

      console.log(fullMergeIds);
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
  }, []);

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
            console.log(`peers/add/${ip}:${port}`);
            const result = await apiCall(`peers/add/${ip}:${port}`);
            console.log(result);
            console.log(result.data);
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
            console.log(`peers/delete/${selectedPeer}`);
            const result = await apiCall(`peers/delete/${selectedPeer}`);
            console.log(result);
            console.log(result.data);
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
          disabled={!selectedId}
          sx={{
            mt: 1,
            mb: 1,
          }}
          onClick={async () => {
            setAction(`transfer/${selectedId}/${destinationId}/${amount}/`);
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
    [selectedId, destinationId, latestTick, amount]
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
              <TableCell>Transactions</TableCell>
              <TableCell align='right'>Balance</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {identities.map((item, index) => (
              <TableRow key={item.id}>
                <TableCell>
                  {item.encrypted === 'true' ? (
                    <LockIcon style={{ color: 'orange' }}></LockIcon>
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
                      color: selectedId === item.id ? 'yellow' : 'white',
                    }}
                    onClick={() => handleSelect(item.id)}
                  />
                </TableCell>
                <TableCell>
                  {selectedId === item.id ? (
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
    [identities, selectedId, amount, destinationId, selectedTxId]
  );

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
          onClick={async () => {
            let result = '';
            console.log('action', action);

            if (
              action.startsWith('transfer') ||
              action.startsWith('asset/transfer')
            ) {
              const tick = await apiCall('tick');
              if (!tick.success) setConnected(false);
              setLatestTick(tick.data);
              if (action.startsWith('asset/transfer'))
                result = await apiCall(
                  `${action}${tick.data + 10}/${password}`
                );
              else
                result = await apiCall(
                  `${action}${tick.data + 10}/${password}`
                );
              // /asset/transfer/<asset_name>/<issuer>/<source_id>/<dest_id>/<num_tokens>/<tick>/<password>
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
            } else result = await apiCall(`${action}${password}`);
            // download special case
            if (action === '/wallet/download') {
              console.log(result.data);

              let csvContent = '';
              if (result.data.split(',').length < 2) {
                console.log('Invalid Password!');
                csvContent += 'Invalid Password!';
              } else {
                csvContent += result.data;
              }
              // Create a temporary link element and trigger download
              const link = document.createElement('a');
              link.href =
                'data:text/csv;charset=utf-8,' + encodeURI(csvContent);
              link.download = 'rubic-db-decrypted.csv';
              document.body.appendChild(link);
              link.click();
              document.body.removeChild(link);
            }

            setPassword('');
            setAction('');
            console.log(result);
            console.log(result.data);
            if (
              result.data === 'Invalid Password' ||
              result.data === 'Invalid Password!' ||
              result.data === 'Must Enter A Password!'
            ) {
              setInvalidPassword('Invalid Password');
            } else {
              setInvalidPassword('');
            }
          }}
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
    [password, action]
  );

  const renderProgress = useCallback(
    () => (
      <>
        <Typography variant='h7' sx={{ mb: 3 }}>
          {!invalidPassword
            ? `Latest Tick: ${latestTick
                ?.toString()
                .replace(/\B(?=(\d{3})+(?!\d))/g, ',')} `
            : `${invalidPassword}`}
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
                      onClick={
                        () => console.log('oh')
                        // qOrder(
                        //   tabLabels['CFB'],
                        //   type === 'Ask' ? 'removeSell' : 'removeBuy',
                        //   item.price,
                        //   item.numberOfShares
                        // )
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
    [id, tabLabels, showProgress]
  );

  const renderAssetList = useCallback(() => {
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
              mt: 1,
            }}
            direction='row'
            spacing={1}
          >
            {assets.map((option) => (
              <>
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
              </>
            ))}
          </Stack>
        </Box>
        <Box>
          <Typography
            sx={{ display: 'flex', alignItems: 'center', mb: 3, mt: 3 }}
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
                    .assets.find((asset) => asset?.name === selectedAsset)
                    ?.balance
                }`
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
                  width: '750px', // optional: custom width
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
              mb: 5,
            }}
            onClick={async () => {
              console.log(assetSource);
              console.log(
                '#hhhhoha',
                identities
                  .find((el) => el.id === assetSource.substring(0, 60))
                  .assets.find((asset) => asset?.name === selectedAsset).issuer
              );
              setAction(
                `asset/transfer/${selectedAsset}/${
                  identities
                    .find((el) => el.id === assetSource.substring(0, 60))
                    .assets.find((asset) => asset?.name === selectedAsset)
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
                    .assets.find((asset) => asset?.name === selectedAsset)
                    ?.balance
                }`
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
                  width: '750px', // optional: custom width
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
    assets,
    identities,
    selectedAsset,
    assetsBalance,
    assetAmount,
    assetSource,
    assetDestination,
  ]);

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
                            setAction(
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
          mt: 1, // margin-right: theme.spacing(2)
        }}
        color='primary'
        onClick={async () => {
          setAction(`/wallet/download/`);
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
        ></Box>

        <CsvDownloadButton></CsvDownloadButton>
        <Divider sx={{ borderColor: 'grey.300', my: 2, mt: 6 }} />
        <Box
          sx={{
            display: 'flex',
          }}
        >
          <Typography sx={{ display: 'flex', alignItems: 'center', mr: 2 }}>
            TICK OFFSET
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
        <Divider sx={{ borderColor: 'grey.300', my: 2, mt: 6 }} />
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
            label='Enter seed for new ID'
            type={showSeed ? 'text' : 'password'}
            value={seed}
            error={!!seedError}
            helperText={seedError}
            onChange={(e) => {
              console.log(e.target.value);
              if (!seedRegex.test(e.target.value)) {
                console.log('oh');
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
              console.log(result.data);
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
                console.log(result);
                console.log(result.data);
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
              setAction(`identity/add/${seed}/`);
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
                // setAction('identity/new/');
                const result = await apiCall(`identity/new/${'0'}`);
                console.log(result);
                console.log(result.data);
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
              setAction('identity/new/');
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
        console.log(result1.data);

        const result = await apiCall(`wallet/encrypt/${password}`);
        console.log(result);
        console.log(result.data);
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
                // pointerEvents: 'none',
                // opacity: 0.5,
              }}
            >
              <TabList
                onChange={handleTabChange}
                aria-label='lab API tabs example'
              >
                <Tab label='Wallet' value='1' />
                <Tab label='Assets' value='2' />
                <Tab label='QX' value='3' />
                <Tab label='Transactions' value='4' />
                <Tab label='Peers' value='5' />
                <Tab label='Settings' value='6' />
              </TabList>
            </Box>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='1'
            >
              {renderProgress()}
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
              {renderAssetList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='3'
            >
              {renderQX(askOrders, 'Ask')}
              {renderQX(bidOrders, 'Bid')}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='4'
            >
              {renderTxList()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='5'
            >
              {renderAddPeer()} {renderPeersTable()}
            </TabPanel>
            <TabPanel
              sx={{
                pointerEvents: action ? 'none' : 'all',
                opacity: action ? 0.3 : 1,
              }}
              value='6'
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
