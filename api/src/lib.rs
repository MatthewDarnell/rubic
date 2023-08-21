pub mod header;
pub mod response;
extern crate crypto;
extern crate identity;
use identity::get_public_key_from_identity;
use crypto::random::random_bytes;
use crate::header::{ entity_type, request_response_header };
use crate::response::handle_response;

//Takes a public key
#[derive(Debug, Copy, Clone)]
pub struct requested_entity {
    pub public_key: [u8; 32]
}


#[derive(Debug, Clone)]
pub struct qubic_api_t {
    pub header: request_response_header,
    pub data: Vec<u8>,
    pub response_data: Option<Vec<u8>>
}
impl qubic_api_t {
    pub fn get_identity_balance(id: &str) -> Self {
        let entity: entity_type = entity_type::REQUEST_ENTITY;
        let mut header = request_response_header::new();
        header.set_type(entity_type::REQUEST_ENTITY);

        let mut data: Vec<u8> = get_public_key_from_identity(id).unwrap();
println!("Passed identity {}, got pub key {:?}", id, &data);
        //let mut data: Vec<u8> = id.as_bytes().try_into().unwrap();
      //  for el in data.iter_mut() {
         //   *el = *el - 65;
       // }
        let size = std::mem::size_of::<request_response_header>() + data.len();
        header.set_size(size);
        qubic_api_t {
            header: header,
            data: data,
            response_data: None
        }
    }
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut res: Vec<u8> = self.header.as_bytes();
        res.append(&mut self.data);
        res
    }
    pub fn set_response(&mut self, data: Vec<u8>) {
        handle_response(&data);
    }
    pub fn new(data: &Vec<u8>) -> Self {    //todo: remove this, should only be able to create by specific api call
        qubic_api_t {
            header: request_response_header::new(),
            data: data.clone(),
            response_data: None
        }
    }
}

#[cfg(test)]
pub mod entity_tests {
    use crate::entity;
    #[test]
    fn create_identity_balance_request_entity() {
        let req = entity::qubic_api_t::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        println!("{:?}", req);
    }

    #[test]
    fn create_entity_get_full_request_as_bytes() {
        let mut req = entity::qubic_api_t::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        req.header.zero_dejavu();   //Dejavu is random 3 byte value
        let bytes = req.as_bytes();
        assert_eq!(bytes.len(), 68);
        assert_eq!(bytes.as_slice(),
                   vec![68, 0, 0, 0, 0, 0, 0, 31, 69, 80, 89, 87, 68, 82, 69, 68, 78, 76, 72, 88, 79, 70, 89, 86, 71, 81, 85, 75, 80, 72, 74, 71, 79, 77, 80, 66, 83, 76, 68, 68, 71, 90, 68, 80, 75, 86, 81, 85, 77, 70, 88, 65, 73, 81, 89, 77, 90, 71, 69, 72, 80, 90, 84, 65, 65, 87, 79, 78]);
    }
}