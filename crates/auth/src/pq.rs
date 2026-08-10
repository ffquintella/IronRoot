//! Post-quantum sealing layer for stored password hashes.
//!
//! ## Design
//!
//! 1. The server generates an **ML-KEM-768** keypair (NIST FIPS 203). The
//!    decapsulation key (`dk`) stays on the server; the encapsulation key
//!    (`ek`) is also held by the server here — both are loaded into the
//!    [`PqSealer`] at startup.
//! 2. **Sealing** a hash:
//!    - encapsulate against `ek` → `(ct, shared_secret)`
//!    - derive a 32-byte AEAD key with `SHA-256(shared_secret)`
//!    - encrypt the password-hash bytes with ChaCha20-Poly1305 using a fresh
//!      random 12-byte nonce
//!    - the resulting [`SealedHash`] is `version || ct || nonce || ciphertext`
//! 3. **Unsealing**:
//!    - parse the envelope, decapsulate `ct` with `dk` to recover the same
//!      shared secret, derive the same AEAD key, and decrypt.
//!
//! Each call uses a fresh ML-KEM encapsulation, so two seals of the same hash
//! produce different envelopes. This protects stored hashes against an
//! attacker who exfiltrates only the database: without the ML-KEM
//! decapsulation key they cannot recover the Argon2id PHC string to brute-
//! force offline.
//!
//! Sealed envelopes are encoded as URL-safe base64 (no padding) so they can
//! be stored in a plain `TEXT` column.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
// ml-kem 0.3 deprecated the *expanded* decapsulation-key encoding in favour of
// 64-byte seeds. We deliberately keep using the expanded form here: it is the
// format `keypair_bytes` has always emitted, and `DecapsulationKey::to_seed`
// returns `None` for keys loaded from an expanded encoding — so switching would
// strand every already-persisted keypair. See the note on `keypair_bytes`.
#[allow(deprecated)]
use ml_kem::ExpandedKeyEncoding;
use ml_kem::{array::Array, Decapsulate, Encapsulate, Kem, KeyExport, MlKem768};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::AuthError;

const VERSION: u8 = 1;

/// A sealed Argon2id PHC string.
///
/// Stored on disk as URL-safe base64 (no padding). Round-trip with
/// [`SealedHash::encode`] / [`SealedHash::decode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedHash {
    ct: Vec<u8>,
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

impl SealedHash {
    /// Encode the sealed envelope as a URL-safe base64 string without padding.
    pub fn encode(&self) -> String {
        let mut buf = Vec::with_capacity(1 + self.ct.len() + 12 + self.ciphertext.len());
        buf.push(VERSION);
        buf.extend_from_slice(&(self.ct.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.ct);
        buf.extend_from_slice(&self.nonce);
        buf.extend_from_slice(&self.ciphertext);
        b64_encode(&buf)
    }

    /// Parse an envelope previously produced by [`SealedHash::encode`].
    pub fn decode(s: &str) -> Result<Self, AuthError> {
        let raw = b64_decode(s).map_err(|e| AuthError::PqSeal(format!("base64: {e}")))?;
        if raw.len() < 1 + 4 + 12 + 16 {
            return Err(AuthError::PqSeal("envelope truncated".into()));
        }
        if raw[0] != VERSION {
            return Err(AuthError::PqSeal(format!("unknown version {}", raw[0])));
        }
        let ct_len = u32::from_be_bytes([raw[1], raw[2], raw[3], raw[4]]) as usize;
        let header = 5;
        if raw.len() < header + ct_len + 12 + 16 {
            return Err(AuthError::PqSeal("envelope length mismatch".into()));
        }
        let ct = raw[header..header + ct_len].to_vec();
        let nonce_start = header + ct_len;
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&raw[nonce_start..nonce_start + 12]);
        let ciphertext = raw[nonce_start + 12..].to_vec();
        Ok(SealedHash {
            ct,
            nonce,
            ciphertext,
        })
    }
}

/// Server-side sealer/unsealer using ML-KEM-768 + ChaCha20-Poly1305.
pub struct PqSealer {
    dk: <MlKem768 as Kem>::DecapsulationKey,
    ek: <MlKem768 as Kem>::EncapsulationKey,
}

