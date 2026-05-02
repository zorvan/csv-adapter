//! Parallel Operations Example
//!
//! This example demonstrates concurrent Right creation and queries,
//! useful for high-throughput scenarios like batch processing or gaming.
//!
//! Run with: `cargo run --example parallel_verification --features "all-chains,tokio" --release`

use csv_adapter::prelude::*;
use std::thread;
use std::time::Instant;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CSV Adapter: Parallel Operations Demo ===\n");

    let client = CsvClient::builder()
        .with_all_chains()
        .with_store_backend(StoreBackend::InMemory)
        .build()?;

    // Sequential creation benchmark
    println!("Sequential Right Creation:");
    println!("-------------------------");

    let num_rights = 100;
    let start = Instant::now();

    for i in 0..num_rights {
        let commitment = Hash::from([i as u8; 32]);
        let _ = client.rights().create(commitment, Chain::Bitcoin);
    }

    let seq_duration = start.elapsed();
    println!("  Created {} rights in {:.2?}", num_rights, seq_duration);
    println!("  Throughput: {:.0} rights/sec\n",
        num_rights as f64 / seq_duration.as_secs_f64());

    // Parallel creation using threads
    println!("Parallel Right Creation (using threads):");
    println!("----------------------------------------");

    let client_arc = Arc::new(client);
    let num_threads = 4;
    let rights_per_thread = 25;

    let start = Instant::now();

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let client_ref = Arc::clone(&client_arc);
            thread::spawn(move || {
                for i in 0..rights_per_thread {
                    let commitment = Hash::from([(thread_id * rights_per_thread + i) as u8; 32]);
                    let _ = client_ref.rights().create(commitment, Chain::Ethereum);
                }
                rights_per_thread
            })
        })
        .collect();

    let total_created: usize = handles.into_iter()
        .map(|h| h.join().unwrap())
        .sum();

    let par_duration = start.elapsed();
    println!("  Created {} rights across {} threads in {:.2?}",
        total_created, num_threads, par_duration);
    println!("  Throughput: {:.0} rights/sec\n",
        total_created as f64 / par_duration.as_secs_f64());

    // Query benchmark
    println!("Parallel Query Benchmark:");
    println!("-------------------------");

    // First create a right to query repeatedly
    let test_right = client_arc.rights().create(
        Hash::from([255u8; 32]),
        Chain::Bitcoin,
    )?;

    let num_queries = 1000;
    let start = Instant::now();

    for _ in 0..num_queries {
        let _ = client_arc.rights().get(&test_right.id);
    }

    let query_duration = start.elapsed();
    let avg_query_time = query_duration / num_queries as u32;

    println!("  Performed {} queries in {:.2?}", num_queries, query_duration);
    println!("  Average latency: {:?}\n", avg_query_time);

    // Speedup calculation
    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    println!("=== Results ===");
    println!("Parallel speedup: {:.2}x", speedup);
    println!("Sequential throughput: {:.0} rights/sec",
        num_rights as f64 / seq_duration.as_secs_f64());
    println!("Parallel throughput: {:.0} rights/sec",
        total_created as f64 / par_duration.as_secs_f64());
    println!("Query latency: {:?}", avg_query_time);

    println!("\n=== Use Cases ===");
    println!("- Batch creation of gaming assets");
    println!("- High-throughput credential issuance");
    println!("- Parallel proof verification (when available)");
    println!("- Multi-threaded indexing operations");

    Ok(())
}
