use super::{create_test_blockchain, stress_tests};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: i64,
    pub tps: f64,
    pub avg_latency: f64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_throughput: u64,
}

pub struct RegressionMonitor {
    metrics_history: Vec<PerformanceMetrics>,
    baseline: Option<PerformanceMetrics>,
    storage_path: String,
}

impl RegressionMonitor {
    pub fn new(storage_path: &str) -> Self {
        let metrics_history = Self::load_history(storage_path);
        
        Self {
            metrics_history,
            baseline: None,
            storage_path: storage_path.to_string(),
        }
    }

    pub fn record_metrics(&mut self, metrics: PerformanceMetrics) -> io::Result<()> {
        // Store metrics in history
        self.metrics_history.push(metrics.clone());
        
        // Append to storage file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)?;
            
        writeln!(file, "{}", serde_json::to_string(&metrics)?)?;
        
        Ok(())
    }

    pub fn check_for_regressions(&self) -> Option<RegressionReport> {
        if self.metrics_history.len() < 2 {
            return None;
        }

        let latest = self.metrics_history.last()?;
        let previous = self.metrics_history.get(self.metrics_history.len() - 2)?;

        let mut regressions = HashMap::new();
        
        if latest.tps < previous.tps * 0.9 {
            regressions.insert("tps", (previous.tps, latest.tps));
        }
        
        if latest.avg_latency > previous.avg_latency * 1.1 {
            regressions.insert("latency", (previous.avg_latency, latest.avg_latency));
        }

        if latest.cpu_usage > previous.cpu_usage * 1.2 {
            regressions.insert("cpu_usage", (previous.cpu_usage, latest.cpu_usage));
        }

        if latest.memory_usage > previous.memory_usage * 1.2 {
            regressions.insert("memory_usage", (previous.memory_usage, latest.memory_usage));
        }

        if latest.network_throughput < previous.network_throughput * 0.9 {
            regressions.insert("network_throughput", (previous.network_throughput, latest.network_throughput));
        }

        if regressions.is_empty() {
            None
        } else {
            Some(RegressionReport {
                timestamp: Utc::now().timestamp(),
                regressions,
            })
        }
    }

    fn load_history(path: &str) -> Vec<PerformanceMetrics> {
        if !Path::new(path).exists() {
            return Vec::new();
        }

        let content = fs::read_to_string(path).unwrap_or_default();
        content.lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }
}

pub struct RegressionReport {
    pub timestamp: i64,
    pub regressions: HashMap<&'static str, (f64, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_detection() {
        let mut monitor = RegressionMonitor::new("test_metrics.json");
        
        let metrics1 = PerformanceMetrics {
            timestamp: Utc::now().timestamp(),
            tps: 10_000.0,
            avg_latency: 0.5,
            cpu_usage: 50.0,
            memory_usage: 1_000_000,
            network_throughput: 10_000_000,
        };
        
        let metrics2 = PerformanceMetrics {
            timestamp: Utc::now().timestamp(),
            tps: 8_000.0, // 20% regression
            avg_latency: 0.6, // 20% regression
            cpu_usage: 70.0, // 40% increase
            memory_usage: 1_500_000, // 50% increase
            network_throughput: 10_000_000,
        };
        
        monitor.record_metrics(metrics1).unwrap();
        monitor.record_metrics(metrics2).unwrap();
        
        let report = monitor.check_for_regressions().unwrap();
        assert_eq!(report.regressions.len(), 4);
    }
}
