use protocol::AsBytes;
use protocol::transfer::TransferTransaction;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_identity, get_public_key_from_identity, get_subseed, sign_raw};
use protocol::identity::Identity;
pub use crate::qx::{QxProcedure, QX_ADDRESS};




//IMPL
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct QxOrderActionInput {
    pub issuer: [u8; 32],
    pub asset_name: u64,
    pub price: i64,
    pub number_of_shares: i64
}

impl QxOrderActionInput {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.issuer.as_slice() {
            bytes.push(*k);
        }
        for c in self.asset_name.to_le_bytes() {
            bytes.push(c);
        }
        for c in self.price.to_le_bytes() {
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
pub struct QxOrderTransaction {
    pub tx: TransferTransaction,
    pub order_tx: QxOrderActionInput,
    pub _signature: Vec<u8>
}

impl QxOrderTransaction {
    pub fn from_signed_data(
        tx: TransferTransaction,
        issuer: &str,
        asset_name: &str,
        price: u64,
        number_of_shares: u64,
        sig: &[u8]) -> Self
    {
        let mut name: [u8; 8] = [0; 8];
        name[0..asset_name.len()].copy_from_slice(asset_name.as_bytes());

        let order_tx = QxOrderActionInput {
            issuer: get_public_key_from_identity(&issuer.to_string()).unwrap(),
            asset_name: u64::from_le_bytes(name),
            price: price as i64,
            number_of_shares: number_of_shares as i64
        };

        QxOrderTransaction {
            tx,
            order_tx,
            _signature: sig.to_vec()
        }
    }

    pub fn from_vars(procedure: QxProcedure, source_identity: &Identity, asset_name: &str, issuer: &str, price: u64, amount: u64, tick: u32) -> Self {
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
        let tx_amount: u64 = match procedure {
            QxProcedure::QxAddBidOrder  => price * amount,
            _ => 1u64
        };
        let mut tx: TransferTransaction = TransferTransaction::from_vars(
            source_identity,
            QX_ADDRESS,
            tx_amount,
            tick
        );
        tx._input_type = procedure as u16;
        tx._input_size = size_of::<QxOrderActionInput>() as u16;

        let mut name: [u8; 8] = [0; 8];
        name[0..asset_name.len()].copy_from_slice(asset_name.as_bytes());
        // fill the input
        let input = QxOrderActionInput {
            issuer: get_public_key_from_identity(&issuer.to_string()).unwrap(),
            asset_name: u64::from_le_bytes(name),
            price: price as i64,
            number_of_shares: amount as i64
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
        QxOrderTransaction {
            tx,
            order_tx: input,
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
        for k in self.order_tx.as_bytes().as_slice() {
            bytes.push(*k);
        }
        bytes
    }

    pub fn txid(&self) -> String {
        let digest: [u8; 32] = k12_bytes(&self.as_bytes()).try_into().unwrap();
        get_identity(&digest).to_lowercase()
    }

}


impl AsBytes for QxOrderTransaction {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.tx.as_bytes_without_signature().as_slice() {
            bytes.push(*k);
        }
        for k in self.order_tx.as_bytes().as_slice() {
            bytes.push(*k);
        }
        for k in self._signature.as_slice() {
            bytes.push(*k);
        }
        bytes
    }
}
