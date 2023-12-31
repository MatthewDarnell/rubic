use crypto::random::random_bytes;
#[derive(Debug, Copy, Clone)]
pub struct RequestResponseHeader {
    pub _size: [u8; 3],
    pub _type: u8,
    pub _dejavu: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum EntityType {
    ERROR = 55, //This is for internal message passing, not a real value
    UNKNOWN = -1,
    ExchangePeers = 0,
    BroadcastTransaction = 24,
    RequestCurrentTickInfo = 27,
    RespondCurrentTickInfo = 28,
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

        let r: [u8; 4] = [vec[4], vec[5], vec[6], vec[7]];
        header._dejavu = u32::from_le_bytes(r);
        return header;
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(8);
        bytes.push(self._size[0]);
        bytes.push(self._size[1]);
        bytes.push(self._size[2]);

        bytes.push(self._type);

        for v in self._dejavu.to_le_bytes() {
            bytes.push(v);
        }

        bytes
    }
    pub fn new() -> Self {
        let r: [u8; 4] = random_bytes(4).as_slice().try_into().unwrap();
        RequestResponseHeader {
            _size: [0; 3],
            _type: 0,
            _dejavu: u32::from_le_bytes(r)
        }
    }
    pub fn zero_dejavu(&mut self) {
        self._dejavu = 0;
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
            24 => EntityType::BroadcastTransaction,
            27 => EntityType::RequestCurrentTickInfo,
            28 => EntityType::RespondCurrentTickInfo,
            31 => EntityType::RequestEntity,
            32 => EntityType::ResponseEntity,
            55 => EntityType::ERROR,
            _ => EntityType::UNKNOWN
        }
    }
}