use crypto::hash::k12_bytes;

#[derive(Debug, Clone)]
pub struct Tick {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub millisecond: u16,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,

    pub prev_resource_testing_digest: u32,
    pub salted_resource_testing_digest: u32,

    pub prev_transaction_body_digest: u32,
    pub salted_transaction_body_digest: u32,

    pub prev_spectrum_digest: [u8; 32],
    pub prev_universe_digest: [u8; 32],
    pub prev_computer_digest: [u8; 32],
    pub salted_spectrum_digest: [u8; 32],
    pub salted_universe_digest: [u8; 32],
    pub salted_computer_digest: [u8; 32],


    pub transaction_digest: [u8; 32],
    pub expected_next_tick_transaction_digest: [u8; 32],
    
    pub signature: [u8; 64]
}

impl Tick {
    pub fn print(&self) {
        println!("-----\n\tcomputor.({})\n\tepoch.({})\n\ttick.({})\n\tms.({})\n\tsec.({})\n\tmin.({})\n\thour.({})\n\tday.({})\n\tmonth.({})\n\tyear.({})\n",
                 self.computor_index,
                 self.epoch,
                 self.tick,
            self.millisecond,
            self.second,
            self.minute,
            self.hour,
            self.day,
            self.month,
            self.year
             );
        
        
        println!("\nExpected Next Tick Transaction Digest: {:?}\n\t\t\t\t\t\t\tSignature: {:?}\n\n", self.expected_next_tick_transaction_digest, self.signature);
        
    }
    
    pub fn as_bytes(&self) -> [u8; size_of::<Tick>()] {
        let mut bytes = [0u8; size_of::<Tick>()];
        bytes[0..2].copy_from_slice(&self.computor_index.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.tick.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.millisecond.to_le_bytes());
        bytes[10..11].copy_from_slice(&self.second.to_le_bytes());
        bytes[11..12].copy_from_slice(&self.minute.to_le_bytes());
        bytes[12..13].copy_from_slice(&self.hour.to_le_bytes());
        bytes[13..14].copy_from_slice(&self.day.to_le_bytes());
        bytes[14..15].copy_from_slice(&self.month.to_le_bytes());
        bytes[15..16].copy_from_slice(&self.year.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.prev_resource_testing_digest.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.salted_resource_testing_digest.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.prev_transaction_body_digest.to_le_bytes());
        bytes[28..32].copy_from_slice(&self.salted_transaction_body_digest.to_le_bytes());
        
        bytes[32..64].copy_from_slice(&self.prev_spectrum_digest);
        bytes[64..96].copy_from_slice(&self.prev_universe_digest);
        bytes[96..128].copy_from_slice(&self.prev_computer_digest);
        bytes[128..160].copy_from_slice(&self.salted_spectrum_digest);
        bytes[160..192].copy_from_slice(&self.salted_universe_digest);
        bytes[192..224].copy_from_slice(&self.salted_computer_digest);
        bytes[224..256].copy_from_slice(&self.transaction_digest);
        bytes[256..288].copy_from_slice(&self.expected_next_tick_transaction_digest);
        bytes[288..352].copy_from_slice(&self.signature);
        bytes
    }

    pub fn as_bytes_without_signature(&self) -> [u8; size_of::<Tick>() - 64] {
        let mut bytes = [0u8; size_of::<Tick>() - 64];
        bytes.copy_from_slice(&self.as_bytes()[..size_of::<Tick>()-64]);
        bytes
    }
    
    pub fn hash(&self) -> [u8; 32] {
        let digest: [u8; 32] = <[u8; 32]>::try_from(k12_bytes(&self.as_bytes_without_signature().to_vec())).unwrap();
        digest
    }
    
    pub fn new(data: &Vec<u8>) -> Tick {
        let mut arrs: Vec<[u8; 32]> = data[32..data.len()-64].chunks_exact(32).map(|chunk| <[u8; 32]>::try_from(chunk).unwrap()).collect();
        
        let mut sig: [u8; 64] = [0u8; 64];
        sig.copy_from_slice(&data[data.len()-64..]);
        Tick {
            computor_index: u16::from_le_bytes([data[0], data[1]]),
            epoch: u16::from_le_bytes([data[2], data[3]]),
            tick: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),

            millisecond: u16::from_le_bytes([data[8], data[9]]),
            second: u8::from_le_bytes([data[10]]),
            minute: u8::from_le_bytes([data[11]]),
            hour: u8::from_le_bytes([data[12]]),
            day: u8::from_le_bytes([data[13]]),
            month: u8::from_le_bytes([data[14]]),
            year: u8::from_le_bytes([data[15]]),

            prev_resource_testing_digest: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            salted_resource_testing_digest: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),

            prev_transaction_body_digest: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            salted_transaction_body_digest: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),


            prev_spectrum_digest: arrs[0],
            prev_universe_digest: arrs[1],
            prev_computer_digest: arrs[2],
            salted_spectrum_digest: arrs[3],
            salted_universe_digest: arrs[4],
            salted_computer_digest: arrs[5],

            transaction_digest: arrs[6],
            expected_next_tick_transaction_digest: arrs[7],
            signature: sig
        }
    }
}


