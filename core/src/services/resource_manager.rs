//! # Resource Management System
//!
//! This module provides functionality for monitoring and controlling system resources
//! used by the blockchain node, including CPU, memory, network bandwidth, and disk space.
//! It helps ensure that the node operates within specified resource limits and provides
//! metrics for resource usage.

use crate::types::{Result, Error};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread::{self, JoinHandle};
use log::{debug, error, info, warn};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt};

/// Configuration for the resource management system
#[derive(Clone, Debug)]
pub struct ResourceManagerConfig {
    /// Maximum CPU usage percentage (0-100)
    pub max_cpu_usage: u8,
    
    /// Maximum memory usage in MB
    pub max_memory_usage: u32,
    
    /// Maximum network bandwidth in KB/s
    pub max_network_bandwidth: u32,
    
    /// Maximum disk space usage in MB
    pub max_disk_usage: u64,
    
    /// Minimum free disk space to maintain in MB
    pub min_free_disk_space: u64,
    
    /// Resource monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    
    /// Whether to enforce resource limits
    pub enforce_limits: bool,
    
    /// Path to the data directory for disk space monitoring
    pub data_directory: String,
}

impl Default for ResourceManagerConfig {
    fn default() -> Self {
        Self {
            max_cpu_usage: 20,
            max_memory_usage: 500,
            max_network_bandwidth: 1024, // 1 MB/s
            max_disk_usage: 10 * 1024,   // 10 GB
            min_free_disk_space: 1024,   // 1 GB
            monitoring_interval_ms: 5000, // 5 seconds
            enforce_limits: true,
            data_directory: ".sebure".to_string(),
        }
    }
}

/// Resource usage statistics
#[derive(Clone, Debug)]
pub struct ResourceUsage {
    /// Current CPU usage percentage (0-100)
    pub cpu_usage: f32,
    
    /// Current memory usage in MB
    pub memory_usage: f32,
    
    /// Current network bandwidth usage in KB/s
    pub network_bandwidth: f32,
    
    /// Current disk space usage in MB
    pub disk_usage: u64,
    
    /// Available free disk space in MB
    pub free_disk_space: u64,
    
    /// Timestamp of the last measurement
    pub timestamp: Instant,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_bandwidth: 0.0,
            disk_usage: 0,
            free_disk_space: 0,
            timestamp: Instant::now(),
        }
    }
}

/// Resource allocation status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceStatus {
    /// Resources are within limits
    Normal,
    
    /// Resources are approaching limits
    Warning,
    
    /// Resources have exceeded limits
    Critical,
    
    /// Resource manager is not monitoring
    Unknown,
}

/// Resource type for specific resource status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    /// CPU resource
    Cpu,
    
    /// Memory resource
    Memory,
    
    /// Network bandwidth resource
    Network,
    
    /// Disk space resource
    Disk,
}

/// Resource allocation recommendation
#[derive(Clone, Debug)]
pub struct ResourceRecommendation {
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Current status of the resource
    pub status: ResourceStatus,
    
    /// Recommended action
    pub recommendation: String,
}

/// Resource manager for monitoring and controlling system resources
pub struct ResourceManager {
    /// Resource manager configuration
    config: ResourceManagerConfig,
    
    /// Current resource usage
    usage: Arc<RwLock<ResourceUsage>>,
    
    /// System information
    system: Arc<Mutex<System>>,
    
    /// Process ID
    pid: u32,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
    
    /// Monitoring thread
    monitor_thread: Option<JoinHandle<()>>,
    
    /// Network usage tracking
    last_network_rx: Arc<RwLock<u64>>,
    last_network_tx: Arc<RwLock<u64>>,
    
    /// Resource status
    status: Arc<RwLock<ResourceStatus>>,
    
    /// Resource recommendations
    recommendations: Arc<RwLock<Vec<ResourceRecommendation>>>,
}

