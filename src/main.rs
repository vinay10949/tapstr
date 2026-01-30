use axum::{
    routing::{get, post},
    Router, response::Html,
};
use secp256k1::{Secp256k1, SecretKey, Message, Keypair, hashes::sha256};
use bitcoin::{Amount, Txid, hashes::Hash};
use rand::rngs::OsRng;
use std::str::FromStr;
use ::tapstr::adaptor;
use ::tapstr::bitcoin_utils;
use tapstr::tapstr;

struct SwapState {
    logs: Vec<String>,
    status: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(get_ui))
        .route("/start", post(start_swap));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Atomic Swap Demo UI running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn get_ui() -> Html<String> {
    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Atomic Swap Demo</title>
    <style>
    body {
        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: #333;
        margin: 0;
        padding: 0;
        min-height: 100vh;
        display: flex;
        justify-content: center;
        align-items: center;
    }
    .container {
        background: white;
        border-radius: 15px;
        box-shadow: 0 10px 30px rgba(0,0,0,0.3);
        padding: 40px;
        max-width: 600px;
        width: 90%;
        text-align: center;
    }
    h1 {
        color: #4a5568;
        margin-bottom: 20px;
        font-size: 2.5em;
    }
    .status {
        font-size: 1.2em;
        margin: 20px 0;
        padding: 10px;
        border-radius: 10px;
        background: #e6fffa;
        color: #2d3748;
    }
    .description {
        font-size: 1.1em;
        margin: 20px 0;
        color: #4a5568;
    }
    button {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        border: none;
        padding: 15px 30px;
        font-size: 1.2em;
        border-radius: 25px;
        cursor: pointer;
        transition: transform 0.2s, box-shadow 0.2s;
        margin-top: 20px;
    }
    button:hover {
        transform: translateY(-2px);
        box-shadow: 0 5px 15px rgba(0,0,0,0.2);
    }
    .logo {
        font-size: 3em;
        margin-bottom: 20px;
    }
    </style>
    </head>
    <body>
    <div class="container">
    <div class="logo">⚡🔄📜</div>
    <h1>Atomic Swap Demo</h1>
    <p class="status">Status: Ready</p>
    <p class="description">Experience the power of atomic swaps between Bitcoin and Nostr using Taproot and Schnorr adaptor signatures.</p>
    <form action="/start" method="post">
    <button type="submit">🚀 Start Swap Demo</button>
    </form>
    </div>
    </body>
    </html>
    "#;
    Html(html.to_string())
}

