extern crate crypto;
use crypto::random::random_bytes;

//#define REQUEST_ENTITY 31
//
// typedef struct
// {
//     unsigned char publicKey[32];
// } RequestedEntity;
#[derive(Debug, Copy, Clone)]
pub enum entity_type {
    REQUEST_ENTITY = 31
}


//Takes a public key
#[derive(Debug, Copy, Clone)]
pub struct requested_entity {
    pub public_key: [u8; 32]
}


//struct RequestResponseHeader
// {
// private:
//     unsigned char _size[3];
//     unsigned char _protocol;
//     unsigned char _dejavu[3];
//     unsigned char _type;
#[derive(Debug, Copy, Clone)]
pub struct request_response_header {
    pub _size: [u8; 3],
    pub _protocol: u8,
    pub _dejavu: [u8; 3],
    pub _type: u8
}

impl request_response_header {
    pub fn new() -> Self {
        request_response_header {
            _size: [0; 3],
            _protocol: 0,
            _dejavu: random_bytes(3).as_slice().try_into().unwrap(),
            _type: 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct qubic_request {
    pub entity_t: entity_type,
    pub header: request_response_header,
    pub data: Vec<u8>
}

impl qubic_request {
    pub fn new(entity_t: entity_type, data: &Vec<u8>) -> Self {
        qubic_request {
            entity_t: entity_t,
            header: request_response_header::new(),
            data: data.clone()
        }
    }
}
