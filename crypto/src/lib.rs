#![feature(ascii_char)]
#![feature(ascii_char_variants)]
mod fourq;

const A_LOWERCASE_ASCII: u8 = 97u8;


//#[cfg(feature = "hash")]
pub mod hash {
    use sodiumoxide::hex::encode;
    use tiny_keccak::{Hasher, KangarooTwelve};

    pub fn k12(input: &str) -> String {
        let ret_val = k12_bytes(&input.as_bytes().to_vec());
        let val = encode(ret_val);
        return val;
    }

    pub fn k12_bytes(input: &Vec<u8>) -> Vec<u8> {
        let mut digest = [0; 32];
        let mut kangaroo = KangarooTwelve::new(b"");
        kangaroo.update(input.as_slice());
        kangaroo.finalize(&mut digest);
        return Vec::from(digest);
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


pub mod qubic_identities {
    use core::ptr::copy_nonoverlapping;
    use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};
    use crate::{A_LOWERCASE_ASCII, hash};
    use hash::k12_bytes;
    use crate::fourq::consts::{CURVE_ORDER_0, CURVE_ORDER_1, CURVE_ORDER_2, CURVE_ORDER_3, MONTGOMERY_R_PRIME, ONE};
    use crate::fourq::ops::{addcarry_u64, ecc_mul_fixed, encode, montgomery_multiply_mod_order, subborrow_u64};
    use crate::fourq::types::{PointAffine};

    //     fn getPublicKey(privateKey: *const u8, publicKey: *mut u8);
    //     fn getIdentity(publicKey: *const u8, identity: *const u8, isLowerCase: bool);

    pub fn get_subseed(seed: &str) -> Result<Vec<u8>, String> {
        let mut seed_bytes: [u8; 55] = [0; 55];
        if seed.len() != 55 {
            return Err(String::from("Invalid Seed Length!"))
        }
        for (index, el) in &mut seed.chars().enumerate() {
            if !el.is_alphabetic() {
                return Err(String::from("Invalid Seed!"));
            }
            seed_bytes[index] = el.to_ascii_lowercase() as u8 - A_LOWERCASE_ASCII;

        }
        Ok(k12_bytes(&seed_bytes.to_vec()))
    }
    pub fn get_private_key(subseed: &Vec<u8>) -> Vec<u8> {
        k12_bytes(subseed)
    }

    /*
    pub fn get_public_key(sk: &Vec<u8>) -> Vec<u8> {
        println!("Got : {:?}", &sk);
        let mut p = PointAffine::default();
        let private_key = sk.as_slice().chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
        println!("{:?}", &private_key);
        ecc_mul_fixed(&private_key, &mut p);
        let mut pk: [u8; 60] = [0; 60];
        encode(&mut p, &mut pk);
        pk.to_vec()
    }
     */

    pub fn get_public_key(private_key: &Vec<u8>) -> [u8; 32] {
        let mut ret_val: [u8; 32] = [0; 32];
        let mut p = PointAffine::default();
        let private_key = private_key.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
        ecc_mul_fixed(&private_key, &mut p);
        encode(&mut p, &mut ret_val);
        ret_val
    }


    pub fn get_identity(public_key: &[u8; 32]) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment = u64::from_le_bytes(public_key[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'A';
                public_key_fragment /= 26;
            }
        }
        let mut identity_bytes_checksum = [0u8; 3];
        let bytes: Vec<u8> = k12_bytes(&public_key.to_vec());
        identity_bytes_checksum[0] = bytes[0];
        identity_bytes_checksum[1] = bytes[1];
        identity_bytes_checksum[2] = bytes[2];
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64 | (identity_bytes_checksum[1] as u64) << 8 | (identity_bytes_checksum[2] as u64) << 16;
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'A';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }

