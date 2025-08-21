use argon2::Error::SaltTooLong;
use base64::{Engine as _, engine::general_purpose};
use crate::hash;
use sha2::Sha256;
use pbkdf2::pbkdf2_hmac;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Nonce
};
use logger::error;
use crate::random::random_bytes;

pub const SALTLENGTH: usize = 32;
pub const NONCELENGTH: usize = 12;

pub fn encrypt(plaintext: &str, password: &str) -> Option<([u8; SALTLENGTH + NONCELENGTH], Vec<u8>)> {
    let salt: [u8; SALTLENGTH] = random_bytes(SALTLENGTH as u32).as_slice().try_into().unwrap();
    let nonce_slice: [u8; NONCELENGTH] = random_bytes(NONCELENGTH as u32).as_slice().try_into().unwrap();

    //pbkdf2 Key Derivation From Salt
    // number of iterations
    let n = 600_000;
    let mut key: [u8; SALTLENGTH] = [0u8; SALTLENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, n, &mut key);

    let cipher = ChaCha20Poly1305::new_from_slice(&key).unwrap();
    let nonce: Nonce = Nonce::from(nonce_slice.clone());
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).unwrap();
    
    let s: [u8; SALTLENGTH] = <[u8; SALTLENGTH]>::try_from(salt.as_ref()).unwrap();
    let n: [u8; NONCELENGTH] = <[u8; NONCELENGTH]>::try_from(nonce_slice.as_ref()).unwrap();

    let mut s_n: [u8; SALTLENGTH + NONCELENGTH] = [0u8; SALTLENGTH + NONCELENGTH];
    s_n[0..SALTLENGTH].copy_from_slice(&s);
    s_n[SALTLENGTH..(SALTLENGTH + NONCELENGTH)].copy_from_slice(&n);

    let mut ret_val: Vec<u8> = Vec::with_capacity(ciphertext.len());
    ret_val.resize(ciphertext.len(), 0);
    ret_val.copy_from_slice(ciphertext.as_slice());
    Some((s_n, ret_val))
}


pub fn decrypt(salt_bytes: &[u8; SALTLENGTH + NONCELENGTH], encrypted: &Vec<u8>, password: &str) -> Result<String, ()> {
    if encrypted.len() < 1 {
        return Err(());
    }
    let mut salt: [u8; SALTLENGTH] = [0; SALTLENGTH];
    salt.clone_from_slice((&salt_bytes[0..SALTLENGTH]));

    let mut nonce_slice: [u8; NONCELENGTH] = [0; NONCELENGTH];
    nonce_slice.clone_from_slice((&salt_bytes[SALTLENGTH..SALTLENGTH + NONCELENGTH]));
    
    let ct: &[u8] = &encrypted;

    //pbkdf2 Key Derivation From Salt
    // number of iterations
    let n = 600_000;
    let mut key: [u8; SALTLENGTH] = [0u8; SALTLENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, n, &mut key);

    let cipher = ChaCha20Poly1305::new_from_slice(&key).unwrap();
    let nonce: Nonce = Nonce::from(nonce_slice.clone());
    match cipher.decrypt(&nonce, ct) {
        Ok(decrypted) => {
            match std::str::from_utf8(decrypted.as_slice()) {
                Ok(decrypted_text) => Ok(decrypted_text.to_string()),
                Err(_) => Err(()),
            }
        },
        Err(_) => {
            error!("Error Decrypting!");
            Err(())
        }
    }
}


#[cfg(test)]
pub mod encryption_tests {
    use crate::encryption::{decrypt, encrypt, NONCELENGTH, SALTLENGTH};

    #[test]
    fn encrypt_a_plaintext() {
        let (salt, encrypted) = encrypt("hello", "thisisalongenoughkeytoencryptavalue").unwrap();
        assert_eq!(salt.len(), SALTLENGTH + NONCELENGTH);
        assert_ne!(encrypted.len(), "hello".len());
    }


    #[test]
    fn encrypt_and_decrypt_a_text() {
        let (salt, encrypted) = encrypt("hello", "thisisalongenoughkeytoencryptavalue").unwrap();
        match decrypt(&salt, &encrypted, "thisisalongenoughkeytoencryptavalue") {
            Ok(v) => {
                assert_eq!(v.as_str(), "hello");
            },
            Err(_) => {
                println!("Failed To Decrypt Test Value!");
                assert_eq!(1, 2);
            }
        }
    }
    
 
}

 