impl ResourceManager {
    /// Create a new resource manager with the given configuration
    pub fn new(config: ResourceManagerConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let pid = std::process::id();
        
        Self {
            config,
            usage: Arc::new(RwLock::new(ResourceUsage::default())),
            system: Arc::new(Mutex::new(system)),
            pid,
            running: Arc::new(RwLock::new(false)),
            monitor_thread: None,
            last_network_rx: Arc::new(RwLock::new(0)),
            last_network_tx: Arc::new(RwLock::new(0)),
            status: Arc::new(RwLock::new(ResourceStatus::Unknown)),
            recommendations: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start the resource manager
    pub fn start(&mut self) -> Result<()> {
        let mut running = self.running.write().unwrap();
        if *running {
            return Err(Error::Validation("Resource manager already running".to_string()));
        }
        
        *running = true;
        
        // Clone Arc references for the monitoring thread
        let config = self.config.clone();
        let usage = self.usage.clone();
        let system = self.system.clone();
        let pid = self.pid;
        let running = self.running.clone();
        let last_network_rx = self.last_network_rx.clone();
        let last_network_tx = self.last_network_tx.clone();
        let status = self.status.clone();
        let recommendations = self.recommendations.clone();
        
        // Create and start the monitoring thread
        let monitor_thread = thread::spawn(move || {
            info!("Resource monitoring thread started");
            
            while *running.read().unwrap() {
                // Refresh system information
                {
                    let mut sys = system.lock().unwrap();
                    sys.refresh_all();
                }
                
                // Update resource usage
                Self::update_resource_usage(
                    &system,
                    &usage,
                    pid,
                    &last_network_rx,
                    &last_network_tx,
                    &config,
                );
                
                // Check resource limits and update status
                Self::check_resource_limits(
                    &usage,
                    &config,
                    &status,
                    &recommendations,
                );
                
                // Sleep until next monitoring interval
                thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
            }
            
            info!("Resource monitoring thread terminated");
        });
        
        self.monitor_thread = Some(monitor_thread);
        
        info!("Resource manager started with config: {:?}", self.config);
        Ok(())
    }
    
    /// Stop the resource manager
    pub fn stop(&mut self) -> Result<()> {
        let mut running = self.running.write().unwrap();
        if !*running {
            return Err(Error::Validation("Resource manager not running".to_string()));
        }
        
        *running = false;
        
        // Wait for monitoring thread to finish
        if let Some(thread) = self.monitor_thread.take() {
            let _ = thread.join();
        }
        
        // Reset status
        *self.status.write().unwrap() = ResourceStatus::Unknown;
        
        info!("Resource manager stopped");
        Ok(())
    }
    
    /// Get the current resource usage
    pub fn get_usage(&self) -> ResourceUsage {
        self.usage.read().unwrap().clone()
    }
    
    /// Get the current resource status
    pub fn get_status(&self) -> ResourceStatus {
        *self.status.read().unwrap()
    }
    
    /// Get resource recommendations
    pub fn get_recommendations(&self) -> Vec<ResourceRecommendation> {
        self.recommendations.read().unwrap().clone()
    }
    
    /// Update the resource manager configuration
    pub fn update_config(&mut self, config: ResourceManagerConfig) {
        self.config = config;
        info!("Resource manager configuration updated: {:?}", self.config);
    }
    
    /// Check if a specific resource is within limits
    pub fn is_resource_available(&self, resource_type: ResourceType, amount: f32) -> bool {
        let usage = self.usage.read().unwrap();
        let config = &self.config;
        
        match resource_type {
            ResourceType::Cpu => {
                usage.cpu_usage + amount <= config.max_cpu_usage as f32
            },
            ResourceType::Memory => {
                usage.memory_usage + amount <= config.max_memory_usage as f32
            },
            ResourceType::Network => {
                usage.network_bandwidth + amount <= config.max_network_bandwidth as f32
            },
            ResourceType::Disk => {
                usage.free_disk_space >= amount as u64
            },
        }
    }
    
    /// Reserve resources for an operation
    pub fn reserve_resources(&self, cpu: f32, memory: f32, network: f32, disk: u64) -> Result<()> {
        if !self.config.enforce_limits {
            return Ok(());
        }
        
        let usage = self.usage.read().unwrap();
        let config = &self.config;
        
        // Check CPU availability
        if usage.cpu_usage + cpu > config.max_cpu_usage as f32 {
            return Err(Error::Validation(format!(
                "CPU usage limit exceeded: current {:.2}% + requested {:.2}% > max {}%",
                usage.cpu_usage, cpu, config.max_cpu_usage
            )));
        }
        
        // Check memory availability
        if usage.memory_usage + memory > config.max_memory_usage as f32 {
            return Err(Error::Validation(format!(
                "Memory usage limit exceeded: current {:.2} MB + requested {:.2} MB > max {} MB",
                usage.memory_usage, memory, config.max_memory_usage
            )));
        }
        
        // Check network availability
        if usage.network_bandwidth + network > config.max_network_bandwidth as f32 {
            return Err(Error::Validation(format!(
                "Network bandwidth limit exceeded: current {:.2} KB/s + requested {:.2} KB/s > max {} KB/s",
                usage.network_bandwidth, network, config.max_network_bandwidth
            )));
        }
        
        // Check disk availability
        if usage.free_disk_space < disk {
            return Err(Error::Validation(format!(
                "Insufficient disk space: available {} MB < requested {} MB",
                usage.free_disk_space, disk
            )));
        }
        
        Ok(())
    }
    
    /// Update resource usage statistics
    fn update_resource_usage(
        system: &Arc<Mutex<System>>,
        usage: &Arc<RwLock<ResourceUsage>>,
        pid: u32,
        last_network_rx: &Arc<RwLock<u64>>,
        last_network_tx: &Arc<RwLock<u64>>,
        config: &ResourceManagerConfig,
    ) {
        let sys = system.lock().unwrap();
        let mut usage_data = usage.write().unwrap();
        
        // Update CPU usage
        let global_cpu = sys.global_cpu_info().cpu_usage();
        
        // Get process-specific CPU usage if available
        if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
            usage_data.cpu_usage = process.cpu_usage();
        } else {
            // Fall back to global CPU usage
            usage_data.cpu_usage = global_cpu;
        }
        
        // Update memory usage
        if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
            usage_data.memory_usage = (process.memory() / 1024 / 1024) as f32; // Convert to MB
        } else {
            // Fall back to estimation
            usage_data.memory_usage = (sys.used_memory() / 1024) as f32 * 0.1; // Estimate 10% of system memory
        }
        
        // Update network bandwidth
        let mut total_rx = 0;
        let mut total_tx = 0;
        
        for (_, network) in sys.networks() {
            total_rx += network.received();
            total_tx += network.transmitted();
        }
        
        let mut last_rx = last_network_rx.write().unwrap();
        let mut last_tx = last_network_tx.write().unwrap();
        
        if *last_rx > 0 && *last_tx > 0 {
            // Calculate bandwidth in KB/s
            let rx_diff = total_rx.saturating_sub(*last_rx);
            let tx_diff = total_tx.saturating_sub(*last_tx);
            let total_diff = rx_diff + tx_diff;
            
            usage_data.network_bandwidth = (total_diff / 1024) as f32 / (config.monitoring_interval_ms as f32 / 1000.0);
        }
        
        *last_rx = total_rx;
        *last_tx = total_tx;
        
        // Update disk usage
        let data_path = std::path::Path::new(&config.data_directory);
        if let Ok(metadata) = std::fs::metadata(data_path) {
            if metadata.is_dir() {
                // Get disk information for the data directory
                if let Some(disk) = sys.disks().iter().find(|d| {
                    if let Ok(mount_path) = std::fs::canonicalize(d.mount_point()) {
                        data_path.starts_with(mount_path)
                    } else {
                        false
                    }
                }) {
                    usage_data.disk_usage = (disk.total_space() - disk.available_space()) / 1024 / 1024; // MB
                    usage_data.free_disk_space = disk.available_space() / 1024 / 1024; // MB
                }
            }
        }
        
        // Update timestamp
        usage_data.timestamp = Instant::now();
        
        debug!("Resource usage updated: CPU: {:.2}%, Memory: {:.2} MB, Network: {:.2} KB/s, Disk: {} MB free",
              usage_data.cpu_usage, usage_data.memory_usage, usage_data.network_bandwidth, usage_data.free_disk_space);
    }
    