    pub fn get_public_key_from_identity(identity: &String) -> Result<[u8; 32], bool> {
        let id: &[u8] = identity.as_bytes();
        let mut public_key: [u8; 32] = [0; 32];
        for i in 0..4 {
            public_key[i << 3..((i<<3) + 8)].copy_from_slice(&u64::to_le_bytes(0u64));
            for j in 0..14 {
                let index = 14 - j - 1;
                if id[i * 14 + index] < b'A' || (id[i * 14 + index]) > b'Z' {
                  return Err(false);
                 }
                let _bytes: [u8; 8] = public_key[i << 3..((i << 3) + 8)].try_into().unwrap();
                let temp: u64 = u64::from_le_bytes(_bytes) * 26u64 +
                    ((id[i * 14 + index] - b'A') as u64);
                public_key[i << 3..((i<<3) + 8)].copy_from_slice(&u64::to_le_bytes(temp));

            }
        }
        #[allow(unused_assignments)]
        let mut identity_bytes_checksum: u32 = 0;
        let hash: Vec<u8> = k12_bytes(&public_key.to_vec());
        let bytes: [u8; 4] = hash[0..4].try_into().unwrap();
        identity_bytes_checksum = u32::from_le_bytes(bytes);
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            if (identity_bytes_checksum % 26) as u8 + b'A' != identity.as_bytes()[56 + i] {
                return Err(false)
            }
            identity_bytes_checksum /= 26;
        }
        Ok(public_key)
    }

    pub fn sign_raw(subseed: &Vec<u8>, public_key: &[u8; 32], message_digest: [u8; 32]) -> [u8; 64] {
        let mut r_a = PointAffine::default();
        let (mut k, mut h, mut temp) = ([0u8; 64], [0u8; 64], [0u8; 96]);
        let mut r = [0u8; 64];


        let mut kg = KangarooTwelve::new(b"");
        kg.update(subseed.as_slice());
        kg.into_xof().squeeze(&mut k);

        let mut signature = [0u8; 64];

        unsafe {
            copy_nonoverlapping(k.as_ptr().offset(32), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);


            let mut kg = KangarooTwelve::new(b"");
            kg.update(&temp[32..]);
            let mut im = [0u8; 64];
            kg.into_xof().squeeze(&mut im);

            copy_nonoverlapping(im.as_ptr(), r.as_mut_ptr(), 64);
            let k: [u64; 8] = k.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            let mut r: [u64; 8] = r.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            ecc_mul_fixed(&r, &mut r_a);

            encode(&mut r_a, &mut signature);
            let mut signature_i: [u64; 8] = signature.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();

            copy_nonoverlapping(signature_i.as_ptr() as *mut u8, temp.as_mut_ptr(), 32);
            copy_nonoverlapping(public_key.as_ptr(), temp.as_mut_ptr().offset(32), 32);


            let mut kg = KangarooTwelve::new(b"");
            kg.update(&temp);
            kg.into_xof().squeeze(&mut h);

            let mut h: [u64; 8] = h.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            let r_i = r;
            montgomery_multiply_mod_order(&r_i, &MONTGOMERY_R_PRIME, &mut r);
            let r_i = r;
            montgomery_multiply_mod_order(&r_i, &ONE, &mut r);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &MONTGOMERY_R_PRIME, &mut h);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &ONE, &mut h);
            montgomery_multiply_mod_order(&k, &MONTGOMERY_R_PRIME, &mut signature_i[4..]);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &MONTGOMERY_R_PRIME, &mut h);
            let mut s_i = [0u64; 4];
            s_i.copy_from_slice(&signature_i[4..]);
            montgomery_multiply_mod_order(&s_i, &h, &mut signature_i[4..]);
            s_i.copy_from_slice(&signature_i[4..]);
            montgomery_multiply_mod_order(&s_i, &ONE, &mut signature_i[4..]);

            if subborrow_u64(subborrow_u64(subborrow_u64(subborrow_u64(0, r[0], signature_i[4], &mut signature_i[4]), r[1], signature_i[5], &mut signature_i[5]), r[2], signature_i[6], &mut signature_i[6]), r[3], signature_i[7], &mut signature_i[7]) != 0 {
                addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(0, signature_i[4], CURVE_ORDER_0, &mut signature_i[4]), signature_i[5], CURVE_ORDER_1, &mut signature_i[5]), signature_i[6], CURVE_ORDER_2, &mut signature_i[6]),signature_i[7], CURVE_ORDER_3, &mut signature_i[7]);
            }

            signature = signature_i.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>().try_into().unwrap();
        }
        signature
    }


    #[cfg(test)]
    pub mod qubic_identity_primitive_tests {
        use crate::hash::k12_bytes;
        use crate::qubic_identities::{get_identity, get_private_key, get_public_key, get_public_key_from_identity, get_subseed, sign_raw};
        #[test]
        fn get_a_subseed() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let subseed = get_subseed(seed).unwrap();
            let encoded = sodiumoxide::hex::encode(subseed);
            assert_eq!(encoded, "d3420abb5f3e0527b588b361fa0a513335833af8b4a4aae23a2958195c3209dc".to_string())
        }
        #[test]
        fn get_a_private_key() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let subseed = get_subseed(seed).unwrap();
            let private_key = get_private_key(&subseed);
            let encoded = sodiumoxide::hex::encode(private_key);
            assert_eq!(encoded, "11531fcea5e11a4a384e211165ff8bcf458595b32c5374ec76cfa1b1da102238".to_string())
        }
        #[test]
        fn get_a_public_key() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let subseed = get_subseed(seed).unwrap();
            let private_key = get_private_key(&subseed);
            let public_key = get_public_key(&private_key);
            let encoded = sodiumoxide::hex::encode(public_key);
            assert_eq!(encoded, "aa873e4cfd37e4bf528a2aa01eecef36547c99caaabd1bbdf7253a65b041771a".to_string())
        }
        #[test]
        fn get_an_identity() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let subseed = get_subseed(seed).unwrap();
            let private_key = get_private_key(&subseed);
            let public_key = get_public_key(&private_key);
            let identity = get_identity(&public_key);
            assert_eq!(identity, "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON".to_string())
        }
        #[test]
        fn get_a_public_key_from_identity() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let subseed = get_subseed(seed).unwrap();
            let private_key = get_private_key(&subseed);
            let public_key = get_public_key(&private_key);
            let identity = get_identity(&public_key);
            let pub_key_from_id = get_public_key_from_identity(&identity).unwrap();

            assert_eq!(public_key, pub_key_from_id)
        }

        #[test]
        fn test_sign_a_message() {
            let seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
            let message: [u8; 32] = [1; 32];
            let digest = k12_bytes(&message.to_vec());
            let subseed = get_subseed(seed).unwrap();
            let private_key = get_private_key(&subseed);
            let public_key = get_public_key(&private_key);
            let identity = get_identity(&public_key);
            let pub_key_from_id = get_public_key_from_identity(&identity).unwrap();
            let result = sign_raw(&subseed, &public_key, <[u8; 32]>::try_from(digest.as_slice()).expect("Failed!"));
            println!("{:?}", result);
            assert_eq!(public_key, pub_key_from_id)
        }


    }
}



//#[cfg(feature = "random")]
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

//#[cfg(feature = "encryption")]
pub mod encryption {
    use base64::{Engine as _, engine::general_purpose};
    use crate::hash;
    use sodiumoxide::crypto::secretbox::{ Key, Nonce };
    use sodiumoxide::crypto::secretbox;
    use logger::error;

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
                error!("Error Decoding Nonce From Base64! : {}", err);
                return Err(());
            }
        };
        let p_nonce: [u8; 24] = p_n.as_slice().try_into().unwrap();
        let nonce_to_use: Nonce = Nonce(p_nonce);
        let c_t: Vec<u8> = match general_purpose::STANDARD_NO_PAD.decode::<&str>(ciphertext) {
            Ok(value) => value,
            Err(err) => {
                error!("Error Decoding CipherText From Base64! : {}", err.to_string());
                return Err(());
            }
        };
        match secretbox::open(c_t.as_slice(), &nonce_to_use, &key) {
            Ok(decrypted) => {
                let plaintext: String = std::str::from_utf8(decrypted.as_slice()).unwrap().to_string();
                return Ok(plaintext);
            },
            Err(_) => {
                error!("Error Decrypting!");
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