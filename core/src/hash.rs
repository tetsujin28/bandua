//! Algorithmes de hash. xxHash64 par défaut (standard DIT, compatible MHL).

use md5::{Digest as _, Md5};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use xxhash_rust::xxh3::Xxh3;
use xxhash_rust::xxh64::Xxh64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Algo {
    Xxh64,
    Xxh3,
    Md5,
    Sha1,
}

impl Algo {
    pub fn label(self) -> &'static str {
        match self {
            Algo::Xxh64 => "xxHash64",
            Algo::Xxh3 => "XXH3-64",
            Algo::Md5 => "MD5",
            Algo::Sha1 => "SHA-1",
        }
    }
    /// Nom de balise utilisé dans les fichiers MHL.
    pub fn mhl_tag(self) -> &'static str {
        match self {
            Algo::Xxh64 => "xxhash64be",
            Algo::Xxh3 => "xxh3",
            Algo::Md5 => "md5",
            Algo::Sha1 => "sha1",
        }
    }
}

pub enum Hasher {
    Xxh64(Box<Xxh64>),
    Xxh3(Box<Xxh3>),
    Md5(Md5),
    Sha1(Sha1),
}

impl Hasher {
    pub fn new(algo: Algo) -> Self {
        match algo {
            Algo::Xxh64 => Hasher::Xxh64(Box::new(Xxh64::new(0))),
            Algo::Xxh3 => Hasher::Xxh3(Box::new(Xxh3::new())),
            Algo::Md5 => Hasher::Md5(Md5::new()),
            Algo::Sha1 => Hasher::Sha1(Sha1::new()),
        }
    }
    pub fn update(&mut self, data: &[u8]) {
        match self {
            Hasher::Xxh64(h) => h.update(data),
            Hasher::Xxh3(h) => h.update(data),
            Hasher::Md5(h) => h.update(data),
            Hasher::Sha1(h) => h.update(data),
        }
    }
    /// Hash en hexadécimal minuscule (big-endian pour xxHash, comme dans les MHL).
    pub fn finish_hex(self) -> String {
        match self {
            Hasher::Xxh64(h) => format!("{:016x}", h.digest()),
            Hasher::Xxh3(h) => format!("{:016x}", h.digest()),
            Hasher::Md5(h) => hex(&h.finalize()),
            Hasher::Sha1(h) => hex(&h.finalize()),
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_vectors() {
        let mut h = Hasher::new(Algo::Xxh64);
        h.update(b"");
        assert_eq!(h.finish_hex(), "ef46db3751d8e999");
        let mut h = Hasher::new(Algo::Md5);
        h.update(b"abc");
        assert_eq!(h.finish_hex(), "900150983cd24fb0d6963f7d28e17f72");
    }
}
