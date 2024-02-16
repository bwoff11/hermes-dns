use crate::dns::Message;
use crate::requests::Request;
use crate::settings::ListenersSettings;
use std::io;
use std::sync::Arc;
use tokio::net::{tcp, TcpListener, UdpSocket};
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
        let (tx, rx): (Sender<Request>, Receiver<Request>) = mpsc::channel(100);

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

        if let Some(tcp_listener) = self.tcp_listener.as_ref() {
            let tcp_sender = tx.clone();
            tokio::spawn(Self::handle_tcp(tcp_listener.clone(), tcp_sender));
            println!(
                "Listening for TCP requests on {}",
                tcp_listener.local_addr().unwrap()
            ); // Consider handling the Result properly
        } else {
            println!("TCP listener is disabled");
        }

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


    async fn handle_tcp(listener: Arc<TcpListener>, sender: Sender<Request>) -> io::Result<()> {
        loop {
            /*let (stream, addr) = listener.accept().await?;
            let sender = sender.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 512];
                let size = stream.read(&mut buf).await.unwrap();
                match Message::deserialize(&buf[..size]) {
                    Ok(msg) => {
                        let request = Request::new_tcp(Arc::new(tokio::sync::Mutex::new(stream)), msg);
                        if sender.send(request).await.is_err() {
                            eprintln!("Failed to send TCP request through channel");
                        }
                    }
                    Err(e) => eprintln!("Failed to deserialize message: {}", e),
                }
            });*/
        }
    }
}
