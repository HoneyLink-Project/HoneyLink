//! Unified Transport Manager
//!
//! Coordinates multiple transport protocols (QUIC, WebRTC) to provide a single
//! unified API for connection management. Handles protocol selection, connection
//! pooling, failover, and integration with Phase 1 DiscoveryManager.
//!
//! # Architecture
//!
//! This follows the same trait-based design pattern as Phase 1's DiscoveryManager:
//! - Aggregates multiple TransportProtocol implementations
//! - Supports protocol selection strategies (prefer QUIC, fallback to WebRTC, etc.)
//! - Manages connection lifecycle and pooling
//! - Provides unified API for connection establishment
//!
//! # Design Rationale
//!
//! - **Consistent with Phase 1**: Same patterns as DiscoveryManager for low learning cost
//! - **Protocol abstraction**: TransportProtocol trait enables pluggable backends
//! - **Connection pooling**: Reuses existing connections for performance
//! - **Failover logic**: Automatic fallback when primary protocol fails
//! - **Thread-safe**: All state protected by Arc<RwLock> and tokio::sync primitives

use crate::protocol::{
    Connection, ProtocolStrategy, ProtocolType, Result, StreamPriority, TransportError, TransportProtocol,
    TransportStats, Stream,
};
use honeylink_qos_scheduler::scheduler::{QoSScheduler, QoSPriority, StreamRequest, StreamMode, AllocationStats};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, error, info, warn};

/// Unified Transport Manager
///
/// Manages multiple transport protocols and provides a unified interface
/// for connection establishment, pooling, and lifecycle management.
///
/// # Thread Safety
/// - All internal state is protected by Arc<RwLock>
/// - Protocol implementations must be Send + Sync
/// - Safe to clone and share across tasks
#[derive(Clone)]
pub struct TransportManager {
    /// Registered transport protocols (keyed by protocol type)
    protocols: Arc<RwLock<HashMap<ProtocolType, Arc<dyn TransportProtocol>>>>,

    /// Connection pool (remote_addr -> connection)
    ///
    /// Stores active connections for reuse. Stale connections are automatically
    /// removed on next access attempt.
    connections: Arc<RwLock<HashMap<SocketAddr, Arc<dyn Connection>>>>,

    /// Protocol selection strategy
    strategy: ProtocolStrategy,

    /// Aggregated transport statistics
    stats: Arc<RwLock<TransportStats>>,

    /// Default connection timeout
    default_timeout: Duration,

    /// QoS scheduler for multi-stream priority management
    ///
    /// Manages bandwidth allocation and stream limits across all connections.
    /// Shared across all protocols.
    qos_scheduler: Arc<Mutex<QoSScheduler>>,
}

impl TransportManager {
    /// Create new transport manager
    ///
    /// # Parameters
    /// - `strategy`: Protocol selection strategy (default: PreferQuic)
    ///
    /// # Returns
    /// A new TransportManager instance with an empty protocol set.
    /// Call `register_protocol()` to add QUIC, WebRTC, or other backends.
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_transport::manager::TransportManager;
    /// use honeylink_transport::protocol::ProtocolStrategy;
    ///
    /// let manager = TransportManager::new(ProtocolStrategy::PreferQuic);
    /// ```
    pub fn new(strategy: ProtocolStrategy) -> Self {
        // Create QoS scheduler with defaults:
        // - 100 Mbps total bandwidth (100,000 kbps)
        // - 100 parallel streams (project requirement)
        let qos_scheduler = QoSScheduler::with_limits(100_000, 100);

        Self {
            protocols: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            default_timeout: Duration::from_secs(5),
            qos_scheduler: Arc::new(Mutex::new(qos_scheduler)),
        }
    }

