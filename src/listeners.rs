use crate::dns::Message;
use crate::requests::Request;
use crate::settings::ListenersSettings;
use std::io;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Listeners {
    udp_listener: Option<Arc<UdpSocket>>,
    tcp_listener: Option<Arc<TcpListener>>,
}

impl Listeners {
    pub async fn new(settings: &ListenersSettings) -> io::Result<Self> {
        let udp_listener = if settings.udp.enabled {
            let address = format!("{}:{}", settings.udp.address, settings.udp.port);
            Some(Arc::new(UdpSocket::bind(&address).await?))
        } else {
            None
        };

        let tcp_listener = if settings.tcp.enabled {
            let address = format!("{}:{}", settings.tcp.address, settings.tcp.port);
            Some(Arc::new(TcpListener::bind(&address).await?))
        } else {
            None
        };

        Ok(Self {
            udp_listener,
            tcp_listener,
        })
    }

    pub async fn listen(&self) -> io::Result<Receiver<Request>> {
        let (tx, mut rx): (Sender<Request>, Receiver<Request>) = mpsc::channel(100);

        if let Some(udp_socket) = self.udp_listener.as_ref() {
            let udp_sender = tx.clone();
            tokio::spawn(Self::handle_udp(udp_socket.clone(), udp_sender));
            println!(
                "Listening for UDP requests on {}",
                udp_socket.local_addr().unwrap()
            ); // Consider handling the Result properly
        } else {
            println!("UDP listener is disabled");
        }

        // Tcp implementation will be added here later

        Ok(rx)
    }

    async fn handle_udp(socket: Arc<UdpSocket>, sender: Sender<Request>) -> io::Result<()> {
        let mut buf = [0u8; 512];
        loop {
            let (size, addr) = socket.recv_from(&mut buf).await?;
            match Message::deserialize(&buf[..size]) {
                Ok(msg) => {
                    let request = Request::new_udp(socket.clone(), addr, msg);
                    if sender.send(request).await.is_err() {
                        eprintln!("Failed to send UDP request through channel");
                    }
                }
                Err(e) => eprintln!("Failed to deserialize message: {}", e),
            }
        }
    }

    // Placeholder for TCP handler
    // async fn handle_tcp(...) { ... }
}
