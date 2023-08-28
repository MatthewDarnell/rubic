


pub fn handle_response_entity(data: &Vec<u8>) {
    println!("Got Exchange Peer Data: {:?}", data);

    let sliced_incoming = &data.as_slice()[8 + 32..8+32 + 8];
    let sliced_outgoing = &data.as_slice()[8 + 32 + 8..8+32 + 8 + 8];

    let numberIncomingTransfers = &data.as_slice()[8 + 32 + 8 + 8..8+32 + 8 + 8 + 4];
    let numberOfOutgoingTransfers = &data.as_slice()[8 + 32 + 8 + 8 + 4..8+32 + 8 + 8 + 4 + 4];

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

    println!("Incoming: {:?} => ({})", sliced_incoming, incoming);
    println!("Outgoing: {:?} => ({})", sliced_outgoing, outgoing);
    println!("Final Balance: <{}>", incoming-outgoing);
    println!("numberIncomingTransfers: {:?}", numberIncomingTransfers);
    println!("numberOfOutgoingTransfers: {:?}", numberOfOutgoingTransfers);
    println!("latestIncomingTransferTick: {:?} => ({})", latestIncomingTransferTick, latest_incoming_transfer_tick);
    println!("latestOutgoingTransferTick: {:?} => ({})", latestOutgoingTransferTick, latest_outgoing_transfer_tick);

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

    println!("tick: {:?} => ({})", tick, num_tick);
    println!("spectrumIndex: {:?} = ({})", spectrumIndex, num_spectrum_index);
}