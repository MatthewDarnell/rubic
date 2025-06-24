use identity::Identity;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_subseed, get_public_key_from_identity, sign_raw, get_identity};
use logger::info;
use crate::AsBytes;
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

static TICK_OFFSET: u32 = 15;

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
    
    pub fn digest(&self) -> Vec<u8> {
        k12_bytes(&self.as_bytes())
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let _signature = match bytes.len() > 80 {
            true => { bytes[80..].to_vec() },
            false => { Vec::<u8>::with_capacity(64) }
        };
        
        TransferTransaction {
            _source_public_key: bytes[0..32].to_vec(),
            _source_destination_public_key: bytes[32..64].to_vec(),
            _amount: read_le_u64(&mut &bytes[64..]),
            _tick: read_le_u32(&mut &bytes[72..]),
            _input_size: read_le_u16(&mut &bytes[76..]),
            _input_type: read_le_u16(&mut &bytes[78..]),
            _signature
        }
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

impl AsBytes for TransferTransaction {
    fn as_bytes(&self) -> Vec<u8> {
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
       95, 0, 0, 0,
       //input type: u16
       0, 0,
       //input size: u16
       0, 0,
       //signature: u64
       255, 164, 192, 74, 223, 72, 39, 62, 63, 24, 180, 239, 143, 222, 170, 19, 69, 213, 145, 118, 196, 171, 146, 
       114, 58, 72, 68, 143, 240, 121, 232, 54, 6, 35, 130, 134, 111, 160, 239, 39, 223,
       191, 101, 105, 20, 7, 191, 238, 235, 70, 155, 184, 142, 34, 27, 41, 150, 34, 233, 198, 15, 93, 16, 0
    ];

    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
    assert_eq!(t.txid().as_str(), "rifeyehuvbytdhumybpflzqvtaugvpykvpmuvhagcfhwhcnnpbarbbfhgvze");
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
      223, 237, 170,   0,
        0,   0,   0,   0,

        /*begin signature */
        124, 178, 91, 84, 107, 43, 5, 18, 30, 225, 35, 62, 156, 63, 119, 88, 144, 58, 57, 
        194, 106, 119, 49, 32, 179, 83, 215, 232, 5, 90, 201, 137, 243, 69, 191, 6, 28,
        178, 160, 182, 146, 254, 189, 53, 99, 173, 221, 87, 58, 247, 250, 71, 232, 43, 213,
        2, 35, 73, 38, 42, 50, 102, 10, 0

    /*
   250, 222, 115, 210,
   15,  11,  22,  11, 210, 206, 106, 144, 254, 178,   6,  38,
  170,  40, 192, 122, 224, 242, 77,  35, 200,  90, 125,  75,
    3,  86, 132, 160,  73,  63,  40, 119, 116, 227,  46, 249,
   27,  40,   3, 234, 99, 187,  24, 212, 147,  79, 197,  92,
   31, 156, 134, 46, 127,  72,  48, 237, 142, 193,  32,   0
*/
    ];
    
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
        223, 237, 170,   0,
        0,   0,   0,   0,
        /*begin signature */ 124, 178, 91, 84, 107, 43, 5, 18, 30, 
        225, 35, 62, 156, 63, 119, 88, 144, 58, 57, 194, 106, 119, 
        49, 32, 179, 83, 215, 232, 5, 90, 201, 137, 243, 69, 191,
        6, 28, 178, 160, 182, 146, 254, 189, 53, 99, 173, 221, 87,
        58, 247, 250, 71, 232, 43, 213, 2, 35, 73, 38, 42, 50, 102,
        10, 0];

    assert_eq!(t2.as_bytes().as_slice(), expected.as_slice());
}

#[test]
fn decode_transfer_from_bytes() {
    let bytes: [u8; 144] = [
        166, 41, 241, 226, 116, 35, 157, 108, 212, 167, 113, 153, 176, 79, 33, 74, 13, 230, 165, 250, 7, 
        252, 247, 41, 6, 184, 109, 194, 112, 39, 244, 142, 106, 235, 5, 192, 171, 45, 199, 46, 175, 10,
        157, 100, 205, 136, 236, 111, 160, 220, 202, 77, 251, 59, 207, 90, 198, 106, 15, 178, 17, 231, 29, 
        90, 123, 0, 0, 0, 0, 0, 0, 0, 255, 122, 142, 1, 0, 0, 0, 0, 58, 232, 176, 8, 64, 18, 193, 54, 82, 
        219, 127, 162, 148, 121, 0, 113, 194, 214, 214, 243, 158, 31, 91, 59, 157, 250, 193, 57, 255, 155, 
        217, 22, 107, 108, 109, 78, 118, 169, 147, 157, 120, 45, 121, 222, 216, 241, 234, 60, 243, 68, 227, 
        159, 233, 16, 148, 118, 145, 104, 59, 100, 83, 192, 27, 0
    ];
    let tx = TransferTransaction::from_bytes(&bytes);
    let source_id = get_identity(<&[u8; 32]>::try_from(tx._source_public_key.as_slice()).unwrap());
    let dest_id = get_identity(<&[u8; 32]>::try_from(tx._source_destination_public_key.as_slice()).unwrap());
    assert_eq!(source_id, "SKMWVDLPBBJAEDIWAWWAXCBDJZDCZOSAQMQDZOYRFBUBYFHYTESONYDEXBFA".to_string());
    assert_eq!(dest_id, "WGGASSAPFMJIJBVPFQELBGORINGDKJVBPFMNIZUOQCSEBRWBGSDKFBQCNFZJ".to_string());
    assert_eq!(tx._amount, 123);
    assert_eq!(tx._tick, 26114815);
    assert_eq!(tx._input_size, 0);
    assert_eq!(tx._input_type, 0);
    assert_eq!(tx._signature.len(), 64);
    assert_eq!(tx._signature[0], 58);
}