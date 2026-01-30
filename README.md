### 🔢 Mathematical Foundations
- **Schnorr Signatures**: Complete mathematical formulation
  - Key generation: `P = x · G`
  - Signing: `s = k + e · x`
  - Verification: `s · G = R + e · P`
- **BIP340 Details**: Tagged hashing, parity handling
- **Adaptor Signatures**: How they enable atomic swaps
  - Modified signing with adaptor point `T = t · G`
  - Signature completion: `s = s' - t`
  - Secret extraction: `t = s' - s`

### 🏗️ Implementation Details
- **crypto.rs**: Core cryptographic utilities
- **adaptor.rs**: Adaptor signature implementation
- **bitcoin_utils.rs**: Taproot transaction handling
- **nostr_utils.rs**: Nostr event management

### 🌐 Web Interface
- Interactive demo with technical details display
- Shows actual cryptographic values during execution

