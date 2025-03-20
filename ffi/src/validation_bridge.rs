//! Validation Service IPC Bridge
//!
//! This module provides the interface between the Flutter UI and the Rust validation service,
//! enabling communication through FFI and managing the validation service lifecycle.

use sebure_core::{ValidationService, ValidationServiceConfig, TaskPriority, ServiceStatus, Blockchain, Result};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_ulonglong};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};

// Global instance of the validation service
lazy_static! {
    static ref VALIDATION_SERVICE: Mutex<Option<ValidationService>> = Mutex::new(None);
    static ref SERVICE_INSTANCES: Mutex<HashMap<u32, Arc<RwLock<ValidationService>>>> = Mutex::new(HashMap::new());
    static ref NEXT_SERVICE_ID: Mutex<u32> = Mutex::new(1);
}

/// Create and initialize a validation service
///
/// # Safety
///
/// This function is unsafe because it takes a raw pointer and modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_create(
    max_cpu_usage: c_uint,
    max_memory_usage: c_uint,
    queue_size_limit: c_uint,
    processing_time_slot_ms: c_uint,
    batch_size: c_uint,
) -> c_uint {
    // Get shared resources
    let blockchain = match super::get_blockchain() {
        Some(blockchain) => blockchain,
        None => {
            error!("Failed to get blockchain instance");
            return 0;
        }
    };
    
    // Create validation service configuration
    let config = ValidationServiceConfig {
        max_cpu_usage: max_cpu_usage.min(100) as u8,  // Ensure max_cpu_usage is at most 100
        max_memory_usage: max_memory_usage,
        queue_size_limit: queue_size_limit as usize,
        processing_time_slot_ms: processing_time_slot_ms as u64,
        batch_size: batch_size as usize,
        ..ValidationServiceConfig::default()
    };
    
    // Create validation service
    let validation_service = ValidationService::new(blockchain, config);
    
    // Generate a new service ID
    let mut next_id = NEXT_SERVICE_ID.lock().unwrap();
    let service_id = *next_id;
    *next_id += 1;
    
    // Store the service instance
    let service_instance = Arc::new(RwLock::new(validation_service));
    SERVICE_INSTANCES.lock().unwrap().insert(service_id, service_instance);
    
    info!("Created validation service with ID: {}", service_id);
    service_id
}

/// Start a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_start(service_id: c_uint) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Start the service
    let mut service = service.write().unwrap();
    match service.start() {
        Ok(_) => {
            info!("Started validation service with ID: {}", service_id);
            0
        }
        Err(e) => {
            error!("Failed to start validation service: {}", e);
            -1
        }
    }
}

/// Stop a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_stop(service_id: c_uint) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Stop the service
    let mut service = service.write().unwrap();
    match service.stop() {
        Ok(_) => {
            info!("Stopped validation service with ID: {}", service_id);
            0
        }
        Err(e) => {
            error!("Failed to stop validation service: {}", e);
            -1
        }
    }
}

/// Destroy a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_destroy(service_id: c_uint) -> c_int {
    // Get the service instance
    let mut instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service.clone(),
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Stop the service if it's running
    {
        let mut service = service.write().unwrap();
        if service.status() != ServiceStatus::Stopped {
            if let Err(e) = service.stop() {
                error!("Failed to stop validation service: {}", e);
                return -1;
            }
        }
    }
    
    // Remove the service instance
    instances.remove(&service_id);
    info!("Destroyed validation service with ID: {}", service_id);
    0
}

/// Get the status of a validation service
///
/// # Safety
///
/// This function is unsafe because it accesses global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_status(service_id: c_uint) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Get the service status
    let service = service.read().unwrap();
    service.status() as c_int
}

/// Pause a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_pause(service_id: c_uint) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Pause the service
    let service = service.read().unwrap();
    match service.pause() {
        Ok(_) => {
            info!("Paused validation service with ID: {}", service_id);
            0
        }
        Err(e) => {
            error!("Failed to pause validation service: {}", e);
            -1
        }
    }
}

/// Resume a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_resume(service_id: c_uint) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Resume the service
    let service = service.read().unwrap();
    match service.resume() {
        Ok(_) => {
            info!("Resumed validation service with ID: {}", service_id);
            0
        }
        Err(e) => {
            error!("Failed to resume validation service: {}", e);
            -1
        }
    }
}

