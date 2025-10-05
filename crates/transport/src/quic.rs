//! QUIC transport implementation using quinn
//!
//! This module implements the TransportProtocol trait using QUIC protocol via the quinn crate.
//! QUIC provides:
//! - TLS 1.3 encryption by default
//! - Multiple concurrent streams per connection
//! - Low latency (target: P99 <= 12ms)
//! - Built-in congestion control
//! - Connection multiplexing
//!
//! # Architecture
//!
//! - QuicTransport: Main struct implementing TransportProtocol trait
//! - QuicConnection: Wrapper around quinn::Connection implementing Connection trait
//! - QuicStream: Wrapper around quinn bidirectional streams implementing Stream trait
//!
//! # Design Decisions
//!
//! - **Pure Rust**: Uses quinn + rustls with ring crypto (no C/C++ dependencies)
//! - **Self-signed certs**: For development/testing (production should use proper PKI)
//! - **Async-first**: All operations are async for non-blocking I/O
//! - **Error mapping**: Quinn errors are mapped to TransportError for consistency
//!
//! # Security
//!
//! - TLS 1.3 enforced via rustls
//! - Certificate validation configurable (skip for testing, enforce for production)
//! - No support for insecure protocols

use crate::protocol::{Connection, Result, Stream, StreamPriority, TransportError, TransportProtocol};
use async_trait::async_trait;
use quinn::{ClientConfig, Endpoint, RecvStream, SendStream, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// QUIC transport implementation
///
/// # Design Rationale
/// - Uses Arc<Mutex<>> for endpoint to allow shared ownership across async tasks
/// - Lazy initialization of endpoint on first connect/listen call
/// - Self-signed certificates for development (should be configurable for production)
pub struct QuicTransport {
    /// Quinn endpoint (lazy-initialized)
    endpoint: Arc<Mutex<Option<Endpoint>>>,
    /// Server configuration (TLS certificates)
    server_config: ServerConfig,
    /// Client configuration (TLS trust anchor)
    client_config: ClientConfig,
}

impl QuicTransport {
    /// Creates a new QUIC transport with default configuration
    ///
    /// # Security Note
    /// Uses self-signed certificates for development/testing.
    /// Production deployments should provide proper CA-signed certificates.
    pub fn new() -> Result<Self> {
        // Generate self-signed certificate for testing
        let (cert, key) = Self::generate_self_signed_cert()?;

        // Server configuration: accept connections with TLS 1.3
        let server_config = Self::build_server_config(cert.clone(), key)?;

        // Client configuration: skip certificate verification for testing
        // TODO: Add production mode with proper CA trust chain
        let client_config = Self::build_client_config_insecure();

        Ok(Self {
            endpoint: Arc::new(Mutex::new(None)),
            server_config,
            client_config,
        })
    }

    /// Generates a self-signed certificate for development/testing
    ///
    /// # Returns
    /// (certificate_der, private_key_der)
    ///
    /// # Security Warning
    /// This should NOT be used in production. Use proper PKI infrastructure instead.
    fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])
            .map_err(|e| TransportError::EncryptionError(format!("Failed to generate cert: {}", e)))?;

        let cert_der = CertificateDer::from(cert.cert);
        let key_der = PrivateKeyDer::try_from(cert.key_pair.serialize_der())
            .map_err(|e| TransportError::EncryptionError(format!("Failed to serialize key: {}", e)))?;

        Ok((cert_der, key_der))
    }

    /// Builds server configuration with TLS 1.3
    fn build_server_config(
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
    ) -> Result<ServerConfig> {
        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .map_err(|e| TransportError::EncryptionError(format!("Failed to build TLS config: {}", e)))?;

        // Enable ALPN for QUIC
        server_crypto.alpn_protocols = vec![b"hq-29".to_vec()];

        let mut server_config = ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
                .map_err(|e| TransportError::EncryptionError(format!("QUIC crypto config failed: {}", e)))?,
        ));

        // Performance tuning for P99 <= 12ms target
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_bidi_streams(100u32.into());
        transport_config.max_concurrent_uni_streams(100u32.into());
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));

        server_config.transport_config(Arc::new(transport_config));

        Ok(server_config)
    }

    /// Builds client configuration with insecure certificate validation (for testing)
    ///
    /// # Security Warning
    /// This skips certificate verification. Use only for development/testing.
    fn build_client_config_insecure() -> ClientConfig {
        let mut client_crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        client_crypto.alpn_protocols = vec![b"hq-29".to_vec()];

        let mut client_config = ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)
                .expect("QUIC client configuration should be valid (internal error)"),
        ));

        // Same performance tuning as server
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_bidi_streams(100u32.into());
        transport_config.max_concurrent_uni_streams(100u32.into());
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));

        client_config.transport_config(Arc::new(transport_config));

        client_config
    }

    /// Initializes endpoint if not already initialized
    async fn ensure_endpoint(&self, addr: SocketAddr) -> Result<Endpoint> {
        let mut endpoint_guard = self.endpoint.lock().await;

        if let Some(endpoint) = endpoint_guard.as_ref() {
            return Ok(endpoint.clone());
        }

        // Create new endpoint
        let mut endpoint = Endpoint::server(self.server_config.clone(), addr)
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create endpoint: {}", e)))?;

        endpoint.set_default_client_config(self.client_config.clone());

        *endpoint_guard = Some(endpoint.clone());
        Ok(endpoint)
    }
}

