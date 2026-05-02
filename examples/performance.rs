//! Performance Benchmarking Example
//!
//! This example demonstrates performance characteristics of the CSV Adapter,
//! including Right creation and transfer throughput.
//!
//! Run with: `cargo run --example performance --features "all-chains,tokio" --release`

use csv_adapter::prelude::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CSV Adapter: Performance Benchmarks ===\n");

    let client = CsvClient::builder()
        .with_all_chains()
        .with_store_backend(StoreBackend::InMemory)
        .build()?;

    // Benchmark 1: Right creation throughput
    println!("Benchmark 1: Right Creation Throughput");
    println!("-------------------------------------");

    let iterations = 1000;
    let start = Instant::now();

    for i in 0..iterations {
        let commitment = Hash::from([i as u8; 32]);
        let _ = client.rights().create(commitment, Chain::Bitcoin);
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!("  Iterations: {}", iterations);
    println!("  Total time: {:.2?}", duration);
    println!("  Throughput: {:.0} rights/second\n", throughput);

    // Benchmark 2: Query latency
    println!("Benchmark 2: Right Query Latency");
    println!("---------------------------------");

    // Create a right to query
    let test_commitment = Hash::from([255u8; 32]);
    let test_right = client.rights().create(test_commitment, Chain::Bitcoin)?;

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = client.rights().get(&test_right.id);
    }

    let duration = start.elapsed();
    let avg_latency = duration / iterations as u32;

    println!("  Iterations: {}", iterations);
    println!("  Average latency: {:.2?} per query\n", avg_latency);

    // Benchmark 3: Cross-chain transfer flow
    println!("Benchmark 3: Cross-Chain Transfer Flow");
    println!("---------------------------------------");

    let start = Instant::now();

    // Create and transfer a right
    let right = client.rights().create(
        Hash::from([42u8; 32]),
        Chain::Bitcoin,
    )?;

    let transfer_id = client.transfers()
        .cross_chain(right.id.clone(), Chain::Ethereum)
        .to_address("0x1234567890abcdef".to_string())
        .execute()?;

    let duration = start.elapsed();

    println!("  Right creation + transfer initiation: {:.2?}", duration);
    println!("  Transfer ID: {}\n", transfer_id);

    // List all rights
    println!("Benchmark 4: Rights Listing");
    println!("--------------------------");

    let start = Instant::now();
    let rights = client.rights().list(RightFilters::default())?;
    let list_duration = start.elapsed();

    println!("  Listed {} rights in {:.2?}\n", rights.len(), list_duration);

    // Summary
    println!("=== Performance Summary ===");
    println!("Right creation: {:.0} ops/sec", throughput);
    println!("Query latency: {:?}", avg_latency);
    println!("Cross-chain flow: {:?} end-to-end", duration);
    println!("List operation: {:?}", list_duration);

    Ok(())
}
