# HoneyLink P2P Key Management Specification

**Badges:** ` P2P Design` ` Serverless` ` No HSM/Vault` ` No C/C++ Dependencies`

---

## 1. Overview

HoneyLink uses **TOFU (Trust On First Use)** based P2P key management. No centralized PKI, HSM, Vault, or Control Plane servers are required. All cryptographic operations use **Pure Rust** implementations (RustCrypto suite).

**Design Principles:**
- **Local-First:** All keys stored locally (`~/.honeylink/keys/`, OS Keychain)
- **Zero Server Dependency:** No HSM, no Vault clusters, no mTLS certificate authorities
- **Physical Proximity Security:** QR code / PIN / NFC pairing ensures MITM protection
- **Pure Rust Cryptography:** `x25519-dalek`, `chacha20poly1305`, `hkdf`, `ed25519-dalek` crates (no C/C++ dependencies)

---

## 2. Key Hierarchy (P2P Model)

| Layer | Purpose | Generation | Storage | Rotation |
|-------|---------|------------|---------|----------|
| **Device Identity Key** | X25519 long-term device keypair | First launch (CSPRNG) | `~/.honeylink/keys/device_key.pem` (0600) + OS Keychain | Manual (device reset) |
| **Peer Trust Public Keys** | Trusted remote device public keys | Received during pairing | `~/.honeylink/trusted_peers.json` (0600) | Removed on unpair |
| **Session Key** | ChaCha20-Poly1305 AEAD | ECDH + HKDF-SHA512 per session | Volatile memory only | Per session (ephemeral) |
| **Signing Key** | Ed25519 for audit log | First launch (CSPRNG) | OS Keychain only | Manual (device reset) |

**No Root CA / Intermediate CA:** P2P design eliminates traditional PKI hierarchy.

---

## 3. Key Generation

### 3.1. Device Identity Key (X25519)

