use criterion::{black_box, Criterion, Throughput};
use sebure_core::{
    blockchain::{Blockchain, BlockchainConfig},
    crypto::{signature::Signature, Hash},
    types::{Block, BlockHeader, Transaction, TransactionData},
    utils::generate_keypair,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn create_test_blockchain() -> Blockchain {
    let config = BlockchainConfig {
        block_time: Duration::from_secs(10),
        difficulty: 1,
        max_block_size: 1024 * 1024,
        ..BlockchainConfig::default()
    };
    
    Blockchain::new(config)
}

pub fn transaction_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_throughput");
    
    // Test baseline throughput
    group.throughput(Throughput::Elements(1));
    group.bench_function("baseline", |b| {
        let blockchain = create_test_blockchain();
        let tx = Arc::new(Transaction::new(TransactionData::default()));
        
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
                .map(|_| Arc::new(Transaction::new(TransactionData::default())))
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
    let mut group = c.benchmark_group("block_propagation");
    
    // Test block creation and propagation
    group.bench_function("create_and_propagate", |b| {
        let blockchain = create_test_blockchain();
        let (_, public_key) = generate_keypair();
        
        b.iter(|| {
            // Create a block
            let header = BlockHeader {
                version: 1,
                height: 1,
                timestamp: 0,
                previous_hash: Hash::default(),
                merkle_root: Hash::default(),
                signature: Signature::default(),
                public_key: public_key.clone(),
                nonce: 0,
                difficulty: 1,
            };
            
            let block = Block::new(header, vec![]);
            
            // Process the block
            blockchain.process_block(black_box(block));
        });
    });

    group.finish();
}

pub fn mempool_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mempool_operations");
    
    // Test adding transactions to mempool
    group.bench_function("add_transaction", |b| {
        let blockchain = create_test_blockchain();
        let tx = Arc::new(Transaction::new(TransactionData::default()));
        
        b.iter(|| {
            blockchain.mempool().add_transaction(black_box(tx.clone()));
        });
    });

    // Test removing transactions from mempool
    group.bench_function("remove_transaction", |b| {
        let blockchain = create_test_blockchain();
        let tx = Arc::new(Transaction::new(TransactionData::default()));
        let tx_id = tx.id.clone();
        
        blockchain.mempool().add_transaction(&tx).unwrap();
        
        b.iter(|| {
            blockchain.mempool().remove_transaction(black_box(&tx_id));
        });
    });

    group.finish();
}
