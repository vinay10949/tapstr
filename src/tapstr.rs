use bitcoin::{Amount, Address, Transaction, TxOut, Txid, ScriptBuf};
use secp256k1::{Secp256k1, XOnlyPublicKey};

#[derive(Debug)]
pub struct Buyer {
    pub locking_tx: Option<Transaction>,
    pub output_script: Option<ScriptBuf>,
    // Add more fields as needed, e.g., adaptor_sig, keys, etc.
}

#[derive(Debug)]
pub struct Seller {
    // Fields for seller, e.g., keys, adaptor_sig
}

impl Buyer {
    pub fn new() -> Self {
        Buyer {
            locking_tx: None,
            output_script: None,
        }
    }

    pub fn create_locking_transaction(
        &mut self,
        prev_txid: Txid,
        prev_vout: u32,
        amount: Amount,
        recipient_pubkey: XOnlyPublicKey,
    ) {
        self.locking_tx = Some(crate::bitcoin_utils::create_locking_transaction(
            prev_txid,
            prev_vout,
            amount,
            recipient_pubkey,
        ));
        self.output_script = Some(ScriptBuf::new_p2tr(&Secp256k1::new(), recipient_pubkey, None));
    }

    pub fn create_spending_transaction(
        &self,
        prev_txid: Txid,
        prev_vout: u32,
        prev_txout: &TxOut,
        amount: Amount,
        recipient: Address,
        signer_keypair: &secp256k1::Keypair,
    ) -> Result<Transaction, anyhow::Error> {
        crate::bitcoin_utils::create_spending_transaction(
            prev_txid,
            prev_vout,
            prev_txout,
            amount,
            recipient,
            signer_keypair,
        )
    }

    pub fn verify_adaptor_signature(
        &self,
        adaptor_sig: &crate::adaptor::AdaptorSignature,
        secp: &Secp256k1<secp256k1::All>,
    ) -> bool {
        adaptor_sig.verify(secp)
    }
}

impl Seller {
    pub fn new() -> Self {
        Seller {}
    }

    // Add methods for seller as needed
}