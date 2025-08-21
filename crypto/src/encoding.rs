use hex;
use hex::FromHexError;

pub fn bytes_to_hex(bytes: &Vec<u8>) -> String { hex::encode(bytes) }
pub fn to_hex(string: &str) -> String { hex::encode(string) }
pub fn from_hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, FromHexError> { hex::decode(hex_string) }
