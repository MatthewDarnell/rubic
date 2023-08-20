use std::time::Duration;
use std::io::prelude::*;
use std::net::TcpStream;
extern crate network;
extern crate crypto;
use network::entity::qubic_request;
fn main() {
    println!("Connecting");
    let mut stream = TcpStream::connect("136.243.81.157:21841").expect("Failed To Connect");
    stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
    loop {
        let request = qubic_request::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON").as_bytes();
        println!("{:?}", &request);
        println!("Writing");
        stream.write(request.as_slice()).unwrap();
        println!("Wrote");
        let mut readable: [u8; 30000] = [0; 30000];
        println!("Reading");
        match stream.read(&mut readable) {
            Ok(bytes_read) => {
                println!("bytes read: {} GOT {:?}", bytes_read, &readable);
            },
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}