use super::create_test_blockchain;
use criterion::{black_box, Criterion, Throughput};
use sebure_core::types::{Transaction, TransactionData};
use std::time::Instant;

pub fn transaction_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_throughput");
    
    // Test baseline throughput
    group.throughput(Throughput::Elements(1));
    group.bench_function("baseline", |b| {
        let blockchain = create_test_blockchain();
        let tx = Transaction::new(TransactionData::default());
        
        b.iter(|| {
            blockchain.process_transaction(black_box(tx.clone()));
        });
    });

    // Test with varying transaction sizes
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_function(&format!("batch_size_{}", size), |b| {
            let blockchain = create_test_blockchain();
            let txs: Vec<_> = (0..*size)
                .map(|_| Transaction::new(TransactionData::default()))
                .collect();
                
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for tx in &txs {
                        blockchain.process_transaction(black_box(tx.clone()));
                    }
                }
                start.elapsed()
            });
        });
    }

    group.finish();
}

pub fn block_propagation(c: &mut Criterion) {
    // TODO: Implement block propagation benchmarks
}
