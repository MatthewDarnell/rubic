
use protocol::identity::Identity;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_subseed, get_public_key_from_identity, sign_raw, get_identity};
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
pub struct CustomTransferTransaction {
    pub _source_public_key: Vec<u8>,
    pub _source_destination_public_key: Vec<u8>,
    pub _amount: u64,
    pub _tick: u32,
    pub _input_type: u16,
    pub _input_size: u16,
    pub _extra_data: Vec<u8>,
    pub _signature: Vec<u8>
}

impl CustomTransferTransaction {

    pub fn from_vars(source_identity: &Identity, dest: &str, amount: u64, input_type: u16, scheduled_tick: u32, data: &Vec<u8>) -> Self {
        if data.len() > u16::MAX as usize {
            panic!("Trying To Embed Too Much Data In CustomTransferTransaction!");
        }
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
        let mut t: CustomTransferTransaction = CustomTransferTransaction {
            _source_public_key: pub_key_src.to_vec(),
            _source_destination_public_key: pub_key_dest.to_vec(),
            _amount: amount,
            _tick: scheduled_tick,
            _input_type: input_type,
            _input_size: data.len() as u16,
            _extra_data: data.to_vec(),
            _signature: Vec::with_capacity(64)
        };
        let digest: Vec<u8> = k12_bytes(&t.as_bytes_without_signature());
        //let mut sub_seed: [u8; 32] = [0; 32];
        let sub_seed: Vec<u8> = get_subseed(source_identity.seed.as_str()).expect("Failed To Get SubSeed!");
        #[allow(unused_assignments)]
        let mut sig: [u8; 64] = [0; 64];
        sig = sign_raw(&sub_seed, &pub_key_src, digest.as_slice().try_into().unwrap());
        t._signature = sig.to_vec();
        t
    }

    /// Rebuilds a transaction from its stored parts, signature included, so a
    /// recorded transaction can be broadcast again without the seed.
    pub fn from_signed_parts(source_pub: &[u8; 32], dest_pub: &[u8; 32], amount: u64, tick: u32, input_type: u16, data: &[u8], signature: &[u8]) -> Self {
        CustomTransferTransaction {
            _source_public_key: source_pub.to_vec(),
            _source_destination_public_key: dest_pub.to_vec(),
            _amount: amount,
            _tick: tick,
            _input_type: input_type,
            _input_size: data.len() as u16,
            _extra_data: data.to_vec(),
            _signature: signature.to_vec(),
        }
    }

