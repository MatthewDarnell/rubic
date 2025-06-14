use std::convert::TryFrom;
use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_public_key_from_identity, verify};
use crate::{ARBITRATOR, NUMBER_COMPUTORS};

pub type ComputorPubKey = [u8; 32];
#[derive(Debug, Clone)]
pub struct BroadcastComputors {
    pub epoch: u16,
    pub pub_keys: [ComputorPubKey; NUMBER_COMPUTORS],
    pub signature: [u8; 64]
}

impl BroadcastComputors {
    pub fn new(data: &[u8; size_of::<BroadcastComputors>()]) -> Self {
        let mut pub_keys: Vec<ComputorPubKey> = Vec::with_capacity(NUMBER_COMPUTORS);
        let (_, right) = data.split_at(size_of::<u16>());
        for bytes in right[0..right.len()-64].chunks_exact(size_of::<ComputorPubKey>()) {
            pub_keys.push(bytes.try_into().unwrap());
        }
        BroadcastComputors {
            epoch: u16::from_le_bytes([data[0], data[1]]),
            pub_keys: <[ComputorPubKey; NUMBER_COMPUTORS]>::try_from(pub_keys.as_slice().to_owned()).unwrap(),
            signature: right[right.len()-64..].try_into().unwrap()
        }
    }
    pub fn hash_without_signature(&self) -> Vec<u8> {
        let mut bytes: [u8; size_of::<BroadcastComputors>() - 64] = [0u8; size_of::<BroadcastComputors>() - 64];
        bytes[..2].copy_from_slice(&u16::to_le_bytes(self.epoch));
        for (index, pub_key) in self.pub_keys.iter().enumerate() {
            bytes[2 + (index * size_of::<ComputorPubKey>())..2 + (index * size_of::<ComputorPubKey>()) + 32].copy_from_slice(pub_key.as_slice()); 
        }
        k12_bytes(&bytes.to_vec())
    }
    pub fn validate(&self) -> bool {
        let sig: [u8; 64] = self.signature;
        let arbitrator: [u8; 32] = get_public_key_from_identity(&String::from(ARBITRATOR)).unwrap();
        let hash: Vec<u8> = self.hash_without_signature();
        let message_digest: [u8; 32] = hash.as_slice().to_owned().try_into().unwrap();
        let verified: bool = verify(&arbitrator, &message_digest, &sig);
        verified
    }
}


mod test_computors {
    #![allow(dead_code, unused)]
    use crypto::qubic_identities::get_identity;
    use crate::computor::{BroadcastComputors, ComputorPubKey};
    use crate::consensus_tests::epoch_163_computors;

    #[test]
    fn test_broadcast_computors_validates() {
        let bc: BroadcastComputors = BroadcastComputors::new(epoch_163_computors());
        assert_eq!(bc.validate(), true);
    }
    
    #[test]
    fn test_computor_pub_key() {
        let pub_key: [u8; 32] = [
            0x57, 0xB2, 0xAE, 0xCF, 0x5D, 0x2B, 0xFD, 0x15, 0xCE, 0x24, 0x3D, 0xA4, 0x85, 0x24, 0x58, 0x31,
            0x6C, 0x5A, 0xE6, 0x72, 0x12, 0xCC, 0x50, 0x25, 0xFB, 0x40, 0x70, 0xEE, 0xBE, 0xEE, 0x5B, 0x12];
        
        let computor: ComputorPubKey = pub_key;
        let identity: String = get_identity(&computor);
        assert_eq!(identity.as_str(), "RRPOICEYZBSPQAUDVHGSBFKKTGLBKNKZSNGZSLPECBJNKXGWQCTGLWNANVOI");
    }   
}