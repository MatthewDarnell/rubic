use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use api::header::{entity_type, request_response_header};
use api::{requested_entity, qubic_api_t};


//                    RequestedEntity request;
//                     *((__m256i*)request.publicKey) = *((__m256i*)ownPublicKey);
//                     initiateRequest(REQUEST_ENTITY, &request, sizeof(request), true);

/*

static bool initiateRequest(unsigned char requestType, void* request, unsigned short requestSize, bool randomizeDejavu)
{
    if (requestingThreadIsBusy)
    {
        return false;
    }
    else
    {
        requestingThreadIsBusy = true;

        RequestResponseHeader* requestHeader = (RequestResponseHeader*)&requestResponseBuffer[0];
        requestHeader->setSize(sizeof(RequestResponseHeader) + requestSize);
        requestHeader->setProtocol();
        if (randomizeDejavu)
        {
            requestHeader->randomizeDejavu();
        }
        else
        {
            requestHeader->zeroDejavu();
        }
        requestHeader->setType(requestType);
        CopyMemory(&requestResponseBuffer[sizeof(RequestResponseHeader)], request, requestSize);

        CreateThread(NULL, 0, requestingThreadProc, (LPVOID)randomizeDejavu, 0, NULL);

        return true;
    }
}
*/

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

fn copy_slice(dst: &mut [u8], src: &[u8], start_offset: usize) -> usize {
    let mut c = 0;
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *(d) = *s;
        c += 1;
    }
    c
}

pub struct request_buffer {
    header: [u8; 8],
    data: [u8; 65535]
}

impl request_buffer {
    pub fn new(request: &qubic_api_t) -> Self {
        let size = std::mem::size_of::<request_response_header>() + request.data.len();
        let mut header_bytes: [u8; 8] = [0; 8];
        //3 byte size
        header_bytes[0] = (size & 0xFF) as u8;
        header_bytes[1] = ((size >> 8) & 0xFF) as u8;
        header_bytes[2] = ((size >> 16) & 0xFF) as u8;
        //3 byte dejavu (zeroed)
        header_bytes[3] = 0;
        header_bytes[4] = 0;
        header_bytes[5] = 0;
        //1 byte type
        header_bytes[6] = 0;
        //1 byte request entity type
        //header_bytes[7] = request.entity_t as u8;
        let mut data_bytes: [u8; 65535] = [0; 65535];
        for (index, val) in request.data.iter().enumerate() {
            data_bytes[index] = *val;
        }
        request_buffer {
            header: header_bytes,
            data: data_bytes
        }

    }
    pub fn get_bytes(&self) -> [u8; 65543] {
        let mut res: [u8; 65543] = [0; 65543];
        for (index, value) in self.header.iter().enumerate() {
            res[index] = *value;
        }
        for (index, value) in self.data.iter().enumerate() {
            res[index + 8] = *value;
        }
        res
    }
}

pub async fn initiate_request(peer: &str, request: &mut qubic_api_t) -> Result<(), Box<dyn Error>> {
    println!("Initiating Request!");
    let size = std::mem::size_of::<request_response_header>() + request.data.len();
    println!("Request Size: {}", size);
    request.header._size[0] = (size & 0xFF) as u8;
    request.header._size[1] = ((size >> 8) & 0xFF) as u8;
    request.header._size[2] = ((size >> 16) & 0xFF) as u8;
    request.header._protocol = 0;
    request.header._dejavu = [0; 3];
    request.header._type = 31;

    let buffer: request_buffer = request_buffer::new(request);
    println!("Sending Request Bytes To Peer: {}", peer);
    println!("{:?}", &buffer.get_bytes());
    // Connect to a peer
    if let Ok(std_stream) = std::net::TcpStream::connect(peer) {
        std_stream.set_nonblocking(true)?;
        let mut stream = TcpStream::from_std(std_stream)?;
        // Write some data.
       // let first_bytes: [u8; 24]= [24, 0, 0, 163, 186, 70, 116, 0, 77, 163, 95, 60, 95, 217, 141, 102, 136, 243, 36, 246, 45, 67, 139, 81];
       // stream.write_all(&first_bytes).await?;
        stream.write_all(&buffer.get_bytes()).await?;
        println!("Data Sent!");
        let mut count = 0;
        let mut msg = vec![0; 66000];

        loop {
            // Wait for the socket to be readable
            stream.readable().await?;
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(&mut msg) {
                Ok(n) => {
                    msg.truncate(n);

                    if msg.len() > 0 {
                        println!("GOT = {:?}", &msg);
                        break;

                    } else {
                        continue;
                    }
                   // return Ok(());
                    //continue;
                    //break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    continue;
                    //return Err(e.into());
                }
            }
        }
        //let s = std::str::from_utf8(&msg[5..9]).expect("invalid utf-8 sequence");
       // println!("{}", s);
        Ok(())
    } else {
        println!("Failed To Connnect To Server {}", peer);
        //Err(())
        Ok(())
    }
    //let mut std_stream = std::net::TcpStream::connect(peer)?;

}


/*
#[cfg(test)]
mod network {
    pub mod requests {
        use crate::entity::qubic_api_t;
        use crate::request;
        use identity;
        use identity::Identity;
        use crate::entity::entity_type::REQUEST_ENTITY;
        use crate::request::initiate_request;

        #[tokio::test]
        async fn send_a_request() {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            let pub_key: Vec<u8> = id.get_public_key().unwrap();

            let mut request: qubic_api_t = qubic_api_t::new(REQUEST_ENTITY, &pub_key);
            initiate_request("95.179.220.69:21841", &mut request).await;

        }
    }
}

 */