extern crate libc;

extern {
    //ECCRYPTO_STATUS SchnorrQ_FullKeyGeneration(unsigned char* SecretKey, unsigned char* PublicKey);
    //fn SchnorrQ_FullKeyGeneration(SecretKey: *mut u8, PublicKey: *mut u8) -> u8;

    //ECCRYPTO_STATUS SchnorrQ_KeyGeneration(const unsigned char* SecretKey, unsigned char* PublicKey);
    //fn SchnorrQ_KeyGeneration(SecretKey: *const u8, PublicKey: *mut u8);

    //ECCRYPTO_STATUS CompressedKeyGeneration(unsigned char* SecretKey, unsigned char* PublicKey);
    fn getPublicKey(privateKey: *const u8, publicKey: *mut u8) -> u8;
    //fn CompressedKeyGeneration(SecretKey: *const u8, PublicKey: *mut u8) -> u8;
}

fn string_to_seed_bytes(string: &str) -> Result<Vec<u8>, &'static str> {
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    let mut byte_array: Vec<u8> = Vec::with_capacity(55);
    for char in string.chars() {
        byte_array.push(
            match alphabet.chars().position(|c| c == char) {
                Some(index) => index as u8,
                None => {
                    println!("Error Converting Seed! Char ${} Not Found", char);
                    return Err("Error Converting Seed! Char Not Found");
                }
            }
        );
    }
    Ok(byte_array)
}


fn subseed(string: &str, mut index: u32) -> Result<Vec<u8>, &'static str> {
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    match string_to_seed_bytes(string) {
        Ok(mut seed_bytes) => {
            let mut preimage: Vec<u8> = seed_bytes.clone();
            while index > 0 {
                for (i, val) in seed_bytes.iter_mut().enumerate() {
                    preimage[i] = preimage[i] + 1;
                    if val > &mut (alphabet.len() as u8) { preimage[i] = 1; }
                    else { break; }
                }
                index = index - 1;
            }
            let hash1 = kangarootwelve_xkcp::hash(preimage.as_slice());
            Ok(hash1.as_bytes().to_owned().to_vec())
        },
        Err(err) => Err(err)
    }
}

fn identity(seed: &str, index: u32) -> Result<Vec<u8>, &'static str> {
    match subseed(seed, index) {
        Ok(hashed_vec) => {
            let generated_secret_key: [u8; 32] = hashed_vec.try_into().unwrap();
            let mut generated_public_key: [u8; 32] = [0; 32];
            println!("Generated Secret Key: {:?}", generated_secret_key);
            println!("Before Generating Public Key: {:?}", &generated_public_key);
            unsafe { getPublicKey(generated_secret_key.as_ptr(), generated_public_key.as_mut_ptr()) };
            println!("Generated PubKey {:?}", generated_public_key);
            Ok(generated_public_key.to_vec())
        },
        Err(err) => Err(err)
    }
}


fn main() {

    /*  Full Key Generation (Randomly Generated Secret) */
    //let mut generated_secret_key: [u8; 32] = [0; 32];
    //let mut generated_public_key: [u8; 64] = [0; 64];
    //let full_key_ret_val = unsafe { SchnorrQ_FullKeyGeneration(generated_secret_key.as_mut_ptr(), generated_public_key.as_mut_ptr() ) };
    //println!("SchnorrQ_FullKeyGeneration returned: {:?}", full_key_ret_val);
    //println!("SchnorrQ_FullKeyGeneration Secret Key: (Len={:?}) {:?}", generated_secret_key.len(), generated_secret_key);
    //println!("SchnorrQ_FullKeyGeneration Public Key: (Len={:?}) {:?}", generated_public_key.len(), generated_public_key);

    let test_seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
    let val = identity(test_seed, 0).unwrap();
    println!("Seed=({})", &test_seed);
    println!("identity returned (Len={}) - {:?}", val.len(), val);


}
