//! # Network Transport
//! 
//! This module implements the transport layer for network communications.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use crate::network::{Message, Protocol, Peer, PeerInfo, ConnectionState};
use crate::types::{Result, Error};

/// Maximum message size for transport (4MB)
const MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Read timeout in seconds
    pub read_timeout: u64,
    
    /// Write timeout in seconds
    pub write_timeout: u64,
    
    /// Maximum message size
    pub max_message_size: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            connection_timeout: 10,
            read_timeout: 30,
            write_timeout: 30,
            max_message_size: MAX_MESSAGE_SIZE,
        }
    }
}

/// Network transport error
#[derive(Debug)]
pub enum TransportError {
    /// I/O error
    IoError(std::io::Error),
    
    /// Connection closed
    ConnectionClosed,
    
    /// Message too large
    MessageTooLarge(usize),
    
    /// Serialization error
    SerializationError(bincode::Error),
    
    /// Timeout
    Timeout,
    
    /// Other error
    Other(String),
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::IoError(err)
    }
}

impl From<bincode::Error> for TransportError {
    fn from(err: bincode::Error) -> Self {
        TransportError::SerializationError(err)
    }
}

/// Transport represents a network transport layer
pub struct Transport {
    /// Transport configuration
    config: TransportConfig,
    
    /// Protocol instance
    protocol: Arc<Protocol>,
    
    /// Active connections
    connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    
    /// Running state
    running: Arc<Mutex<bool>>,
    
    /// Listener thread handle
    listener_thread: Option<thread::JoinHandle<()>>,
}

impl Transport {
    /// Create a new transport instance
    pub fn new(config: TransportConfig, protocol: Protocol) -> Self {
        Transport {
            config,
            protocol: Arc::new(protocol),
            connections: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            listener_thread: None,
        }
    }
    
