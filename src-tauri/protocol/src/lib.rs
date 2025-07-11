pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

pub mod identity;
pub mod transfer;