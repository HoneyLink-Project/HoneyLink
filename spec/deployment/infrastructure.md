# HoneyLink Client Distribution Infrastructure

**Badges:** ` P2P Design` ` Serverless` ` No C/C++ Dependencies`

> HoneyLink P2P client distribution strategy for all platforms. No central servers, no cloud infrastructure.

## Table of Contents
- [Overview](#overview)
- [Platform-Specific Installers](#platform-specific-installers)
- [Local Storage Configuration](#local-storage-configuration)
- [Network Configuration](#network-configuration)
- [NAT Traversal Setup](#nat-traversal-setup)
- [Firewall Configuration](#firewall-configuration)

## Overview

**HoneyLink P2P Architecture = No Servers Required**

Unlike traditional applications, HoneyLink uses pure peer-to-peer architecture:
- No backend infrastructure: No Kubernetes, no databases
- No cloud hosting costs: Users run locally
- Deployment target: Client apps (Windows/macOS/Linux/iOS/Android)
- Network requirement: Only STUN servers for NAT traversal (public, free)

### Architecture Comparison

| Component | Traditional | HoneyLink P2P |
|-----------|-------------|---------------|
| Backend API | Required | Not needed  |
| Database | Required | Not needed  |
| Cloud Hosting | Required | Not needed  |
| Client App | Required | Required  |
| STUN/TURN | Optional | Required  |

## Platform-Specific Installers

### Windows
- Format: MSI (Windows Installer)
- Architecture: x64, ARM64
- Min OS: Windows 10 21H2+
- Install Path: %PROGRAMFILES%\HoneyLink\
- Data Path: %USERPROFILE%\.honeylink\
- Distribution: Direct download, winget, Chocolatey

### macOS
- Format: DMG with signed APP
- Architecture: Apple Silicon (ARM64), Intel (x64)
- Min OS: macOS 12 (Monterey)+
- Install Path: /Applications/HoneyLink.app
- Data Path: ~/.honeylink/
- Distribution: Direct download, Homebrew, App Store (future)

### Linux
- Formats: DEB, RPM, AppImage, Snap, Flatpak
- Architecture: x64, ARM64
- Min Kernel: 5.15+ (BLE support)
- Install Paths: /usr/bin/honeylink, ~/.honeylink/
- Distribution: Direct download, APT, DNF, Snap, Flatpak

### iOS
- Distribution: Apple App Store
- Min iOS: 15.0+
- Permissions: Bluetooth, Local Network, Camera
- Storage: App sandbox + Keychain

### Android
- Distribution: Google Play, F-Droid, APK
- Min Android: 10 (API 29)+
- Permissions: Bluetooth, Internet, Camera
- Storage: Private storage + Android Keystore
## Local Storage Configuration

Directory: `~/.honeylink/` (all platforms)

```
~/.honeylink/
 keys/
    device_key.pem       # X25519 private key (0600)
    device_key.pub        # X25519 public key (0644)
 trusted_peers.json        # TOFU list (0600)
 config.toml               # Configuration (0644)
 metrics/metrics.db        # Local SQLite (0644)
 logs/honeylink.log        # Logs (0644, max 50MB)
```

### Permissions (Unix)
```bash
chmod 700 ~/.honeylink/
chmod 600 ~/.honeylink/keys/device_key.pem
chmod 600 ~/.honeylink/trusted_peers.json
```

## Network Configuration

### Required Access
- mDNS: UDP 5353 (local multicast)
- BLE: Bluetooth hardware
- QUIC: UDP 7843 (default, configurable)
- WebRTC: UDP dynamic ports (ICE)

### NAT Traversal
- STUN: UDP 3478, 19302 (stun.l.google.com)
- TURN: UDP/TCP 3478 (optional, user-provided)

### Default config.toml
```toml
[device]
device_id = "auto-uuid"
device_name = "My HoneyLink Device"

[discovery]
protocols = ["mdns", "ble"]

[pairing]
methods = ["qr_code", "pin"]
pin_length = 6

[transport]
protocols = ["quic", "webrtc"]
quic_port = 7843
webrtc_stun_servers = ["stun:stun.l.google.com:19302"]

[storage]
key_storage_path = "~/.honeylink/keys"
trusted_peers_path = "~/.honeylink/trusted_peers.json"

[telemetry]
local_storage_path = "~/.honeylink/metrics"
export_enabled = false  # No server upload
```

## Firewall Configuration

### Windows
```powershell
New-NetFirewallRule -DisplayName "HoneyLink QUIC" -Direction Inbound -Protocol UDP -LocalPort 7843 -Action Allow
New-NetFirewallRule -DisplayName "HoneyLink mDNS" -Direction Inbound -Protocol UDP -LocalPort 5353 -Action Allow
```

### Linux (ufw)
```bash
sudo ufw allow 7843/udp comment "HoneyLink QUIC"
sudo ufw allow 5353/udp comment "HoneyLink mDNS"
```

### macOS
Automatic: System prompts "Allow HoneyLink to accept incoming connections?"

## CI/CD Pipeline

### Build Matrix (GitHub Actions)
- Windows: x86_64-pc-windows-msvc  .msi
- macOS: aarch64-apple-darwin  .dmg
- Linux: x86_64-unknown-linux-gnu  .deb/.rpm

### Quality Gates
- All tests pass: `cargo test`
- Linter passes: `cargo clippy -- -D warnings`
- No vulnerabilities: `cargo audit`
- Coverage  85%

### Release Steps
1. Build all platform installers
2. Sign (Authenticode/Apple/GPG)
3. Upload to GitHub Releases
4. Publish to package managers
5. Update app stores (iOS/Android)

## Security

### Code Signing
- Windows: Authenticode (DigiCert)
- macOS: Apple Developer ID + Notarization
- Linux: GPG signatures

### Supply Chain
- All dependencies: Pure Rust only (no C/C++)
- Audit: `cargo audit` in CI
- Reproducible builds

## Compliance

### Privacy (GDPR/CCPA)
- No server data collection
- All data local only
- No tracking, no analytics without opt-in

### License
- MIT License (open source)
- Repository: github.com/HoneyLink-Project/HoneyLink

### Accessibility
- WCAG 2.1 Level AA
- Keyboard navigation
- Screen reader support

### i18n
- English, Japanese, Spanish, Chinese (Phase 1)

## Definition of Done
- [x] Platform installers documented (5 platforms)
- [x] Local storage structure defined
- [x] Network/firewall configuration specified
- [x] NAT traversal (STUN/TURN) explained
- [x] CI/CD pipeline defined
- [x] Security (signing, supply chain) covered
- [x] Compliance (GDPR, licensing) documented
- [x] No server infrastructure (pure P2P)

---

**Last Updated:** 2025-01-04  
**Related:** [architecture/overview.md](../architecture/overview.md), [security/auth.md](../security/auth.md)
