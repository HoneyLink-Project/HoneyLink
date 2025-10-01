// HoneyLink Key Generation CLI Tool
//
// Provides command-line interface for:
// - X25519 keypair generation
// - HKDF key derivation (hierarchical)
// - Key rotation management
// - Key export/import in Base64 format
//
// Zero C/C++ dependencies - pure Rust using RustCrypto crates.

use base64::Engine;
use clap::{Parser, Subcommand};
use honeylink_crypto::{KeyHierarchy, KeyRotationManager, KeyScope};
use std::process;

#[derive(Parser)]
#[command(name = "honeylink-keygen")]
#[command(about = "HoneyLink key generation and management tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new root key
    GenerateRoot {
        /// Output file path (optional, prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Derive child key from parent
    Derive {
        /// Parent key in Base64 format
        #[arg(short, long)]
        parent: String,

        /// Target scope (root, device, session, stream)
        #[arg(short, long)]
        scope: String,

        /// Output file path (optional, prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Create a new key rotation manager with default policies
    InitRotation {
        /// Output file path for rotation state
        #[arg(short, long)]
        output: String,
    },

    /// Add a new key version to rotation manager
    AddVersion {
        /// Rotation state file
        #[arg(short, long)]
        state: String,

        /// Key scope (root, device, session, stream)
        #[arg(short = 'c', long)]
        scope: String,

        /// Key material in Base64 format
        #[arg(short, long)]
        key: String,
    },

    /// Check rotation status
    Status {
        /// Rotation state file
        #[arg(short, long)]
        state: String,
    },

    /// Show key hierarchy demonstration
    Demo,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::GenerateRoot { output } => {
            generate_root(output)?;
        }
        Commands::Derive { parent, scope, output } => {
            derive_key(&parent, &scope, output)?;
        }
        Commands::InitRotation { output } => {
            init_rotation(&output)?;
        }
        Commands::AddVersion { state, scope, key } => {
            add_version(&state, &scope, &key)?;
        }
        Commands::Status { state } => {
            show_status(&state)?;
        }
        Commands::Demo => {
            run_demo()?;
        }
    }

    Ok(())
}

fn generate_root(output: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use rand::Rng;
    use rand::rngs::OsRng;

    // Generate X25519 keypair using OS random
    let mut secret_bytes = [0u8; 32];
    OsRng.fill(&mut secret_bytes);
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&secret_bytes);    if let Some(path) = output {
        std::fs::write(&path, encoded)?;
        eprintln!("✅ Root key generated and saved to: {}", path);
    } else {
        println!("{}", encoded);
    }

    Ok(())
}

fn derive_key(
    parent_b64: &str,
    scope_str: &str,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse parent key
    let parent_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parent_b64)?;
    if parent_bytes.len() != 32 {
        return Err("Parent key must be 32 bytes".into());
    }
    let mut parent_key = [0u8; 32];
    parent_key.copy_from_slice(&parent_bytes);

    // Parse scope
    let scope = match scope_str.to_lowercase().as_str() {
        "root" => KeyScope::Root,
        "device" | "device-master" => KeyScope::DeviceMaster,
        "session" => KeyScope::Session,
        "stream" => KeyScope::Stream,
        _ => return Err(format!("Invalid scope: {}", scope_str).into()),
    };

    // Derive child key
    let hierarchy = KeyHierarchy::from_bytes(parent_key);
    let child_key = hierarchy.derive_simple(scope)?;
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(child_key);

    if let Some(path) = output {
        std::fs::write(&path, encoded)?;
        eprintln!("✅ Derived {:?} key and saved to: {}", scope, path);
    } else {
        println!("{}", encoded);
    }

    Ok(())
}

fn init_rotation(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manager = KeyRotationManager::new();
    let json = serde_json::to_string_pretty(&manager)?;
    std::fs::write(output, json)?;
    eprintln!("✅ Rotation manager initialized: {}", output);
    Ok(())
}

