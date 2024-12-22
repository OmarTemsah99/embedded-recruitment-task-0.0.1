use embedded_recruitment_task::{
    message::{ client_message, server_message, AddRequest, EchoMessage },
    server::Server,
};
use std::{ sync::Arc, thread::{ self, JoinHandle }, time::{ Duration, Instant } };

mod client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

fn create_server(port: u16) -> Arc<Server> {
    Arc::new(Server::new(&format!("localhost:{}", port)).expect("Failed to start server"))
}

#[test]
fn test_client_connection() {
    let server = create_server(8080);
    let handle = setup_server_thread(server.clone());

    // Wait for the server to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut client = client::Client::new("localhost", 8080, 2000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");
    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_client_echo_message() {
    let server = create_server(8081);
    let handle = setup_server_thread(server.clone());
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut client = client::Client::new("localhost", 8081, 2000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let start = Instant::now();
    let response = client.receive().expect("Failed to receive response for EchoMessage");

    assert!(start.elapsed() < Duration::from_secs(1), "Response took too long"); // Adjust timeout as needed

    match response.message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(echo.content, "Hello, World!", "Echoed message content does not match");
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_multiple_echo_messages() {
    let server = create_server(8082);
    let handle = setup_server_thread(server.clone());
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut client = client::Client::new("localhost", 8082, 2000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let messages = vec!["Hello, World!", "How are you?", "Goodbye!"];

    for content in &messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = content.to_string();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        assert!(client.send(message).is_ok(), "Failed to send message");

        let start = Instant::now();
        let response = client.receive().expect("Failed to receive response for EchoMessage");

        assert!(start.elapsed() < Duration::from_secs(1), "Response took too long"); // Adjust timeout as needed

        match response.message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(echo.content, *content, "Echoed message content does not match");
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_multiple_clients() {
    let server = create_server(8083);
    let handle = setup_server_thread(server.clone());
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut clients = vec![
        client::Client::new("localhost", 8083, 2000),
        client::Client::new("localhost", 8083, 2000),
        client::Client::new("localhost", 8083, 2000)
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    let messages = vec!["Hello, World!", "How are you?", "Goodbye!"];

    for content in &messages {
        for client in clients.iter_mut() {
            let mut echo_message = EchoMessage::default();
            echo_message.content = content.to_string();
            let message = client_message::Message::EchoMessage(echo_message.clone());

            assert!(client.send(message).is_ok(), "Failed to send message");

            let start = Instant::now();
            let response = client.receive().expect("Failed to receive response for EchoMessage");

            assert!(start.elapsed() < Duration::from_secs(1), "Response took too long"); // Adjust timeout as needed

            match response.message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(echo.content, *content, "Echoed message content does not match");
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    for client in clients.iter_mut() {
        assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");
    }

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_client_add_request() {
    let server = create_server(8084);
    let handle = setup_server_thread(server.clone());
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut client = client::Client::new("localhost", 8084, 2000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let start = Instant::now();
    let response = client.receive().expect("Failed to receive response for AddRequest");

    assert!(start.elapsed() < Duration::from_secs(1), "Response took too long"); // Adjust timeout as needed

    match response.message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(add_response.result, 30, "AddResponse result does not match");
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}