    pub fn digest(&self) -> Vec<u8> {
        k12_bytes(&self.as_bytes())
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

        for byte in self._extra_data.as_slice() {
            bytes.push(*byte)
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

        for byte in self._extra_data.as_slice() {
            bytes.push(*byte)
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let input_size: u16 = u16::from_le_bytes([bytes[78], bytes[79]]);


        let data: Vec<u8> = match input_size > 0 {
            true => bytes[80..80 + input_size as usize].to_vec(),
            false => Vec::<u8>::new()
        };

        let _signature = match bytes.len() - input_size as usize > 80 {
            true => { bytes[80 + input_size as usize..].to_vec() },
            false => { Vec::<u8>::with_capacity(64) }
        };

        CustomTransferTransaction {
            _source_public_key: bytes[0..32].to_vec(),
            _source_destination_public_key: bytes[32..64].to_vec(),
            _amount: read_le_u64(&mut &bytes[64..]),
            _tick: read_le_u32(&mut &bytes[72..]),
            _input_type: read_le_u16(&mut &bytes[76..]),
            _input_size: read_le_u16(&mut &bytes[78..]),
            _extra_data: data.to_vec(),
            _signature
        }

    }

    pub fn txid(&self) -> String {
        let digest: [u8; 32] = k12_bytes(&self.as_bytes()).try_into().unwrap();
        get_identity(&digest).to_lowercase()
    }

}


#[test]
fn create_custom_transfer_without_data_and_check_txid() {
    let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let d: Vec<u8> = vec![];

    let t: CustomTransferTransaction = CustomTransferTransaction::from_vars(&id, "DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANMIG", 100, 1, 80, &d);
    let expected: Vec<u8> = vec![
        //source pub key: u32
        170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26,
        //dest pub key: u32
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //amount: u64
        100, 0, 0, 0, 0, 0, 0, 0,
        //tick: u32
        80, 0, 0, 0,
        //input type: u16
        1, 0,
        //input size: u16
        0, 0,
        //signature: u64
        92, 248, 181, 161, 207, 113, 104, 43, 239, 93, 200, 156, 44, 172, 19, 32, 177, 129, 194, 206, 142, 37, 5, 97, 231, 163, 116, 196, 143, 185,
        120, 183, 11, 239, 172, 196, 173, 125, 24, 148, 85, 82, 102, 74, 240, 251, 71, 49, 174, 6, 77, 238, 164, 83, 147, 61, 189, 5, 49, 149, 117,
        10, 31, 0
    ];
    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
    assert_eq!(t.txid().as_str(), "emiausxqvqkqzbzxphibziijnsrcyvnsznienuwdhagwbcacalsercsfdsbh");
}


#[test]
fn create_custom_transfer_and_check_txid() {
    let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
    let d: Vec<u8> = vec![1, 1, 1, 1, 1, 1, 1, 1];

    let t: CustomTransferTransaction = CustomTransferTransaction::from_vars(&id, "DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANMIG", 100, 1, 80, &d);
    let expected: Vec<u8> = vec![
        //source pub key: u32
        170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26,
        //dest pub key: u32
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //amount: u64
        100, 0, 0, 0, 0, 0, 0, 0,
        //tick: u32
        80, 0, 0, 0,
        //input type: u16
        1, 0,
        //input size: u16
        8, 0,
        //data: [u8]
        1, 1, 1, 1, 1, 1, 1, 1,
        //signature: u64
        228, 145, 83, 143, 193, 138, 101, 164, 172, 210, 196, 39, 171, 154, 77, 4, 51, 175, 190, 234, 61, 16, 166, 206, 149, 48, 157, 31, 247,
        123, 216, 79, 151, 51, 62, 92, 66, 143, 53, 111, 176, 139, 160, 104, 211, 180, 59, 214, 113, 81, 66, 86, 251, 28, 81, 39, 32, 67, 10,
        238, 57, 12, 0, 0
    ];
    assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
    assert_eq!(t.txid().as_str(), "rfuawzsiyjcskdxpfbucrdcsttkcbezfrwonwmcvaaejviyhgytxgysajwwa");
}



#[test]
fn decode_custom_transfer_from_bytes() {
    let bytes: Vec<u8> = vec![
        //source pub key: u32
        170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26,
        //dest pub key: u32
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //amount: u64
        100, 0, 0, 0, 0, 0, 0, 0,
        //tick: u32
        80, 0, 0, 0,
        //input type: u16
        1, 0,
        //input size: u16
        8, 0,
        //data: [u8]
        1, 1, 1, 1, 1, 1, 1, 1,
        //signature: u64
        228, 145, 83, 143, 193, 138, 101, 164, 172, 210, 196, 39, 171, 154, 77, 4, 51, 175, 190, 234, 61, 16, 166, 206, 149, 48, 157, 31, 247,
        123, 216, 79, 151, 51, 62, 92, 66, 143, 53, 111, 176, 139, 160, 104, 211, 180, 59, 214, 113, 81, 66, 86, 251, 28, 81, 39, 32, 67, 10,
        238, 57, 12, 0, 0
    ];
    let tx = CustomTransferTransaction::from_bytes(&bytes);

    let source_id = get_identity(<&[u8; 32]>::try_from(tx._source_public_key.as_slice()).unwrap());
    let dest_id = get_identity(<&[u8; 32]>::try_from(tx._source_destination_public_key.as_slice()).unwrap());
    let digest = tx.digest();
    assert_eq!(source_id, "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON".to_string());
    assert_eq!(dest_id, "DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANMIG".to_string());
    assert_eq!(tx._amount, 100);
    assert_eq!(tx._tick, 80);
    assert_eq!(tx._input_type, 1);
    assert_eq!(tx._input_size, 8);
    assert_eq!(tx._signature.len(), 64);
    assert_eq!(tx._extra_data[7], 1);
    assert_eq!(tx._signature[0], 228);
    
    let _digest = vec![227, 173, 164, 44, 48, 180, 118, 117, 185, 163, 55, 12, 211, 153, 29, 
                       83, 133, 89, 240, 3, 167, 21, 19, 1, 2, 219, 106, 43, 222, 253, 18, 25];
    let equal = digest[..].iter().zip(_digest[..].iter()).all(|(a,b)| a == b);
    assert!(equal);

}
