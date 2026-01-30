use bitcoin::{
    Amount, Address, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness, absolute::LockTime,
    key::Keypair, sighash::{SighashCache, Prevouts, TapSighashType}, transaction::Version,
};
use secp256k1::{Secp256k1, Scalar, XOnlyPublicKey, Message};
use anyhow::Result;
use rand::rngs::OsRng;

/// Create a P2TR locking transaction
pub fn create_locking_transaction(
    prev_txid: Txid,
    prev_vout: u32,
    amount: Amount,
    recipient_pubkey: XOnlyPublicKey,
) -> Transaction {
    let secp = Secp256k1::new();
    let script_pubkey = ScriptBuf::new_p2tr(&secp, recipient_pubkey, None);

    Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: prev_txid, vout: prev_vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: amount,
            script_pubkey,
        }],
    }
}

/// Create a spending transaction for P2TR using Schnorr signature
pub fn create_spending_transaction(
    prev_txid: Txid,
    prev_vout: u32,
    prev_txout: &TxOut,
    amount: Amount,
    recipient: Address,
    signer_keypair: &Keypair,
) -> Result<Transaction> {
    let secp = Secp256k1::new();
    let mut tx = Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: prev_txid, vout: prev_vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: amount,
            script_pubkey: recipient.script_pubkey(),
        }],
    };

    let prevouts = Prevouts::All(&[prev_txout]);
    let sighash = SighashCache::new(&tx).taproot_key_spend_signature_hash(0, &prevouts, TapSighashType::Default)?;

    let mut rng = OsRng;
    let sig = secp.sign_schnorr_with_rng(&Message::from(sighash), signer_keypair, &mut rng);

    tx.input[0].witness = Witness::from_slice(&[sig.as_ref()]);

    Ok(tx)
}

/// Create Taproot output tweaking key with commitment
pub fn create_nostr_signature_lock_script(commitment: [u8; 32], internal_key: XOnlyPublicKey) -> Result<XOnlyPublicKey> {
    let secp = Secp256k1::new();
    let tweak = Scalar::from_be_bytes(commitment)?;
    let (tweaked_key, _) = internal_key.add_tweak(&secp, &tweak)?;
    Ok(tweaked_key)
}