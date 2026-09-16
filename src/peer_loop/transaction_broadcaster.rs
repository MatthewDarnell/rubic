use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use smart_contract::qx::asset_transfer::AssetTransferTransaction;
use protocol::transfer::TransferTransaction;
use crypto::qubic_identities::get_public_key_from_identity;
use logger::{error, info};
use network::peers::PeerSet;
use smart_contract::qx::order::QxOrderTransaction;
use store::{get_db_path, sqlite};
use store::sqlite::{tick, transfer};
use store::sqlite::transfer::set_transfer_as_broadcast;

/// A pending transaction is re-sent to every peer whenever the known tick has
/// advanced by at least this much since its previous send, until the current
/// tick reaches its expiration tick or it confirms. With the tick poll hitting
/// every peer, the known tick moves roughly once a second, so this is ~1 resend
/// per tick per pending transaction.
const REBROADCAST_EVERY_TICKS: u32 = 1;
/// RANDOM session steps are resent this often (to two peers) after their first send.
const STEP_REBROADCAST_EVERY_TICKS: u32 = 3;

pub fn broadcast_transactions(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            /*
            *
            *   SECTION <Look For Pending Transfers To Broadcast Or Re-Broadcast>
            *
            */
            let latest_tick: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
                Ok(t) => t.parse::<u32>().unwrap_or(0),
                Err(_) => 0,
            };

            match transfer::fetch_transfers_to_broadcast(get_db_path().as_str(), latest_tick, REBROADCAST_EVERY_TICKS) {
                Ok(transfers_to_broadcast) => {
                    for transfer_map in transfers_to_broadcast {
                        let first_send = transfer_map.get("broadcast").map(|b| b == "0" || b == "false").unwrap_or(true);
                        let source_id = transfer_map.get("source").unwrap();
                        let dest_id = transfer_map.get("destination").unwrap();
                        // RANDOM session steps are many and already sit in every peer's
                        // per-tick pool after the first send: resend them only every few
                        // ticks, and to a couple of peers, so they never crowd out the
                        // tick polls the whole session depends on.
                        let session_step = dest_id == miner::random::RANDOM_CONTRACT_IDENTITY;
                        if session_step && !first_send {
                            let last: u32 = transfer_map.get("last_broadcast_tick").and_then(|t| t.parse().ok()).unwrap_or(0);
                            if last + STEP_REBROADCAST_EVERY_TICKS > latest_tick {
                                continue;
                            }
                        }

                        let amount = transfer_map.get("amount").unwrap();
                        let tick = transfer_map.get("tick").unwrap();

                        let amt: u64 = amount.parse().unwrap();
                        let tck: u32 = tick.parse().unwrap();

                        let source_pub_key = get_public_key_from_identity(source_id).unwrap();
                        let des_pub_key = get_public_key_from_identity(dest_id).unwrap();

                        let signature = transfer_map.get("signature").unwrap();
                        let sig_arr = hex::decode(signature).unwrap();
                        let txid = transfer_map.get("txid").unwrap();

                        let mut tx = TransferTransaction::from_signed_data(
                            &source_pub_key,
                            &des_pub_key,
                            amt,
                            tck,
                            0,
                            0,
                            sig_arr.as_slice()
                        );
                        let _broadcast: Option<api::QubicApiPacket>;
                        match sqlite::asset::asset_transfer::fetch_transfer_by_txid(get_db_path().as_str(), txid.as_str()) {
                            Ok(_asset_tx) => {
                                if _asset_tx.is_some() {
                                    let asset_tx = _asset_tx.unwrap();
                                    //This is an Asset Transfer
                                    let issuer = asset_tx.get("issuer").unwrap();
                                    let name = asset_tx.get("name").unwrap();
                                    let _num_shares = asset_tx.get("num_shares").unwrap();
                                    let input_size = asset_tx.get("input_size").unwrap();
                                    let input_type = asset_tx.get("input_type").unwrap();
                                    let new_owner_and_possessor = asset_tx.get("new_owner_and_possessor").unwrap();
                                    
                                    tx._input_size = u16::from_str(input_size).unwrap();
                                    tx._input_type = u16::from_str(input_type).unwrap();
                                    
                                    let atx: AssetTransferTransaction = AssetTransferTransaction::from_signed_data(tx, issuer, new_owner_and_possessor, name.as_str(), i64::from_str(_num_shares).unwrap(), sig_arr.as_slice());
                                    _broadcast = Some(api::QubicApiPacket::broadcast_transaction(atx));
                                } else {
                                    match sqlite::qx::order::fetch_qx_order_by_txid(get_db_path().as_str(), txid.as_str()) {
                                        Ok(_order_tx) => {
                                            if _order_tx.is_some() {
                                                let order_tx = _order_tx.unwrap();
                                                //This is an Asset Transfer
                                                let issuer = order_tx.get("issuer").unwrap();
                                                let name = order_tx.get("name").unwrap();
                                                let _num_shares = order_tx.get("num_shares").unwrap();
                                                let input_size = order_tx.get("input_size").unwrap();
                                                let input_type = order_tx.get("input_type").unwrap();
                                                let price = order_tx.get("price").unwrap();

                                                tx._input_size = u16::from_str(input_size).unwrap();
                                                tx._input_type = u16::from_str(input_type).unwrap();

                                                let otx: QxOrderTransaction = QxOrderTransaction::from_signed_data(tx, issuer, name.as_str(), u64::from_str(price).unwrap(), u64::from_str(_num_shares).unwrap(), sig_arr.as_slice());
                                                //println!("{:?}", &otx);
                                                //println!("Re-Constructed Tx: {}", otx.txid());
                                                _broadcast = Some(api::QubicApiPacket::broadcast_transaction(otx));
                                            } else {
                                                // A RANDOM session step carries the contract input.
                                                _broadcast = match crate::miner::rebuild_step_transaction(txid.as_str()) {
                                                    Some(round_tx) => Some(api::QubicApiPacket::broadcast_custom_transaction(&round_tx)),
                                                    None => Some(api::QubicApiPacket::broadcast_transaction(tx)),
                                                };
                                            }
                                        },
                                        Err(_) => {
                                            _broadcast = None;
                                        }
                                    }
                                }
                            },
                            Err(_) => {
                                _broadcast = None;
                            }
                        };
                        {
                            if let Some(broadcast) = _broadcast {
                                let sent = {
                                    let mut lock = peer_set.lock().unwrap();
                                    if first_send { lock.make_request_urgent(broadcast) }
                                    else if session_step { lock.make_request_high_priority(broadcast) }
                                    else { lock.make_request(broadcast) }
                                };
                                match sent {
                                    Ok(_) => {
                                        match set_transfer_as_broadcast(get_db_path().as_str(), txid.as_str(), latest_tick) {
                                            Ok(_) => {
                                                if first_send {
                                                    //println!("Transaction {} Broadcast (tick {})", txid, latest_tick);
                                                    info!("Transaction {} Broadcast (tick {})", txid, latest_tick);
                                                } else {
                                                    info!("Transaction {} Re-Broadcast (tick {}, expires {})", txid, latest_tick, tck);
                                                }
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
                    }
                },
                Err(_) => {}
            }

        }
    });
}