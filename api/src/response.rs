use crate::request_response_header;
use crate::header::entity_type;

pub fn handle_response(response: &Vec<u8>) {

    let header = request_response_header::from_vec(response);
    let size = std::mem::size_of::<request_response_header>();
    if header.get_type().is_none() {
        println!("unknown type");
        return ;
    }
    let resp_type = header.get_type().unwrap();
    let data_size = header.get_size() ;
    if data_size == 0 {
        return;
    }

    println!("Full Response: {:?}", response);

    //println!("Num Bytes Of Response: {}", response.len());

    //println!("Data Size: {}", data_size);

    let header_slice = &response.as_slice()[..8];
    let address_slice = &response.as_slice()[8..8+32];
   // let data_slice = &response.as_slice()[8..data_size];
    match resp_type {
        entity_type::EXCHANGE_PEERS => {
           // println!("exchange peers");
            return;
        },
        entity_type::REQUEST_ENTITY => {
            println!("request entity");

        },
        entity_type::RESPONSE_ENTITY => {
           // println!("response entity");
//println!("address slice: {:?}", address_slice);

//let addr = match std::str::from_utf8(address_slice) {
  //  Ok(v) => v,
 //   Err(err) => panic!("{}", err.to_string())
//}            ;
           // println!("Address Decoded: {}", addr);

//    unsigned int tick;
//     int spectrumIndex;
//     unsigned char siblings[SPECTRUM_DEPTH][32];

            let sliced_incoming = &response.as_slice()[8 + 32..8+32 + 8];
            let sliced_outgoing = &response.as_slice()[8 + 32 + 8..8+32 + 8 + 8];
            
            
            let numberIncomingTransfers = &response.as_slice()[8 + 32 + 8 + 8..8+32 + 8 + 8 + 4];
            let numberOfOutgoingTransfers = &response.as_slice()[8 + 32 + 8 + 8 + 4..8+32 + 8 + 8 + 4 + 4];

//Incoming: [65, 152, 31, 205, 0, 0, 0, 0] => (4726562774254092288)
// Outgoing: [145, 52, 143, 102, 0, 0, 0, 0] => (10463145502537940992)
// Final Balance: <5736582728283848704>

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

            
            
            let latestIncomingTransferTick = &response.as_slice()[8 + 32 + 8 + 8 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4];
            let latestOutgoingTransferTick = &response.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4];

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
            
            
            let tick = &response.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4];
            let spectrumIndex = &response.as_slice()[8 + 32 + 8 + 8 + 4 + 4 + 4 + 4 + 4..8+32 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4];


            println!("Incoming: {:?} => ({})", sliced_incoming, incoming);
            println!("Outgoing: {:?} => ({})", sliced_outgoing, outgoing);
            println!("Final Balance: <{}>", incoming-outgoing);
            println!("numberIncomingTransfers: {:?}", numberIncomingTransfers);
            println!("numberOfOutgoingTransfers: {:?}", numberOfOutgoingTransfers);
            println!("latestIncomingTransferTick: {:?} => ({})", latestIncomingTransferTick, latest_incoming_transfer_tick);
            println!("latestOutgoingTransferTick: {:?} => ({})", latestOutgoingTransferTick, latest_outgoing_transfer_tick);

            //tick: [90, 239, 121, 0] => (4017684480)
            // spectrumIndex: [255, 255, 255, 255] = (-256)

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


           // println!("result: {}", s);

        }
        _ => {
            println!("unknown type");
        }
    }

    println!("Handling Response\nReading Response: {:?}", header);
    println!("Num Bytes Of Response: {}", response.len());

    println!("Response Type: {:?}", resp_type);
    println!("Data Size: {}", data_size);

    println!("Header: {:?}", header_slice);
   // println!("Data: {:?}", data_slice);
}