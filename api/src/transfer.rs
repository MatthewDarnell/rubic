use std::ffi::c_uchar;
use identity::{Identity, get_public_key_from_identity};
use crypto::hash::k12_bytes;

extern {
    fn sign(subseed: *const u8, publicKey: *const c_uchar, messageDigest: *const c_uchar, signature: *mut c_uchar);
    fn getSubseed(seed: *const c_uchar, subseed: *mut c_uchar) -> bool;
    //bool getSubseed(const unsigned char* seed, unsigned char* subseed)
    //void sign(const unsigned char* subseed, const unsigned char* publicKey, const unsigned char* messageDigest, unsigned char* signature)
}


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

static TICK_OFFSET: u32 = 10;

impl TransferTransaction {
    pub fn from_vars(source_identity: &Identity, dest: &str, amount: u64, tick: u32) -> Self {
        if source_identity.encrypted {
            panic!("Trying to Transfer From Encrypted Wallet!");
        }
        if source_identity.seed.len() != 55 {
            panic!("Trying To Transfer From Corrupted Identity!");
        }
        let pub_key_src = match get_public_key_from_identity(source_identity.identity.as_str()) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };
        let pub_key_dest = match get_public_key_from_identity(dest) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };

        let mut t: TransferTransaction = TransferTransaction {
            _source_public_key: pub_key_src.clone(),
            _source_destination_public_key: pub_key_dest.clone(),
            _amount: amount,
            _tick: tick + TICK_OFFSET,
            _input_type: 0,
            _input_size: 0,
            _signature: Vec::with_capacity(64)
        };
        let digest: Vec<u8> = k12_bytes(&t.as_bytes_without_signature());
        let mut sub_seed: [u8; 32] = [0; 32];
        unsafe {
            getSubseed(source_identity.seed.as_str().as_ptr(), sub_seed.as_mut_ptr());
        }
        let mut sig: [u8; 64] = [0; 64];
        unsafe {
            sign(sub_seed.as_ptr(), pub_key_src.as_ptr(), digest.as_ptr(), sig.as_mut_ptr());
        }
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

}


  
#[test]
fn create_transfer() {
    let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let t: TransferTransaction = TransferTransaction::from_vars(&id, "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 100, 100);
    let expected: Vec<u8> = vec![170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26, 170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 110, 0, 0, 0, 105, 55, 108, 214, 255, 246, 151, 81, 6, 214, 129, 65, 96, 14, 146, 66, 206, 140, 212, 149, 217, 230, 189, 217, 106, 16, 216, 3, 208, 51, 185, 179, 25, 89, 215, 168, 85, 62, 9, 204, 52, 238, 245, 199, 48, 2, 43, 52, 117, 72, 109, 119, 84, 236, 135, 240, 56, 179, 194, 36, 96, 124, 32, 0];
    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
}