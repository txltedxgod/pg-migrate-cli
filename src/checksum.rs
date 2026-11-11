use sha2::{Digest, Sha256};

pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::compute_sha256;

    #[test]
    fn test_compute_sha256_known_vector_and_determinism() {
        let h1 = compute_sha256("CREATE TABLE users (id INT PRIMARY KEY);");
        let h2 = compute_sha256("CREATE TABLE users (id INT PRIMARY KEY);");
        let empty = compute_sha256("");

        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert_eq!(
            empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
