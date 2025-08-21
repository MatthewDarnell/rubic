use rand::prelude::*;
pub fn random_bytes(length: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(length as usize);
    data.resize(length as usize, 0);
    data = data.chunks_exact_mut(1).map(|mut chunk| rand::rng().random()).collect();
    data
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