    /// Check resource limits and update status
    fn check_resource_limits(
        usage: &Arc<RwLock<ResourceUsage>>,
        config: &ResourceManagerConfig,
        status: &Arc<RwLock<ResourceStatus>>,
        recommendations: &Arc<RwLock<Vec<ResourceRecommendation>>>,
    ) {
        let usage_data = usage.read().unwrap();
        let mut current_status = ResourceStatus::Normal;
        let mut new_recommendations = Vec::new();
        
        // Check CPU usage
        let cpu_percentage = (usage_data.cpu_usage / config.max_cpu_usage as f32) * 100.0;
        if cpu_percentage >= 90.0 {
            current_status = ResourceStatus::Critical;
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Cpu,
                status: ResourceStatus::Critical,
                recommendation: format!(
                    "CPU usage is critical at {:.2}%. Consider reducing max_cpu_usage or optimizing workload.",
                    usage_data.cpu_usage
                ),
            });
        } else if cpu_percentage >= 75.0 {
            if current_status != ResourceStatus::Critical {
                current_status = ResourceStatus::Warning;
            }
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Cpu,
                status: ResourceStatus::Warning,
                recommendation: format!(
                    "CPU usage is high at {:.2}%. Monitor for potential performance issues.",
                    usage_data.cpu_usage
                ),
            });
        }
        
        // Check memory usage
        let memory_percentage = (usage_data.memory_usage / config.max_memory_usage as f32) * 100.0;
        if memory_percentage >= 90.0 {
            current_status = ResourceStatus::Critical;
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Memory,
                status: ResourceStatus::Critical,
                recommendation: format!(
                    "Memory usage is critical at {:.2} MB. Consider increasing max_memory_usage or reducing workload.",
                    usage_data.memory_usage
                ),
            });
        } else if memory_percentage >= 75.0 {
            if current_status != ResourceStatus::Critical {
                current_status = ResourceStatus::Warning;
            }
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Memory,
                status: ResourceStatus::Warning,
                recommendation: format!(
                    "Memory usage is high at {:.2} MB. Monitor for potential memory pressure.",
                    usage_data.memory_usage
                ),
            });
        }
        
        // Check network bandwidth
        let network_percentage = (usage_data.network_bandwidth / config.max_network_bandwidth as f32) * 100.0;
        if network_percentage >= 90.0 {
            current_status = ResourceStatus::Critical;
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Network,
                status: ResourceStatus::Critical,
                recommendation: format!(
                    "Network bandwidth is critical at {:.2} KB/s. Consider increasing max_network_bandwidth or reducing network activity.",
                    usage_data.network_bandwidth
                ),
            });
        } else if network_percentage >= 75.0 {
            if current_status != ResourceStatus::Critical {
                current_status = ResourceStatus::Warning;
            }
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Network,
                status: ResourceStatus::Warning,
                recommendation: format!(
                    "Network bandwidth is high at {:.2} KB/s. Monitor for potential network congestion.",
                    usage_data.network_bandwidth
                ),
            });
        }
        
        // Check disk space
        if usage_data.free_disk_space < config.min_free_disk_space {
            current_status = ResourceStatus::Critical;
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Disk,
                status: ResourceStatus::Critical,
                recommendation: format!(
                    "Free disk space is critically low at {} MB. Free up disk space or increase storage capacity.",
                    usage_data.free_disk_space
                ),
            });
        } else if usage_data.free_disk_space < config.min_free_disk_space * 2 {
            if current_status != ResourceStatus::Critical {
                current_status = ResourceStatus::Warning;
            }
            new_recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Disk,
                status: ResourceStatus::Warning,
                recommendation: format!(
                    "Free disk space is low at {} MB. Consider freeing up disk space.",
                    usage_data.free_disk_space
                ),
            });
        }
        
        // Update status and recommendations
        let mut status_write = status.write().unwrap();
        let mut recommendations_write = recommendations.write().unwrap();
        
        if *status_write != current_status {
            match current_status {
                ResourceStatus::Normal => {
                    info!("Resource status changed to Normal");
                },
                ResourceStatus::Warning => {
                    warn!("Resource status changed to Warning");
                },
                ResourceStatus::Critical => {
                    error!("Resource status changed to Critical");
                },
                ResourceStatus::Unknown => {
                    warn!("Resource status changed to Unknown");
                },
            }
            
            *status_write = current_status;
        }
        
        *recommendations_write = new_recommendations;
    }
    
    /// Calculate the optimal batch size based on available resources
    pub fn calculate_optimal_batch_size(&self, cpu_per_item: f32, memory_per_item: f32) -> usize {
        let usage = self.usage.read().unwrap();
        let config = &self.config;
        
        // Calculate available resources
        let available_cpu = (config.max_cpu_usage as f32 - usage.cpu_usage).max(0.0);
        let available_memory = (config.max_memory_usage as f32 - usage.memory_usage).max(0.0);
        
        // Calculate batch sizes based on each resource
        let cpu_batch_size = if cpu_per_item > 0.0 {
            (available_cpu / cpu_per_item) as usize
        } else {
            usize::MAX
        };
        
        let memory_batch_size = if memory_per_item > 0.0 {
            (available_memory / memory_per_item) as usize
        } else {
            usize::MAX
        };
        
        // Use the minimum batch size to respect all resource constraints
        let batch_size = cpu_batch_size.min(memory_batch_size);
        
        // Ensure a minimum batch size of 1
        batch_size.max(1)
    }
    
    /// Get disk usage information for a specific directory
    pub fn get_directory_size(&self, path: &str) -> Result<u64> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(Error::Validation(format!("Path does not exist: {}", path.display())));
        }
        
        if !path.is_dir() {
            return Err(Error::Validation(format!("Path is not a directory: {}", path.display())));
        }
        
        let mut total_size = 0;
        
        for entry in walkdir::WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
        
        Ok(total_size / 1024 / 1024) // Convert to MB
    }
    
    /// Check if the system has enough resources to run the node
    pub fn check_system_requirements(&self) -> Result<()> {
        let sys = self.system.lock().unwrap();
        
        // Check CPU cores
        let cpu_count = sys.cpus().len();
        if cpu_count < 2 {
            return Err(Error::Validation(format!(
                "Insufficient CPU cores: {} (minimum 2 required)",
                cpu_count
            )));
        }
        
        // Check total memory
        let total_memory = sys.total_memory() / 1024 / 1024; // MB
        if total_memory < 1024 {
            return Err(Error::Validation(format!(
                "Insufficient memory: {} MB (minimum 1024 MB required)",
                total_memory
            )));
        }
        
        // Check disk space
        let data_path = std::path::Path::new(&self.config.data_directory);
        if let Some(disk) = sys.disks().iter().find(|d| {
            if let Ok(mount_path) = std::fs::canonicalize(d.mount_point()) {
                data_path.starts_with(mount_path)
            } else {
                false
            }
        }) {
            let available_space = disk.available_space() / 1024 / 1024; // MB
            if available_space < self.config.min_free_disk_space {
                return Err(Error::Validation(format!(
                    "Insufficient disk space: {} MB available (minimum {} MB required)",
                    available_space, self.config.min_free_disk_space
                )));
            }
        }
        
        Ok(())
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        if *self.running.read().unwrap() {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_resource_manager_config_default() {
        let config = ResourceManagerConfig::default();
        
        assert_eq!(config.max_cpu_usage, 20);
        assert_eq!(config.max_memory_usage, 500);
        assert_eq!(config.max_network_bandwidth, 1024);
        assert_eq!(config.max_disk_usage, 10 * 1024);
        assert_eq!(config.min_free_disk_space, 1024);
        assert_eq!(config.monitoring_interval_ms, 5000);
        assert!(config.enforce_limits);
    }
    
    #[test]
    fn test_resource_manager_creation() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);
        
        assert_eq!(manager.get_status(), ResourceStatus::Unknown);
        
        let usage = manager.get_usage();
        assert_eq!(usage.cpu_usage, 0.0);
        assert_eq!(usage.memory_usage, 0.0);
        assert_eq!(usage.network_bandwidth, 0.0);
    }
    
    #[test]
    fn test_resource_manager_start_stop() {
        let config = ResourceManagerConfig {
            monitoring_interval_ms: 100, // Faster for testing
            ..ResourceManagerConfig::default()
        };
        
        let mut manager = ResourceManager::new(config);
        
        // Start the manager
        assert!(manager.start().is_ok());
        
        // Starting again should fail
        assert!(manager.start().is_err());
        
        // Let it run for a bit to collect some data
        thread::sleep(Duration::from_millis(300));
        
        // Check that we have some data
        let usage = manager.get_usage();
        assert!(usage.cpu_usage >= 0.0);
        
        // Stop the manager
        assert!(manager.stop().is_ok());
        
        // Stopping again should fail
        assert!(manager.stop().is_err());
    }
    
    #[test]
    fn test_resource_availability() {
        let config = ResourceManagerConfig {
            max_cpu_usage: 20,
            max_memory_usage: 500,
            max_network_bandwidth: 1000,
            ..ResourceManagerConfig::default()
        };
        
        let mut manager = ResourceManager::new(config);
        
        // Manually set usage for testing
        {
            let mut usage = manager.usage.write().unwrap();
            usage.cpu_usage = 10.0;
            usage.memory_usage = 200.0;
            usage.network_bandwidth = 500.0;
            usage.free_disk_space = 2000;
        }
        
        // Check resource availability
        assert!(manager.is_resource_available(ResourceType::Cpu, 5.0)); // 10 + 5 <= 20
        assert!(!manager.is_resource_available(ResourceType::Cpu, 15.0)); // 10 + 15 > 20
        
        assert!(manager.is_resource_available(ResourceType::Memory, 200.0)); // 200 + 200 <= 500
        assert!(!manager.is_resource_available(ResourceType::Memory, 400.0)); // 200 + 400 > 500
        
        assert!(manager.is_resource_available(ResourceType::Network, 400.0)); // 500 + 400 <= 1000
        assert!(!manager.is_resource_available(ResourceType::Network, 600.0)); // 500 + 600 > 1000
        
        assert!(manager.is_resource_available(ResourceType::Disk, 1000.0)); // 2000 >= 1000
        assert!(!manager.is_resource_available(ResourceType::Disk, 3000.0)); // 2000 < 3000
    }
    
    #[test]
    fn test_resource_reservation() {
        let config = ResourceManagerConfig {
            max_cpu_usage: 20,
            max_memory_usage: 500,
            max_network_bandwidth: 1000,
            enforce_limits: true,
            ..ResourceManagerConfig::default()
        };
        
        let mut manager = ResourceManager::new(config.clone());
        
        // Manually set usage for testing
        {
            let mut usage = manager.usage.write().unwrap();
            usage.cpu_usage = 10.0;
            usage.memory_usage = 200.0;
            usage.network_bandwidth = 500.0;
            usage.free_disk_space = 2000;
        }
        
        // Valid reservation
        assert!(manager.reserve_resources(5.0, 100.0, 200.0, 1000).is_ok());
        
        // Invalid CPU reservation
        assert!(manager.reserve_resources(15.0, 100.0, 200.0, 1000).is_err());
        
        // Invalid memory reservation
        assert!(manager.reserve_resources(5.0, 400.0, 200.0, 1000).is_err());
        
        // Invalid network reservation
        assert!(manager.reserve_resources(5.0, 100.0, 600.0, 1000).is_err());
        
        // Invalid disk reservation
        assert!(manager.reserve_resources(5.0, 100.0, 200.0, 3000).is_err());
        
        // With enforce_limits = false, all reservations should succeed
        let mut config2 = config.clone();
        config2.enforce_limits = false;
        manager.update_config(config2);
        
        assert!(manager.reserve_resources(15.0, 400.0, 600.0, 3000).is_ok());
    }
    
    #[test]
    fn test_optimal_batch_size() {
        let config = ResourceManagerConfig {
            max_cpu_usage: 20,
            max_memory_usage: 500,
            ..ResourceManagerConfig::default()
        };
        
        let manager = ResourceManager::new(config);
        
        // Manually set usage for testing
        {
            let mut usage = manager.usage.write().unwrap();
            usage.cpu_usage = 10.0;
            usage.memory_usage = 200.0;
        }
        
        // Calculate optimal batch size
        let batch_size = manager.calculate_optimal_batch_size(2.0, 50.0);
        
        // Check that the batch size respects both CPU and memory constraints
        assert_eq!(batch_size, 5); // 10 + 2*5 = 20 (CPU), 200 + 50*5 = 450 (Memory)
    }
}

