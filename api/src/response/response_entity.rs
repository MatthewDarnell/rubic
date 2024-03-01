use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use crate::crypto::qubic_identities::get_identity;
#[derive(Debug, Clone)]
pub struct ResponseEntity {
    pub identity: String,
    pub peer: String,
    pub incoming: u64,
    pub outgoing: u64,
    pub final_balance: u64,
    pub number_incoming_transactions: u32,
    pub number_outgoing_transactions: u32,
    pub latest_incoming_transfer_tick: u32,
    pub latest_outgoing_transfer_tick: u32,
    pub tick: u32,
    pub spectrum_index: i32
}

impl ResponseEntity {
    pub fn new(identity: &str, peer: &str, inc: u64, out: u64, num_in_txs: u32, num_out_txs: u32, lt_in_tx: u32, lt_out_tx: u32, tick: u32, s_in: i32) -> ResponseEntity {
        ResponseEntity {
            identity: identity.to_string(),
            peer: peer.to_string(),
            incoming: inc,
            outgoing: out,
            final_balance: match inc > out {
                true => inc - out,
                false => out - inc
            },
            number_incoming_transactions: num_in_txs,
            number_outgoing_transactions: num_out_txs,
            latest_incoming_transfer_tick: lt_in_tx,
            latest_outgoing_transfer_tick: lt_out_tx,
            tick: tick,
            spectrum_index: s_in
        }
    }
    pub fn print(&self) {
        println!("----Response Entity----");
        println!("Identity: ({})", self.identity.as_str());
        println!("Peer: ({})", self.peer.as_str());
        println!("Incoming: ({}) Qus", self.incoming);
        println!("Outgoing: ({}) Qus", self. outgoing);
        println!("Final Balance: <{}> Qus", self.final_balance);
        println!("raw_num_incoming_transfers: {}", self.number_incoming_transactions);
        println!("raw_num_outgoing_transfers: {}", self.number_outgoing_transactions);
        println!("raw_latest_incoming_transfer_tick: ({})", self.latest_incoming_transfer_tick);
        println!("raw_latest_outgoing_transfer_tick: ({})", self.latest_outgoing_transfer_tick);
        println!("Latest Tick: {}", self.tick);
        println!("Spectrum Index: {}", self.spectrum_index);
        println!("-----------------------");
    }
}

impl FormatQubicResponseDataToStructure for ResponseEntity {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Self {handle_response_entity(response).unwrap() /* check it's valid before calling! */ }
}


pub fn handle_response_entity(response: &mut QubicApiPacket) -> Option<ResponseEntity> {
    let data: &Vec<u8> = &response.as_bytes();
    if data.len() < 8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4 {
        return None;
    }
    //println!("Got Response Entity Peer Data: {:?}", data);
    let mut slice: [u8; 32] = [0; 32];
    for (idx, p) in data.as_slice()[8..40].iter().enumerate() {
        slice[idx] = *p;
    }
    let sliced_identity = get_identity(&slice);
    //let sliced_identity = get_identity_from_pub_key(&data.as_slice()[8..40]);
    let sliced_incoming = &data.as_slice()[8 + 32..8+32 + 8];
    let sliced_outgoing = &data.as_slice()[8 + 32 + 8..8+32 + 8 + 8];

    let raw_num_incoming_transfers = &data.as_slice()[8 + 32 + 8 + 8..8+32 + 8 + 8 + 4];
    let raw_num_outgoing_transfers = &data.as_slice()[8 + 32 + 8 + 8 + 4..8+32 + 8 + 8 + 4 + 4];

    let mut number_incoming_transfers: u32 = raw_num_incoming_transfers[3] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= raw_num_incoming_transfers[2] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= raw_num_incoming_transfers[1] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= raw_num_incoming_transfers[0] as u32 & 0xFF;

    let mut number_outgoing_transfers: u32 = raw_num_outgoing_transfers[3] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= raw_num_outgoing_transfers[2] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= raw_num_outgoing_transfers[1] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= raw_num_outgoing_transfers[0] as u32 & 0xFF;


    let mut incoming: u64 = sliced_incoming[7] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[6] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[5] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[4] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[3] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[2] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[1] as u64 & 0xFF;
    incoming <<= 8;
    incoming |= sliced_incoming[0] as u64 & 0xFF;

    let mut outgoing: u64 = sliced_outgoing[7] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[6] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[5] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[4] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[3] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[2] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[1] as u64 & 0xFF;
    outgoing <<= 8;
    outgoing |= sliced_outgoing[0] as u64 & 0xFF;

    let raw_latest_incoming_transfer_tick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4];
    let raw_latest_outgoing_transfer_tick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4];

    let mut latest_incoming_transfer_tick: u32 = raw_latest_incoming_transfer_tick[3] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= raw_latest_incoming_transfer_tick[2] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= raw_latest_incoming_transfer_tick[1] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= raw_latest_incoming_transfer_tick[0] as u32 & 0xFF;

    let mut latest_outgoing_transfer_tick: u32 = raw_latest_outgoing_transfer_tick[3] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= raw_latest_outgoing_transfer_tick[2] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= raw_latest_outgoing_transfer_tick[1] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= raw_latest_outgoing_transfer_tick[0] as u32 & 0xFF;

    let tick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4];
    let raw_spectrum_index = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4];

    let mut num_tick: u32 = tick[3] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[2] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[1] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[0] as u32 & 0xFF;

    let mut num_spectrum_index: i32 = raw_spectrum_index[3] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= raw_spectrum_index[2] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= raw_spectrum_index[1] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= raw_spectrum_index[0] as i32 & 0xFF;

    Some(ResponseEntity::new(
        sliced_identity.as_str(),
        response.peer.as_ref().unwrap().as_str(),
        incoming,
        outgoing,
        number_incoming_transfers,
        number_outgoing_transfers,
        latest_incoming_transfer_tick,
        latest_outgoing_transfer_tick,
        num_tick,
        num_spectrum_index
    ))
}