#[axum::debug_handler]
async fn start_swap() -> Html<String> {
    let mut logs = vec![];

    let log = |logs: &mut Vec<String>, msg: String| {
        println!("{}", msg);
        logs.push(msg.clone());
    };

    log(&mut logs, "Starting atomic swap between Bitcoin and Nostr using Taproot and Schnorr adaptor signatures".to_string());

    let secp = Secp256k1::new();
    let mut rng = OsRng;

    // Establish Seller and Buyer
    let seller_nostr_keypair = Keypair::new(&secp, &mut rng);
    let buyer_bitcoin_keypair = Keypair::new(&secp, &mut rng);

    let _seller = tapstr::Seller::new();
    let mut buyer = tapstr::Buyer::new();

    log(&mut logs, "Seller and Buyer established.".to_string());

    // Step 1: Seller creates a Nostr event and a commitment
    let content = "Buy this digital item".to_string();
    let message = Message::from_hashed_data::<sha256::Hash>(content.as_bytes());
    let t = SecretKey::new(&mut rng);
    let adaptor_sig = adaptor::AdaptorSignature::new(&secp, &seller_nostr_keypair, &message, &t);
    log(&mut logs, "Seller created adaptor signature.".to_string());

    // Commitment: hash of the adaptor s for simplicity
    let commitment = sha256::Hash::const_hash(adaptor_sig.s.as_ref());
    log(&mut logs, format!("Seller created commitment: {:?}", commitment));

    // Step 2: Buyer creates a lock transaction
    // Simulated previous tx
    let prev_txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let prev_vout = 0;
    let amount = Amount::from_sat(10000);

    // Tweaked key for locking
    let seller_pubkey = seller_nostr_keypair.x_only_public_key().0;
    let tweaked_key = bitcoin_utils::create_nostr_signature_lock_script(*commitment.as_byte_array(), seller_pubkey).unwrap();
    buyer.create_locking_transaction(prev_txid, prev_vout, amount, tweaked_key);
    log(&mut logs, "Buyer created locking transaction.".to_string());

    // Step 3: Buyer verifies the adaptor signature
    let verified = adaptor_sig.verify(&secp);
    log(&mut logs, format!("Buyer verified adaptor signature: {}", verified));

    // Step 4: Seller completes the signature and reveals the secret
    // In the protocol, seller provides the real Nostr sig
    let s_prime = adaptor_sig.complete(&t);
    let _final_nostr_sig = adaptor_sig.generate_final_signature(&s_prime);
    log(&mut logs, "Seller completed the Nostr signature and revealed it.".to_string());

    // Step 5: Buyer verifies the secret
    // Buyer extracts t from the sig
    let extracted_t = adaptor_sig.extract_secret(&s_prime);
    let t_matches = extracted_t == t;
    log(&mut logs, format!("Buyer verified the secret: {}", t_matches));

    log(&mut logs, "Atomic swap completed successfully!".to_string());

    // Collect technical details
    let seller_pubkey = seller_nostr_keypair.x_only_public_key().0;
    let buyer_pubkey = buyer_bitcoin_keypair.x_only_public_key().0;
    let adaptor_details = format!("Nonce Point: {:?}, s: {:?}, ex: {:?}", adaptor_sig.nonce_point, adaptor_sig.s, adaptor_sig.ex);
    let revealed_secret = format!("t: {:?}", t);
    let math_details = r#"
    <h4>Schnorr Signature Mathematics:</h4>
    <p><strong>Standard Schnorr:</strong> s = k + e * x, where e = H(R || P || m), R = k * G</p>
    <p><strong>Verification:</strong> s * G = R + e * P</p>
    <p><strong>Adaptor Signature:</strong> s' = s + t, where t is the secret</p>
    <p><strong>Challenge:</strong> e = H(R' || P || m) where R' = R + T, T = t * G</p>
    <p><strong>BIP340:</strong> Uses tagged hash for e to prevent collision attacks</p>
    "#;

    let logs_html = logs.iter().enumerate().map(|(i, log)| format!("<div class='log-item' style='animation-delay: {}s'><span class='step'>{}</span> {}</div>", i as f32 * 0.3, i + 1, log)).collect::<String>();
    let details_html = format!(r#"
    <div class="details">
    <h3>🔐 Technical Details</h3>
    <div class="detail-section">
    <h4>Keys:</h4>
    <p><strong>Seller Public Key:</strong> {}</p>
    <p><strong>Buyer Public Key:</strong> {}</p>
    </div>
    <div class="detail-section">
    <h4>Adaptor Signature:</h4>
    <p>{}</p>
    </div>
    <div class="detail-section">
    <h4>Revealed Secret:</h4>
    <p>{}</p>
    </div>
    <div class="detail-section">
    {}
    </div>
    </div>
    "#, seller_pubkey, buyer_pubkey, adaptor_details, revealed_secret, math_details);
    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Atomic Swap Demo - Results</title>
    <style>
    body {{
        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: #333;
        margin: 0;
        padding: 0;
        min-height: 100vh;
        display: flex;
        justify-content: center;
        align-items: center;
    }}
    .container {{
        background: white;
        border-radius: 15px;
        box-shadow: 0 10px 30px rgba(0,0,0,0.3);
        padding: 40px;
        max-width: 800px;
        width: 90%;
        text-align: center;
    }}
    h1 {{
        color: #4a5568;
        margin-bottom: 20px;
        font-size: 2.5em;
    }}
    .status {{
        font-size: 1.2em;
        margin: 20px 0;
        padding: 10px;
        border-radius: 10px;
        background: #c6f6d5;
        color: #2d3748;
    }}
    .logs {{
        text-align: left;
        margin: 30px 0;
        max-height: 400px;
        overflow-y: auto;
        border: 1px solid #e2e8f0;
        border-radius: 10px;
        padding: 20px;
        background: #f7fafc;
    }}
    .log-item {{
        margin-bottom: 10px;
        padding: 10px;
        border-radius: 5px;
        background: white;
        border-left: 4px solid #667eea;
        animation: fadeInUp 0.6s ease forwards;
        opacity: 0;
        transform: translateY(20px);
    }}
    .step {{
        font-weight: bold;
        color: #667eea;
        margin-right: 10px;
    }}
    @keyframes fadeInUp {{
        to {{
            opacity: 1;
            transform: translateY(0);
        }}
    }}
    a {{
        display: inline-block;
        background: #667eea;
        color: white;
        padding: 10px 20px;
        text-decoration: none;
        border-radius: 5px;
        margin-top: 20px;
        transition: background 0.2s;
    }}
    a:hover {{
        background: #5a67d8;
    }}
    .success {{
        color: #38a169;
        font-size: 1.5em;
        margin: 20px 0;
    }}
    .details {{
        margin-top: 40px;
        text-align: left;
        background: #f8f9fa;
        padding: 20px;
        border-radius: 10px;
        border: 1px solid #e2e8f0;
    }}
    .detail-section {{
        margin-bottom: 20px;
    }}
    .detail-section h4 {{
        color: #4a5568;
        margin-bottom: 10px;
    }}
    .detail-section p {{
        margin: 5px 0;
        font-family: monospace;
        background: #ffffff;
        padding: 5px;
        border-radius: 3px;
        word-break: break-all;
    }}
    </style>
    </head>
    <body>
    <div class="container">
    <h1>🎉 Swap Completed!</h1>
    <p class="status">Status: Completed</p>
    <div class="success">✅ Atomic swap between Bitcoin and Nostr successful!</div>
    <div class="logs">
    <h3>📋 Execution Logs:</h3>
    {}
    </div>
    {}
    <a href="/">🔄 Try Again</a>
    </div>
    </body>
    </html>
    "#, logs_html, details_html);
    Html(html)
}