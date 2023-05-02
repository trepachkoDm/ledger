use blake2::{Blake2s256, Digest};

use crate::comands::Block;

pub type Hash = Vec<u8>;

pub fn hash(block: &Block) -> Hash {
    let mut hasher = Blake2s256::new();
    hasher.update(format!("{:?}", block).as_bytes());
    let res = hasher.finalize();
    let mut vector = Vec::new();
    vector.extend_from_slice(&res);
    vector
}
pub fn calculate_random_number(hash: Hash) -> f64 {
    let mut array = [0u8; 8];
    for (i, &byte) in hash.iter().enumerate().take(8) {
        array[i] = byte;
    }
    let num = u64::from_be_bytes(array);
    num as f64 / u64::MAX as f64
}