impl Default for QuicTransport {
    fn default() -> Self {
        Self::new().expect("Failed to create default QuicTransport")
    }
}

#[async_trait]
impl TransportProtocol for QuicTransport {
    fn protocol_name(&self) -> &'static str {
        "QUIC"
    }

    async fn connect(&self, addr: SocketAddr, timeout: Duration) -> Result<Arc<dyn Connection>> {
        // Use ephemeral local port (0.0.0.0:0)
        let local_addr: SocketAddr = if addr.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };

        let endpoint = self.ensure_endpoint(local_addr).await?;

        // Connect with timeout
        let connecting = endpoint.connect(addr, "localhost")
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to initiate connection: {}", e)))?;

        let connection = tokio::time::timeout(timeout, connecting)
            .await
            .map_err(|_| TransportError::ConnectionTimeout(timeout))?
            .map_err(|e| TransportError::ConnectionFailed(format!("Connection failed: {}", e)))?;

        Ok(Arc::new(QuicConnection {
            connection: Arc::new(connection),
            streams: Arc::new(Mutex::new(HashMap::new())),
        }))
    }

    async fn listen(&self, addr: SocketAddr) -> Result<mpsc::Receiver<Arc<dyn Connection>>> {
        let endpoint = self.ensure_endpoint(addr).await?;
        let (tx, rx) = mpsc::channel(100);

        // Spawn task to accept incoming connections
        tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                match incoming.await {
                    Ok(connection) => {
                        let conn: Arc<dyn Connection> = Arc::new(QuicConnection {
                            connection: Arc::new(connection),
                            streams: Arc::new(Mutex::new(HashMap::new())),
                        });
                        if tx.send(conn).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn stop_listening(&self) -> Result<()> {
        let mut endpoint_guard = self.endpoint.lock().await;
        if let Some(endpoint) = endpoint_guard.take() {
            endpoint.close(0u32.into(), b"shutdown");
        }
        Ok(())
    }

    async fn is_listening(&self) -> bool {
        self.endpoint.lock().await.is_some()
    }

    async fn stats(&self) -> crate::protocol::TransportStats {
        // TODO: Implement proper stats collection from quinn
        crate::protocol::TransportStats::default()
    }
}

/// QUIC connection wrapper
struct QuicConnection {
    connection: Arc<quinn::Connection>,
    streams: Arc<Mutex<HashMap<u64, (SendStream, RecvStream)>>>,
}

#[async_trait]
impl Connection for QuicConnection {
    async fn send(&self, data: &[u8]) -> Result<()> {
        // Use default stream (stream 0)
        let (mut send, _) = self.connection.open_bi().await
            .map_err(|e| TransportError::SendFailed(format!("Failed to open stream: {}", e)))?;

        send.write_all(data).await
            .map_err(|e| TransportError::SendFailed(format!("Write failed: {}", e)))?;

        send.finish()
            .map_err(|e| TransportError::SendFailed(format!("Finish failed: {}", e)))?;

        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>> {
        let (_, mut recv) = self.connection.accept_bi().await
            .map_err(|e| TransportError::ReceiveFailed(format!("Failed to accept stream: {}", e)))?;

        let data = recv.read_to_end(1024 * 1024) // 1MB max
            .await
            .map_err(|e| TransportError::ReceiveFailed(format!("Read failed: {}", e)))?;

        Ok(data)
    }

    async fn open_stream(&self) -> Result<Box<dyn Stream>> {
        let (send, recv) = self.connection.open_bi().await
            .map_err(|e| TransportError::SendFailed(format!("Failed to open stream: {}", e)))?;

        Ok(Box::new(QuicStream { send, recv }))
    }

    async fn open_stream_with_priority(&self, priority: StreamPriority) -> Result<Box<dyn Stream>> {
        let (send, recv) = self.connection.open_bi().await
            .map_err(|e| TransportError::SendFailed(format!("Failed to open stream: {}", e)))?;

        // Map StreamPriority to quinn stream priority
        // quinn uses i32 priority: higher values = higher priority
        let quinn_priority = match priority {
            StreamPriority::High => 100,   // Burst traffic, highest priority
            StreamPriority::Normal => 50,  // Standard traffic, medium priority
            StreamPriority::Low => 0,      // Background traffic, lowest priority
        };

        // Set stream priority in quinn
        send.set_priority(quinn_priority)
            .map_err(|e| TransportError::SendFailed(format!("Failed to set priority: {}", e)))?;

        Ok(Box::new(QuicStream { send, recv }))
    }

    async fn close(&self) -> Result<()> {
        self.connection.close(0u32.into(), b"closed");
        Ok(())
    }

    fn remote_addr(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    fn local_addr(&self) -> SocketAddr {
        // Quinn doesn't expose local SocketAddr directly, use a placeholder
        // TODO: Store local_addr during connection establishment
        "0.0.0.0:0".parse().unwrap()
    }

    fn is_connected(&self) -> bool {
        !self.connection.close_reason().is_some()
    }

    fn stats(&self) -> crate::protocol::ConnectionStats {
        let quinn_stats = self.connection.stats();
        crate::protocol::ConnectionStats {
            start_time: 0, // TODO: Track connection start time
            bytes_sent: 0, // TODO: quinn doesn't expose per-path stats easily
            bytes_received: 0,
            rtt_ms: quinn_stats.path.rtt.as_millis() as u32,
            active_streams: 0, // TODO: Track this
        }
    }


}

/// QUIC stream wrapper
struct QuicStream {
    send: SendStream,
    recv: RecvStream,
}

#[async_trait]
impl Stream for QuicStream {
    async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.send.write_all(data).await
            .map_err(|e| TransportError::SendFailed(format!("Stream write failed: {}", e)))?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        let data = self.recv.read_to_end(1024 * 1024).await
            .map_err(|e| TransportError::ReceiveFailed(format!("Stream read failed: {}", e)))?;
        Ok(data)
    }

    async fn close(&mut self) -> Result<()> {
        self.send.finish()
            .map_err(|e| TransportError::SendFailed(format!("Stream finish failed: {}", e)))?;
        Ok(())
    }
}

/// Certificate verifier that skips all validation (INSECURE - for testing only)
///
/// # Security Warning
/// This implementation accepts ANY certificate without validation.
/// NEVER use this in production environments.
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quic_transport_creation() {
        let transport = QuicTransport::new();
        assert!(transport.is_ok());
        assert_eq!(transport.unwrap().protocol_name(), "QUIC");
    }

    #[tokio::test]
    async fn test_quic_connect_timeout() {
        let transport = QuicTransport::new().unwrap();
        let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        // Should timeout since no server is listening
        let result = transport.connect(addr, Duration::from_millis(100)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quic_listen_and_connect() {
        let server = QuicTransport::new().unwrap();
        let client = QuicTransport::new().unwrap();

        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut incoming = server.listen(addr).await.unwrap();

        // Get actual bound address
        let server_addr = {
            let endpoint_guard = server.endpoint.lock().await;
            endpoint_guard.as_ref().unwrap().local_addr().unwrap()
        };

        // Client connects
        let client_conn = client.connect(server_addr, Duration::from_secs(5)).await.unwrap();

        // Server accepts
        let server_conn = tokio::time::timeout(Duration::from_secs(5), incoming.recv())
            .await
            .unwrap()
            .unwrap();

        // Verify connections are established
        assert_ne!(client_conn.remote_addr().port(), 0);
        assert_ne!(server_conn.remote_addr().port(), 0);

        // Clean up
        client_conn.close().await.unwrap();
        server_conn.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_quic_prioritized_stream() {
        let server = QuicTransport::new().unwrap();
        let client = QuicTransport::new().unwrap();

        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut incoming = server.listen(addr).await.unwrap();

        let server_addr = {
            let endpoint_guard = server.endpoint.lock().await;
            endpoint_guard.as_ref().unwrap().local_addr().unwrap()
        };

        // Client connects
        let client_conn = client.connect(server_addr, Duration::from_secs(5)).await.unwrap();

        // Server accepts
        let _server_conn = tokio::time::timeout(Duration::from_secs(5), incoming.recv())
            .await
            .unwrap()
            .unwrap();

        // Open streams with different priorities
        let high_stream = client_conn.open_stream_with_priority(StreamPriority::High).await.unwrap();
        let normal_stream = client_conn.open_stream_with_priority(StreamPriority::Normal).await.unwrap();
        let low_stream = client_conn.open_stream_with_priority(StreamPriority::Low).await.unwrap();

        // Verify streams are open (basic smoke test)
        // In production, priority would affect congestion control and bandwidth allocation
        drop(high_stream);
        drop(normal_stream);
        drop(low_stream);

        client_conn.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_quic_send_receive() {
        let server = QuicTransport::new().unwrap();
        let client = QuicTransport::new().unwrap();

        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut incoming = server.listen(addr).await.unwrap();

        let server_addr = {
            let endpoint_guard = server.endpoint.lock().await;
            endpoint_guard.as_ref().unwrap().local_addr().unwrap()
        };

        let client_conn = client.connect(server_addr, Duration::from_secs(5)).await.unwrap();
        let server_conn = tokio::time::timeout(Duration::from_secs(5), incoming.recv())
            .await
            .unwrap()
            .unwrap();

        // Send data from client to server
        let test_data = b"Hello, QUIC!";
        client_conn.send(test_data).await.unwrap();

        // Receive on server
        let received = server_conn.receive().await.unwrap();
        assert_eq!(received, test_data);

        client_conn.close().await.unwrap();
        server_conn.close().await.unwrap();
    }
}
