use sha2::{Sha256, Digest};
use base64::{encode, decode};

pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    encode(hasher.finalize())
}

pub fn verify_hash(data: &str, hash: &str) -> bool {
    hash_data(data) == hash
}

pub fn encode_base64(data: &[u8]) -> String {
    encode(data)
}

pub fn decode_base64(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    decode(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let data = "test_data";
        let hash = hash_data(data);
        assert!(verify_hash(data, &hash));
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"hello world";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(data, &decoded[..]);
    }
}
