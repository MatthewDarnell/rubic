use std::collections::BTreeSet;
use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use smart_contract::qx::orderbook::{AssetOrder, OrderBook};

impl FormatQubicResponseDataToStructure for OrderBook {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_asset_order_response(response) }
}

pub fn handle_asset_order_response(response: &mut QubicApiPacket) -> Option<OrderBook> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + (std::mem::size_of::<AssetOrder>() * 256)) {
        println!("Wrong Size! {}, {:?}", data_len, &response.data[0..8]);
        return None;
    }
    let mut ret_val: OrderBook = OrderBook {
        full_order_list: Vec::new(),
        cached_order_set: BTreeSet::new()
    };
    ret_val.full_order_list = response.data
        .chunks_exact(size_of::<AssetOrder>())
        .map(|x| AssetOrder::from_bytes(x).unwrap())
        .filter(|order| order.price > 0)
        .collect();
    
    for order in &ret_val.full_order_list { 
        if !ret_val.cached_order_set.contains(&order) {
            ret_val.cached_order_set.insert(order.clone());
            continue;
        }
        let existing_sum: i64 = ret_val.cached_order_set.get(&order)?.num_shares;
        ret_val.cached_order_set.remove(&order);
        let mut new_order: AssetOrder = order.clone();
        new_order.num_shares = existing_sum + new_order.num_shares;
        ret_val.cached_order_set.insert(new_order);
    }
    Some(ret_val)
}