impl PqSealer {
    /// Generate a fresh ML-KEM-768 keypair and return a ready-to-use sealer.
    ///
    /// Persist the keypair via [`PqSealer::keypair_bytes`] if you need it to
    /// survive a restart; reload with [`PqSealer::from_keypair_bytes`].
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let (dk, ek) = MlKem768::generate_keypair_from_rng(&mut rng);
        Self { dk, ek }
    }

    /// Return the serialised `(decapsulation_key, encapsulation_key)` for
    /// persistence. The decapsulation key is sensitive — store it like any
    /// other server secret.
    pub fn keypair_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        #[allow(deprecated)]
        let dk_bytes = self.dk.to_expanded_bytes().to_vec();
        (dk_bytes, self.ek.to_bytes().to_vec())
    }

    /// Reconstruct a sealer from previously-serialised bytes.
    pub fn from_keypair_bytes(dk_bytes: &[u8], ek_bytes: &[u8]) -> Result<Self, AuthError> {
        let dk_arr = Array::try_from(dk_bytes)
            .map_err(|_| AuthError::PqSeal("invalid decapsulation key length".into()))?;
        let ek_arr = Array::try_from(ek_bytes)
            .map_err(|_| AuthError::PqSeal("invalid encapsulation key length".into()))?;
        #[allow(deprecated)]
        let dk = <MlKem768 as Kem>::DecapsulationKey::from_expanded_bytes(&dk_arr)
            .map_err(|_| AuthError::PqSeal("invalid decapsulation key".into()))?;
        let ek = <MlKem768 as Kem>::EncapsulationKey::new(&ek_arr)
            .map_err(|_| AuthError::PqSeal("invalid encapsulation key".into()))?;
        Ok(Self { dk, ek })
    }

    /// Seal a plain-text payload (typically an Argon2id PHC string).
    pub fn seal(&self, plaintext: &[u8]) -> Result<SealedHash, AuthError> {
        let mut rng = rand::rng();
        let (ct, shared) = self.ek.encapsulate_with_rng(&mut rng);

        let key = derive_key(shared.as_slice());
        let aead = ChaCha20Poly1305::new(&Key::from(key));
        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);
        let ciphertext = aead
            .encrypt(&Nonce::from(nonce), plaintext)
            .map_err(|e| AuthError::PqSeal(format!("aead encrypt: {e}")))?;
        Ok(SealedHash {
            ct: ct.as_slice().to_vec(),
            nonce,
            ciphertext,
        })
    }

    /// Unseal an envelope previously produced by [`PqSealer::seal`].
    pub fn unseal(&self, sealed: &SealedHash) -> Result<Vec<u8>, AuthError> {
        let ct_arr = Array::try_from(sealed.ct.as_slice())
            .map_err(|_| AuthError::PqSeal("invalid ciphertext length".into()))?;
        let shared = self.dk.decapsulate(&ct_arr);

        let key = derive_key(shared.as_slice());
        let aead = ChaCha20Poly1305::new(&Key::from(key));
        aead.decrypt(&Nonce::from(sealed.nonce), sealed.ciphertext.as_ref())
            .map_err(|e| AuthError::PqSeal(format!("aead decrypt: {e}")))
    }
}

fn derive_key(shared: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ironroot-auth/pq-seal/v1");
    h.update(shared);
    let out = h.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out);
    key
}

// --- minimal URL-safe base64 (no padding) ---------------------------------

const ALPH: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn b64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() * 4 + 2) / 3);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(ALPH[(b0 >> 2) as usize] as char);
        out.push(ALPH[(((b0 & 0b11) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            out.push(ALPH[(((b1 & 0b1111) << 2) | (b2 >> 6)) as usize] as char);
        }
        if chunk.len() >= 3 {
            out.push(ALPH[(b2 & 0b111111) as usize] as char);
        }
    }
    out
}

fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut vals = Vec::with_capacity(s.len());
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return Err(format!("invalid char {:?}", c as char)),
        };
        vals.push(v);
    }
    let mut out = Vec::with_capacity(vals.len() * 3 / 4);
    for chunk in vals.chunks(4) {
        let v0 = chunk[0];
        let v1 = chunk.get(1).copied().unwrap_or(0);
        out.push((v0 << 2) | (v1 >> 4));
        if let Some(&v2) = chunk.get(2) {
            out.push(((v1 & 0b1111) << 4) | (v2 >> 2));
            if let Some(&v3) = chunk.get(3) {
                out.push(((v2 & 0b11) << 6) | v3);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_unseal_roundtrip() {
        let sealer = PqSealer::generate();
        let plaintext = b"$argon2id$v=19$m=19456,t=2,p=1$abcd$efgh...";
        let sealed = sealer.seal(plaintext).unwrap();
        let opened = sealer.unseal(&sealed).unwrap();
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn envelopes_are_non_deterministic() {
        let sealer = PqSealer::generate();
        let a = sealer.seal(b"hello").unwrap();
        let b = sealer.seal(b"hello").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let sealer = PqSealer::generate();
        let sealed = sealer.seal(b"payload").unwrap();
        let encoded = sealed.encode();
        let decoded = SealedHash::decode(&encoded).unwrap();
        assert_eq!(sealed, decoded);
        assert_eq!(sealer.unseal(&decoded).unwrap(), b"payload");
    }

    #[test]
    fn unseal_with_wrong_key_fails() {
        let a = PqSealer::generate();
        let b = PqSealer::generate();
        let sealed = a.seal(b"secret").unwrap();
        assert!(b.unseal(&sealed).is_err());
    }

    #[test]
    fn keypair_persistence_roundtrip() {
        let original = PqSealer::generate();
        let sealed = original.seal(b"persisted").unwrap();
        let (dk, ek) = original.keypair_bytes();
        let restored = PqSealer::from_keypair_bytes(&dk, &ek).unwrap();
        assert_eq!(restored.unseal(&sealed).unwrap(), b"persisted");
    }

    #[test]
    fn base64_roundtrip_arbitrary_bytes() {
        for n in [0usize, 1, 2, 3, 4, 5, 31, 32, 33, 64, 100] {
            let bytes: Vec<u8> = (0..n).map(|i| (i * 7 % 251) as u8).collect();
            let encoded = b64_encode(&bytes);
            let decoded = b64_decode(&encoded).unwrap();
            assert_eq!(decoded, bytes, "roundtrip failed for n={n}");
        }
    }
}
