extern crate libc;

use std::convert::TryInto;

extern {
    //ECCRYPTO_STATUS SchnorrQ_FullKeyGeneration(unsigned char* SecretKey, unsigned char* PublicKey);
    //fn SchnorrQ_FullKeyGeneration(SecretKey: *mut u8, PublicKey: *mut u8) -> u8;

    //ECCRYPTO_STATUS SchnorrQ_KeyGeneration(const unsigned char* SecretKey, unsigned char* PublicKey);
    //fn SchnorrQ_KeyGeneration(SecretKey: *const u8, PublicKey: *mut u8);

    //ECCRYPTO_STATUS CompressedKeyGeneration(unsigned char* SecretKey, unsigned char* PublicKey);

    //bool getSubseed(const unsigned char* seed, unsigned char* subseed)
    //void getPrivateKey(unsigned char* subseed, unsigned char* privateKey)
    fn getSubseed(seed: *const u8, subseed: *mut u8) -> bool;
    fn getPrivateKey(subseed: *const u8, privateKey: *mut u8);

    fn getPublicKey(privateKey: *const u8, publicKey: *mut u8);

    //void getIdentity(unsigned char* publicKey, char* identity, bool isLowerCase)
    fn getIdentity(publicKey: *const u8, identity: *const u8, isLowerCase: bool);

    //fn CompressedKeyGeneration(SecretKey: *const u8, PublicKey: *mut u8) -> u8;

    //void KangarooTwelve(unsigned char* input, unsigned int inputByteLen, unsigned char* output, unsigned int outputByteLen)
    fn KangarooTwelve(input: *const u8, inputByteLen: u32, output: *mut u8, outputByteLen: u32);
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
            let mut hash1: [u8; 32] = [0; 32];
            unsafe {
                KangarooTwelve(preimage.as_slice().as_ptr(), preimage.len() as u32, hash1.as_mut_ptr(), 32);
            }
            Ok(hash1.to_owned().to_vec())
        },
        Err(err) => Err(err)
    }
}

fn identity(seed: &str, index: u32) -> Result<Vec<u8>, &'static str> {
    match subseed(seed, index) {
        Ok(hashed_vec) => {
            let generated_secret_key: [u8; 32] = hashed_vec.try_into().unwrap();
            let mut generated_public_key: [u8; 32] = [0; 32];
            let mut generated_identity: [u8; 60] = [0; 60];
            println!("Generated Secret Key: {:?}", generated_secret_key);
            println!("Before Generating Public Key: {:?}", &generated_public_key);
            unsafe { getPublicKey(generated_secret_key.as_ptr(), generated_public_key.as_mut_ptr()) };
            unsafe {
                getIdentity(generated_public_key.as_ptr(), generated_identity.as_mut_ptr(), false);
            }
            println!("Generated PubKey {:?}", generated_public_key);

            Ok(generated_identity.to_vec())
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

/*
    let val = identity(test_seed, 0).unwrap();
    let id = std::str::from_utf8(val.as_slice()).unwrap();
    println!("Seed=({})", &test_seed);
    println!("identity returned (Len={}) - {:?}", val.len(), id);
*/

    let test_seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";

    let mut ownSubseed: [u8; 32] = [0; 32];
    let mut privateKey: [u8; 32] = [0; 32];
    let mut publicKey: [u8; 32] = [0; 32];
    let mut identity: [u8; 60] = [0; 60];

    let seed_bytes: Vec<u8> = test_seed.try_into().unwrap();
    println!("Seed {:?}", test_seed);
    println!("Seed Bytes: {:?}", &seed_bytes);
    unsafe {
        getSubseed(seed_bytes.as_slice().as_ptr(), ownSubseed.as_mut_ptr());
        getPrivateKey(ownSubseed.as_ptr(), privateKey.as_mut_ptr());
        getPublicKey(privateKey.as_ptr(), publicKey.as_mut_ptr());
        getIdentity(publicKey.as_ptr(), identity.as_mut_ptr(), false);
    }
    let id = std::str::from_utf8(identity.as_slice()).unwrap();
    println!("identity returned (Len={}) - {:?}", id.len(), id);

}
