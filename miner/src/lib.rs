//! Rubic as an entropy provider for the RANDOM smart contract.
//!
//! One round: commit the K12 digest of a 512-byte secret at tick T, then at
//! T + 3 (the next tick of the same stream) reveal the secret with an empty
//! commit. The contract mixes the reveal into its entropy and refunds the
//! collateral locked with the commit plus the amount attached to the reveal.

pub mod random;
