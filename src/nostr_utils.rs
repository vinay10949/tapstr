use nostr::{Event, Keys, Kind, Tag, EventBuilder};
use secp256k1::SecretKey;

pub fn create_signed_event(keys: &Keys, kind: Kind, content: &str, tags: Vec<Tag>) -> Event {
    EventBuilder::new(kind, content, tags).to_event(keys).unwrap()
}

pub fn extract_secret_from_signature(adaptor_sig: &crate::adaptor::AdaptorSignature, nostr_sig: &secp256k1::schnorr::Signature) -> SecretKey {
    let sig_bytes = nostr_sig.as_ref();
    let s_prime = SecretKey::from_slice(&sig_bytes[32..64]).unwrap();
    adaptor_sig.extract_secret(&s_prime)
}