extern crate libc;
use std::str::Utf8Error;
extern {
    fn getSubseed(seed: *const u8, subseed: *mut u8) -> bool;
    fn getPrivateKey(subseed: *const u8, privateKey: *mut u8);
    fn getPublicKey(privateKey: *const u8, publicKey: *mut u8);
    fn getIdentity(publicKey: *const u8, identity: *const u8, isLowerCase: bool);
    fn KangarooTwelve(input: *const u8, inputByteLen: u32, output: *mut u8, outputByteLen: u32);
    // bool getSharedKey(const unsigned char* privateKey, const unsigned char* publicKey, unsigned char* sharedKey)
    //void sign(const unsigned char* subseed, const unsigned char* publicKey, const unsigned char* messageDigest, unsigned char* signature)
    //bool verify(const unsigned char* publicKey, const unsigned char* messageDigest, const unsigned char* signature)
}

fn identity(seed: &str, index: u32) -> Vec<u8> {
    let mut ownSubseed: [u8; 32] = [0; 32];
    let mut privateKey: [u8; 32] = [0; 32];
    let mut publicKey: [u8; 32] = [0; 32];
    let mut identity: [u8; 60] = [0; 60];
    unsafe {
        getSubseed(seed.as_ptr(), ownSubseed.as_mut_ptr());
        getPrivateKey(ownSubseed.as_ptr(), privateKey.as_mut_ptr());
        getPublicKey(privateKey.as_ptr(), publicKey.as_mut_ptr());
        getIdentity(publicKey.as_ptr(), identity.as_mut_ptr(), false);
    }
    identity.to_owned().to_vec()
}

fn identity_to_address(identity: &Vec<u8>) -> Result<String, Utf8Error> {
    match  std::str::from_utf8(identity.as_slice()) {
        Ok(val) => Ok(val.to_string()),
        Err(err) => Err(err)
    }
}

#[derive(Debug)]
pub struct Identity {
    pub seed: String,
    pub seed_ct: String,
    pub hash: String,
    pub salt: String,
    pub identity: String,
    pub index: u32
}

impl Identity {
    pub fn from_vars(seed: &str, seed_ct: &str, hash: &str, salt: &str, identity: &str, index: u32) -> Self {
        Identity {
            seed: String::from(seed),
            seed_ct: String::from(""),
            hash: String::from(""),
            salt: String::from(""),
            identity: String::from(identity),
            index: index
        }
    }
    pub fn contains_seed(identity: &Identity) -> bool { identity.seed.len()== 55}
    pub fn new(seed: &str, index: u32) -> Self {
        let id = identity(seed, index);
        match identity_to_address(&id) {
            Ok(address) => {
                Identity {
                    seed: String::from(seed),
                    seed_ct: String::from(""),
                    hash: String::from(""),
                    salt: String::from(""),
                    identity: address,
                    index: index
                }
            },
            Err(err) => {
                Identity {
                    seed: String::from(""),
                    seed_ct: String::from(""),
                    hash: String::from(""),
                    salt: String::from(""),
                    identity: String::from(""),
                    index: 0
                }
            }
        }
    }
}


#[test]
fn create_new_identity() {
    let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", 0);
    println!("{:?}", &id);
    assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
}

#[test]
fn create_new_identity_from_vars() {
    //let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", 0);
    let id: Identity = Identity::from_vars(
    "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf",
    "",
    "",
    "",
    "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON",
    0
    );
    assert_eq!(id.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
}
