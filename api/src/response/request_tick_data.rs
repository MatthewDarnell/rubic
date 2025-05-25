use crypto::qubic_identities::get_identity;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use crate::response::response_entity::ResponseEntity;

#[derive(Debug, Clone)]
struct TransactionDigest([u8; 1024]);

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
    pub fn new(data: &Vec<u8>) -> TickData {
        let mut tx_digests = Vec::<TransactionDigest>::with_capacity(32);
        let mut tx_digest_iter = data[48..32816].chunks_exact(1024);
        while let Some(value) = tx_digest_iter.next() {
            let temp: TransactionDigest = TransactionDigest(value[0..1024].try_into().unwrap());
            tx_digests.push(temp);
        }

        let mut contract_fees = Vec::<i64>::with_capacity(1024);
        let mut contract_fee_iter = data[32817..32817 + (1024 * 8)].chunks_exact(8);
        while let Some(value) = contract_fee_iter.next() {
            let temp: i64 = i64::from_le_bytes(value.try_into().unwrap());
            contract_fees.push(temp);
        }
        
        let signature: Vec<u8> = data[32817 + (1024 * 8) + 1..].to_vec();
        TickData {
            computor_index: u16::from_le_bytes([data[0], data[1]]),
            epoch: u16::from_le_bytes([data[2], data[3]]),
            tick: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            millisecond: u8::from_le_bytes([data[8]]),
            second: u8::from_le_bytes([data[9]]),
            minute: u8::from_le_bytes([data[10]]),
            hour: u8::from_le_bytes([data[11]]),
            day: u8::from_le_bytes([data[12]]),
            month: u8::from_le_bytes([data[13]]),
            year: u8::from_le_bytes([data[14]]),

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