use crate::qubic_api_t;
use crate::response::FormatQubicResponseDataToStructure;

#[derive(Debug, Copy, Clone)]
pub struct ResponseEntity {
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
    pub fn new(inc: u64, out: u64, num_in_txs: u32, num_out_txs: u32, lt_in_tx: u32, lt_out_tx: u32, tick: u32, s_in: i32) -> ResponseEntity {
        ResponseEntity {
            incoming: inc,
            outgoing: out,
            final_balance: out - inc,
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
        println!("Incoming: ({}) Qus", self.incoming);
        println!("Outgoing: ({}) Qus", self. outgoing);
        println!("Final Balance: <{}> Qus", self.final_balance);
        println!("numberIncomingTransfers: {}", self.number_incoming_transactions);
        println!("numberOfOutgoingTransfers: {}", self.number_outgoing_transactions);
        println!("latestIncomingTransferTick: ({})", self.latest_incoming_transfer_tick);
        println!("latestOutgoingTransferTick: ({})", self.latest_outgoing_transfer_tick);
        println!("Latest Tick: {}", self.tick);
        println!("Spectrum Index: {}", self.spectrum_index);
        println!("-----------------------");
    }
}

impl FormatQubicResponseDataToStructure for ResponseEntity {
    fn format_qubic_response_data_to_structure(response: &mut qubic_api_t) -> Self {handle_response_entity(response).unwrap() /* check it's valid before calling! */ }
}


pub fn handle_response_entity(response: &mut qubic_api_t) -> Option<ResponseEntity> {
    let data: &Vec<u8> = &response.as_bytes();
    if data.len() < 8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4 {
        return None;
    }
    println!("Got Response Entity Peer Data: {:?}", data);

    let sliced_incoming = &data.as_slice()[8 + 32..8+32 + 8];
    let sliced_outgoing = &data.as_slice()[8 + 32 + 8..8+32 + 8 + 8];

    let numberIncomingTransfers = &data.as_slice()[8 + 32 + 8 + 8..8+32 + 8 + 8 + 4];
    let numberOfOutgoingTransfers = &data.as_slice()[8 + 32 + 8 + 8 + 4..8+32 + 8 + 8 + 4 + 4];

    let mut number_incoming_transfers: u32 = numberIncomingTransfers[3] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= numberIncomingTransfers[2] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= numberIncomingTransfers[1] as u32 & 0xFF;
    number_incoming_transfers <<= 8;
    number_incoming_transfers |= numberIncomingTransfers[0] as u32 & 0xFF;

    let mut number_outgoing_transfers: u32 = numberOfOutgoingTransfers[3] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= numberOfOutgoingTransfers[2] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= numberOfOutgoingTransfers[1] as u32 & 0xFF;
    number_outgoing_transfers <<= 8;
    number_outgoing_transfers |= numberOfOutgoingTransfers[0] as u32 & 0xFF;


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

    let latestIncomingTransferTick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4];
    let latestOutgoingTransferTick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4];

    let mut latest_incoming_transfer_tick: u32 = latestIncomingTransferTick[3] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= latestIncomingTransferTick[2] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= latestIncomingTransferTick[1] as u32 & 0xFF;
    latest_incoming_transfer_tick <<= 8;
    latest_incoming_transfer_tick |= latestIncomingTransferTick[0] as u32 & 0xFF;

    let mut latest_outgoing_transfer_tick: u32 = latestOutgoingTransferTick[3] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= latestOutgoingTransferTick[2] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= latestOutgoingTransferTick[1] as u32 & 0xFF;
    latest_outgoing_transfer_tick <<= 8;
    latest_outgoing_transfer_tick |= latestOutgoingTransferTick[0] as u32 & 0xFF;

    let tick = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4];
    let spectrumIndex = &data.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4];

    let mut num_tick: u32 = tick[3] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[2] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[1] as u32 & 0xFF;
    num_tick <<= 8;
    num_tick |= tick[0] as u32 & 0xFF;

    let mut num_spectrum_index: i32 = spectrumIndex[3] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= spectrumIndex[2] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= spectrumIndex[1] as i32 & 0xFF;
    num_spectrum_index <<= 8;

    num_spectrum_index |= spectrumIndex[0] as i32 & 0xFF;

    Some(ResponseEntity::new(
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