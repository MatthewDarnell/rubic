use identity::Identity;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_subseed, get_public_key_from_identity, sign_raw, get_identity};
use logger::info;

#[derive(Debug, Clone)]
pub struct TransferTransaction {
    pub _source_public_key: Vec<u8>,
    pub _source_destination_public_key: Vec<u8>,
    pub _amount: u64,
    pub _tick: u32,
    pub _input_type: u16,
    pub _input_size: u16,
    pub _signature: Vec<u8>
}

static TICK_OFFSET: u32 = 25;

impl TransferTransaction {

    pub fn from_signed_data(
                            src_pub_key: &[u8; 32],
                            dest_pub_key: &[u8; 32],
                            amount: u64,
                            tick: u32,
                            input_type: u16,
                            input_size: u16,
                            sig: &[u8]) -> Self
    {
        TransferTransaction {
            _source_public_key: src_pub_key.to_vec(),
            _source_destination_public_key: dest_pub_key.to_vec(),
            _amount: amount,
            _tick: tick,
            _input_type: input_type,
            _input_size: input_size,
            _signature: sig.to_vec()
        }
    }
    pub fn from_vars(source_identity: &Identity, dest: &str, amount: u64, tick: u32) -> Self {
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
        let pub_key_dest = match get_public_key_from_identity(&String::from(dest)) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };
        let mut t: TransferTransaction = TransferTransaction {
            _source_public_key: pub_key_src.to_vec(),
            _source_destination_public_key: pub_key_dest.to_vec(),
            _amount: amount,
            _tick: tick + TICK_OFFSET,
            _input_type: 0,
            _input_size: 0,
            _signature: Vec::with_capacity(64)
        };
        info!("Setting Expiration Tick For Transaction To {}", tick + TICK_OFFSET);
        let digest: Vec<u8> = k12_bytes(&t.as_bytes_without_signature());
        //let mut sub_seed: [u8; 32] = [0; 32];
        let sub_seed: Vec<u8> = get_subseed(source_identity.seed.as_str()).expect("Failed To Get SubSeed!");
        #[allow(unused_assignments)]
        let mut sig: [u8; 64] = [0; 64];
        sig = sign_raw(&sub_seed, &pub_key_src, digest.as_slice().try_into().unwrap());
        t._signature = sig.to_vec();
        t
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self._source_public_key.as_slice() {
            bytes.push(*k);
        }
        for k in self._source_destination_public_key.as_slice() {
            bytes.push(*k);
        }
        for c in self._amount.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._tick.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_type.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_size.to_le_bytes() {
            bytes.push(c);
        }

        for k in self._signature.as_slice() {
            bytes.push(*k);
        }
        bytes
    }

    pub fn as_bytes_without_signature(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self._source_public_key.as_slice() {
            bytes.push(*k);
        }
        for k in self._source_destination_public_key.as_slice() {
            bytes.push(*k);
        }
        for c in self._amount.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._tick.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_type.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_size.to_le_bytes() {
            bytes.push(c);
        }

        bytes
    }
    
    pub fn txid(&self) -> String {
        let digest: [u8; 32] = k12_bytes(&self.as_bytes()).try_into().unwrap();
        get_identity(&digest).to_lowercase()
    }

}


