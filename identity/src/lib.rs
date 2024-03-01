#![allow(unused_assignments)]
#![allow(dead_code)]
use crypto;
use core::str::Utf8Error;

fn identity_to_address(identity: &Vec<u8>) -> Result<String, Utf8Error> {
    match  std::str::from_utf8(identity.as_slice()) {
        Ok(val) => Ok(val.to_string()),
        Err(err) => Err(err)
    }
}

#[derive(Debug)]
pub struct Identity {
    pub seed: String,
    pub hash: String,
    pub salt: String,
    pub identity: String,
    pub encrypted: bool
}

impl Identity {
    pub fn get_public_key(&self) -> Result<Vec<u8>, String> {
        if !self.contains_seed() {
            Err("Invalid Seed! Can't Get Public Key!".to_string())
        } else {
            let mut own_subseed: Vec<u8> = vec![];
            let mut private_key: Vec<u8> = vec![];
            let mut public_key: [u8; 32] = [0; 32];
            own_subseed = crypto::qubic_identities::get_subseed(self.seed.as_str()).expect("Failed To Get SubSeed!");
            private_key = crypto::qubic_identities::get_private_key(&own_subseed);
            public_key = crypto::qubic_identities::get_public_key(&private_key);
            own_subseed = crypto::qubic_identities::get_subseed(self.seed.as_str()).expect("Failed To Get SubSeed!");
            Ok(Vec::from(public_key))
        }
    }
    pub fn from_vars(seed: &str, hash: &str, salt: &str, identity: &str, is_encrypted: bool) -> Self {
        Identity {
            seed: String::from(seed),
            hash: String::from(hash),
            salt: String::from(salt),
            identity: String::from(identity),
            encrypted: is_encrypted
        }
    }
    pub fn contains_seed(&self) -> bool { self.seed.len() == 55}
    pub fn new(seed: &str) -> Self {
        let subseed = crypto::qubic_identities::get_subseed(seed).expect("Failed To Get SubSeed!");
        let private_key = crypto::qubic_identities::get_private_key(&subseed);
        let public_key = crypto::qubic_identities::get_public_key(&private_key);
        let id = crypto::qubic_identities::get_identity(&public_key);
        Identity {
            seed: String::from(seed),
            hash: String::from(""),
            salt: String::from(""),
            identity: id,
            encrypted: false
        }
    }
    pub fn encrypt_identity(&mut self, password: &str) -> Result<Self, String> {
        if !self.contains_seed() {
            return Err("Unable To Encrypt Identity With Missing Seed!".to_string());
        }
        match crypto::encryption::encrypt(self.seed.as_str(), password) {
            Some((nonce, ciphertext)) => {
                match crypto::passwords::hash_password(password) {
                    Ok(hashed_password) => {
                        Ok(Identity {
                            seed: ciphertext,
                            hash: hashed_password,
                            salt: nonce,
                            identity: self.identity.to_owned(),
                            encrypted: true
                        })
                    },
                    Err(err) => Err(err)
                }
            },
            None => {
                Err("Failed To Encrypt Seed!".to_string())
            }
        }
    }

    pub fn decrypt_identity(&mut self, password: &str) -> Result<Self, String> {
        if !self.encrypted {
            return Err("Unable To Decrypt an Unencrypted Identity!".to_string());
        }
        match crypto::passwords::verify_password(password, self.hash.as_str()) {
            Ok(_) => {
                match crypto::encryption::decrypt(self.salt.as_str(), self.seed.as_str(), password) {
                    Ok(seed) => {
                        Ok(Identity {
                            seed: seed,
                            hash: self.hash.to_owned(),
                            salt: self.salt.to_owned(),
                            identity: self.identity.to_owned(),
                            encrypted: false
                        })
                    },
                    Err(_) => {
                        Err("Failed To Decrypt Identity! (Memory Corruption?)".to_string())
                    }
                }
            },
            Err(_) => {
                Err("Invalid Password!".to_string())
            }
        }
    }
}


#[cfg(test)]
mod create_identity {

    pub mod create {
        use crate::Identity;

        #[test]
        fn create_new_identity() {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        }

        #[test]
        fn create_new_identity_from_vars() {
            let id: Identity = Identity::from_vars(
                "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf",
                "",
                "",
                "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON",
                false
            );
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        }
    }

    pub mod encryption {
        use crate::Identity;
        #[test]
        fn encrypt_an_identity() {
            let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            assert_eq!(id.encrypted, false);
            assert_eq!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            id =  id.encrypt_identity("password").unwrap();
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
            assert_eq!(id.encrypted, true);
            assert_ne!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
        }

        #[test]
        fn decrypt_an_identity() {
            let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            id =  id.encrypt_identity("password").unwrap();
            assert_eq!(id.encrypted, true);
            id = id.decrypt_identity("password").unwrap();
            assert_eq!(id.encrypted, false);
            assert_eq!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
        }
    }
}