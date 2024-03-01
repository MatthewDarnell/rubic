use std::ffi::c_uchar;
use identity::Identity;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{ get_subseed, get_public_key_from_identity };
use logger::info;
extern {
    //extern ECCRYPTO_STATUS SchnorrQ_Sign(const unsigned char* SecretKey, const unsigned char* PublicKey, const unsigned char* Message, const unsigned int SizeMessage, unsigned char* Signature);
    fn sign(subseed: *const u8, publicKey: *const c_uchar, messageDigest: *const c_uchar, signature: *mut c_uchar);
    //fn SchnorrQ_Sign(subseed: *const u8, publicKey: *const c_uchar, messageDigest: *const c_uchar, SizeMessage: u32, signature: *mut c_uchar);
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

static TICK_OFFSET: u32 = 30;

impl TransferTransaction {
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
        let mut sub_seed: Vec<u8> = get_subseed(source_identity.seed.as_str()).expect("Failed To Get SubSeed!");
        unsafe {
            getSubseed(source_identity.seed.as_str().as_ptr(), sub_seed.as_mut_ptr());
        }
        let mut sig: [u8; 64] = [0; 64];
        unsafe {
            sign(sub_seed.as_slice().as_ptr(), pub_key_src.as_ptr(), digest.as_ptr(), sig.as_mut_ptr());
            //SchnorrQ_Sign(sub_seed.as_ptr(), pub_key_src.as_ptr(), digest.as_ptr(), 32, sig.as_mut_ptr());
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