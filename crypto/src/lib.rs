extern crate libc;
extern crate core;


pub mod hash {
    use sodiumoxide::hex::encode;
    extern {
        fn KangarooTwelve(input: *const u8, inputByteLen: u32, output: *mut u8, outputByteLen: u32);
    }
    pub fn k12(input: &str) -> String {
        let mut output: [u8; 32] = [0; 32];
        unsafe { KangarooTwelve(input.as_ptr(), input.len() as u32, output.as_mut_ptr(), 32); }
        let val = encode(output);
        return val;
    }

    #[cfg(test)]
    pub mod kangaroo12_tests {
        use crate::hash::k12;
        #[test]
        fn hash_a_value() {
            let value = k12("inputText");
            assert_eq!(value, "2459b095c4d5b1759a14f5e4924f26a813c020979fab5ef2cad7321af37808d3".to_string())
        }
    }
}

pub mod random {
    use sodiumoxide::randombytes::randombytes;
    pub fn random_bytes(length: u32) -> Vec<u8> {
        randombytes(length as usize)
    }


    #[cfg(test)]
    pub mod random_tests {
        use std::collections::HashSet;
        use crate::random::random_bytes;

        #[test]
        fn get_a_random_vector() {
            let vec_one = random_bytes(32);
            let vec_two = random_bytes(32);
            let s1: HashSet<_> = vec_one.iter().copied().collect();
            let s2: HashSet<_> = vec_two.iter().copied().collect();
            let diff: Vec<_> = s1.difference(&s2).collect();
            assert!(diff.len() > 0);
        }
    }


}

pub mod encryption {
    use base64::{Engine as _, engine::general_purpose};
    use crate::hash;
    use sodiumoxide::crypto::secretbox::{ Key, Nonce };
    use sodiumoxide::crypto::secretbox;

    pub fn encrypt(plaintext: &str, password: &str) -> Option<(String, String)> {
        let hashed_password: String = hash::k12(password);
        let p_key: [u8; 32] = match hashed_password.as_bytes()[..32].try_into() {
            Ok(v) => v,
            Err(_) => { return None; }
        };
        let key: Key = Key(p_key);
        let nonce = secretbox::gen_nonce();
        let ciphertext = secretbox::seal(plaintext.as_bytes(), &nonce, &key);
        let nonce_string: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(nonce.as_ref());
        let ciphertext_string: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(ciphertext.as_ref());
        return Some((nonce_string, ciphertext_string));
    }

    pub fn decrypt(nonce: &str, ciphertext: &str, password: &str) -> Result<String, ()> {
        let hashed_password: String = hash::k12(password);
        let p_key: [u8; 32] = match hashed_password.as_bytes()[..32].try_into() {
            Ok(v) => v,
            Err(_) => { return Err(()); }
        };
        let key: Key = Key(p_key);
        let p_n: Vec<u8> = match general_purpose::STANDARD_NO_PAD.decode::<&str>(nonce) {
            Ok(value) => value,
            Err(err) => {
                println!("Error Decoding Nonce From Base64! : {}", err.to_string());
                return Err(());
            }
        };
        let p_nonce: [u8; 24] = p_n.as_slice().try_into().unwrap();
        let nonce_to_use: Nonce = Nonce(p_nonce);
        let c_t: Vec<u8> = match general_purpose::STANDARD_NO_PAD.decode::<&str>(ciphertext) {
            Ok(value) => value,
            Err(err) => {
                println!("Error Decoding CipherText From Base64! : {}", err.to_string());
                return Err(());
            }
        };
        match secretbox::open(c_t.as_slice(), &nonce_to_use, &key) {
            Ok(decrypted) => {
                let plaintext: String = std::str::from_utf8(decrypted.as_slice()).unwrap().to_string();
                return Ok(plaintext);
            },
            Err(_) => {
                println!("Error Decrypting!");
                Err(())
            }
        }
    }

    #[cfg(test)]
    pub mod encryption_tests {
        use crate::encryption::{decrypt, encrypt};

        #[test]
        fn encrypt_a_plaintext() {
            encrypt("hello", "thisisalongenoughkeytoencryptavalue").unwrap();
        }

        #[test]
        fn encrypt_and_decrypt_a_text() {
            let (nonce, ct) = encrypt("hello", "thisisalongenoughkeytoencryptavalue").unwrap();
            match decrypt(nonce.as_str(), ct.as_str(), "thisisalongenoughkeytoencryptavalue") {
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
}


pub mod passwords {
    use bcrypt::{DEFAULT_COST, hash, verify};
    pub fn hash_password(password: &str) -> Result<String, String> {
        match hash(password, DEFAULT_COST) {
            Ok(hashed) => {
                Ok(hashed)
            },
            Err(err) => {
                println!("Error Hashing Password! : {}", err.to_string().as_str());
                Err(err.to_string())
            }
        }
    }
    pub fn verify_password(password: &str, ciphertext: &str) -> Result<bool, String> {
        match  verify(password, ciphertext) {
            Ok(result) => Ok(result),
            Err(err) => {
                println!("Error Verifying Password! : {:?}", err.to_string().as_str());
                Err(err.to_string())
            }
        }
    }

    #[cfg(test)]
    pub mod bcrypt_tests {
        use crate::passwords::{ hash_password, verify_password };

        #[test]
        fn hash_a_password_correct_len() {
            let res = hash_password("hello").unwrap();
            assert_eq!(res.len(), 60)
        }

        #[test]
        fn hash_a_password_ensure_unique() {
            let res = hash_password("wrong_password_for_result").unwrap();
            assert_ne!(res.as_str(), "$2b$12$tfOakqclS.o0bV5Tht57tOumU7Nyumh0qjjd3LqgMV1gLQBD68jT6")
        }

        #[test]
        fn verify_a_password() {
            match verify_password("hello", "$2b$12$tfOakqclS.o0bV5Tht57tOumU7Nyumh0qjjd3LqgMV1gLQBD68jT6") {
                Ok(_) => {},
                Err(_) => { assert_eq!(1, 2) }
            }
        }

        #[test]
        fn verify_a_password_invalid_hash_throws() {
            match verify_password("hello", "") {
                Ok(_) => assert_eq!(1, 2),
                Err(_) => {}
            }
        }
    }
}