export const TIMEOUT_MS = 10000;
export const TICK_OFFSET = 30;
export const MAX_AMOUNT = 1000000000000000;
// Where the local Rubic server listens. Override for development/testing with a
// VITE_RUBIC_API entry in ui/.env.local (e.g. to point at a mock on another port).
export const serverIp = import.meta.env.VITE_RUBIC_API || 'http://127.0.0.1:3000';
// let globalLatestTick = 0;
// let expirationPendingTick = -1;
// let transactionPending = false;
export const passwordNotSetDefaultMessage =
  'You Can Set A Master Password In Settings.';
