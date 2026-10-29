use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Sha256, Sha512};
use zeroize::Zeroizing;

pub const ENC_V1: u8 = 1;

const NONCE_LEN: usize = 12;

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    Malformed,
    Decrypt,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Malformed => f.write_str("sealed data is malformed"),
            CryptoError::Decrypt => f.write_str("decryption failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

pub struct MasterKey(Zeroizing<[u8; 64]>);

impl MasterKey {
    pub fn generate() -> Self {
        let mut k = Zeroizing::new([0u8; 64]);
        OsRng.fill_bytes(k.as_mut());
        MasterKey(k)
    }

    pub fn from_bytes(b: &[u8; 64]) -> Self {
        let mut k = Zeroizing::new([0u8; 64]);
        k.copy_from_slice(b);
        MasterKey(k)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

pub struct ItemKey(Zeroizing<[u8; 32]>);

impl ItemKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

fn hkdf_expand<const N: usize>(ikm: &[u8], salt: Option<&[u8]>, info: &[u8]) -> Zeroizing<[u8; N]> {
    let mut out = Zeroizing::new([0u8; N]);
    Hkdf::<Sha512>::new(salt, ikm).expand(info, out.as_mut()).expect("hkdf output length is valid");
    out
}

pub fn item_key(root: &[u8], salt: &[u8; 32]) -> ItemKey {
    ItemKey(hkdf_expand(root, Some(salt), b"item"))
}

pub struct DedupKey(Zeroizing<[u8; 64]>);

pub fn dedup_key(m: &MasterKey) -> DedupKey {
    DedupKey(hkdf_expand(m.as_bytes(), None, b"dedup"))
}

pub fn dedup_tag(k: &DedupKey, data: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(k.0.as_ref()).expect("hmac accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

#[derive(Clone)]
pub struct LockRoot(Zeroizing<[u8; 64]>);

impl LockRoot {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

pub fn lock_root(m: &MasterKey, signature: &[u8]) -> LockRoot {
    let mut ikm = Zeroizing::new(Vec::with_capacity(64 + signature.len()));
    ikm.extend_from_slice(m.as_bytes());
    ikm.extend_from_slice(signature);
    LockRoot(hkdf_expand(&ikm, None, b"lock"))
}

pub fn lock_dedup_key(r: &LockRoot) -> DedupKey {
    DedupKey(hkdf_expand(r.as_bytes(), None, b"dedup"))
}

pub struct Keys {
    pub master: MasterKey,
    pub dedup: DedupKey,
    pub lock_root: Option<LockRoot>,
}

impl Keys {
    pub fn new(master: MasterKey) -> Self {
        let dedup = dedup_key(&master);
        Keys { master, dedup, lock_root: None }
    }
}

pub fn new_salt() -> [u8; 32] {
    let mut s = [0u8; 32];
    OsRng.fill_bytes(&mut s);
    s
}

pub fn seal(k: &ItemKey, plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(k.as_bytes()));
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ct = cipher.encrypt(Nonce::from_slice(&nonce), plaintext).expect("aes-gcm encryption does not fail for in-memory buffers");
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn open(k: &ItemKey, sealed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if sealed.len() < NONCE_LEN + 16 {
        return Err(CryptoError::Malformed);
    }
    let (nonce, ct) = sealed.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(k.as_bytes()));
    cipher.decrypt(Nonce::from_slice(nonce), ct).map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip() {
        let m = MasterKey::generate();
        let k = item_key(m.as_bytes(), &[7u8; 32]);
        assert_eq!(open(&k, &seal(&k, b"merhaba")).unwrap(), b"merhaba");
    }

    #[test]
    fn nonce_is_fresh_each_time() {
        let k = item_key(MasterKey::generate().as_bytes(), &[1u8; 32]);
        assert_ne!(seal(&k, b"x"), seal(&k, b"x"));
    }

    #[test]
    fn wrong_key_and_tamper_fail() {
        let m = MasterKey::generate();
        let k = item_key(m.as_bytes(), &[1u8; 32]);
        let other = item_key(m.as_bytes(), &[2u8; 32]);
        let mut s = seal(&k, b"x");
        assert!(open(&other, &s).is_err());
        let last = s.len() - 1;
        s[last] ^= 1;
        assert!(open(&k, &s).is_err());
    }

    #[test]
    fn item_key_depends_on_salt_only() {
        let m = MasterKey::generate();
        assert_eq!(item_key(m.as_bytes(), &[3u8; 32]).as_bytes(), item_key(m.as_bytes(), &[3u8; 32]).as_bytes());
        assert_ne!(item_key(m.as_bytes(), &[3u8; 32]).as_bytes(), item_key(m.as_bytes(), &[4u8; 32]).as_bytes());
    }

    #[test]
    fn dedup_tag_is_keyed() {
        let a = MasterKey::generate();
        let b = MasterKey::generate();
        assert_eq!(dedup_tag(&dedup_key(&a), b"x"), dedup_tag(&dedup_key(&a), b"x"));
        assert_ne!(dedup_tag(&dedup_key(&a), b"x"), dedup_tag(&dedup_key(&b), b"x"));
    }

    #[test]
    fn lock_root_depends_on_signature() {
        let m = MasterKey::from_bytes(&[9u8; 64]);
        assert_ne!(lock_root(&m, b"a").as_bytes(), lock_root(&m, b"b").as_bytes());
        assert_eq!(lock_root(&m, b"a").as_bytes(), lock_root(&m, b"a").as_bytes());
    }
}
