use crate::dns::Message as DNSMessage;
use crate::requests::ConnectionInfo;
use crate::requests::Request;
use crate::settings::ListenersSettings;
use std::collections::VecDeque;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

trait Listener {
    fn start(&self, queue: Arc<Mutex<VecDeque<Request>>>, is_running: Arc<AtomicBool>);
}

struct UDPListener {
    port: u16,
}

pub struct Listeners {
    udp: UDPListener,
    //tcp: TCPListener, // Assume TCPListener is defined similarly to UDPListener
    queue: Arc<Mutex<VecDeque<Request>>>,
    is_running: Arc<AtomicBool>,
}

impl Listener for UDPListener {
    fn start(&self, queue: Arc<Mutex<VecDeque<Request>>>, is_running: Arc<AtomicBool>) {
        let addr = format!("0.0.0.0:{}", self.port);
        let socket = UdpSocket::bind(&addr).expect("Failed to bind UDP socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set UDP socket to non-blocking");

        let socket_arc = Arc::new(socket);

        thread::spawn(move || {
            let mut buf = [0; 512];
            while is_running.load(Ordering::SeqCst) {
                let socket = Arc::clone(&socket_arc);
                match socket.recv_from(&mut buf) {
                    Ok((size, src)) => {
                        if let Ok(msg) = DNSMessage::deserialize(&buf[..size]) {
                            let request = Request {
                                connection_info: ConnectionInfo::UDP {
                                    socket: Arc::clone(&socket),
                                    addr: src,
                                },
                                message: msg,
                            };
                            let mut q = queue.lock().unwrap();
                            q.push_back(request);
                        }
                    }
                    Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                        eprintln!("UDP error: {:?}", e);
                    }
                    _ => {}
                }
                thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }
}

impl Listeners {
    pub fn new(settings: &ListenersSettings) -> Self {
        Listeners {
            udp: UDPListener {
                port: settings.udp.port,
            },
            // Assuming TCPListener is defined similarly to UDPListener
            //tcp: TCPListener { port: settings.tcp.port },
            queue: Arc::new(Mutex::new(VecDeque::new())),
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    // Adjust the return type to reflect that the queue now holds Request objects
    pub fn start(&self) -> Arc<Mutex<VecDeque<Request>>> {
        self.udp
            .start(Arc::clone(&self.queue), Arc::clone(&self.is_running));
        // self.tcp.start(Arc::clone(&self.queue), Arc::clone(&self.is_running));
        Arc::clone(&self.queue)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    // Adjust the return type to reflect that the queue now holds Request objects
    pub fn queue(&self) -> Arc<Mutex<VecDeque<Request>>> {
        Arc::clone(&self.queue)
    }
}