Generated on first application launch. Storage:
- **Primary:** OS Keychain (encrypted by OS)
- **Backup:** `~/.honeylink/keys/device_key.pem` (permissions 0600, PKCS#8 format)

### 3.2. Session Key Derivation (Per Connection)

Ephemeral session key derived using ECDH + HKDF. Properties:
- Ephemeral: Generated per session, discarded on session termination
- Forward Secrecy: Past session keys cannot be recovered even if device key is compromised
- Zero Persistence: Never written to disk

---

## 4. Key Distribution (Pairing Protocols)

### 4.1. QR Code Pairing (Recommended)

**Workflow:**
1. Device A generates QR code containing device_id, public_key, expires_at
2. Device B scans QR code
3. Device B displays Device A's device_id and prompts user confirmation
4. User confirms physical proximity verification  Device B saves Device A's public key to `~/.honeylink/trusted_peers.json`
5. Device B sends its public key to Device A via BLE/QUIC
6. Device A saves Device B's public key  Pairing complete

**Security:**
- QR code visibility = physical proximity (MITM protection)
- 5-minute expiration (prevents replay attacks)
- User confirmation required

### 4.2. PIN Code Pairing (Fallback)

**Workflow:**
1. Device A displays 6-digit PIN (valid 30 seconds)
2. User enters PIN on Device B
3. Device B sends PIN + public key to Device A via BLE
4. Device A verifies PIN  Saves Device B's public key
5. Device A responds with its public key  Pairing complete

**Security:**
- 30-second validity (1,000,000 combinations, 30s window = low brute-force risk)
- BLE short range (~100m) = physical proximity

### 4.3. NFC Tap-to-Pair (Phase 2 - Future)

**Workflow:**
1. User taps Device A to Device B (NFC range ~10cm)
2. Public keys exchanged via NFC NDEF
3. Pairing complete (1-second operation)

**Security:** 10cm range = strongest MITM protection

---

## 5. Key Storage

### 5.1. Local File System

**Structure:**
```
~/.honeylink/
 keys/
    device_key.pem          # X25519 private key (0600)
    signing_key.pem         # Ed25519 private key (0600)
 trusted_peers.json          # Trusted peer public keys (0600)
 config.toml                 # Application config (0644)
 logs/
    audit.log               # Signed audit log (0600)
 metrics/
     metrics.db              # Local SQLite (0644)
```

### 5.2. OS Keychain Integration (Recommended)

**Platform-Specific Storage:**
- **Windows:** DPAPI (`CryptProtectData` API)
- **macOS:** Keychain Services (`SecItemAdd` with `kSecAttrAccessibleWhenUnlocked`)
- **Linux:** Secret Service API (GNOME Keyring / KWallet)

**Implementation:** Use `keyring` crate for cross-platform abstraction.

**Priority:**
1. OS Keychain (encrypted by OS, survives file deletion)
2. Local file (`.pem` files, 0600 permissions)

---

## 6. Key Lifecycle

### 6.1. Device Identity Key

**Generation:** First application launch  saved to OS Keychain + `~/.honeylink/keys/device_key.pem`

**Rotation:** Manual only (device reset / security incident)

**Rotation Procedure:**
1. User initiates "Reset Device Identity" in settings
2. Application generates new X25519 keypair
3. Old key overwritten with secure erase (Zeroize crate)
4. All peers unpaired (trusted_peers.json cleared)
5. User must re-pair with all devices

**Emergency Rotation:** If device key compromised:
- User initiates emergency reset on all trusted devices
- Compromised device removed from `trusted_peers.json`
- Key change detected on next connection attempt (see section 7)

### 6.2. Session Key

**Generation:** Per session (ECDH + HKDF)

**Rotation:** Automatic every session (ephemeral)

**Zeroization:** Session key memory zeroized on session termination using `zeroize` crate

### 6.3. Trusted Peer Keys

**Addition:** During pairing (QR/PIN/NFC)

**Removal:**
- User-initiated unpair: Delete entry from `trusted_peers.json`
- Key change detected: Display warning, allow user to re-verify or block

**Retention:** Indefinite (until user unpairs)

---

## 7. Key Change Detection (MITM Protection)

**Scenario:** Remote device's public key changes (device reset / key rotation / MITM attack)

**Detection Mechanism:**
1. Device A attempts connection to Device B
2. Device B's public key in handshake  saved public key in `trusted_peers.json`
3. Application displays ** Key Change Warning**

**User Actions:**
- **Block Connection:** Refuse connection, keep old key
- **Re-Verify with QR/PIN:** Perform new pairing ceremony to confirm legitimacy
- **Trust New Key:** Replace old key with new key (dangerous if MITM)

**Security:** Physical proximity verification (QR/PIN/NFC) prevents silent key substitution.

---

## 8. Revocation & Destruction

### 8.1. Peer Key Revocation

**User-Initiated Unpair:**
1. User selects device in trusted devices list
2. Clicks "Unpair"
3. Application removes entry from `trusted_peers.json`
4. Next connection attempt from that device rejected with "Untrusted Device" error

**No CRL/OCSP:** P2P design eliminates centralized revocation lists.

### 8.2. Device Key Destruction

**Scenarios:** Device decommissioning, Factory reset, Security incident

**Procedure:**
1. Overwrite `device_key.pem` with random data (3 passes)
2. Delete OS Keychain entry (`keyring` crate)
3. Clear `trusted_peers.json`
4. Secure erase audit logs
5. Zeroize all memory containing key material (`zeroize` crate)

**Compliance:** NIST SP 800-88 sanitization guidelines (Clear / Purge).

---

## 9. Cryptographic Primitives (Pure Rust)

| Primitive | Algorithm | Crate | Purpose |
|-----------|-----------|-------|---------|
| Key Exchange | X25519 ECDH | `x25519-dalek` | Session key derivation |
| Key Derivation | HKDF-SHA512 | `hkdf` | Derive session keys from shared secret |
| AEAD Encryption | ChaCha20-Poly1305 | `chacha20poly1305` | Stream encryption/authentication |
| Digital Signatures | Ed25519 | `ed25519-dalek` | Audit log signing |
| CSPRNG | OS Random | `rand_core::OsRng` | Key generation |
| Zeroization | Memory Clearing | `zeroize` | Secure key erasure |

**Zero C/C++ Dependencies:** All cryptography implemented in Pure Rust (RustCrypto project).

---

## 10. Security Properties

| Property | Mechanism | Threat Mitigation |
|----------|-----------|-------------------|
| **Forward Secrecy** | Ephemeral ECDH per session | Past sessions safe if device key compromised |
| **MITM Protection** | Physical proximity pairing (QR/PIN/NFC) | Attacker cannot impersonate without physical access |
| **Key Change Detection** | Compare saved vs. received public keys | Alerts user to potential compromise |
| **Local-Only Storage** | No network key transmission | Zero server breach risk |
| **OS-Level Encryption** | Keychain/DPAPI/Secret Service | File system access  key access |

---

## 11. Audit & Observability

### 11.1. Audit Log

**Events Logged:** Device key generation, Peer pairing (QR/PIN/NFC), Peer unpair, Key change detection warnings, Emergency key rotation

**Log Format:** JSON Lines (`~/.honeylink/logs/audit.log`, 0600 permissions)

**Signing:** Ed25519 signature using device signing key (prevents log tampering).

### 11.2. Metrics

| Metric | Type | Labels |
|--------|------|--------|
| `key_pairing_total` | Counter | method (qr, pin, nfc), result (success, failure) |
| `key_rotation_total` | Counter | reason (manual, emergency) |
| `key_change_detected_total` | Counter | action (block, re-verify, trust) |

**Storage:** Local SQLite (`~/.honeylink/metrics/metrics.db`, 500MB max, 13-month retention)

**No Server Upload:** Metrics never leave device without explicit user consent (optional OTLP export).

---

## 12. Threat Model & Mitigations

| Threat | P2P Mitigation | Traditional PKI Risk |
|--------|----------------|----------------------|
| **HSM Compromise** | N/A (no HSM) | Single point of failure |
| **CA Breach** | N/A (no CA) | All certificates revoked |
| **Vault Data Leak** | N/A (no Vault) | All secrets exposed |
| **MITM Attack** | Physical proximity pairing | Relies on trusted CA chain |
| **Key Theft** | OS Keychain + 0600 permissions | HSM still vulnerable to insider |
| **Replay Attack** | Ephemeral session keys | Depends on nonce management |

**Conclusion:** P2P TOFU model eliminates entire classes of server-side attacks.

---

## 13. Compliance & Best Practices

**Standards:** NIST SP 800-56A (ECDH), NIST SP 800-108 (Key Derivation), NIST SP 800-88 (Secure Erasure), RFC 7748 (X25519 / Ed25519), RFC 8439 (ChaCha20-Poly1305)

**Best Practices:**
-  Use CSPRNG (`OsRng`) for all key generation
-  Zeroize keys after use (`zeroize` crate)
-  Set file permissions to 0600 for key files
-  Use OS Keychain when available
-  Implement key change detection with user confirmation
-  Sign audit logs to prevent tampering
-  Never transmit private keys over network
-  Never store keys in plaintext logs/metrics
-  Never use C/C++ cryptographic libraries

---

## 14. Migration from Old Server-Centric Design

**Removed Components:**
-  Root Trust Anchor (RTA) + HSM
-  Intermediate CA + Vault clusters
-  Control Plane registration API
-  mTLS certificate chains
-  CRL/OCSP revocation services

**Replaced With:**
-  Local device identity keys (X25519)
-  TOFU trust model (`trusted_peers.json`)
-  QR/PIN/NFC pairing
-  OS Keychain integration
-  Pure Rust cryptography

**Backup:** See `key-management-old-server.md` for historical server-centric design.

---

## 15. Related Specifications

- **Authentication:** `spec/security/auth.md` (TOFU pairing protocols)
- **Encryption:** `spec/security/encryption.md` (ChaCha20-Poly1305 AEAD)
- **Architecture:** `spec/architecture/overview.md` (P2P design principles)
- **Crypto Module:** `crates/crypto/README.md` (Implementation details)

---

**Version:** 2.0 (P2P)  
**Last Updated:** 2025-01-15  
**Changelog:**
- 2025-01-15: Complete P2P rewrite (removed HSM/Vault/CA, added TOFU/QR/PIN/NFC)
- 2024-XX-XX: Original server-centric design (archived as `key-management-old-server.md`)