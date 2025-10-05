//! Performance Benchmarks for HoneyLink Transport
//!
//! Measures key performance metrics:
//! - Connection establishment latency (P50/P95/P99)
//! - Stream throughput (single and multi-stream)
//! - QoS overhead (priority switching cost)
//! - Multi-stream scalability (concurrent streams)
//!
//! Target: spec/performance/benchmark.md
//! - P99 latency ≤ 120ms
//! - Throughput ≥ 95% of expected
//! - Packet loss ≤ 0.2%

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use honeylink_transport::{
    manager::TransportManager,
    protocol::{ProtocolStrategy, ProtocolType, StreamPriority},
    quic::QuicTransport,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark: Connection establishment latency
///
/// Measures time to establish QUIC connection
fn bench_connection_establishment(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("connection_establishment", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
                
                let addr = "127.0.0.1:10000".parse().unwrap();
                let _ = transport.connect(black_box(addr)).await;
            });
        });
    });
}

/// Benchmark: Stream opening latency
///
/// Measures time to open a stream on established connection
fn bench_stream_opening(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("stream_opening", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
                
                let addr = "127.0.0.1:10001".parse().unwrap();
                if let Ok(conn) = transport.connect(addr).await {
                    let _ = transport
                        .open_prioritized_stream(&conn, StreamPriority::Normal, 1000)
                        .await;
                    let _ = conn.close().await;
                }
            });
        });
    });
}

/// Benchmark: QoS priority assignment overhead
///
/// Measures cost of priority-based stream allocation
fn bench_qos_priority_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("qos_priority");
    
    for priority in [StreamPriority::Low, StreamPriority::Normal, StreamPriority::High] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", priority)),
            &priority,
            |b, &prio| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                        let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                        transport.register_protocol(ProtocolType::Quic, quic).await;
                        
                        let addr = "127.0.0.1:10002".parse().unwrap();
                        if let Ok(conn) = transport.connect(addr).await {
                            let _ = transport
                                .open_prioritized_stream(&conn, black_box(prio), 1000)
                                .await;
                            let _ = conn.close().await;
                        }
                    });
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Multi-stream concurrent allocation
///
/// Measures scalability with multiple concurrent streams
fn bench_multi_stream_scalability(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("multi_stream_scalability");
    
    for stream_count in [10, 50, 100] {
        group.throughput(Throughput::Elements(stream_count));
        group.bench_with_input(
            BenchmarkId::from_parameter(stream_count),
            &stream_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                        let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                        transport.register_protocol(ProtocolType::Quic, quic).await;
                        
                        let addr = "127.0.0.1:10003".parse().unwrap();
                        if let Ok(conn) = transport.connect(addr).await {
                            let mut streams = Vec::new();
                            for _ in 0..black_box(count) {
                                if let Ok(stream) = transport
                                    .open_prioritized_stream(&conn, StreamPriority::Normal, 1000)
                                    .await
                                {
                                    streams.push(stream);
                                }
                            }
                            let _ = conn.close().await;
                        }
                    });
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: QoS stats retrieval
///
/// Measures overhead of stats API
fn bench_qos_stats_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("qos_stats_retrieval", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
                
                let _ = black_box(transport.qos_stats().await);
            });
        });
    });
}

/// Benchmark: Connection pooling efficiency
///
/// Measures performance of connection reuse
fn bench_connection_pooling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("connection_pooling", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
                
                let addr = "127.0.0.1:10004".parse().unwrap();
                
                // First connection
                if let Ok(conn1) = transport.connect(black_box(addr)).await {
                    let _ = conn1.close().await;
                }
                
                // Second connection (should reuse or be faster)
                if let Ok(conn2) = transport.connect(black_box(addr)).await {
                    let _ = conn2.close().await;
                }
            });
        });
    });
}

/// Benchmark: Priority switching overhead
///
/// Measures cost of changing stream priorities
fn bench_priority_switching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("priority_switching", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(ProtocolStrategy::PreferQuic);
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
                
                let addr = "127.0.0.1:10005".parse().unwrap();
                if let Ok(conn) = transport.connect(addr).await {
                    // Open low priority stream
                    if transport
                        .open_prioritized_stream(&conn, StreamPriority::Low, 1000)
                        .await
                        .is_ok()
                    {
                        // Then open high priority stream (simulates switching)
                        let _ = transport
                            .open_prioritized_stream(&conn, StreamPriority::High, 1000)
                            .await;
                    }
                    let _ = conn.close().await;
                }
            });
        });
    });
}

/// Benchmark: TransportManager initialization
///
/// Measures overhead of manager setup
fn bench_manager_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("manager_initialization", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transport = TransportManager::new(black_box(ProtocolStrategy::PreferQuic));
                let quic = Arc::new(QuicTransport::new().expect("Failed to create QUIC"));
                transport.register_protocol(ProtocolType::Quic, quic).await;
            });
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets = 
        bench_connection_establishment,
        bench_stream_opening,
        bench_qos_priority_overhead,
        bench_multi_stream_scalability,
        bench_qos_stats_retrieval,
        bench_connection_pooling,
        bench_priority_switching,
        bench_manager_initialization
}

criterion_main!(benches);
