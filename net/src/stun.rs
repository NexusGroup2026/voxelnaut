//! STUN client for NAT traversal
//!
//! Implements STUN (RFC 5389) for NAT type detection and public IP discovery.

use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;

/// STUN server response
#[derive(Debug, Clone)]
pub struct StunResponse {
    pub public_addr: SocketAddr,
    pub nat_type: NatType,
}

/// NAT type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// Open internet - no NAT
    Open,
    /// Full cone NAT - any external can connect
    FullCone,
    /// Restricted cone - external must have sent to internal first
    RestrictedCone,
    /// Port restricted cone - restrictions on port too
    PortRestrictedCone,
    /// Symmetric NAT - different mapping per destination
    Symmetric,
    /// Unknown or unable to determine
    Unknown,
}

impl NatType {
    pub fn can_direct_connect(&self) -> bool {
        matches!(self, NatType::Open | NatType::FullCone)
    }

    pub fn requires_relay(&self) -> bool {
        matches!(self, NatType::Symmetric)
    }
}

/// STUN client
pub struct StunClient {
    server: String,
    timeout: Duration,
}

impl StunClient {
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Get public IP address via STUN
    pub fn get_public_ip(&self) -> std::io::Result<SocketAddr> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(self.timeout))?;
        socket.set_write_timeout(Some(self.timeout))?;

        // Send STUN binding request
        let server_addr: SocketAddr = self.server.parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid STUN server address")
        })?;

        let request = self.build_binding_request();
        socket.send_to(&request, server_addr)?;

        // Receive response
        let mut buf = [0u8; 1024];
        let (bytes, from) = socket.recv_from(&mut buf)?;

        // Parse STUN response
        self.parse_binding_response(&buf[..bytes], from)
    }

    /// Detect NAT type
    pub fn detect_nat_type(&self) -> std::io::Result<NatType> {
        // Simplified NAT detection
        // In production, would send requests to multiple STUN servers and compare
        
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(self.timeout))?;
        
        // Try to connect to STUN server
        let server_addr: SocketAddr = self.server.parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid STUN server address")
        })?;

        // Send binding request
        let request = self.build_binding_request();
        socket.send_to(&request, server_addr)?;

        // Check if we get a response from same address we sent to
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((_, addr)) => {
                if addr == server_addr {
                    // Could be Open or Full Cone - need more testing
                    Ok(NatType::PortRestrictedCone) // Conservative estimate
                } else {
                    Ok(NatType::Symmetric)
                }
            }
            Err(_) => Ok(NatType::Symmetric),
        }
    }

    fn build_binding_request(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        
        // Message type: Binding Request (0x0001)
        msg.push(0x00);
        msg.push(0x01);
        
        // Message length (excluding header)
        msg.push(0x00);
        msg.push(0x00);
        
        // Magic cookie
        msg.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);
        
        // Transaction ID (96 bits = 12 bytes)
        let tid = generate_transaction_id();
        msg.extend_from_slice(&tid);
        
        // Attributes would go here (FINGERPRINT, etc.)
        
        msg
    }

    fn parse_binding_response(&self, data: &[u8], from: SocketAddr) -> std::io::Result<SocketAddr> {
        if data.len() < 20 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "STUN response too short",
            ));
        }

        // Check message type (should be 0x0101 for Binding Success Response)
        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        if msg_type != 0x0101 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not a STUN binding response",
            ));
        }

        // Look for XOR-MAPPED-ADDRESS attribute (0x0020)
        let mut pos = 20;
        while pos < data.len() - 4 {
            let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            
            if attr_type == 0x0020 && attr_len >= 8 {
                // XOR-MAPPED-ADDRESS
                // Skip family (1 byte) and port (2 bytes), then IP (4 bytes)
                let xport = u16::from_be_bytes([data[pos + 6], data[pos + 7]]);
                let xaddr = u32::from_be_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]]);
                
                let port = xport ^ 0x2112;
                let addr = xaddr ^ 0x2112A442;
                
                return Ok(SocketAddr::from((addr, port)));
            }
            
            pos += 4 + attr_len;
            // Align to 4 bytes
            pos = (pos + 3) & !3;
        }

        // Fallback: use source address if no MAPPED-ADDRESS found
        Ok(from)
    }
}

fn generate_transaction_id() -> [u8; 12] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut tid = [0u8; 12];
    for (i, byte) in tid.iter_mut().enumerate() {
        *byte = ((timestamp >> (i * 8)) & 0xFF) as u8;
    }
    tid
}

/// TURN client for relay fallback
/// 
/// When direct P2P connection is not possible due to symmetric NAT,
/// TURN (RFC 5766) provides a relay server to forward traffic.
/// 
/// This is a placeholder - full TURN implementation requires
/// authentication and is typically provided by services like
/// Twilio, Xirsys, or custom TURN servers.
pub struct TurnClient {
    server: String,
    username: Option<String>,
    password: Option<String>,
}

impl TurnClient {
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            username: None,
            password: None,
        }
    }

    pub fn with_credentials(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// Allocate a relay address
    /// Returns the relay address to share with peers
    pub fn allocate(&self) -> std::io::Result<String> {
        // In production, this would:
        // 1. Connect to TURN server
        // 2. Send Allocate request with credentials
        // 3. Receive relay address
        // 4. Return relay address for sharing
        
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TURN relay not implemented - requires external service",
        ))
    }
}