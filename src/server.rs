use crate::message::{ AddRequest, AddResponse, EchoMessage };
use log::{ debug, error, info, warn };
use prost::Message;
use std::{
    io::{ self, ErrorKind, Read, Write },
    net::{ TcpListener, TcpStream },
    sync::{ atomic::{ AtomicBool, Ordering }, Arc, Mutex },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 1024]; // Increased buffer size to handle larger messages
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    info!("Client disconnected.");
                    break;
                }
                Ok(bytes_read) => {
                    debug!("Received raw bytes: {:?}", &buffer[..bytes_read]);

                    if let Ok(echo_message) = EchoMessage::decode(&buffer[..bytes_read]) {
                        info!("Processed EchoMessage: {}", echo_message.content);
                        let payload = echo_message.encode_to_vec();
                        self.stream.write_all(&payload)?;
                        self.stream.flush()?;
                    } else if let Ok(add_request) = AddRequest::decode(&buffer[..bytes_read]) {
                        info!(
                            "Processed AddRequest: {} + {} = {}",
                            add_request.a,
                            add_request.b,
                            add_request.a + add_request.b
                        );
                        let add_response = AddResponse {
                            result: add_request.a + add_request.b,
                        };
                        let payload = add_response.encode_to_vec();
                        self.stream.write_all(&payload)?;
                        self.stream.flush()?;
                    } else {
                        error!("Failed to decode message: {:?}", &buffer[..bytes_read]);
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Non-blocking read timeout
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    error!("Error reading from client: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

pub struct Server {
    listener: Arc<Mutex<TcpListener>>,
    is_running: Arc<AtomicBool>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?; // Set non-blocking mode at listener creation
        let listener = Arc::new(Mutex::new(listener));
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        info!("Server is running on {}", self.listener.lock().unwrap().local_addr()?);

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.lock().unwrap().accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    let is_running = self.is_running.clone();
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No connection ready, wait for a short period
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
