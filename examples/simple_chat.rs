//! Simple P2P Chat Example
//!
//! Demonstrates basic usage of HoneyLink Transport API:
//! - Connection establishment
//! - Prioritized stream creation
//! - QoS statistics monitoring
//!
//! This is a conceptual example showing API patterns.
//! Stream read/write operations require implementing AsyncRead/AsyncWrite
//! for the Stream trait in production code.

use honeylink_transport::{
    manager::TransportManager,
    protocol::{ProtocolStrategy, ProtocolType, StreamPriority},
    quic::QuicTransport,
    logging::init_tracing,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    init_tracing();

    info!("🐝 HoneyLink P2P Chat Example");
    info!("==============================");

    // Step 1: Create Transport Manager
    info!("1. Creating Transport Manager...");
    let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);

    // Step 2: Register QUIC protocol
    info!("2. Registering QUIC protocol...");
    let quic = Arc::new(QuicTransport::new()?);
    transport.register_protocol(ProtocolType::Quic, quic).await;

    // Step 3: Connect to peer
    // In a real application, this would be the discovered peer address from mDNS
    let peer_addr = "127.0.0.1:8080".parse()?;
    info!("3. Connecting to peer at {}", peer_addr);

    match transport.connect(peer_addr).await {
        Ok(connection) => {
            info!("   ✅ Connected successfully!");

            // Step 4: Open a high-priority stream for chat messages
            info!("4. Opening high-priority stream for chat...");
            match transport
                .open_prioritized_stream(&connection, StreamPriority::High, 5000)
                .await
            {
                Ok(_stream) => {
                    info!("   ✅ Stream opened (5 Mbps bandwidth allocated)");

                    // Step 5: In production, you would send/receive data like this:
                    println!("5. Sending chat message (conceptual)...");
                    println!("   Code pattern:");
                    println!("   ```rust");
                    println!("   // Convert stream to tokio::io types");
                    println!("   let message = b\"Hello from HoneyLink P2P!\";");
                    println!("   stream.write_all(message).await?;");
                    println!("   ```");

                    // Step 6: Reading response (conceptual)
                    println!("6. Reading response (conceptual)...");
                    println!("   Code pattern:");
                    println!("   ```rust");
                    println!("   let mut buffer = vec![0u8; 1024];");
                    println!("   let n = stream.read(&mut buffer).await?;");
                    println!("   println!(\"Received: {{}}\", String::from_utf8_lossy(&buffer[..n]));");
                    println!("   ```");
                }
                Err(e) => {
                    eprintln!("   ❌ Failed to open stream: {}", e);
                }
            }

            // Step 7: Get QoS statistics
            println!("7. Checking QoS statistics...");
            let stats = transport.qos_stats().await;
            println!("   📊 QoS Stats:");
            println!("      - Total streams: {}", stats.total_streams);
            println!("      - Allocated bandwidth: {} kbps", stats.allocated_bandwidth_kbps);
            println!("      - Available bandwidth: {} kbps", stats.available_bandwidth_kbps);

            // Step 8: Close connection
            println!("8. Closing connection...");
            if let Err(e) = connection.close().await {
                eprintln!("   ❌ Failed to close connection: {}", e);
            } else {
                println!("   ✅ Connection closed gracefully");
            }
        }
        Err(e) => {
            eprintln!("   ❌ Connection failed: {}", e);
            eprintln!("\n💡 Note: This example requires a QUIC server running at 127.0.0.1:8080");
            eprintln!("   In a real application, you would:");
            eprintln!("   1. Discover peers using mDNS (honeylink-discovery crate)");
            eprintln!("   2. Display nearby devices in UI");
            eprintln!("   3. Establish P2P connection after user selection");
        }
    }

    println!("\n✨ Example complete!");
    Ok(())
}
