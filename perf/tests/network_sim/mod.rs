use sebure_core::network::{NetworkMessage, NetworkSimulator};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub latency: Duration,
    pub packet_loss: f32,
    pub bandwidth: u64, // bytes per second
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            latency: Duration::from_millis(50),
            packet_loss: 0.0,
            bandwidth: 10_000_000, // 10 MB/s
        }
    }
}

pub struct SimulatedNetwork {
    config: NetworkConfig,
    nodes: Vec<Arc<NetworkSimulator>>,
}

impl SimulatedNetwork {
    pub fn new(config: NetworkConfig, node_count: usize) -> Self {
        let nodes = (0..node_count)
            .map(|_| Arc::new(NetworkSimulator::new()))
            .collect();
            
        Self { config, nodes }
    }

    pub fn send_message(&self, from: usize, to: usize, message: NetworkMessage) {
        if rand::random::<f32>() < self.config.packet_loss {
            return; // Simulate packet loss
        }

        let latency = self.config.latency;
        let message_size = message.size_bytes() as f64;
        let transfer_time = Duration::from_secs_f64(
            message_size / self.config.bandwidth as f64
        );

        let total_delay = latency + transfer_time;
        
        let to_node = self.nodes[to].clone();
        let message = message.clone();
        
        std::thread::spawn(move || {
            std::thread::sleep(total_delay);
            to_node.receive_message(message);
        });
    }

    pub fn broadcast_message(&self, from: usize, message: NetworkMessage) {
        for (i, node) in self.nodes.iter().enumerate() {
            if i != from {
                self.send_message(from, i, message.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sebure_core::network::NetworkMessageType;

    #[test]
    fn test_network_simulation() {
        let config = NetworkConfig {
            latency: Duration::from_millis(100),
            packet_loss: 0.1,
            bandwidth: 1_000_000, // 1 MB/s
        };
        
        let network = SimulatedNetwork::new(config, 3);
        let message = NetworkMessage::new(
            NetworkMessageType::Transaction,
            vec![1, 2, 3]
        );
        
        network.send_message(0, 1, message.clone());
        network.broadcast_message(0, message);
    }
}
