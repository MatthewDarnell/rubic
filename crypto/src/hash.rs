use hex::encode;
use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};

pub fn k12(input: &str) -> String {
    let ret_val = k12_bytes(&input.as_bytes().to_vec());
    encode(ret_val)
}

pub fn k12_bytes(input: &Vec<u8>) -> Vec<u8> {
    let mut digest = [0; 32];
    let mut kangaroo = KangarooTwelve::new(b"");
    kangaroo.update(input.as_slice());
    kangaroo.finalize(&mut digest);
    Vec::from(digest)
}

pub fn k12_64(input: &Vec<u8>) -> Vec<u8> {
    let mut output = [0u8; 64];
    let mut hasher = KangarooTwelve::new(b"");
    hasher.update(input);
    let mut xof = hasher.into_xof();
    xof.squeeze(&mut output[..32]);
    xof.squeeze(&mut output[32..]);
    output.to_vec()
}

#[cfg(test)]
pub mod kangaroo12_tests {
    use crate::hash::{k12, k12_64};
    #[test]
    fn hash_a_value() {
        let value = k12("inputText");
        assert_eq!(value, "2459b095c4d5b1759a14f5e4924f26a813c020979fab5ef2cad7321af37808d3".to_string())
    }

    #[test]
    fn hash_64_length() {
        let input: [u8; 4] = [0x01, 0x01, 0x01, 0x01];
        let hashed = k12_64(&input.to_vec());
        let expected: [u8; 64] = [
            100, 235, 75, 154, 91, 247, 195, 9, 136,
            147, 220, 63, 23, 226, 96, 132, 155, 107,
            59, 67, 118, 117, 162, 17, 227, 251, 205,
            254, 76, 238, 111, 21, 192, 78, 194, 235,
            42, 157, 3, 130, 70, 32, 213, 124, 202,
            89, 29, 227, 15, 207, 172, 130, 201, 118,
            62, 69, 247, 170, 185, 2, 1, 148, 177, 160];
        assert_eq!(hashed.as_slice(), &expected)
    }
}

