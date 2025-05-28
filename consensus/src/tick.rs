#[derive(Debug, Clone)]
pub struct Tick {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub millisecond: u8,
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


    signature: [u8; 64]
}

impl Tick {
    pub fn print(&self) {
        println!("-----\nTick:\n\ttick.({})\n\tcomputor.({})\n\tepoch.({})\n\t\n\tyear.({})\n\tsig: {:?}...\n-----",
                 self.tick,
                 self.computor_index,
                 self.epoch,
                 self.year,
                 &self.signature[..8]);
    }
    pub fn new(data: &Vec<u8>) -> Tick {
        Tick {
            computor_index: u16::from_le_bytes([data[0], data[1]]),
            epoch: u16::from_le_bytes([data[2], data[3]]),
            tick: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),

            millisecond: u8::from_le_bytes([data[8]]),
            second: u8::from_le_bytes([data[9]]),
            minute: u8::from_le_bytes([data[10]]),
            hour: u8::from_le_bytes([data[11]]),
            day: u8::from_le_bytes([data[12]]),
            month: u8::from_le_bytes([data[13]]),
            year: u8::from_le_bytes([data[14]]),

            prev_resource_testing_digest: u32::from_le_bytes([data[15], data[16], data[17], data[18]]),
            salted_resource_testing_digest: u32::from_le_bytes([data[19], data[20], data[21], data[22]]),

            prev_transaction_body_digest: u32::from_le_bytes([data[23], data[24], data[25], data[26]]),
            salted_transaction_body_digest: u32::from_le_bytes([data[27], data[28], data[29], data[30]]),


            prev_spectrum_digest: data[31..63].try_into().unwrap(),
            prev_universe_digest: data[63..95].try_into().unwrap(),
            prev_computer_digest: data[95..127].try_into().unwrap(),
            salted_spectrum_digest: data[127..159].try_into().unwrap(),
            salted_universe_digest: data[159..191].try_into().unwrap(),
            salted_computer_digest: data[191..223].try_into().unwrap(),

            transaction_digest: data[223..255].try_into().unwrap(),
            expected_next_tick_transaction_digest: data[255..287].try_into().unwrap(),

            signature: data[287..351].try_into().unwrap()
        }
    }
}
