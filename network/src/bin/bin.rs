use std::time::Duration;
use std::io::prelude::*;
use std::net::TcpStream;
extern crate crypto;
use crypto::random::random_bytes;
fn main() {
    //    unsigned char _size[3];
    //     unsigned char _protocol;
    //     unsigned char _dejavu[3];
    //     unsigned char _type;
    //                                           vvvv type 31 = get info
    let mut request: [u8; 40] = [40, 0, 0, 0, 0, 0, 0, 31, 170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26];
    println!("Connecting");



    let mut stream = TcpStream::connect("136.243.81.157:21841").expect("Failed To Connect");

    stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
    loop {
        let r = random_bytes(3);
        println!("{:?}", r);
        request[4] = r[0];
        request[5] = r[1];
        request[6] = r[2];
        println!("{:?}", &request);


        println!("Writing");
        stream.write(&request).unwrap();
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
