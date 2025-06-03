#![allow(dead_code, unused)]
mod fourq;

const A_LOWERCASE_ASCII: u8 = 97u8;

pub fn initialize() { sodiumoxide::init().expect("Failed To Initialize SodiumOxide!"); }

//#[cfg(feature = "hash")]
pub mod hash {
    use sodiumoxide::hex::encode;
    use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};

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
}


pub mod qubic_identities {
    use core::ptr::copy_nonoverlapping;
    use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};
    use crate::{A_LOWERCASE_ASCII, hash};
    use hash::k12_bytes;
    use crate::fourq::consts::{CURVE_ORDER_0, CURVE_ORDER_1, CURVE_ORDER_2, CURVE_ORDER_3, MONTGOMERY_R_PRIME, ONE};
    use crate::fourq::ops::{addcarry_u64, decode, ecc_mul_double, ecc_mul_fixed, encode, montgomery_multiply_mod_order, subborrow_u64};
    use crate::fourq::types::{PointAffine};
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

    #[inline]   //Thanks Mineco!
    pub fn verify(public_key: &[u8; 32], message_digest: &[u8; 32], signature: &[u8; 64]) -> bool {
        let mut a = PointAffine::default();
        let mut temp: [u8; 96] = [0; 96];
        let mut h: [u8; 64] = [0; 64];
        if (public_key[15] & 0x80 == 1) || (signature[15] & 0x80 == 1) || (signature[62] & 0xC0 == 1) || (signature[63] == 1) {
            return false;  
        }
        if !decode(public_key, &mut a) {  // Also verifies that A is on the curve, if it is not it fails
            return false;
        }

        unsafe {
            copy_nonoverlapping(signature.as_ptr(), temp.as_mut_ptr(), 32);
            copy_nonoverlapping(public_key.as_ptr(), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);
        }

        let mut ull_sig: [u64; 8] = signature
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();


        let mut kg = KangarooTwelve::new(b"");
        kg.update(&temp);
        kg.into_xof().squeeze(&mut h);
        
        let mut ull_h: [u64; 8] = h
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        
        if !ecc_mul_double(&mut ull_sig[4..], &mut ull_h, &mut a) {
            return false;
        }
        
        let mut a_bytes: [u8; 64] = [0; 64];
        encode(&mut a, &mut a_bytes);
        
        a_bytes[..32].iter().zip(signature[..32].iter()).all(|(a,b)| a == b)
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
    use sodiumoxide::crypto::secretbox::{Key, Nonce, NONCEBYTES};
    use sodiumoxide::crypto::{pwhash, secretbox};
    use sodiumoxide::crypto::pwhash::SALTBYTES;
    use logger::error;
    
    pub fn encrypt(plaintext: &str, password: &str) -> Option<([u8; SALTBYTES + NONCEBYTES], Vec<u8>)> {
        let salt = pwhash::gen_salt();
        let nonce = secretbox::gen_nonce();
        let mut key = Key([0; secretbox::KEYBYTES]);
        let mut ciphertext: Vec<u8> = Vec::new();
        {
            let Key(ref mut kb) = key;
            match pwhash::derive_key(kb, password.as_bytes(), &salt,
                               pwhash::OPSLIMIT_INTERACTIVE,
                               pwhash::MEMLIMIT_INTERACTIVE) {
                Ok(_) => {
                    ciphertext = secretbox::seal(plaintext.as_bytes(), &nonce, &key);
                },
                Err(_) => { return None; }
            }
        }
        let s: [u8; SALTBYTES] = <[u8; SALTBYTES]>::try_from(salt.as_ref()).unwrap();
        let n: [u8; NONCEBYTES] = <[u8; NONCEBYTES]>::try_from(nonce.as_ref()).unwrap();
        
        let mut s_n: [u8; SALTBYTES + NONCEBYTES] = [0u8; SALTBYTES + NONCEBYTES];
        s_n[0..SALTBYTES].copy_from_slice(&s);
        s_n[SALTBYTES..(SALTBYTES + NONCEBYTES)].copy_from_slice(&n);
        
        let mut ret_val: Vec<u8> = Vec::with_capacity(ciphertext.len());
        ret_val.resize(ciphertext.len(), 0);
        ret_val.copy_from_slice(ciphertext.as_slice());
        Some((s_n, ret_val))
    }
    
    
    pub fn decrypt(salt_bytes: &[u8; SALTBYTES + NONCEBYTES], encrypted: &Vec<u8>, password: &str) -> Result<String, ()> {
        if encrypted.len() < 1 {
            return Err(());
        }
        let salt: pwhash::Salt = pwhash::Salt::from_slice(&salt_bytes[0..SALTBYTES]).unwrap();
        let nonce: Nonce = Nonce::from_slice(&salt_bytes[SALTBYTES..SALTBYTES + NONCEBYTES]).unwrap();
        let ct: &[u8] = &encrypted;
        let mut key = Key([0; secretbox::KEYBYTES]);
        {
            let Key(ref mut kb) = key;
            match pwhash::derive_key(kb, password.as_bytes(), &salt,
                                     pwhash::OPSLIMIT_INTERACTIVE,
                                     pwhash::MEMLIMIT_INTERACTIVE) {
                Ok(_) => {
                    match secretbox::open(ct, &nonce, &key) {
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
                },
                Err(_) => Err(())
            }
        }
    }

    #[cfg(test)]
    pub mod encryption_tests {
        use sodiumoxide::hex;
        use crate::encryption::{decrypt, encrypt};

        #[test]
        fn encrypt_a_plaintext() {
            let (salt, encrypted) = encrypt("hello", "thisisalongenoughkeytoencryptavalue").unwrap();
            assert_eq!(salt.len(), 56);
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
}


pub mod passwords {
    use sodiumoxide::crypto::{pwhash::argon2id13, secretbox};
    use random::random_bytes;
    use crate::random;

    pub fn hash_password(password: &str) -> Result<String, String> {
        match argon2id13::pwhash(password.as_bytes(),
                             argon2id13::OPSLIMIT_INTERACTIVE,
                             argon2id13::MEMLIMIT_INTERACTIVE) {
            Ok(ph) => Ok(sodiumoxide::hex::encode(ph.as_ref())),
            Err(_) => Err("Could not hash password".to_string())
        }
    }
    pub fn verify_password(password: &str, ciphertext: &str) -> Result<bool, String> {
            match sodiumoxide::hex::decode(ciphertext) {
                Ok(ct_vec) => {
                    match argon2id13::HashedPassword::from_slice(&ct_vec) {
                        Some(ph) => {
                            Ok(argon2id13::pwhash_verify(&ph, password.as_bytes()))
                        },
                        None => Err("Could Not Get Password To Verify. Memory Corruption?".to_string())
                    }
                },
                Err(_) => Err("Could Not Validate Password".to_string())
            }
        }

    #[cfg(test)]
    pub mod password_hash_tests {
        use crate::passwords::{hash_password, verify_password};

        #[test]
        fn hash_a_password_correct_len() {
            let res = hash_password("superSecretPassword").unwrap();
            assert_ne!(res.len(), "superSecretPassword".len())
        }

        #[test]
        fn hash_a_password_ensure_unique() {
            let res = hash_password("wrong_password_for_result").unwrap();
            assert_ne!(res.as_str(), "$argon2id$v=19$m=19456,t=2,p=1$OGvJ/8bPLl5ReXN9h0uzZOESj6bGfv/K/6KIf9heg+s$3PDL8se7IfHxVNfIYikDxERwRtF9lNy5kxlSfwIWPJg")
        }

        #[test]
        fn verify_a_password() {
            match hash_password("superSecretPassword") {
                Ok(ph) => {
                    match verify_password("superSecretPassword", ph.as_str()) {
                        Ok(verified) => { assert_eq!(verified, true) },
                        Err(_) => { assert_eq!(1, 2) }
                    }
                },
                Err(_) => {
                    assert_eq!(1, 2)
                }
            }
        }

        #[test]
        fn verify_a_password_invalid_hash_throws() {
            match hash_password("superSecretPassword") {
                Ok(ph) => {
                    match verify_password("bogusPassword", ph.as_str()) {
                        Ok(verified) => { assert_eq!(verified, false) },
                        Err(_) => { assert_eq!(1, 2) }
                    }
                },
                Err(_) => {
                    assert_eq!(1, 2)
                }
            }
        }
    }
}