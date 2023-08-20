extern crate crypto;
use crypto::random::random_bytes;
use crate::entity::entity_type::REQUEST_ENTITY;

#[derive(Debug, Copy, Clone)]
pub enum entity_type {
    REQUEST_ENTITY = 31
}


//Takes a public key
#[derive(Debug, Copy, Clone)]
pub struct requested_entity {
    pub public_key: [u8; 32]
}

#[derive(Debug, Copy, Clone)]
pub struct request_response_header {
    pub _size: [u8; 3],
    pub _protocol: u8,
    pub _dejavu: [u8; 3],
    pub _type: u8
}

impl request_response_header {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(8);
        bytes.push(self._size[0]);
        bytes.push(self._size[1]);
        bytes.push(self._size[2]);

        bytes.push(self._protocol);

        bytes.push(self._dejavu[0]);
        bytes.push(self._dejavu[1]);
        bytes.push(self._dejavu[2]);

        bytes.push(self._type);
        bytes
    }
    pub fn new() -> Self {
        request_response_header {
            _size: [0; 3],
            _protocol: 0,
            _dejavu: random_bytes(3).as_slice().try_into().unwrap(),
            _type: 0
        }
    }
    pub fn zero_dejavu(&mut self) {
        self._dejavu[0] = 0;
        self._dejavu[1] = 0;
        self._dejavu[2] = 0;
    }
    pub fn set_size(&mut self, _size: usize) {
        self._size[0] = (_size & 0xFF) as u8;
        self._size[1] = ((_size >> 8) & 0xFF) as u8;
        self._size[2] = ((_size >> 16) & 0xFF) as u8;
    }
    pub fn set_type(&mut self, _type: entity_type) {
        self._type = _type as u8;
    }
}

#[derive(Debug, Clone)]
pub struct qubic_request {
    pub header: request_response_header,
    pub data: Vec<u8>
}

impl qubic_request {
    pub fn get_identity_balance(identity: &str) -> Self {
        let entity: entity_type = REQUEST_ENTITY;
        let mut header = request_response_header::new();
        header.set_type(REQUEST_ENTITY);
        let data: Vec<u8> = identity.as_bytes().try_into().unwrap();
        let size = std::mem::size_of::<request_response_header>() + data.len();
        header.set_size(size);
        qubic_request {
            header: header,
            data: data
        }
    }
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut res: Vec<u8> = self.header.as_bytes();
        res.append(&mut self.data);
        res
    }
    pub fn new(data: &Vec<u8>) -> Self {    //todo: remove this, should only be able to create by specific api call
        qubic_request {
            header: request_response_header::new(),
            data: data.clone()
        }
    }
}

#[cfg(test)]
pub mod entity_tests {
    use crate::entity;
    #[test]
    fn create_identity_balance_request_entity() {
        let req = entity::qubic_request::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        println!("{:?}", req);
    }

    #[test]
    fn create_entity_get_full_request_as_bytes() {
        let mut req = entity::qubic_request::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        req.header.zero_dejavu();   //Dejavu is random 3 byte value
        let bytes = req.as_bytes();
        assert_eq!(bytes.len(), 68);
        assert_eq!(bytes.as_slice(),
        vec![68, 0, 0, 0, 0, 0, 0, 31, 69, 80, 89, 87, 68, 82, 69, 68, 78, 76, 72, 88, 79, 70, 89, 86, 71, 81, 85, 75, 80, 72, 74, 71, 79, 77, 80, 66, 83, 76, 68, 68, 71, 90, 68, 80, 75, 86, 81, 85, 77, 70, 88, 65, 73, 81, 89, 77, 90, 71, 69, 72, 80, 90, 84, 65, 65, 87, 79, 78]);
    }
}