//! P2P File Transfer Example
//!
//! Demonstrates multi-stream file transfer with QoS:
//! - High-priority control stream
//! - Normal-priority data streams
//! - Progress tracking
//! - Bandwidth allocation
//!
//! This example shows the API pattern for implementing file transfer.
//! In production, you would add chunking, resume capability, and error recovery.

use honeylink_transport::{
    manager::TransportManager,
    protocol::{ProtocolStrategy, ProtocolType, StreamPriority},
    quic::QuicTransport,
};
use std::sync::Arc;

/// File metadata sent over control stream
#[derive(Debug)]
struct FileMetadata {
    name: String,
    size: u64,
    chunks: usize,
}

impl FileMetadata {
    fn new(name: &str, size: u64, chunk_size: usize) -> Self {
        let chunks = (size as usize + chunk_size - 1) / chunk_size;
        Self {
            name: name.to_string(),
            size,
            chunks,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 HoneyLink P2P File Transfer Example");
    println!("=======================================\n");

    // Configuration
    const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB chunks
    const FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB file (simulated)
    
    let metadata = FileMetadata::new("example_file.bin", FILE_SIZE, CHUNK_SIZE);
    println!("📋 File Transfer Configuration:");
    println!("   - File: {}", metadata.name);
    println!("   - Size: {} MB", metadata.size / (1024 * 1024));
    println!("   - Chunks: {}", metadata.chunks);
    println!();

    // Step 1: Setup transport
    println!("1. Setting up transport...");
    let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
    let quic = Arc::new(QuicTransport::new()?);
    transport.register_protocol(ProtocolType::Quic, quic).await;
    
    // Step 2: Connect to peer
    let peer_addr = "127.0.0.1:8081".parse()?;
    println!("2. Connecting to peer at {}...", peer_addr);
    
    match transport.connect(peer_addr).await {
        Ok(connection) => {
            println!("   ✅ Connected!");
            
            // Step 3: Open control stream (high priority, low bandwidth)
            println!("3. Opening control stream...");
            match transport
                .open_prioritized_stream(&connection, StreamPriority::High, 100)
                .await
            {
                Ok(_control_stream) => {
                    println!("   ✅ Control stream ready");
                    
                    // Send file metadata (conceptual)
                    println!("   📤 Sending metadata (conceptual):");
                    let metadata_msg = format!("FILE:{}:{}:{}", 
                        metadata.name, metadata.size, metadata.chunks);
                    println!("      Message: {}", metadata_msg);
                }
                Err(e) => {
                    eprintln!("   ❌ Failed to open control stream: {}", e);
                    return Ok(());
                }
            }
            
            // Step 4: Open data streams (normal priority, higher bandwidth)
            println!("4. Opening {} data streams...", metadata.chunks.min(4));
            let concurrent_streams = metadata.chunks.min(4); // Max 4 concurrent chunks
            let bandwidth_per_stream = 10000 / concurrent_streams; // 10 Mbps total
            
            for i in 0..concurrent_streams {
                match transport
                    .open_prioritized_stream(
                        &connection, 
                        StreamPriority::Normal, 
                        bandwidth_per_stream as u32
                    )
                    .await
                {
                    Ok(_data_stream) => {
                        println!("   ✅ Data stream {} opened ({} kbps)", i, bandwidth_per_stream);
                        
                        // Simulate sending chunk (conceptual)
                        let chunk_size_kb = CHUNK_SIZE.min(FILE_SIZE as usize) / 1024;
                        println!("   📤 Chunk {} ready to send ({} KB)", i, chunk_size_kb);
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to open data stream {}: {}", i, e);
                    }
                }
            }
            
            // Step 5: Monitor QoS
            println!("5. QoS Statistics:");
            let stats = transport.qos_stats().await;
            println!("   📊 Total streams: {}", stats.total_streams);
            println!("   📊 Allocated bandwidth: {} kbps ({} Mbps)", 
                stats.allocated_bandwidth_kbps,
                stats.allocated_bandwidth_kbps / 1000);
            println!("   📊 Available bandwidth: {} kbps", stats.available_bandwidth_kbps);
            
            // Calculate transfer time estimate
            let transfer_time_seconds = (FILE_SIZE as f64 * 8.0) / 
                (stats.allocated_bandwidth_kbps as f64 * 1000.0);
            println!("   ⏱️  Estimated transfer time: {:.2} seconds", transfer_time_seconds);
            
            // Step 6: Close connection
            println!("6. Closing connection...");
            if let Err(e) = connection.close().await {
                eprintln!("   ❌ Failed to close: {}", e);
            } else {
                println!("   ✅ Transfer complete!");
            }
        }
        Err(e) => {
            eprintln!("   ❌ Connection failed: {}", e);
            eprintln!("\n💡 Note: This example requires a QUIC server running at 127.0.0.1:8081");
            eprintln!("\n   Real-world file transfer would include:");
            eprintln!("   - Chunk checksums for integrity verification");
            eprintln!("   - Resume capability (tracking sent chunks)");
            eprintln!("   - Error recovery and retransmission");
            eprintln!("   - Progress callbacks for UI updates");
            eprintln!("   - Bandwidth throttling based on network conditions");
        }
    }
    
    println!("\n✨ Example complete!");
    println!("\n📖 Key Takeaways:");
    println!("   - Control streams use high priority + low bandwidth");
    println!("   - Data streams use normal priority + high bandwidth");
    println!("   - QoS scheduler allocates bandwidth fairly");
    println!("   - Multiple concurrent streams improve throughput");
    
    Ok(())
}
