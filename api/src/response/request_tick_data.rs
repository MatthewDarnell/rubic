use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;

#[derive(Debug, Clone)]
pub struct TransactionDigest([u8; 1024]);

#[derive(Debug, Clone)]
pub struct TickData {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub millisecond: u8,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,

    pub time_lock: [u8; 32],
    //pub transaction_digests: [TransactionDigest; 32],
    pub transaction_digests: Vec<TransactionDigest>,
    pub contract_fees: Vec<i64>,
    //pub contract_fees: [i64; 1024],
    //signature: [u8; 64]
    signature: Vec<u8>
}

impl TickData {
    pub fn print(&self) {
        println!("-----\nTick:\n\ttick.({})\n\tcomputor.({})\n\tepoch.({})\n\tnum_txs.({})\n\tsig: {:?}\n-----",
        self.tick,
        self.computor_index,
        self.epoch,
        self.transaction_digests.len(),
        &self.signature);
    }
    pub fn new(data: &Vec<u8>) -> TickData {
        let(_, right) = data.split_at(std::mem::size_of::<RequestResponseHeader>());
        let tick: u32 = u32::from_le_bytes([right[4], right[5], right[6], right[7]]);
        //println!("Tick={}", tick);
        let mut tx_digests = Vec::<TransactionDigest>::with_capacity(32);
        let mut tx_digest_iter = right[48..32816].chunks_exact(1024);
        while let Some(value) = tx_digest_iter.next() {
            let temp: TransactionDigest = TransactionDigest(value[0..1024].try_into().unwrap());
            tx_digests.push(temp);
        }
        //println!("Formatted {} Tx Digests For Tick {}", tx_digests.len(), tick);

        let mut contract_fees = Vec::<i64>::with_capacity(1024);
        let mut contract_fee_iter = right[32817..32817 + (1024 * 8)].chunks_exact(8);
        while let Some(value) = contract_fee_iter.next() {
            let temp: i64 = i64::from_le_bytes(value.try_into().unwrap());
            contract_fees.push(temp);
        }
        
        let signature: Vec<u8> = right[32817 + (1024 * 8) + 1..].to_vec();
        TickData {
            computor_index: u16::from_le_bytes([right[0], right[1]]),
            epoch: u16::from_le_bytes([right[2], right[3]]),
            tick: u32::from_le_bytes([right[4], right[5], right[6], right[7]]),
            millisecond: u8::from_le_bytes([right[8]]),
            second: u8::from_le_bytes([right[9]]),
            minute: u8::from_le_bytes([right[10]]),
            hour: u8::from_le_bytes([right[11]]),
            day: u8::from_le_bytes([right[12]]),
            month: u8::from_le_bytes([right[13]]),
            year: u8::from_le_bytes([right[14]]),

            time_lock: data[15..47].try_into().unwrap(),
            transaction_digests: tx_digests,
            contract_fees,
            signature
        }
    }
}

impl FormatQubicResponseDataToStructure for TickData {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_tick_data(response) }
}

pub fn handle_tick_data(response: &mut QubicApiPacket) -> Option<TickData> {
    let data: &Vec<u8> = &response.as_bytes();
    if data.len() < 41074 {
        return None;
    }
    Some(TickData::new(data))
}