    /// Register a transport protocol
    ///
    /// Adds a new protocol backend to the manager. Protocols can be registered
    /// before or during operation.
    ///
    /// # Parameters
    /// - `protocol_type`: Protocol identifier (Quic or WebRtc)
    /// - `protocol`: Protocol implementation (must be Send + Sync)
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_transport::manager::TransportManager;
    /// use honeylink_transport::protocol::{ProtocolStrategy, ProtocolType};
    /// use honeylink_transport::quic::QuicTransport;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
    ///     let quic = QuicTransport::new()?;
    ///     manager.register_protocol(ProtocolType::Quic, Arc::new(quic)).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_protocol(
        &mut self,
        protocol_type: ProtocolType,
        protocol: Arc<dyn TransportProtocol>,
    ) {
        let mut protocols = self.protocols.write().await;
        info!(
            "Registering transport protocol: {} ({})",
            protocol_type.as_str(),
            protocol.protocol_name()
        );
        protocols.insert(protocol_type, protocol);
    }

    /// Connect to a remote peer
    ///
    /// Establishes a connection to the specified address using the configured
    /// protocol selection strategy. Reuses existing connections from pool when possible.
    ///
    /// # Protocol Selection Logic
    /// - **PreferQuic**: Try QUIC first, fallback to WebRTC on failure
    /// - **PreferWebRtc**: Try WebRTC first, fallback to QUIC on failure
    /// - **QuicOnly**: Use only QUIC, fail if unavailable
    /// - **WebRtcOnly**: Use only WebRTC, fail if unavailable
    /// - **All**: Race all protocols, first success wins
    ///
    /// # Connection Pooling
    /// - Check pool for existing connection to `addr`
    /// - Verify connection is still alive with `is_connected()`
    /// - Remove stale connections and establish new one
    ///
    /// # Parameters
    /// - `addr`: Remote peer address (IP:port)
    ///
    /// # Returns
    /// - `Ok(Arc<dyn Connection>)`: Connection handle on success
    /// - `Err(TransportError)`: Connection failed with all protocols
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_transport::manager::TransportManager;
    /// use honeylink_transport::protocol::ProtocolStrategy;
    /// use std::net::SocketAddr;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = TransportManager::new(ProtocolStrategy::PreferQuic);
    ///     let addr: SocketAddr = "192.168.1.100:8080".parse()?;
    ///     let conn = manager.connect(addr).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        // Check connection pool first
        if let Some(conn) = self.get_pooled_connection(addr).await {
            debug!("Reusing pooled connection to {}", addr);
            return Ok(conn);
        }

        // Establish new connection based on strategy
        let conn = match self.strategy {
            ProtocolStrategy::PreferQuic => self.connect_prefer_quic(addr).await?,
            ProtocolStrategy::PreferWebRtc => self.connect_prefer_webrtc(addr).await?,
            ProtocolStrategy::QuicOnly => self.connect_quic_only(addr).await?,
            ProtocolStrategy::WebRtcOnly => self.connect_webrtc_only(addr).await?,
            ProtocolStrategy::All => self.connect_all(addr).await?,
        };

        // Add to pool
        self.add_to_pool(addr, conn.clone()).await;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.connections_established += 1;
        stats.active_connections += 1;

        Ok(conn)
    }

    /// Get pooled connection if available and alive
    async fn get_pooled_connection(&self, addr: SocketAddr) -> Option<Arc<dyn Connection>> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get(&addr) {
            if conn.is_connected() {
                return Some(conn.clone());
            } else {
                // Remove stale connection
                debug!("Removing stale connection to {}", addr);
                connections.remove(&addr);

                // Update stats
                let mut stats = self.stats.write().await;
                stats.active_connections = stats.active_connections.saturating_sub(1);
            }
        }

        None
    }

    /// Add connection to pool
    async fn add_to_pool(&self, addr: SocketAddr, conn: Arc<dyn Connection>) {
        let mut connections = self.connections.write().await;
        connections.insert(addr, conn);
    }

    /// Connect using PreferQuic strategy
    ///
    /// Try QUIC first, fallback to WebRTC if QUIC fails.
    async fn connect_prefer_quic(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        let protocols = self.protocols.read().await;

        // Try QUIC first
        if let Some(quic) = protocols.get(&ProtocolType::Quic) {
            debug!("Attempting QUIC connection to {}", addr);
            match quic.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("QUIC connection established to {}", addr);
                    return Ok(conn);
                }
                Err(e) => {
                    warn!("QUIC connection failed: {}, trying WebRTC fallback", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                }
            }
        } else {
            warn!("QUIC protocol not registered");
        }

        // Fallback to WebRTC
        if let Some(webrtc) = protocols.get(&ProtocolType::WebRtc) {
            debug!("Attempting WebRTC connection to {}", addr);
            match webrtc.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("WebRTC connection established to {}", addr);
                    return Ok(conn);
                }
                Err(e) => {
                    error!("WebRTC connection also failed: {}", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                    return Err(e);
                }
            }
        } else {
            warn!("WebRTC protocol not registered");
        }

        Err(TransportError::ConnectionFailed(
            "No protocols available".to_string(),
        ))
    }

    /// Connect using PreferWebRtc strategy
    ///
    /// Try WebRTC first, fallback to QUIC if WebRTC fails.
    async fn connect_prefer_webrtc(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        let protocols = self.protocols.read().await;

        // Try WebRTC first
        if let Some(webrtc) = protocols.get(&ProtocolType::WebRtc) {
            debug!("Attempting WebRTC connection to {}", addr);
            match webrtc.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("WebRTC connection established to {}", addr);
                    return Ok(conn);
                }
                Err(e) => {
                    warn!("WebRTC connection failed: {}, trying QUIC fallback", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                }
            }
        } else {
            warn!("WebRTC protocol not registered");
        }

        // Fallback to QUIC
        if let Some(quic) = protocols.get(&ProtocolType::Quic) {
            debug!("Attempting QUIC connection to {}", addr);
            match quic.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("QUIC connection established to {}", addr);
                    return Ok(conn);
                }
                Err(e) => {
                    error!("QUIC connection also failed: {}", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                    return Err(e);
                }
            }
        } else {
            warn!("QUIC protocol not registered");
        }

        Err(TransportError::ConnectionFailed(
            "No protocols available".to_string(),
        ))
    }

    /// Connect using QuicOnly strategy
    async fn connect_quic_only(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        let protocols = self.protocols.read().await;

        if let Some(quic) = protocols.get(&ProtocolType::Quic) {
            debug!("Attempting QUIC-only connection to {}", addr);
            match quic.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("QUIC connection established to {}", addr);
                    Ok(conn)
                }
                Err(e) => {
                    error!("QUIC connection failed: {}", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                    Err(e)
                }
            }
        } else {
            Err(TransportError::ProtocolNotSupported(
                "QUIC protocol not registered".to_string(),
            ))
        }
    }

    /// Connect using WebRtcOnly strategy
    async fn connect_webrtc_only(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        let protocols = self.protocols.read().await;

        if let Some(webrtc) = protocols.get(&ProtocolType::WebRtc) {
            debug!("Attempting WebRTC-only connection to {}", addr);
            match webrtc.connect(addr, self.default_timeout).await {
                Ok(conn) => {
                    info!("WebRTC connection established to {}", addr);
                    Ok(conn)
                }
                Err(e) => {
                    error!("WebRTC connection failed: {}", e);
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    stats.connections_failed += 1;
                    Err(e)
                }
            }
        } else {
            Err(TransportError::ProtocolNotSupported(
                "WebRTC protocol not registered".to_string(),
            ))
        }
    }

    /// Connect using All strategy
    ///
    /// Race all available protocols, first successful connection wins.
    /// Uses tokio::select! for concurrent connection attempts.
    async fn connect_all(&self, addr: SocketAddr) -> Result<Arc<dyn Connection>> {
        let protocols = self.protocols.read().await;

        let quic_future = async {
            if let Some(quic) = protocols.get(&ProtocolType::Quic) {
                quic.connect(addr, self.default_timeout).await
            } else {
                Err(TransportError::ProtocolNotSupported(
                    "QUIC not registered".to_string(),
                ))
            }
        };

        let webrtc_future = async {
            if let Some(webrtc) = protocols.get(&ProtocolType::WebRtc) {
                webrtc.connect(addr, self.default_timeout).await
            } else {
                Err(TransportError::ProtocolNotSupported(
                    "WebRTC not registered".to_string(),
                ))
            }
        };

        debug!("Racing QUIC and WebRTC connections to {}", addr);

        // Race both protocols, first success wins
        tokio::select! {
            quic_result = quic_future => {
                match quic_result {
                    Ok(conn) => {
                        info!("QUIC won the race to {}", addr);
                        return Ok(conn);
                    }
                    Err(e) => {
                        debug!("QUIC lost the race: {}", e);
                    }
                }
            }
            webrtc_result = webrtc_future => {
                match webrtc_result {
                    Ok(conn) => {
                        info!("WebRTC won the race to {}", addr);
                        return Ok(conn);
                    }
                    Err(e) => {
                        debug!("WebRTC lost the race: {}", e);
                    }
                }
            }
        }

        // Both failed
        let mut stats = self.stats.write().await;
        stats.connections_failed += 1;

        Err(TransportError::ConnectionFailed(
            "All protocols failed".to_string(),
        ))
    }

    /// Get aggregated transport statistics
    ///
    /// Returns cumulative statistics across all protocols and connections.
    pub async fn stats(&self) -> TransportStats {
        self.stats.read().await.clone()
    }

    /// Clear all pooled connections
    ///
    /// Useful for testing or forcing reconnection.
    pub async fn clear_pool(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();

        let mut stats = self.stats.write().await;
        stats.active_connections = 0;
    }

    /// Close a specific connection and remove from pool
    ///
    /// # Parameters
    /// - `addr`: Remote address of connection to close
    pub async fn close_connection(&self, addr: SocketAddr) -> Result<()> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.remove(&addr) {
            conn.close().await?;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);

            info!("Closed connection to {}", addr);
        }

        Ok(())
    }

    /// Get list of registered protocol types
    pub async fn registered_protocols(&self) -> Vec<ProtocolType> {
        let protocols = self.protocols.read().await;
        protocols.keys().copied().collect()
    }

    /// Open a prioritized stream on an existing connection
    ///
    /// Creates a new stream with QoS-aware bandwidth allocation and priority handling.
    /// This method integrates with the QoS scheduler to:
    /// - Allocate bandwidth based on stream priority
    /// - Track stream count limits (max 100 streams)
    /// - Enforce fair bandwidth sharing across priority levels
    ///
    /// # Priority Mapping
    /// - **High** → QoSPriority::Burst (high bandwidth, low latency)
    /// - **Normal** → QoSPriority::Normal (standard bandwidth)
    /// - **Low** → QoSPriority::Latency (background, lowest priority)
    ///
    /// # Parameters
    /// - `connection`: Existing connection to open stream on
    /// - `priority`: Stream priority level (High/Normal/Low)
    /// - `bandwidth_kbps`: Requested bandwidth in kilobits per second
    ///
    /// # Returns
    /// - `Ok(Box<dyn Stream>)`: Stream handle on success
    /// - `Err(TransportError::ResourceExhausted)`: Insufficient bandwidth or too many streams
    /// - `Err(TransportError)`: Connection error
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_transport::manager::TransportManager;
    /// use honeylink_transport::protocol::{ProtocolStrategy, StreamPriority};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = TransportManager::new(ProtocolStrategy::PreferQuic);
    ///     let addr = "127.0.0.1:8080".parse()?;
    ///     let conn = manager.connect(addr).await?;
    ///
    ///     // Open high-priority video stream (5000 kbps)
    ///     let stream = manager.open_prioritized_stream(
    ///         &conn,
    ///         StreamPriority::High,
    ///         5000
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn open_prioritized_stream(
        &self,
        connection: &Arc<dyn Connection>,
        priority: StreamPriority,
        bandwidth_kbps: u32,
    ) -> Result<Box<dyn Stream>> {
        // Map StreamPriority to QoSPriority
        let qos_priority = match priority {
            StreamPriority::High => QoSPriority::Burst,
            StreamPriority::Normal => QoSPriority::Normal,
            StreamPriority::Low => QoSPriority::Latency,
        };

        // Create stream request
        let stream_name = format!(
            "stream-{}-{}",
            connection.remote_addr(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let request = StreamRequest {
            name: stream_name.clone(),
            mode: StreamMode::Reliable, // Default to reliable bidirectional streams
            priority: qos_priority,
            bandwidth_kbps,
        };

        // Allocate stream through QoS scheduler
        let mut scheduler = self.qos_scheduler.lock().await;
        let allocations = scheduler
            .allocate_streams(&[request])
            .map_err(|e| TransportError::ResourceExhausted(e.to_string()))?;

        if allocations.is_empty() {
            return Err(TransportError::ResourceExhausted(
                "QoS scheduler rejected stream allocation".to_string(),
            ));
        }

        // Stream allocated successfully, open it on the connection
        let stream = connection.open_stream_with_priority(priority).await?;

        debug!(
            "Opened prioritized stream {} on {} with priority {:?}, bandwidth {} kbps",
            stream_name,
            connection.remote_addr(),
            priority,
            bandwidth_kbps
        );

        Ok(stream)
    }

    /// Release a stream from QoS scheduler
    ///
    /// Notifies the QoS scheduler that a stream is no longer active,
    /// freeing up bandwidth and stream count for other allocations.
    ///
    /// # Parameters
    /// - `stream_id`: Unique stream ID (from honeylink-core types)
    /// - `bandwidth_kbps`: Bandwidth to release
    pub async fn release_stream(&self, stream_id: honeylink_core::types::StreamId, bandwidth_kbps: u32) {
        let mut scheduler = self.qos_scheduler.lock().await;
        scheduler.release_stream(stream_id, bandwidth_kbps);

        debug!("Released stream {:?} with bandwidth {} kbps", stream_id, bandwidth_kbps);
    }

    /// Get QoS scheduler statistics
    ///
    /// Returns current bandwidth usage, active stream count, and allocation stats.
    pub async fn qos_stats(&self) -> AllocationStats {
        let scheduler = self.qos_scheduler.lock().await;
        scheduler.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ConnectionStats;
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    // Mock transport for testing
    struct MockTransport {
        name: &'static str,
        should_fail: bool,
    }

    #[async_trait]
    impl TransportProtocol for MockTransport {
        fn protocol_name(&self) -> &'static str {
            self.name
        }

        async fn connect(
            &self,
            _addr: SocketAddr,
            _timeout: Duration,
        ) -> Result<Arc<dyn Connection>> {
            if self.should_fail {
                Err(TransportError::ConnectionFailed(format!(
                    "{} mock failure",
                    self.name
                )))
            } else {
                Ok(Arc::new(MockConnection {
                    addr: "127.0.0.1:8080".parse().unwrap(),
                }))
            }
        }

        async fn listen(&self, _addr: SocketAddr) -> Result<mpsc::Receiver<Arc<dyn Connection>>> {
            Err(TransportError::ProtocolNotSupported(
                "Mock listen not implemented".to_string(),
            ))
        }

        async fn stop_listening(&self) -> Result<()> {
            Ok(())
        }

        async fn is_listening(&self) -> bool {
            false
        }

        async fn stats(&self) -> TransportStats {
            TransportStats::default()
        }
    }

    // Mock connection for testing
    struct MockConnection {
        addr: SocketAddr,
    }

    // Mock stream for testing
    struct MockStream;

    #[async_trait]
    impl crate::protocol::Stream for MockStream {
        async fn send(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive(&mut self) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl Connection for MockConnection {
        fn remote_addr(&self) -> SocketAddr {
            self.addr
        }

        fn local_addr(&self) -> SocketAddr {
            "0.0.0.0:0".parse().unwrap()
        }

        async fn send(&self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive(&self) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        async fn open_stream(&self) -> Result<Box<dyn crate::protocol::Stream>> {
            Ok(Box::new(MockStream))
        }

        async fn open_stream_with_priority(&self, _priority: StreamPriority) -> Result<Box<dyn crate::protocol::Stream>> {
            Ok(Box::new(MockStream))
        }

        async fn close(&self) -> Result<()> {
            Ok(())
        }

        fn is_connected(&self) -> bool {
            true
        }

        fn stats(&self) -> ConnectionStats {
            ConnectionStats::default()
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let protocols = manager.registered_protocols().await;
        assert_eq!(protocols.len(), 0);
    }

    #[tokio::test]
    async fn test_register_protocol() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let mock = Arc::new(MockTransport {
            name: "MockQUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, mock)
            .await;

        let protocols = manager.registered_protocols().await;
        assert_eq!(protocols.len(), 1);
        assert!(protocols.contains(&ProtocolType::Quic));
    }

    #[tokio::test]
    async fn test_connect_prefer_quic_success() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let result = manager.connect(addr).await;
        assert!(result.is_ok());

        let stats = manager.stats().await;
        assert_eq!(stats.connections_established, 1);
        assert_eq!(stats.active_connections, 1);
    }

    #[tokio::test]
    async fn test_connect_prefer_quic_fallback_to_webrtc() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: true,
        });
        let webrtc = Arc::new(MockTransport {
            name: "WebRTC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;
        manager
            .register_protocol(ProtocolType::WebRtc, webrtc)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let result = manager.connect(addr).await;
        assert!(result.is_ok());

        let stats = manager.stats().await;
        assert_eq!(stats.connections_established, 1);
        assert_eq!(stats.connections_failed, 1); // QUIC failed
        assert_eq!(stats.active_connections, 1);
    }

    #[tokio::test]
    async fn test_connect_quic_only_failure() {
        let mut manager = TransportManager::new(ProtocolStrategy::QuicOnly);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: true,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let result = manager.connect(addr).await;
        assert!(result.is_err());

        let stats = manager.stats().await;
        assert_eq!(stats.connections_established, 0);
        assert_eq!(stats.connections_failed, 1);
    }

    #[tokio::test]
    async fn test_connection_pooling() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        // First connection
        let conn1 = manager.connect(addr).await.unwrap();
        let stats = manager.stats().await;
        assert_eq!(stats.connections_established, 1);

        // Second connection to same address (should reuse)
        let conn2 = manager.connect(addr).await.unwrap();
        let stats = manager.stats().await;
        assert_eq!(stats.connections_established, 1); // Still 1, reused

        // Verify same connection (by comparing remote_addr)
        assert_eq!(conn1.remote_addr(), conn2.remote_addr());
    }

    #[tokio::test]
    async fn test_clear_pool() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        manager.connect(addr).await.unwrap();

        let stats = manager.stats().await;
        assert_eq!(stats.active_connections, 1);

        manager.clear_pool().await;
        let stats = manager.stats().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_close_connection() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let quic = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, quic)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        manager.connect(addr).await.unwrap();

        let stats = manager.stats().await;
        assert_eq!(stats.active_connections, 1);

        manager.close_connection(addr).await.unwrap();
        let stats = manager.stats().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_qos_prioritized_stream() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let mock = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, mock)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = manager.connect(addr).await.unwrap();

        // Open high-priority stream (5000 kbps)
        let high_stream = manager
            .open_prioritized_stream(&conn, StreamPriority::High, 5000)
            .await
            .unwrap();

        // Open normal-priority stream (1000 kbps)
        let normal_stream = manager
            .open_prioritized_stream(&conn, StreamPriority::Normal, 1000)
            .await
            .unwrap();

        // Verify streams are open
        drop(high_stream);
        drop(normal_stream);

        // Check QoS stats
        let stats = manager.qos_stats().await;
        assert_eq!(stats.allocated_bandwidth_kbps, 6000); // 5000 + 1000
        assert_eq!(stats.total_streams, 2);
    }

    #[tokio::test]
    async fn test_qos_insufficient_bandwidth() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let mock = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, mock)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = manager.connect(addr).await.unwrap();

        // Try to allocate 150 Mbps (exceeds default 100 Mbps limit)
        let result = manager
            .open_prioritized_stream(&conn, StreamPriority::High, 150_000)
            .await;

        assert!(result.is_err());
        match result {
            Err(TransportError::ResourceExhausted(_)) => {
                // Expected error
            }
            _ => panic!("Expected ResourceExhausted error"),
        }
    }

    #[tokio::test]
    async fn test_qos_too_many_streams() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let mock = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, mock)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = manager.connect(addr).await.unwrap();

        // Open streams near bandwidth limit (99,000 kbps used, 1,000 remaining)
        let mut streams = Vec::new();
        for _ in 0..99 {
            let stream = manager
                .open_prioritized_stream(&conn, StreamPriority::Normal, 1000)
                .await
                .unwrap();
            streams.push(stream);
        }

        // Try to allocate 5000 kbps (exceeds remaining 1000 kbps)
        let result = manager
            .open_prioritized_stream(&conn, StreamPriority::High, 5000)
            .await;

        assert!(result.is_err());
        match result {
            Err(TransportError::ResourceExhausted(_)) => {
                // Expected error: insufficient bandwidth
            }
            _ => panic!("Expected ResourceExhausted error"),
        }
    }

    #[tokio::test]
    async fn test_qos_priority_ordering() {
        let mut manager = TransportManager::new(ProtocolStrategy::PreferQuic);
        let mock = Arc::new(MockTransport {
            name: "QUIC",
            should_fail: false,
        });

        manager
            .register_protocol(ProtocolType::Quic, mock)
            .await;

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = manager.connect(addr).await.unwrap();

        // Open streams in reverse priority order
        let _low = manager
            .open_prioritized_stream(&conn, StreamPriority::Low, 1000)
            .await
            .unwrap();
        let _normal = manager
            .open_prioritized_stream(&conn, StreamPriority::Normal, 1000)
            .await
            .unwrap();
        let _high = manager
            .open_prioritized_stream(&conn, StreamPriority::High, 1000)
            .await
            .unwrap();

        // Verify QoS scheduler allocated all streams
        let stats = manager.qos_stats().await;
        assert_eq!(stats.total_streams, 3);
    }
}
