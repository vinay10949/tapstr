use secp256k1::{Secp256k1, SecretKey, PublicKey, Keypair, schnorr::{Signature as SchnorrSignature}, Message, XOnlyPublicKey};
use secp256k1::scalar::Scalar;
use rand::rngs::OsRng;

pub struct AdaptorSignature {
    pub nonce_point: PublicKey, // R'
    pub s: SecretKey,
    pub ex: SecretKey,
    pub pubkey: PublicKey,      // even-Y adjusted
    pub message: Vec<u8>,
}

impl AdaptorSignature {
    pub fn new(secp: &Secp256k1<secp256k1::All>, keypair: &Keypair, message: &Message, t: &SecretKey) -> Self {
        let mut rng = OsRng;
        let mut k = SecretKey::new(&mut rng);
        let mut x = keypair.secret_key();

        // BIP340 parity checks
        // For R: ensure even y
        let r_full = k.public_key(secp);
        let (_, parity) = r_full.x_only_public_key();
        if parity == secp256k1::Parity::Odd {
            k = k.negate();
        }

        // For P: ensure even y
        let p_full = x.public_key(secp);
        let (_, p_parity) = p_full.x_only_public_key();
        if p_parity == secp256k1::Parity::Odd {
            x = x.negate();
        }

        // Now compute R' = R + T
        let t_pub = t.public_key(secp);
        let r_prime = r_full.combine(&t_pub).unwrap();

        // e = schnorr_challenge(R', P, m)
        let e_scalar = crate::crypto::schnorr_challenge(&r_prime, &x.public_key(secp), message.as_ref());

        // s = k + e * x
        let ex = x.mul_tweak(&e_scalar).unwrap();
        let s = k.add_tweak(&Scalar::from_be_bytes(*ex.as_ref()).unwrap()).unwrap();

        let pubkey = x.public_key(secp);

        AdaptorSignature {
            nonce_point: r_prime,
            s,
            ex,
            pubkey,
            message: message.as_ref().to_vec(),
        }
    }

    pub fn verify(&self, _secp: &Secp256k1<secp256k1::All>) -> bool {
        // For now, assume correct
        let e_scalar = crate::crypto::schnorr_challenge(&self.nonce_point, &self.pubkey, &self.message);
        // TODO: check s * G == nonce_point + e_scalar * pubkey
        true
    }

    pub fn complete(&self, t: &SecretKey) -> SecretKey {
        self.s.add_tweak(&Scalar::from_be_bytes(*t.as_ref()).unwrap()).unwrap()
    }

    pub fn extract_secret(&self, s_prime: &SecretKey) -> SecretKey {
        s_prime.add_tweak(&Scalar::from_be_bytes(*self.s.negate().as_ref()).unwrap()).unwrap()
    }

    pub fn generate_final_signature(&self, s_prime: &SecretKey) -> SchnorrSignature {
        let (r, _) = self.nonce_point.x_only_public_key();
        SchnorrSignature::from_slice(&[&r.serialize()[..], &s_prime[..]].concat()).unwrap()
    }
}

pub struct Swap {
    pub signature: AdaptorSignature,
    pub seller_nostr_pubkey: XOnlyPublicKey,
    pub buyer_bitcoin_pubkey: PublicKey,
    pub message: Message,
}

pub fn initiate_swap(secp: &Secp256k1<secp256k1::All>, seller_keypair: &Keypair, message: Message, t: SecretKey) -> Swap {
    let signature = AdaptorSignature::new(secp, seller_keypair, &message, &t);
    Swap {
        signature,
        seller_nostr_pubkey: seller_keypair.public_key().into(),
        buyer_bitcoin_pubkey: t.public_key(secp), // placeholder
        message,
    }
}

pub fn verify_swap(secp: &Secp256k1<secp256k1::All>, swap: &Swap) -> bool {
    swap.signature.verify(secp)
}

pub fn complete_swap(_secp: &Secp256k1<secp256k1::All>, swap: &Swap, t: &SecretKey) -> SchnorrSignature {
    let s_prime = swap.signature.complete(t);
    swap.signature.generate_final_signature(&s_prime)
}