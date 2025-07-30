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
    BroadcastComputors = 2,
    BroadcastTick = 3,
    
    
    RequestComputors = 11,
    RequestedQuorumTick = 14,

    BroadcastFutureTickData = 8,
    RequestTickData = 16,
    
    BroadcastTransaction = 24,
    RequestCurrentTickInfo = 27,
    RespondCurrentTickInfo = 28,
    RequestEntity = 31,
    ResponseEntity = 32,
    ResponseEnd = 35,
    
    //Assets
    RequestIssuedAssets = 36,
    RespondIssuedAssets = 37,
    RequestOwnedAssets = 38,
    RespondOwnedAssets = 39,
    RequestPossessedAssets = 40,
    RespondPossessedAssets = 41,
    
    //Contracts
    RequestContractFunction = 42,
    RespondContractFunction = 43,
    
    RequestAssets = 52,
    RespondAssets = 53,
}

impl EntityType {
    pub fn to_byte(&self) -> i8 {
        match self {
            EntityType::UNKNOWN => -1,
            EntityType::ExchangePeers => 0,
            EntityType::BroadcastComputors => 2,
            EntityType::BroadcastTick => 3,
            EntityType::RequestComputors => 11,
            EntityType::RequestedQuorumTick => 14,
            EntityType::BroadcastFutureTickData => 8,
            EntityType::RequestTickData => 16,
            EntityType::BroadcastTransaction => 24,
            EntityType::RequestCurrentTickInfo => 27,
            EntityType::RespondCurrentTickInfo => 28,
            EntityType::RequestEntity => 31,
            EntityType::ResponseEntity => 32,
            EntityType::ResponseEnd => 35,
            EntityType::RequestIssuedAssets => 36,
            EntityType::RespondIssuedAssets => 37,
            EntityType::RequestOwnedAssets => 38,
            EntityType::RespondOwnedAssets => 39,
            EntityType::RequestPossessedAssets => 40,
            EntityType::RespondPossessedAssets => 41,
            EntityType::RequestContractFunction => 42,
            EntityType::RespondContractFunction => 43,
            EntityType::RequestAssets => 52,
            EntityType::RespondAssets => 53,
            EntityType::ERROR => 55
        }
    }
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
            2 => EntityType::BroadcastComputors,
            3 => EntityType::BroadcastTick,
            11 => EntityType::RequestComputors,
            14 => EntityType::RequestedQuorumTick,
            16 => EntityType::RequestTickData,
            8 => EntityType::BroadcastFutureTickData,
            24 => EntityType::BroadcastTransaction,
            27 => EntityType::RequestCurrentTickInfo,
            28 => EntityType::RespondCurrentTickInfo,
            31 => EntityType::RequestEntity,
            32 => EntityType::ResponseEntity,
            35 => EntityType::ResponseEnd,
            36 => EntityType::RequestIssuedAssets,
            37 => EntityType::RespondIssuedAssets,
            38 => EntityType::RequestOwnedAssets,
            39 => EntityType::RespondOwnedAssets,
            40 => EntityType::RequestPossessedAssets,
            41 => EntityType::RespondPossessedAssets,
            42 => EntityType::RequestContractFunction,
            43 => EntityType::RespondContractFunction,
            52 => EntityType::RequestAssets,
            53 => EntityType::RespondAssets,
            55 => EntityType::ERROR,
            _ => EntityType::UNKNOWN
        }
    }
    pub fn recv_multiple_packets(&self) -> bool {   //Some Responses Send Multiple Data Packets Until End Response
        match self.get_type() {
            EntityType::BroadcastTick => true,
            EntityType::RespondIssuedAssets => true,
            EntityType::RespondOwnedAssets => true,
            EntityType::RespondPossessedAssets => true,
            EntityType::RespondAssets => true,
            _ => false
        }
    }
}