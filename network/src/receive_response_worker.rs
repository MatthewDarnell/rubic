use api::{QubicApiPacket, response};

pub fn listen_for_api_responses(rx: std::sync::mpsc::Receiver<QubicApiPacket>) {
    loop {
        match rx.recv() {
            Ok(mut r) => response::get_formatted_response(&mut r),
            Err(err) => println!("{}", err.to_string())
        }
    }
}