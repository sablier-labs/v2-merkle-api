use merkle_light::hash::Algorithm;
use sha2::{Digest, Sha256};
use std::hash::Hasher;

pub struct HashingAlgorithm(Sha256);

impl HashingAlgorithm {
    pub fn new() -> HashingAlgorithm {
        HashingAlgorithm(Sha256::new())
    }
}

impl Default for HashingAlgorithm {
    fn default() -> HashingAlgorithm {
        HashingAlgorithm::new()
    }
}

impl Hasher for HashingAlgorithm {
    fn write(&mut self, msg: &[u8]) {
        self.0.update(msg)
    }

    fn finish(&self) -> u64 {
        unimplemented!()
    }
}

impl Algorithm<[u8; 32]> for HashingAlgorithm {
    fn hash(&mut self) -> [u8; 32] {
        let cloned_hasher = self.0.clone();
        let result = cloned_hasher.finalize();
        let h: [u8; 32] = result.into();
        h
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}