#[test]
fn create_transfer_and_check_txid() {
    let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let t: TransferTransaction = TransferTransaction::from_vars(&id, "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 100, 80);
    let expected: Vec<u8> = vec![
       //source pub key: u32
       170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26,
       //dest pub key: u32
       170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26,
       //amount: u64
       100, 0, 0, 0, 0, 0, 0, 0,
       //tick: u32
       110, 0, 0, 0,
       //input type: u16
       0, 0,
       //input size: u16
       0, 0,
       //signature: u64
       179, 108, 100, 1, 209, 21, 45, 198, 110, 190, 137, 194, 107, 157, 36, 76, 124, 94, 142, 45, 125, 220, 238, 70, 17, 253, 181, 125, 147, 192, 126,
       93, 7, 155, 196, 186, 185, 143, 220, 131, 215, 170, 241, 92, 83, 71, 181, 143, 107, 62, 90, 232, 10, 164, 55, 202, 24, 189, 84, 156, 203, 51, 27, 0
    ];

    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
    assert_eq!(t.txid().as_str(), "ncpeapoygdnmibkoxrvydquuifobdotzzjtjjdeacddymugdazstafqbvnug");
}


#[test]
fn create_another_transfer() {
    let id: Identity = Identity::new("lcehvbvddggksfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let t: TransferTransaction = TransferTransaction::from_vars(&id, "XBFULHWOBTSMNBHDGQXGUMPMLKWCECTIDIDEHCAJFCKYILPWPIVGADEGZTYJ", 10, 11202000);
    let expected: Vec<u8> = vec![
        37, 112,  53,  49, 107, 136, 119, 158, 242,  99, 180,  87,
        210, 149,  13,  37, 184, 195,   1, 183,  74, 237, 103, 254,
      177,  29, 206,  92, 194, 153, 169, 137,  21, 147, 195, 140,
       38,  28,  76,  52, 221,  94,  94, 219, 189, 129, 136,  98,
      148, 135, 210,  91,  26,  54, 242,  75,  66, 181,  44, 135,
        8,  85,  12, 212,
        10,   0,   0,   0,   0,   0,   0,   0,
      238, 237, 170,   0,
        0,   0,   0,   0,
        /*begin signature */ 250, 222, 115, 210,
       15,  11,  22,  11, 210, 206, 106, 144, 254, 178,   6,  38,
      170,  40, 192, 122, 224, 242, 77,  35, 200,  90, 125,  75,
        3,  86, 132, 160,  73,  63,  40, 119, 116, 227,  46, 249,
       27,  40,   3, 234, 99, 187,  24, 212, 147,  79, 197,  92,
       31, 156, 134, 46, 127,  72,  48, 237, 142, 193,  32,   0];
    
    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
}

#[test]
fn create_transfer_from_signed_vars() {
    let id: Identity = Identity::new("lcehvbvddggksfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let t: TransferTransaction = TransferTransaction::from_vars(&id, "XBFULHWOBTSMNBHDGQXGUMPMLKWCECTIDIDEHCAJFCKYILPWPIVGADEGZTYJ", 10, 11202000);
    
    let source = get_public_key_from_identity(&id.identity).unwrap();
    let dest = get_public_key_from_identity(&"XBFULHWOBTSMNBHDGQXGUMPMLKWCECTIDIDEHCAJFCKYILPWPIVGADEGZTYJ".to_string()).unwrap();
    let t2: TransferTransaction = TransferTransaction::from_signed_data(&source, &dest, 10, t._tick, 0, 0, t._signature.as_slice());
    
    let expected: Vec<u8> = vec![
        37, 112,  53,  49, 107, 136, 119, 158, 242,  99, 180,  87,
        210, 149,  13,  37, 184, 195,   1, 183,  74, 237, 103, 254,
        177,  29, 206,  92, 194, 153, 169, 137,  21, 147, 195, 140,
        38,  28,  76,  52, 221,  94,  94, 219, 189, 129, 136,  98,
        148, 135, 210,  91,  26,  54, 242,  75,  66, 181,  44, 135,
        8,  85,  12, 212,
        10,   0,   0,   0,   0,   0,   0,   0,
        238, 237, 170,   0,
        0,   0,   0,   0,
        /*begin signature */ 250, 222, 115, 210,
        15,  11,  22,  11, 210, 206, 106, 144, 254, 178,   6,  38,
        170,  40, 192, 122, 224, 242, 77,  35, 200,  90, 125,  75,
        3,  86, 132, 160,  73,  63,  40, 119, 116, 227,  46, 249,
        27,  40,   3, 234, 99, 187,  24, 212, 147,  79, 197,  92,
        31, 156, 134, 46, 127,  72,  48, 237, 142, 193,  32,   0];

    assert_eq!(t2.as_bytes().as_slice(), expected.as_slice());
}