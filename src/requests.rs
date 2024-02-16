use crate::dns::Message;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, AsyncWrite};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;

pub enum ConnectionInfo {
    UDP {
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
    },
    TCP {
        stream: Arc<Mutex<TcpStream>>,
    },
}

pub struct Request {
    pub connection_info: ConnectionInfo,
    pub message: Message,
}

impl Request {
    pub fn new_udp(socket: Arc<UdpSocket>, addr: SocketAddr, message: Message) -> Self {
        Self {
            connection_info: ConnectionInfo::UDP { socket, addr },
            message,
        }
    }

    pub fn new_tcp(stream: Arc<Mutex<TcpStream>>, message: Message) -> Self {
        Self {
            connection_info: ConnectionInfo::TCP { stream },
            message,
        }
    }

    pub async fn send_response(&self, response: &[u8]) -> std::io::Result<()> {
        match &self.connection_info {
            ConnectionInfo::UDP { socket, addr } => {
                socket.send_to(response, addr).await?;
            }
            ConnectionInfo::TCP { stream } => {
                let mut stream = stream.lock().await;
                stream.write_all(response).await?;
                stream.flush().await?;
                // Consider if I need to close the stream, adjust accordingly
                // For protocols that keep the connection open, I might not close here
            }
        }
        Ok(())
    }
}
