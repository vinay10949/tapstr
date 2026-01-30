use secp256k1::{Secp256k1, PublicKey, scalar::Scalar};
use bitcoin_hashes::{sha256, Hash};
use anyhow::{anyhow, Result};

/// PadTo32 adds left zero-padding to ensure the slice has 32 bytes.
/// This is optimized to avoid unnecessary allocations by using a fixed-size array.
pub fn pad_to_32(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let len = data.len().min(32);
    result[32 - len..].copy_from_slice(&data[data.len().saturating_sub(len)..]);
    result
}

/// AddPubKeys returns the sum of two secp256k1 public keys.
/// This implements the EC point addition: R = P1 + P2.
pub fn add_pubkeys(p1: &PublicKey, p2: &PublicKey) -> Result<PublicKey> {
    p1.combine(p2).map_err(|e| anyhow!("Failed to add public keys: {:?}", e))
}

/// NegatePoint returns a new point that is the negation (-P) of the input point P.
/// In elliptic curve cryptography, negating a point means keeping the same x-coordinate but negating the y-coordinate.
pub fn negate_point(p: &PublicKey) -> PublicKey {
    let secp = Secp256k1::new();
    p.negate(&secp)
}

pub fn schnorr_challenge(
    r: &PublicKey,
    p: &PublicKey,
    msg: &[u8],
) -> Scalar {
    let rx = pad_to_32(&r.x_only_public_key().0.serialize());
    let px = pad_to_32(&p.x_only_public_key().0.serialize());

    let mut data = Vec::with_capacity(64 + msg.len());
    data.extend_from_slice(&rx);
    data.extend_from_slice(&px);
    data.extend_from_slice(msg);

    let tag = b"BIP0340/challenge";
    let tag_hash = sha256::Hash::hash(tag);
    let mut preimage = tag_hash.to_byte_array().to_vec();
    preimage.extend_from_slice(&tag_hash.to_byte_array());
    preimage.extend_from_slice(&data);
    let hash = sha256::Hash::hash(&preimage);

    Scalar::from_be_bytes(hash.to_byte_array()).expect("scalar overflow")
}