    /// Start the transport service
    pub fn start(&mut self, listen_addr: SocketAddr) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::Network("Transport already running".to_string()));
        }
        
        *running = true;
        
        // Start listener thread
        let running_clone = self.running.clone();
        let connections_clone = self.connections.clone();
        let protocol_clone = self.protocol.clone();
        let config_clone = self.config.clone();
        
        self.listener_thread = Some(thread::spawn(move || {
            Self::listener_thread(listen_addr, running_clone, connections_clone, protocol_clone, config_clone);
        }));
        
        log::info!("Transport started, listening on {}", listen_addr);
        
        Ok(())
    }
    
    /// Stop the transport service
    pub fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::Network("Transport not running".to_string()));
        }
        
        *running = false;
        
        // Close all connections
        let mut connections = self.connections.lock().unwrap();
        connections.clear();
        
        // Wait for listener thread to end
        if let Some(handle) = self.listener_thread.take() {
            if handle.join().is_err() {
                log::error!("Error joining listener thread");
            }
        }
        
        log::info!("Transport stopped");
        
        Ok(())
    }
    
    /// Connect to a peer
    pub fn connect(&self, addr: SocketAddr) -> std::result::Result<(), TransportError> {
        // Check if already connected
        let mut connections = self.connections.lock().unwrap();
        if connections.contains_key(&addr) {
            return Ok(());
        }
        
        // Create a TCP connection
        let stream = self.create_connection(addr)?;
        
        // Add to connections map
        connections.insert(addr, stream);
        
        Ok(())
    }
    
    /// Disconnect from a peer
    pub fn disconnect(&self, addr: &SocketAddr) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if connections.remove(addr).is_none() {
            return Err(Error::Network(format!("Not connected to {}", addr)));
        }
        
        Ok(())
    }
    
    /// Send a message to a specific peer
    pub fn send(&self, addr: &SocketAddr, message: &Message) -> std::result::Result<(), TransportError> {
        let mut connections = self.connections.lock().unwrap();
        
        if let Some(stream) = connections.get_mut(addr) {
            // Serialize the message
            let data = message.serialize()?;
            
            // Check size limit
            if data.len() > self.config.max_message_size {
                return Err(TransportError::MessageTooLarge(data.len()));
            }
            
            // Send the size prefix
            let size = data.len() as u32;
            let size_buf = size.to_be_bytes();
            
            // Set write timeout
            stream.set_write_timeout(Some(Duration::from_secs(self.config.write_timeout)))?;
            
            // Write size and data
            stream.write_all(&size_buf)?;
            stream.write_all(&data)?;
            stream.flush()?;
            
            Ok(())
        } else {
            Err(TransportError::Other(format!("Not connected to {}", addr)))
        }
    }
    
    /// Receive a message from a specific peer
    pub fn receive(&self, addr: &SocketAddr) -> std::result::Result<Message, TransportError> {
        let mut connections = self.connections.lock().unwrap();
        
        if let Some(stream) = connections.get_mut(addr) {
            // Set read timeout
            stream.set_read_timeout(Some(Duration::from_secs(self.config.read_timeout)))?;
            
            // Read the size prefix
            let mut size_buf = [0u8; 4];
            match stream.read_exact(&mut size_buf) {
                Ok(_) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Err(TransportError::ConnectionClosed);
                },
                Err(e) => return Err(TransportError::IoError(e)),
            }
            
            let size = u32::from_be_bytes(size_buf) as usize;
            
            // Check size limit
            if size > self.config.max_message_size {
                return Err(TransportError::MessageTooLarge(size));
            }
            
            // Read the message data
            let mut data = vec![0u8; size];
            match stream.read_exact(&mut data) {
                Ok(_) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Err(TransportError::ConnectionClosed);
                },
                Err(e) => return Err(TransportError::IoError(e)),
            }
            
            // Deserialize the message
            let message = Message::deserialize(&data)?;
            
            Ok(message)
        } else {
            Err(TransportError::Other(format!("Not connected to {}", addr)))
        }
    }
    
    /// Create a TCP connection
    fn create_connection(&self, addr: SocketAddr) -> std::result::Result<TcpStream, TransportError> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(self.config.connection_timeout))?;
        
        // Configure the stream
        stream.set_nodelay(true)?;
        
        Ok(stream)
    }
    
    /// Listener thread function
    fn listener_thread(
        listen_addr: SocketAddr,
        running: Arc<Mutex<bool>>,
        connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
        protocol: Arc<Protocol>,
        config: TransportConfig,
    ) {
        // Create TCP listener
        let listener = match TcpListener::bind(listen_addr) {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to bind listener: {}", e);
                return;
            }
        };
        
        // Set non-blocking so we can check running state
        if let Err(e) = listener.set_nonblocking(true) {
            log::error!("Failed to set non-blocking: {}", e);
            return;
        }
        
        log::info!("Listener active on {}", listen_addr);
        
        // Accept loop
        while *running.lock().unwrap() {
            match listener.accept() {
                Ok((stream, addr)) => {
                    log::debug!("Accepted connection from {}", addr);
                    
                    // Configure the stream
                    if let Err(e) = stream.set_nodelay(true) {
                        log::warn!("Failed to set nodelay: {}", e);
                        continue;
                    }
                    
                    // Add to connections map
                    let mut conns = connections.lock().unwrap();
                    conns.insert(addr, stream);
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, sleep a bit
                    thread::sleep(Duration::from_millis(100));
                },
                Err(e) => {
                    log::warn!("Error accepting connection: {}", e);
                }
            }
        }
    }
    
    /// Get the number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
    
    /// Check if connected to a peer
    pub fn is_connected(&self, addr: &SocketAddr) -> bool {
        self.connections.lock().unwrap().contains_key(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    use crate::network::{ProtocolConfig, MessageType};
    use crate::types::Priority;
    
    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    fn create_test_protocol() -> Protocol {
        Protocol::new(
            ProtocolConfig::default(),
            "sebure-testnet".to_string(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            "sebure-test/1.0.0".to_string(),
        )
    }
    
    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        
        assert_eq!(config.connection_timeout, 10);
        assert_eq!(config.read_timeout, 30);
        assert_eq!(config.write_timeout, 30);
        assert_eq!(config.max_message_size, MAX_MESSAGE_SIZE);
    }
    
    #[test]
    fn test_transport_creation() {
        let config = TransportConfig::default();
        let protocol = create_test_protocol();
        
        let transport = Transport::new(config, protocol);
        
        assert_eq!(transport.connection_count(), 0);
    }
    
    // Note: More comprehensive transport tests would require
    // actual networking, which is beyond the scope of unit tests.
    // In a real implementation, we would use mocks or integration tests.
}
