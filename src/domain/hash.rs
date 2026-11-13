//! Content hashing and stable dedup keys.
use crate::domain::ContentKind;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// SHA-256 hex digest of UTF-8 text.
pub fn hash_text(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex(&h.finalize())
}

/// SHA-256 hex digest of arbitrary bytes (used for image DIB / PNG bytes).
pub fn hash_bytes(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    hex(&h.finalize())
}

/// SHA-256 over a canonical, sorted, lowercased path list.
/// Sorting makes the hash independent of CF_HDROP ordering.
pub fn hash_paths(paths: &[String]) -> String {
    let mut normalized: Vec<String> = paths.iter().map(|p| p.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    let mut h = Sha256::new();
    for p in &normalized {
        h.update(p.as_bytes());
        h.update(b"\n");
    }
    hex(&h.finalize())
}

/// Stable dedup key scoped by kind, so identical bytes of different kinds never collide.
pub fn dedup_key(kind: ContentKind, hash: &str) -> String {
    format!("{}:{}", kind.as_str(), hash)
}
