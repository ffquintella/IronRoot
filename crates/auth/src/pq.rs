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
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
// ml-kem 0.3 deprecated the *expanded* decapsulation-key encoding in favour of
// 64-byte seeds. We deliberately keep using the expanded form here: it is the
// format `keypair_bytes` has always emitted, and `DecapsulationKey::to_seed`
// returns `None` for keys loaded from an expanded encoding — so switching would
// strand every already-persisted keypair. See the note on `keypair_bytes`.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hkdf::Hkdf;
#[allow(deprecated)]
use ml_kem::ExpandedKeyEncoding;
use ml_kem::{Decapsulate, Encapsulate, Kem, KeyExport, MlKem768, array::Array};
use rand::Rng;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::AuthError;

/// Original envelope: encryption key derived as `SHA-256(context || shared)`.
/// Still accepted on unseal so hashes sealed by earlier versions keep working.
const VERSION_V1_SHA256: u8 = 1;
/// Current envelope: key derived with HKDF-SHA256, the standard construction.
const VERSION_HKDF: u8 = 2;

/// A sealed Argon2id PHC string.
///
/// Stored on disk as URL-safe base64 (no padding). Round-trip with
/// [`SealedHash::encode`] / [`SealedHash::decode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedHash {
    /// Envelope version, which selects the key-derivation function on unseal.
    version: u8,
    ct: Vec<u8>,
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

impl SealedHash {
    /// Encode the sealed envelope as a URL-safe base64 string without padding.
    pub fn encode(&self) -> String {
        let mut buf = Vec::with_capacity(1 + self.ct.len() + 12 + self.ciphertext.len());
        buf.push(self.version);
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
        let version = raw[0];
        if version != VERSION_V1_SHA256 && version != VERSION_HKDF {
            return Err(AuthError::PqSeal(format!("unknown version {version}")));
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
            version,
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
    ///
    /// The decapsulation key is wrapped in [`Zeroizing`] so this copy is wiped
    /// when dropped; it still derefs to `Vec<u8>`, so callers that just write
    /// it out need no changes. Anything *you* copy it into is your
    /// responsibility.
    pub fn keypair_bytes(&self) -> (Zeroizing<Vec<u8>>, Vec<u8>) {
        #[allow(deprecated)]
        let dk_bytes = Zeroizing::new(self.dk.to_expanded_bytes().to_vec());
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

        let key = derive_key(VERSION_HKDF, shared.as_slice());
        let aead = ChaCha20Poly1305::new(&Key::from(*key));
        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);
        let ciphertext = aead
            .encrypt(&Nonce::from(nonce), plaintext)
            .map_err(|e| AuthError::PqSeal(format!("aead encrypt: {e}")))?;
        Ok(SealedHash {
            version: VERSION_HKDF,
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

        let key = derive_key(sealed.version, shared.as_slice());
        let aead = ChaCha20Poly1305::new(&Key::from(*key));
        aead.decrypt(&Nonce::from(sealed.nonce), sealed.ciphertext.as_ref())
            .map_err(|e| AuthError::PqSeal(format!("aead decrypt: {e}")))
    }
}

/// Derive the ChaCha20-Poly1305 key from the ML-KEM shared secret.
///
/// `version` selects the construction so that envelopes sealed by older
/// releases stay readable:
///
/// - [`VERSION_V1_SHA256`] — `SHA-256(context || shared)`. Sound here (the
///   ML-KEM shared secret is already a uniformly random 32 bytes), but
///   non-standard, so it is kept only for decrypting existing data.
/// - [`VERSION_HKDF`] — HKDF-SHA256 with the context as `info`. The standard,
///   auditable choice, and what every new seal uses.
fn derive_key(version: u8, shared: &[u8]) -> Zeroizing<[u8; 32]> {
    const CONTEXT: &[u8] = b"ironroot-auth/pq-seal/v1";
    let mut key = Zeroizing::new([0u8; 32]);
    if version == VERSION_V1_SHA256 {
        let mut h = Sha256::new();
        h.update(CONTEXT);
        h.update(shared);
        key.copy_from_slice(&h.finalize());
    } else {
        let hk = Hkdf::<Sha256>::new(None, shared);
        hk.expand(CONTEXT, key.as_mut())
            .expect("HKDF-SHA256 expand of 32 bytes cannot fail");
    }
    key
}

// --- URL-safe base64 (no padding) -----------------------------------------
//
// Previously hand-rolled here. Swapped for the `base64` crate: identical
// output alphabet and padding behaviour, but a well-tested implementation
// rather than one this project has to maintain and audit itself.

fn b64_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn b64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Envelopes written before the HKDF switch must still unseal, otherwise a
    /// dependency upgrade would silently lock every stored password hash.
    #[test]
    fn v1_sha256_envelopes_still_unseal() {
        use chacha20poly1305::aead::{Aead, KeyInit};

        let sealer = PqSealer::generate();
        let mut rng = rand::rng();
        let (ct, shared) = sealer.ek.encapsulate_with_rng(&mut rng);

        // Hand-build a v1 envelope the way the pre-HKDF code did.
        let key = derive_key(VERSION_V1_SHA256, shared.as_slice());
        let aead = ChaCha20Poly1305::new(&Key::from(*key));
        let nonce = [7u8; 12];
        let ciphertext = aead
            .encrypt(&Nonce::from(nonce), b"$argon2id$legacy".as_ref())
            .unwrap();
        let legacy = SealedHash {
            version: VERSION_V1_SHA256,
            ct: ct.as_slice().to_vec(),
            nonce,
            ciphertext,
        };

        assert_eq!(sealer.unseal(&legacy).unwrap(), b"$argon2id$legacy");

        // ...and it survives an encode/decode round trip.
        let reparsed = SealedHash::decode(&legacy.encode()).unwrap();
        assert_eq!(reparsed.version, VERSION_V1_SHA256);
        assert_eq!(sealer.unseal(&reparsed).unwrap(), b"$argon2id$legacy");
    }

    #[test]
    fn new_seals_use_the_hkdf_envelope() {
        let sealer = PqSealer::generate();
        let sealed = sealer.seal(b"secret").unwrap();
        assert_eq!(sealed.version, VERSION_HKDF);
        assert_eq!(SealedHash::decode(&sealed.encode()).unwrap(), sealed);
    }

    /// The `base64` crate must produce byte-identical output to the hand-rolled
    /// encoder it replaced, or previously stored envelopes would not parse.
    #[test]
    fn base64_matches_the_previous_hand_rolled_encoder() {
        const ALPH: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        fn legacy_encode(bytes: &[u8]) -> String {
            let mut out = String::new();
            for chunk in bytes.chunks(3) {
                let b0 = chunk[0] as u32;
                let b1 = *chunk.get(1).unwrap_or(&0) as u32;
                let b2 = *chunk.get(2).unwrap_or(&0) as u32;
                let n = (b0 << 16) | (b1 << 8) | b2;
                let take = chunk.len() + 1;
                for i in 0..take {
                    out.push(ALPH[((n >> (18 - 6 * i)) & 0x3F) as usize] as char);
                }
            }
            out
        }

        for n in 0..64usize {
            let bytes: Vec<u8> = (0..n).map(|i| (i * 7 + 3) as u8).collect();
            assert_eq!(b64_encode(&bytes), legacy_encode(&bytes), "n={n}");
            assert_eq!(b64_decode(&b64_encode(&bytes)).unwrap(), bytes, "n={n}");
        }
    }

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
