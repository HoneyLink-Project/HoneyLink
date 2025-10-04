//! Simple mDNS discovery example
//!
//! This example demonstrates basic device discovery:
//! 1. Announce device via mDNS
//! 2. Discover nearby devices for 10 seconds
//! 3. Print discovered devices
//! 4. Graceful shutdown
//!
//! Usage:
//!   cargo run --example simple_discovery

use honeylink_discovery::DiscoveryService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 HoneyLink Simple Discovery Example");
    println!("=====================================\n");

    // Get device info from environment or use defaults
    let device_id = std::env::var("DEVICE_ID").unwrap_or_else(|_| "DEV-EXAMPLE-001".to_string());
    let device_name =
        std::env::var("DEVICE_NAME").unwrap_or_else(|_| "Example Device".to_string());
    let device_type = std::env::var("DEVICE_TYPE").unwrap_or_else(|_| "desktop".to_string());

    println!("📱 Device Information:");
    println!("   ID:   {}", device_id);
    println!("   Name: {}", device_name);
    println!("   Type: {}", device_type);
    println!();

    // Create discovery service
    println!("🚀 Starting discovery service...");
    let mut service = DiscoveryService::new(&device_id, &device_name, &device_type)?;

    // Start announcing and browsing
    service.start().await?;
    println!("✅ Service started (announcing via mDNS)");
    println!();

    // Discover devices
    println!("🔎 Discovering nearby devices (10 seconds)...");
    println!("   (Run this example on multiple devices to see them discover each other)");
    println!();

    let devices = service.discover_devices(10).await?;

    println!("\n📊 Discovery Results:");
    println!("   Found {} device(s)", devices.len());
    println!();

    if devices.is_empty() {
        println!("   No devices found.");
        println!("   💡 Tip: Run this example on another device to see discovery in action!");
    } else {
        for (i, device) in devices.iter().enumerate() {
            println!("   Device {}:", i + 1);
            println!("     ID:        {}", device.device_id);
            println!("     Name:      {}", device.device_name);
            println!("     Type:      {:?}", device.device_type);
            println!("     Version:   {}", device.version);
            println!("     Addresses: {:?}", device.addresses);
            println!("     Port:      {}", device.port);
            if let Some(rssi) = device.rssi {
                println!("     RSSI:      {} dBm", rssi);
            }
            println!();
        }
    }

    // Graceful shutdown
    println!("🛑 Shutting down...");
    service.stop().await?;
    println!("✅ Service stopped");

    Ok(())
}
