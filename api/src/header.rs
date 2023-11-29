use crypto::random::random_bytes;
#[derive(Debug, Copy, Clone)]
pub struct RequestResponseHeader {
    pub _size: [u8; 3],
    pub _type: u8,
    pub _dejavu: [u8; 3],
    pub _deprecated_type: u8
}

#[derive(Debug, Copy, Clone)]
pub enum EntityType {
    ERROR = 55, //This is for internal message passing, not a real value
    UNKNOWN = -1,
    ExchangePeers = 0,
    BroadcastTransaction = 24,
    RequestEntity = 31,
    ResponseEntity = 32
}

impl RequestResponseHeader {
    pub fn from_vec(vec: &Vec<u8>) -> Self {
        let mut header = RequestResponseHeader::new();
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
        RequestResponseHeader {
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

    pub fn set_type(&mut self, _type: EntityType) {
        self._type = _type as u8;
    }
    pub fn get_type(&self) -> EntityType {
        match self._type {
            0 => EntityType::ExchangePeers,
            31 => EntityType::RequestEntity,
            32 => EntityType::ResponseEntity,
            55 => EntityType::ERROR,
            _ => EntityType::UNKNOWN
        }
    }
}