New Client Handling Structure
The main focus of the update is in how the server handles clients and their requests. The Client struct now includes two types of messages:

EchoMessage (previously handled)
AddRequest and AddResponse (newly added for arithmetic operations)
Changes to Client:
Loop in handle() method: The handle() method has been refactored to continuously read incoming data from the client in a loop (loop {}) instead of reading data only once. This ensures that the server can handle multiple requests from the same client.

Message Handling: The server now distinguishes between two types of messages:

EchoMessage: The content is echoed back to the client as before.
AddRequest: The server expects an addition request, processes the two integers a and b, and sends back an AddResponse with the result of the sum.
Error Handling: If neither EchoMessage nor AddRequest can be decoded, the server logs an error message.

pub fn handle(&mut self) -> io::Result<()> {
let mut buffer = [0; 512];
loop {
let bytes_read = self.stream.read(&mut buffer)?;
if bytes_read == 0 {
info!("Client disconnected.");
break;
}

        if let Ok(echo_message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Processed EchoMessage: {}", echo_message.content);
            let payload = echo_message.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        } else if let Ok(add_request) = AddRequest::decode(&buffer[..bytes_read]) {
            let result = add_request.a + add_request.b;
            info!("Processed AddRequest: {} + {} = {}", add_request.a, add_request.b, result);
            let add_response = AddResponse { result };
            let payload = add_response.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        } else {
            error!("Failed to decode message");
        }
    }
    Ok(())

}

<!--  -->

Threaded Server with Shared State
The server has been updated to use threading to handle multiple client connections concurrently. The Server struct has the following changes:

Changes to Server:
TcpListener is Wrapped in Mutex: The TcpListener is wrapped in a Mutex and placed inside an Arc (atomic reference counter). This allows the listener to be shared safely across threads while also being mutable.

Threading: Each client connection is now handled in its own thread. This is accomplished using thread::spawn inside the run() method. Each thread manages its own instance of the Client struct, allowing for parallel processing of multiple clients.

Graceful Shutdown: The is_running flag is used to manage the running state of the server. If the server needs to be stopped, this flag is set to false, and the server will exit its main loop. The server will then wait for each client-handling thread to finish by calling join() on each thread.

pub fn run(&self) -> io::Result<()> {
self.is_running.store(true, Ordering::SeqCst);
info!("Server is running on {}", self.listener.lock().unwrap().local_addr()?);

    self.listener.lock().unwrap().set_nonblocking(true)?;

    let mut threads = Vec::new();

    while self.is_running.load(Ordering::SeqCst) {
        match self.listener.clone().lock().unwrap().accept() {
            Ok((stream, addr)) => {
                info!("New client connected: {}", addr);

                let is_running = self.is_running.clone();
                let handle = thread::spawn(move || {
                    let mut client = Client::new(stream);
                    while is_running.load(Ordering::SeqCst) {
                        if let Err(e) = client.handle() {
                            error!("Error handling client: {}", e);
                            break;
                        }
                    }
                });

                threads.push(handle);
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }

    for handle in threads {
        let _ = handle.join();
    }

    info!("Server stopped.");
    Ok(())

}

<!--  -->

Shared State with Atomic Boolean (Arc<AtomicBool>)
The is_running flag is now an AtomicBool wrapped in an Arc to allow it to be shared safely across threads. This flag is used to control whether the server is still running or if it should stop processing new connections.

AtomicBool is thread-safe and allows the server to check the running state without locking it.
Arc (atomic reference counting) ensures that the flag is shared across multiple threads, preventing issues with ownership and borrowing.

<!--  -->

Non-blocking TcpListener
The server’s TcpListener is set to non-blocking mode using set_nonblocking(true) to avoid blocking the server thread while waiting for incoming connections. When there are no connections to accept, the server sleeps for 50 milliseconds to reduce CPU usage.

self.listener.lock().unwrap().set_nonblocking(true)?;

<!--  -->

Graceful Shutdown
The stop() method gracefully stops the server by setting the is_running flag to false. This is checked in the main loop to ensure that the server can cleanly shut down without abruptly terminating client connections.

pub fn stop(&self) {
if self.is_running.load(Ordering::SeqCst) {
self.is_running.store(false, Ordering::SeqCst);
info!("Shutdown signal sent.");
} else {
warn!("Server was already stopped or not running.");
}
}

<!--  -->

Summary of Key Changes:
Client Handling: Multiple message types (EchoMessage and AddRequest) are processed in a loop, and responses are sent back to the client.
Concurrency: The server now supports multiple client connections concurrently using threads, with shared state managed using Arc<Mutex<TcpListener>> and Arc<AtomicBool>.
Graceful Shutdown: The server can be stopped gracefully using the is_running flag.
Non-blocking TCP Listener: The server uses a non-blocking listener to avoid blocking on accept() calls and sleeps when no connections are present.
