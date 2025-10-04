# Physical Adapter Layer

P2P protocol integration for device discovery and transport.

## ⚠️ P2P Design

This crate implements **pure P2P protocols** with no server dependencies:

- ✅ **mDNS:** Local network device discovery (`_honeylink._tcp.local.`)
- ✅ **BLE:** Bluetooth Low Energy advertising for nearby devices
- ✅ **QUIC:** UDP transport with quinn crate (default port 7843)
- ✅ **WebRTC:** Data channels with ICE candidates (STUN/TURN for NAT traversal)
- ❌ **No gRPC/REST servers:** Direct Rust crate usage (mdns-sd, btleplug, quinn, webrtc)

## Features

- **mDNS Discovery:** Find peers on local network (UDP multicast 5353)
- **BLE Advertising:** Find peers via Bluetooth (range ~100m)
- **QUIC Transport:** Low-latency P2P communication (P95 < 20ms)
- **WebRTC Fallback:** NAT traversal for internet connectivity (STUN/TURN)
- **Hot Swap:** Switch between protocols without session interruption
- **Power Modes:** 4-tier power management (UltraLow/Low/Normal/High)

## Usage

```rust
use honeylink_physical_adapter::{PhysicalAdapter, DiscoveryProtocol};

#[tokio::main]
async fn main() {
    // mDNS discovery
    let mut adapter = PhysicalAdapter::new(DiscoveryProtocol::MDns);
    adapter.start_discovery().await.unwrap();
    
    // Wait for peer
    let peer = adapter.wait_for_peer().await.unwrap();
    println!("Found peer: {}", peer.device_name);
    
    // Establish QUIC connection
    adapter.connect(&peer, DiscoveryProtocol::Quic).await.unwrap();
}
```

## Discovery Protocols

### mDNS (Local Network)
- **Service Type:** `_honeylink._tcp.local.`
- **Port:** UDP 5353 (multicast)
- **Range:** Same subnet (~300m)
- **Latency:** P95 < 100ms
- **Crate:** `mdns-sd` (Pure Rust)

### BLE (Bluetooth Low Energy)
- **Service UUID:** HoneyLink-specific UUID
- **Range:** ~100m (open space)
- **Latency:** P95 < 200ms
- **Power:** ~10mA (UltraLow mode)
- **Crate:** `btleplug` (Pure Rust)

### QUIC (UDP Transport)
- **Port:** 7843 (user-configurable)
- **Protocol:** QUIC over UDP
- **Latency:** P95 < 20ms
- **Bandwidth:** 100-1000Mbps
- **Crate:** `quinn` (Pure Rust)

### WebRTC (NAT Traversal)
- **ICE:** STUN/TURN for NAT traversal
- **STUN Servers:** stun.l.google.com:19302, stun.l.google.com:3478
- **TURN:** Optional user-configured servers
- **Success Rate:** 95% NAT traversal
- **Crate:** `webrtc` (Pure Rust)

## Power Modes

| Mode | Power | Throughput | Latency | Protocol |
|------|-------|------------|---------|----------|
| **UltraLow** | ~10mA | < 1Mbps | P95 < 200ms | BLE only |
| **Low** | ~50mA | 10-50Mbps | P95 < 50ms | mDNS + QUIC |
| **Normal** | ~200mA | 100-500Mbps | P95 < 20ms | mDNS + QUIC |
| **High** | ~500mA | 500-1000Mbps | P95 < 10ms | mDNS + QUIC + WebRTC |

## NAT Traversal

```rust
// Configure STUN/TURN for internet connectivity
let config = QuicConfig {
    stun_servers: vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun.l.google.com:3478".to_string(),
    ],
    turn_server: None,  // Optional user-configured TURN
    ..Default::default()
};
```

## Architecture Consistency

- **No REST/gRPC servers:** All communication via P2P protocols
- **No OAuth2/bearer tokens:** Authentication via TOFU (Trust On First Use)
- **Local-first:** Works offline on local network (mDNS/BLE)
- **Pure Rust:** No C/C++ dependencies

## Historical Note

**Previous versions** mentioned "gRPC/REST API integration" for WiFi/5G/THz adapters. This was **removed** to align with P2P design. Current implementation uses direct Rust crates (mdns-sd, btleplug, quinn, webrtc).

## Related Documentation

- [spec/modules/physical-adapter.md](../../spec/modules/physical-adapter.md) - P2P protocol specification
- [spec/deployment/infrastructure.md](../../spec/deployment/infrastructure.md) - NAT traversal setup
