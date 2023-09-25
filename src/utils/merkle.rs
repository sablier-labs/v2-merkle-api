use merkle_light::{hash::Algorithm, proof::Proof};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedProof {
    lemma: Vec<[u8; 32]>,
    path: Vec<bool>,
}

impl SerializedProof {
    pub fn from_proof(proof: &Proof<[u8; 32]>) -> Self {
        SerializedProof {
            lemma: proof.lemma().to_vec(),
            path: proof.path().iter().map(|&r| r).collect(),
        }
    }

    pub fn _to_proof(&self) -> Proof<[u8; 32]> {
        Proof::new(self.lemma.clone(), self.path.clone())
    }
}
