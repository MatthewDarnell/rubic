use std::str::FromStr;
use identity::Identity;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_subseed, get_public_key_from_identity, sign_raw, get_identity};
use crate::AsBytes;
use crate::transfer::TransferTransaction;
/*
    Helper Functions
*/
fn read_le_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_le_bytes(int_bytes.try_into().unwrap())
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}

fn read_le_u16(input: &mut &[u8]) -> u16 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u16>());
    *input = rest;
    u16::from_le_bytes(int_bytes.try_into().unwrap())
}

/*
    End Helper Functions
*/

const QX_ADDRESS: &str = "BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID";
pub const QX_TRANSFER_SHARE: u16 = 2;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TransferAssetOwnershipAndPossessionInput {
    pub issuer: [u8; 32],
    pub new_owner_and_possessor: [u8; 32],
    pub asset_name: u64,
    pub number_of_shares: i64
}

impl TransferAssetOwnershipAndPossessionInput {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.issuer.as_slice() {
            bytes.push(*k);
        }
        for k in self.new_owner_and_possessor.as_slice() {
            bytes.push(*k);
        }
        for c in self.asset_name.to_le_bytes() {
            bytes.push(c);
        }

        for c in self.number_of_shares.to_le_bytes() {
            bytes.push(c);
        }

        bytes
    }
}


#[derive(Debug, Clone)]
#[repr(C)]
pub struct AssetTransferTransaction {
    pub tx: TransferTransaction,
    pub asset_tx: TransferAssetOwnershipAndPossessionInput,
    pub _signature: Vec<u8>
}

impl AssetTransferTransaction {
    pub fn from_signed_data(
        tx: TransferTransaction,
        issuer: &str,
        new_owner_and_possessor: &str,
        asset_name: &str,
        number_of_shares: i64,
        sig: &[u8]) -> Self
    {
        let mut name: [u8; 8] = [0; 8];
        name[0..asset_name.len()].copy_from_slice(asset_name.as_bytes());
        
        let asset_tx = TransferAssetOwnershipAndPossessionInput {
            issuer: get_public_key_from_identity(&issuer.to_string()).unwrap(),
            new_owner_and_possessor: get_public_key_from_identity(&new_owner_and_possessor.to_string()).unwrap(),
            asset_name: u64::from_le_bytes(name),
            number_of_shares
        };
        
        AssetTransferTransaction {
            tx,
            asset_tx,
            _signature: sig.to_vec()
        }
    }
    
    pub fn from_vars(source_identity: &Identity, asset_name: &str, issuer: &str, dest: &str, amount: i64, tick: u32) -> Self {
        if source_identity.encrypted {
            panic!("Trying to Transfer From Encrypted Wallet!");
        }
        if source_identity.seed.len() != 55 {
            panic!("Trying To Transfer From Corrupted Identity!");
        }
        let pub_key_src = match get_public_key_from_identity(&source_identity.identity) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };
        let mut tx: TransferTransaction = TransferTransaction::from_vars(
          source_identity,
          QX_ADDRESS,
          1000000u64,
          tick
        );

        tx._input_type = QX_TRANSFER_SHARE;
        tx._input_size = size_of::<TransferAssetOwnershipAndPossessionInput>() as u16;

        let mut name: [u8; 8] = [0; 8];
        name[0..asset_name.len()].copy_from_slice(asset_name.as_bytes());
        // fill the input
        let input = TransferAssetOwnershipAndPossessionInput {
            issuer: get_public_key_from_identity(&issuer.to_string()).unwrap(),
            new_owner_and_possessor: get_public_key_from_identity(&dest.to_string()).unwrap(),
            asset_name: u64::from_le_bytes(name),
            number_of_shares: amount
        };

        let tx_bytes = tx.as_bytes_without_signature();
        let mut input_bytes = input.as_bytes();
        let mut pre_image: Vec<u8> = Vec::new();
        pre_image.resize(tx_bytes.len(), 0);
        pre_image.copy_from_slice(&tx_bytes);
        pre_image.append(&mut input_bytes);
        
        let hash = k12_bytes(&pre_image);

        let sub_seed: Vec<u8> = get_subseed(source_identity.seed.as_str()).expect("Failed To Get SubSeed!");
        #[allow(unused_assignments)]
        let mut sig: [u8; 64] = [0; 64];
        sig = sign_raw(&sub_seed, &pub_key_src, hash.as_slice().try_into().unwrap());
        AssetTransferTransaction {
            tx,
            asset_tx: input,
            _signature: sig.to_vec()
        }
    }

    pub fn digest(&self) -> Vec<u8> {
        k12_bytes(&self.as_bytes())
    }


    pub fn as_bytes_without_signature(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.tx.as_bytes_without_signature().as_slice() {
            bytes.push(*k);
        }
        for k in self.asset_tx.as_bytes().as_slice() {
            bytes.push(*k);
        }
        bytes
    }

    pub fn txid(&self) -> String {
        let digest: [u8; 32] = k12_bytes(&self.as_bytes()).try_into().unwrap();
        get_identity(&digest).to_lowercase()
    }

}


impl AsBytes for AssetTransferTransaction {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.tx.as_bytes_without_signature().as_slice() {
            bytes.push(*k);
        }
        for k in self.asset_tx.as_bytes().as_slice() {
            bytes.push(*k);
        }
        for k in self._signature.as_slice() {
            bytes.push(*k);
        }
        bytes
    }
}