/// Get statistics from a validation service
///
/// # Safety
///
/// This function is unsafe because it takes a raw pointer and accesses global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_get_stats(
    service_id: c_uint,
    transactions_processed: *mut c_ulonglong,
    blocks_validated: *mut c_ulonglong,
    blocks_generated: *mut c_ulonglong,
    validation_errors: *mut c_ulonglong,
    queue_length: *mut c_uint,
    avg_transaction_time_ms: *mut f64,
    uptime_seconds: *mut c_ulonglong,
    cpu_usage: *mut f32,
    memory_usage: *mut f32,
) -> c_int {
    // Check pointers
    if transactions_processed.is_null() || blocks_validated.is_null() || blocks_generated.is_null()
        || validation_errors.is_null() || queue_length.is_null() || avg_transaction_time_ms.is_null()
        || uptime_seconds.is_null() || cpu_usage.is_null() || memory_usage.is_null()
    {
        error!("Invalid pointer in sebure_validation_service_get_stats");
        return -1;
    }
    
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Get the service stats
    let service = service.read().unwrap();
    let stats = service.stats();
    
    // Fill in the output parameters
    *transactions_processed = stats.transactions_processed;
    *blocks_validated = stats.blocks_validated;
    *blocks_generated = stats.blocks_generated;
    *validation_errors = stats.validation_errors;
    *queue_length = stats.queue_length as c_uint;
    *avg_transaction_time_ms = stats.avg_transaction_time_ms;
    *uptime_seconds = stats.uptime_seconds;
    *cpu_usage = stats.cpu_usage;
    *memory_usage = stats.memory_usage;
    
    0
}

/// Update configuration for a validation service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_update_config(
    service_id: c_uint,
    max_cpu_usage: c_uint,
    max_memory_usage: c_uint,
    queue_size_limit: c_uint,
    processing_time_slot_ms: c_uint,
    batch_size: c_uint,
) -> c_int {
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Create validation service configuration
    let config = ValidationServiceConfig {
        max_cpu_usage: max_cpu_usage.min(100) as u8,  // Ensure max_cpu_usage is at most 100
        max_memory_usage: max_memory_usage,
        queue_size_limit: queue_size_limit as usize,
        processing_time_slot_ms: processing_time_slot_ms as u64,
        batch_size: batch_size as usize,
        ..ValidationServiceConfig::default()
    };
    
    // Update the service configuration
    let mut service = service.write().unwrap();
    service.update_config(config);
    
    info!("Updated configuration for validation service with ID: {}", service_id);
    0
}

/// Add a task to the validation service
///
/// # Safety
///
/// This function is unsafe because it accesses global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_validation_service_add_task(
    service_id: c_uint,
    task_type: c_int,
    priority: c_int,
    data: *const c_char,
    task_id_out: *mut c_ulonglong,
) -> c_int {
    // Check pointers
    if task_id_out.is_null() {
        error!("Invalid pointer in sebure_validation_service_add_task");
        return -1;
    }
    
    // Get the service instance
    let instances = SERVICE_INSTANCES.lock().unwrap();
    let service = match instances.get(&service_id) {
        Some(service) => service,
        None => {
            error!("Invalid service ID: {}", service_id);
            return -1;
        }
    };
    
    // Convert C string to Rust string if provided
    let data_str = if !data.is_null() {
        let data_cstr = CStr::from_ptr(data);
        match data_cstr.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                error!("Invalid UTF-8 in data string");
                return -1;
            }
        }
    } else {
        None
    };
    
    // Convert priority to TaskPriority
    let task_priority = match priority {
        0 => TaskPriority::Low,
        1 => TaskPriority::Medium,
        2 => TaskPriority::High,
        3 => TaskPriority::Critical,
        _ => {
            error!("Invalid task priority: {}", priority);
            return -1;
        }
    };
    
    // Add the task based on type
    let service = service.read().unwrap();
    let result = match task_type {
        0 => {
            // Block generation task
            service.add_block_generation_task(task_priority)
        }
        1 => {
            // Custom task
            if let Some(name) = data_str {
                service.add_custom_task(name, vec![], task_priority)
            } else {
                error!("Custom task requires a name");
                return -1;
            }
        }
        _ => {
            error!("Invalid task type: {}", task_type);
            return -1;
        }
    };
    
    // Handle the result
    match result {
        Ok(id) => {
            *task_id_out = id;
            0
        }
        Err(e) => {
            error!("Failed to add task: {}", e);
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would go here
}
