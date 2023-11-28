use identity::get_public_key_from_identity;
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

impl TransferTransaction {
    pub fn from_vars(src: &String, dest: &String, amount: u64, tick: u32, sig: Vec<u8>) -> Self {
        let pub_key_src = match get_public_key_from_identity(src.as_str()) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };
        let pub_key_dest = match get_public_key_from_identity(src.as_str()) {
            Ok(pub_key) => pub_key,
            Err(err) => panic!("{:?}", err)
        };
        TransferTransaction {
            _source_public_key: pub_key_src,
            _source_destination_public_key: pub_key_dest,
            _amount: amount,
            _tick: tick,
            _input_type: 0,
            _input_size: 0,
            _signature: sig
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(144);
        for k in self._source_public_key.as_slice() {
            bytes.push(*k);
        }
        for k in self._source_destination_public_key.as_slice() {
            bytes.push(*k);
        }
        for c in self._amount.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_type.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._input_size.to_le_bytes() {
            bytes.push(c);
        }

        for c in self._tick.to_le_bytes() {
            bytes.push(c);
        }

        for k in self._signature.as_slice() {
            bytes.push(*k);
        }
        bytes
    }

}



#[test]
fn create_transfer() {
    //[170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    let pub_key_src = identity::get_public_key_from_identity("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON").unwrap();
    let pub_key_dest = identity::get_public_key_from_identity("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON").unwrap();
    let sig: Vec<u8> = vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let t: TransferTransaction = TransferTransaction::from_vars(&"EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON".to_string(), &"EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON".to_string(), 100, 100, sig);
    println!("{:?}", &t);
    let expected: Vec<u8> = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //assert_eq!(t.as_bytes().as_slice(), expected.as_slice());
}