use crate::dns::Message;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::Arc;

pub trait Respondable {
    fn send_response(&self, data: &[u8]) -> std::io::Result<()>;
}

pub struct Request {
    pub connection_info: ConnectionInfo,
    pub message: Message,
}

use std::sync::Mutex;

pub enum ConnectionInfo {
    UDP {
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
    },
    TCP {
        stream: Mutex<TcpStream>,
    },
}

impl Request {
    pub fn respond(&self, message: &Message) -> std::io::Result<()> {
        let response_data = message.serialize();
        self.connection_info.send_response(&response_data)
    }
}

impl Respondable for ConnectionInfo {
    fn send_response(&self, data: &[u8]) -> std::io::Result<()> {
        match self {
            ConnectionInfo::UDP { socket, addr } => {
                socket.send_to(data, addr)?;
            }
            ConnectionInfo::TCP { stream } => {
                let mut stream = stream.lock().unwrap(); // Lock the mutex to get mutable access
                let mut len_buf = [0u8; 2];
                len_buf.copy_from_slice(&(data.len() as u16).to_be_bytes());
                stream.write_all(&len_buf)?;
                stream.write_all(data)?;
            }
        }
        Ok(())
    }
}
