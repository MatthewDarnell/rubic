use std::sync::{Arc, Mutex};
use std::time::Duration;
use api::transfer::TransferTransaction;
use crypto::qubic_identities::get_public_key_from_identity;
use logger::{error, info};
use network::peers::PeerSet;
use store::get_db_path;
use store::sqlite::transfer;
use store::sqlite::transfer::set_transfer_as_broadcast;

pub fn broadcast_transactions(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            /*
            *
            *   SECTION <Look For Pending Transfers To Broadcast>
            *
            */

            match transfer::fetch_transfers_to_broadcast(get_db_path().as_str()) {
                Ok(transfers_to_broadcast) => {
                    for transfer_map in transfers_to_broadcast {
                        let source_id = transfer_map.get("source").unwrap();
                        let dest_id = transfer_map.get("destination").unwrap();

                        let amount = transfer_map.get("amount").unwrap();
                        let tick = transfer_map.get("tick").unwrap();

                        let amt: u64 = amount.parse().unwrap();
                        let tck: u32 = tick.parse().unwrap();

                        let source_pub_key = get_public_key_from_identity(source_id).unwrap();
                        let des_pub_key = get_public_key_from_identity(dest_id).unwrap();

                        let signature = transfer_map.get("signature").unwrap();
                        let sig_arr = hex::decode(signature).unwrap();
                        let txid = transfer_map.get("txid").unwrap();

                        let tx = TransferTransaction::from_signed_data(
                            &source_pub_key,
                            &des_pub_key,
                            amt,
                            tck,
                            0,
                            0,
                            sig_arr.as_slice()
                        );

                        let broadcast = api::QubicApiPacket::broadcast_transaction(&tx);
                        {
                            match peer_set.lock().unwrap().make_request(broadcast) {
                                Ok(_) => {
                                    match set_transfer_as_broadcast(get_db_path().as_str(), txid.as_str()) {
                                        Ok(_) => {
                                            //println!("Transaction {} Broadcast", txid);
                                            info!("Transaction {} Broadcast", txid);
                                        },
                                        Err(err) => {
                                            error!("Failed To Set Transaction <{}> as Broadcast! ({})", txid, err);
                                        }
                                    }
                                },
                                Err(err) => error!("{}", err)
                            }
                        }
                    }
                },
                Err(_) => {}
            }

        }
    });
}