use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::error;
use smart_contract::qx::orderbook::AssetOrdersRequest;
use network::peers::PeerSet;
use smart_contract::qx::QxFunctions;
use store::get_db_path;

pub fn monitor_qx_orderbook(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(3000));
            /*
            *
            *   SECTION <Update QX Orderbooks>
            *
            */
            match store::sqlite::asset::asset_issuance::fetch_issued_assets_with_data(get_db_path().as_str()) {
                Ok(assets) => {
                    for asset in assets {
                        let name = asset.get(&"name".to_string()).unwrap();
                        let issuer = asset.get(&"issuer".to_string()).unwrap();
                        let asset_order_request: AssetOrdersRequest = AssetOrdersRequest::new(QxFunctions::QxGetAssetBidOrder,
                                                                                              name.as_str(),
                                                                                              issuer.as_str(),
                                                                                              0
                        );
                        let request = api::QubicApiPacket::get_asset_qx_orders(&asset_order_request);
                        {
                            match peer_set.lock().unwrap().make_request(request) {
                                Ok(_) => {},
                                Err(err) => error!("{}", err)
                            }
                        }


                        let asset_order_request2: AssetOrdersRequest = AssetOrdersRequest::new(QxFunctions::QxGetAssetAskOrder,
                                                                                               name.as_str(),
                                                                                               issuer.as_str(),
                                                                                               0
                        );
                        let request2 = api::QubicApiPacket::get_asset_qx_orders(&asset_order_request2);
                        {
                            match peer_set.lock().unwrap().make_request(request2) {
                                Ok(_) => {},
                                Err(err) => error!("{}", err)
                            }
                        }
                    }

                    let name = "RANDOM";
                    let issuer = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB";
                    let asset_order_request: AssetOrdersRequest = AssetOrdersRequest::new(QxFunctions::QxGetAssetBidOrder,
                                                                                          name,
                                                                                          issuer,
                                                                                          0
                    );
                    let request = api::QubicApiPacket::get_asset_qx_orders(&asset_order_request);
                    {
                        match peer_set.lock().unwrap().make_request(request) {
                            Ok(_) => {},
                            Err(err) => error!("{}", err)
                        }
                    }
                },
                Err(_) => {}
            }
        }
    });
}