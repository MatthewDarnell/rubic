extern crate libc;
use crypto;
use std::str::Utf8Error;
extern {
    fn getSubseed(seed: *const u8, subseed: *mut u8) -> bool;
    fn getPrivateKey(subseed: *const u8, privateKey: *mut u8);
    fn getPublicKey(privateKey: *const u8, publicKey: *mut u8);
    fn getIdentity(publicKey: *const u8, identity: *const u8, isLowerCase: bool);
    //bool getPublicKeyFromIdentity(const unsigned char* identity, unsigned char* publicKey)
    fn getPublicKeyFromIdentity(identity: *const u8, publicKey: *mut u8);

    // bool getSharedKey(const unsigned char* privateKey, const unsigned char* publicKey, unsigned char* sharedKey)
    //void sign(const unsigned char* subseed, const unsigned char* publicKey, const unsigned char* messageDigest, unsigned char* signature)
    //bool verify(const unsigned char* publicKey, const unsigned char* messageDigest, const unsigned char* signature)
}

fn identity(seed: &str, index: u32) -> Vec<u8> {
    let mut own_subseed: [u8; 32] = [0; 32];
    let mut private_key: [u8; 32] = [0; 32];
    let mut public_key: [u8; 32] = [0; 32];
    let mut identity: [u8; 60] = [0; 60];
    unsafe {
        getSubseed(seed.as_ptr(), own_subseed.as_mut_ptr());
        getPrivateKey(own_subseed.as_ptr(), private_key.as_mut_ptr());
        getPublicKey(private_key.as_ptr(), public_key.as_mut_ptr());
        getIdentity(public_key.as_ptr(), identity.as_mut_ptr(), false);
    }
    identity.to_owned().to_vec()
}

fn identity_to_address(identity: &Vec<u8>) -> Result<String, Utf8Error> {
    match  std::str::from_utf8(identity.as_slice()) {
        Ok(val) => Ok(val.to_string()),
        Err(err) => Err(err)
    }
}

pub fn get_public_key_from_identity(identity: &str) -> Result<Vec<u8>, ()> {
    let mut pub_key: [u8; 60] = [0; 60];
    unsafe { getPublicKeyFromIdentity(identity.as_ptr(), pub_key.as_mut_ptr()) };
    Ok(pub_key.to_owned().to_vec())
}

#[derive(Debug)]
pub struct Identity {
    pub account: String,
    pub seed: String,
    pub hash: String,
    pub salt: String,
    pub identity: String,
    pub index: u32,
    pub encrypted: bool
}

impl Identity {
    pub fn get_public_key(&self) -> Result<Vec<u8>, String> {
        if !self.contains_seed() {
            Err("Invalid Seed! Can't Get Public Key!".to_string())
        } else {
            let mut own_subseed: [u8; 32] = [0; 32];
            let mut private_key: [u8; 32] = [0; 32];
            let mut public_key: [u8; 32] = [0; 32];
            unsafe {
                getSubseed(self.seed.as_str().as_ptr(), own_subseed.as_mut_ptr());
                getPrivateKey(own_subseed.as_ptr(), private_key.as_mut_ptr());
                getPublicKey(private_key.as_ptr(), public_key.as_mut_ptr());
            }
            Ok(Vec::from(public_key))
        }
    }
    pub fn from_vars(account: &str, seed: &str, hash: &str, salt: &str, identity: &str, index: u32, is_encrypted: bool) -> Self {
        Identity {
            account: String::from(account),
            seed: String::from(seed),
            hash: String::from(hash),
            salt: String::from(salt),
            identity: String::from(identity),
            index: index,
            encrypted: is_encrypted
        }
    }
    pub fn contains_seed(&self) -> bool { self.seed.len() == 55}
    pub fn new(seed: &str, account: &str, index: u32) -> Self {
        let id = identity(seed, index);
        match identity_to_address(&id) {
            Ok(address) => {
                Identity {
                    account: String::from(account),
                    seed: String::from(seed),
                    hash: String::from(""),
                    salt: String::from(""),
                    identity: address,
                    index: index,
                    encrypted: false
                }
            },
            Err(err) => {
                println!("Error Generating Identity! : {}", err.to_string());
                Identity {
                    account: String::from(""),
                    seed: String::from(""),
                    hash: String::from(""),
                    salt: String::from(""),
                    identity: String::from(""),
                    index: 0,
                    encrypted: false
                }
            }
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
                            account: self.account.to_owned(),
                            seed: ciphertext,
                            hash: hashed_password,
                            salt: nonce,
                            identity: self.identity.to_owned(),
                            index: self.index,
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
                            account: self.account.to_owned(),
                            seed: seed,
                            hash: self.hash.to_owned(),
                            salt: self.salt.to_owned(),
                            identity: self.identity.to_owned(),
                            index: self.index,
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
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            println!("{:?}", &id);
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        }

        #[test]
        fn create_new_identity_from_vars() {
            let id: Identity = Identity::from_vars(
                "testAccount",
                "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf",
                "",
                "",
                "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON",
                0,
                false
            );
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        }
    }

    pub mod encryption {
        use crate::Identity;
        #[test]
        fn encrypt_an_identity() {
            let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            assert_eq!(id.encrypted, false);
            assert_eq!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            id =  id.encrypt_identity("password").unwrap();
            assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
            assert_eq!(id.encrypted, true);
            assert_ne!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
        }

        #[test]
        fn decrypt_an_identity() {
            let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            id =  id.encrypt_identity("password").unwrap();
            assert_eq!(id.encrypted, true);
            id = id.decrypt_identity("password").unwrap();
            assert_eq!(id.encrypted, false);
            assert_eq!(id.seed.as_str(), "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
        }
    }
}