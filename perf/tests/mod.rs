pub mod benchmarks;
pub mod network_sim;
pub mod regression_monitoring;
pub mod stress_tests;

use criterion::{criterion_group, criterion_main};
use regression_monitoring::RegressionMonitor;

criterion_group!(
    performance_tests,
    benchmarks::transaction_throughput,
    stress_tests::high_load,
    stress_tests::network_partitions,
    stress_tests::resource_monitoring
);

criterion_main!(performance_tests);

pub fn create_test_blockchain() -> sebure_core::blockchain::Blockchain {
    // Create a test blockchain instance with default configuration
    sebure_core::blockchain::Blockchain::new_test_instance()
}

pub fn run_performance_suite() {
    let mut monitor = RegressionMonitor::new("performance_metrics.json");
    
    // Run benchmarks and record metrics
    let metrics = benchmarks::run_benchmarks();
    monitor.record_metrics(metrics).unwrap();
    
    // Check for regressions
    if let Some(report) = monitor.check_for_regressions() {
        eprintln!("Performance regression detected: {:?}", report);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_suite() {
        run_performance_suite();
    }
}
