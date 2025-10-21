use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

pub fn random_id(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn hash_to_floats(text: &str, dims: usize) -> Vec<f32> {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let mut values = Vec::with_capacity(dims);
    for chunk in digest.chunks(4).cycle().take(dims) {
        let mut bytes = [0u8; 4];
        for (idx, byte) in chunk.iter().enumerate() {
            bytes[idx] = *byte;
        }
        let value = f32::from_le_bytes(bytes) / f32::MAX.abs();
        values.push(value);
    }
    values
}
