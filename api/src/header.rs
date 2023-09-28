use crypto::random::random_bytes;
#[derive(Debug, Copy, Clone)]
pub struct request_response_header {
    pub _size: [u8; 3],
    pub _type: u8,
    pub _dejavu: [u8; 3],
    pub _deprecated_type: u8
}

#[derive(Debug, Copy, Clone)]
pub enum entity_type {
    ERROR = 55, //This is for internal message passing, not a real value
    UNKNOWN = -1,
    EXCHANGE_PEERS = 0,
    REQUEST_ENTITY = 31,
    RESPONSE_ENTITY = 32
}

impl request_response_header {
    pub fn from_vec(vec: &Vec<u8>) -> Self {
        let mut header = request_response_header::new();
        header._size[0] = vec[0];
        header._size[1] = vec[1];
        header._size[2] = vec[2];

        header._type = vec[3];

        header._dejavu[0] = vec[4];
        header._dejavu[1] = vec[5];
        header._dejavu[2] = vec[6];

        header._deprecated_type = vec[7];
        return header;
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(8);
        bytes.push(self._size[0]);
        bytes.push(self._size[1]);
        bytes.push(self._size[2]);

        bytes.push(self._type);

        bytes.push(self._dejavu[0]);
        bytes.push(self._dejavu[1]);
        bytes.push(self._dejavu[2]);

        bytes.push(self._deprecated_type);
        bytes
    }
    pub fn new() -> Self {
        request_response_header {
            _size: [0; 3],
            _type: 0,
            _dejavu: random_bytes(3).as_slice().try_into().unwrap(),
            _deprecated_type: 0
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
    pub fn get_size(&self) -> usize {
        let mut size: usize = 0;
        size |= self._size[2] as usize;
        size <<= 8;
        size |= self._size[1] as usize & 0xFF;
        size <<= 8;
        size |= self._size[0] as usize & 0xFF;
        return size;
    }

    pub fn set_type(&mut self, _type: entity_type) {
        self._type = _type as u8;
    }
    pub fn get_type(&self) -> entity_type {
        match self._type {
            0 => entity_type::EXCHANGE_PEERS,
            31 => entity_type::REQUEST_ENTITY,
            32 => entity_type::RESPONSE_ENTITY,
            55 => entity_type::ERROR,
            _ => entity_type::UNKNOWN
        }
    }
}