#[test]
fn create_tick_verify_bytes() {
    let data = vec![
            179, 1, 162, 0, 78, 218, 145, 1, 0, 0, 55, 43, 1, 27, 5, 25, 182, 148, 180, 150, 
            76, 154, 174, 13, 16, 201, 116, 192, 182, 141, 129, 61, 46, 136, 177, 190, 7, 255, 
            107, 15, 233, 183, 56, 140, 9, 242, 110, 133, 173, 26, 125, 241, 64, 181, 236, 55, 
            53, 192, 108, 141, 20, 198, 42, 104, 198, 1, 132, 246, 45, 82, 35, 205, 214, 62, 35, 
            56, 1, 234, 106, 113, 28, 219, 120, 220, 33, 228, 17, 16, 13, 104, 51, 204, 84, 140, 
            159, 233, 35, 143, 84, 38, 162, 102, 229, 91, 82, 50, 161, 198, 72, 143, 183, 99, 
            12, 139, 19, 77, 144, 157, 103, 28, 151, 234, 24, 108, 5, 149, 224, 248, 209, 84, 154,
            135, 37, 24, 156, 151, 121, 239, 217, 233, 227, 236, 3, 55, 134, 252, 6, 161, 188, 127,
            161, 203, 249, 4, 216, 69, 216, 224, 239, 36, 200, 119, 196, 123, 107, 129, 17, 33, 
            78, 46, 196, 109, 51, 94, 138, 198, 108, 18, 210, 47, 153, 184, 172, 250, 202, 217, 176,
            44, 37, 250, 70, 81, 20, 58, 197, 174, 255, 27, 237, 23, 224, 145, 6, 58, 88, 180, 142, 
            140, 143, 106, 151, 252, 237, 56, 218, 227, 85, 232, 191, 249, 119, 219, 106, 139, 178, 
            76, 66, 49, 196, 24, 48, 86, 28, 24, 146, 197, 167, 85, 86, 70, 144, 53, 83, 54, 122, 
            198, 43, 11, 82, 113, 203, 18, 95, 6, 89, 139, 90, 24, 65, 77, 211, 194, 17, 217, 170, 
            221, 6, 224, 141, 205, 105, 41, 49, 91, 247, 209, 181, 212, 239, 220, 177, 78, 162, 18, 
            153, 203, 141, 98, 0, 2, 182, 209, 242, 147, 212, 202, 202, 49, 88, 208, 111, 41, 217, 
            14, 33, 231, 49, 128, 142, 189, 210, 240, 97, 68, 72, 91, 252, 169, 211, 223, 227, 213, 
            39, 253, 104, 79, 159, 15, 221, 136, 206, 123, 250, 21, 74, 165, 238, 222, 77, 243, 203, 
            118, 96, 203, 34, 9, 207, 253, 230, 127, 25, 0
    ];
    let t: Tick = Tick::new(&data);
    assert_eq!(t.computor_index, 435);
    let bytes = t.as_bytes();
    let matching = data.iter().zip(&bytes).filter(|&(a, b)| a == b).count();
    assert_eq!(matching, std::mem::size_of::<Tick>());
    
    
    let bytes = t.as_bytes_without_signature();
    assert_eq!(bytes, [179, 1, 162, 0, 78, 218, 145, 1, 0, 0, 55, 43, 1, 27, 5, 25, 182, 148, 180, 150, 
        76, 154, 174, 13, 16, 201, 116, 192, 182, 141, 129, 61, 46, 136, 177, 190, 7, 255, 107, 15, 233,
        183, 56, 140, 9, 242, 110, 133, 173, 26, 125, 241, 64, 181, 236, 55, 53, 192, 108, 141, 20, 198,
        42, 104, 198, 1, 132, 246, 45, 82, 35, 205, 214, 62, 35, 56, 1, 234, 106, 113, 28, 219, 120, 220,
        33, 228, 17, 16, 13, 104, 51, 204, 84, 140, 159, 233, 35, 143, 84, 38, 162, 102, 229, 91, 82, 50, 
        161, 198, 72, 143, 183, 99, 12, 139, 19, 77, 144, 157, 103, 28, 151, 234, 24, 108, 5, 149, 224,
        248, 209, 84, 154, 135, 37, 24, 156, 151, 121, 239, 217, 233, 227, 236, 3, 55, 134, 252, 6, 161, 
        188, 127, 161, 203, 249, 4, 216, 69, 216, 224, 239, 36, 200, 119, 196, 123, 107, 129, 17, 33, 78,
        46, 196, 109, 51, 94, 138, 198, 108, 18, 210, 47, 153, 184, 172, 250, 202, 217, 176, 44, 37, 250,
        70, 81, 20, 58, 197, 174, 255, 27, 237, 23, 224, 145, 6, 58, 88, 180, 142, 140, 143, 106, 151, 
        252, 237, 56, 218, 227, 85, 232, 191, 249, 119, 219, 106, 139, 178, 76, 66, 49, 196, 24, 48, 86,
        28, 24, 146, 197, 167, 85, 86, 70, 144, 53, 83, 54, 122, 198, 43, 11, 82, 113, 203, 18, 95, 6, 89,
        139, 90, 24, 65, 77, 211, 194, 17, 217, 170, 221, 6, 224, 141, 205, 105, 41, 49, 91, 247, 209, 181,
        212, 239, 220, 177, 78, 162, 18, 153, 203, 141, 0]
    );
    let hash = t.hash();
    assert_eq!(hash, [177, 188, 80, 212, 47, 217, 143, 147, 61, 96, 25, 
                        1, 253, 230, 166, 10, 77, 175, 155,
                        75, 245, 236, 146, 42, 193, 72, 148, 143, 189, 163, 182, 161]
    );
    
    let hash2 = k12_bytes(&bytes.to_vec());
    assert_eq!(hash.to_vec(), hash2);
    
    
}