fn add_version(
    state_path: &str,
    scope_str: &str,
    key_b64: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load manager
    let json = std::fs::read_to_string(state_path)?;
    let mut manager: KeyRotationManager = serde_json::from_str(&json)?;

    // Parse scope
    let scope = match scope_str.to_lowercase().as_str() {
        "root" => KeyScope::Root,
        "device" | "device-master" => KeyScope::DeviceMaster,
        "session" => KeyScope::Session,
        "stream" => KeyScope::Stream,
        _ => return Err(format!("Invalid scope: {}", scope_str).into()),
    };

    // Parse key
    let key_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(key_b64)?;
    if key_bytes.len() != 32 {
        return Err("Key must be 32 bytes".into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    // Add version
    let now = chrono::Utc::now().timestamp();
    let version = manager.add_key_version(scope, key, now)?;

    // Save
    let json = serde_json::to_string_pretty(&manager)?;
    std::fs::write(state_path, json)?;

    eprintln!("✅ Added {:?} key version {}", scope, version);
    Ok(())
}

fn show_status(state_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(state_path)?;
    let manager: KeyRotationManager = serde_json::from_str(&json)?;

    let now = chrono::Utc::now().timestamp();
    let status = manager.get_status(now);

    println!("🔑 Key Rotation Status\n");

    for (scope, info) in status {
        println!("Scope: {:?}", scope);
        println!("  Active Version: {:?}", info.active_version);
        println!("  Usable Versions: {}", info.usable_versions);
        println!("  Needs Rotation: {}", if info.needs_rotation { "⚠️  YES" } else { "✅ No" });
        if let Some(next) = info.next_rotation {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(next, 0)
                .unwrap_or_default();
            println!("  Next Rotation: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
        } else {
            println!("  Next Rotation: Manual only");
        }
        println!();
    }

    Ok(())
}

fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    use rand::Rng;
    use rand::rngs::OsRng;

    println!("🍯 HoneyLink Key Hierarchy Demo\n");
    println!("Demonstrating 4-level key derivation:");
    println!("  k_root → k_device → k_session → k_stream\n");

    // Generate root key
    println!("1️⃣  Generating root key (X25519)...");
    let mut root_key = [0u8; 32];
    OsRng.fill(&mut root_key);
    println!("   Root: {}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(root_key));

    // Derive device key
    println!("\n2️⃣  Deriving device master key (HKDF-SHA512)...");
    let hierarchy = KeyHierarchy::from_bytes(root_key);
    let device_key = hierarchy.derive_simple(KeyScope::DeviceMaster)?;
    println!("   Device: {}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(device_key));

    // Derive session key
    println!("\n3️⃣  Deriving session key...");
    let hierarchy = KeyHierarchy::from_bytes(device_key);
    let session_key = hierarchy.derive_simple(KeyScope::Session)?;
    println!("   Session: {}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(session_key));

    // Derive stream key
    println!("\n4️⃣  Deriving stream key...");
    let hierarchy = KeyHierarchy::from_bytes(session_key);
    let stream_key = hierarchy.derive_simple(KeyScope::Stream)?;
    println!("   Stream: {}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(stream_key));

    println!("\n✅ All keys derived successfully using pure Rust cryptography!");
    println!("   - X25519-dalek for root keypair");
    println!("   - HKDF-SHA512 for key derivation");
    println!("   - Zero C/C++ dependencies\n");

    // Demonstrate rotation
    println!("5️⃣  Key Rotation Demo...");
    let mut manager = KeyRotationManager::new();
    let now = chrono::Utc::now().timestamp();

    // Add root key version
    manager.add_key_version(KeyScope::Root, root_key, now)?;
    println!("   Added Root key v1");

    // Add device key version
    manager.add_key_version(KeyScope::DeviceMaster, device_key, now)?;
    println!("   Added Device key v1");

    // Show status
    let status = manager.get_status(now);
    println!("\n📊 Rotation Status:");
    for (scope, info) in status {
        if info.active_version.is_some() {
            println!("   {:?}: v{} active, {} usable",
                scope,
                info.active_version.unwrap(),
                info.usable_versions
            );
        }
    }

    println!("\n🎉 Demo complete! Use --help to see all commands.");

    Ok(())
}
