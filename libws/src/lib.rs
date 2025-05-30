// Public module for WebSocket client functionality
pub mod ws_client;

use axum::{
    extract::ws::{ Message, WebSocket, WebSocketUpgrade },
    extract::ConnectInfo,
    response::IntoResponse,
};
use futures_util::{ SinkExt, StreamExt };
use serde_json::{ json, Value };
use std::{ collections::HashMap, net::SocketAddr, sync::{ Arc, Mutex } };
use tokio::sync::mpsc::{ self, UnboundedSender };

use libjwt::validate_jwt;

// Type aliases for topic names and subscriber management
pub type Topic = String;
pub type Subscribers = Arc<Mutex<HashMap<Topic, Vec<UnboundedSender<String>>>>>;

/// TODO: NOT SURE IF THIS IS STILL CORRECT -- I need to check my original code for libws 
/// Should the JWT token is passed in the query string of the WebSocket connection request?
/// 
/// From lib.rs in engine, when you load the ws server call a wrapper function that will 
/// call the handle_socket_with_jwt function
//  .route(
//            "/ws",
//            get(handle_socket_adapter),
//        )
// 
/// Adapter function to bridge between server and library
/// async fn handle_socket_adapter(
///    ws: WebSocketUpgrade,
///    ConnectInfo(addr): ConnectInfo<SocketAddr>,
///    State(subscribers): State<Subscribers>,
///    query_params: Option<Query<WebSocketParams>>,  // Add query parameters
/// ) -> impl IntoResponse {
///    // Call the libws handler with query parameters
///    libws::handle_socket(ws, ConnectInfo(addr), query_params, subscribers).await
/// }

/// Handles the WebSocket upgrade and initializes the connection with JWT validation.
pub async fn handle_socket_with_jwt(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    subscribers: Subscribers,
    token: String,
) -> impl IntoResponse {
    println!("[handle_socket_with_jwt] WS connection from {}", addr);

    // Convert both branches to Response<Body> using into_response()
    if validate_jwt(&token).is_ok() {
        println!("[handle_socket_with_jwt] JWT token is valid");
        handle_socket(ws, ConnectInfo(addr), subscribers).await.into_response()
    } else {
        println!("[handle_socket_with_jwt] JWT token is invalid");
        ws.on_upgrade(|socket| async move {
            let _ = socket.close().await;
            println!("[handle_socket_with_jwt] Connection closed due to invalid JWT");
        }).into_response()
    }
}

/// Handles the WebSocket upgrade and initializes the connection.
pub async fn handle_socket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    subscribers: Subscribers
) -> impl IntoResponse {
    println!("[handle_socket] WS connection from {}", addr);

    // Upgrade the connection and run the WebSocket handler
    ws.on_upgrade(move |socket| {
        async move {
            if let Err(e) = run_connection(socket, subscribers).await {
                eprintln!("[handle_socket] Client error: {:?}", e);
            }
        }
    })
}

/// Manages the WebSocket connection, handling messages, subscriptions, and publishing.
async fn run_connection(socket: WebSocket, subscribers: Subscribers) -> Result<(), String> {
    println!("[run_connection] Executing WebSocket connection handler...");

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Track topics the client is subscribed to
    let my_topics = Arc::new(Mutex::new(Vec::<String>::new()));

    // Create a channel for sending messages to the client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tx_clone = tx.clone();
    let subscribers_inner = subscribers.clone();
    let topics_inner = my_topics.clone();

    // Task for sending messages to the client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task for receiving messages from the client
    let receive_task = tokio::spawn(async move {
        let mut client_name = "<unknown>".to_string();
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    // Handle client name registration
                    if let Some(rest) = text.strip_prefix("register-name:") {
                        client_name = rest.trim().to_string();
                        println!("[register-name] => {}", client_name);

                        // Handle topic subscription
                    } else if let Some(rest) = text.strip_prefix("subscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[subscribe] subscriber_name={}, topic={}", client_name, topic);

                        subscribers_inner
                            .lock()
                            .unwrap()
                            .entry(topic.clone())
                            .or_default()
                            .push(tx.clone());

                        topics_inner.lock().unwrap().push(topic);

                        // Handle topic unsubscription
                    } else if let Some(rest) = text.strip_prefix("unsubscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[unsubscribe] {} unsubscribing from {}", client_name, topic);

                        let mut subs = subscribers_inner.lock().unwrap();
                        if let Some(vec) = subs.get_mut(&topic) {
                            vec.retain(|s| !same_channel(s, &tx));
                        }
                        topics_inner
                            .lock()
                            .unwrap()
                            .retain(|t| t != &topic);

                        // Handle JSON message publishing
                    } else if let Some(rest) = text.strip_prefix("publish-json:") {
                        match serde_json::from_str::<Value>(rest) {
                            Ok(parsed) => {
                                let topic = parsed["topic"]
                                    .as_str()
                                    .unwrap_or("<none>")
                                    .to_string();
                                let payload = parsed["payload"].as_str().unwrap_or("").to_string();
                                let publisher = parsed["publisher_name"]
                                    .as_str()
                                    .unwrap_or("<unknown>")
                                    .to_string();
                                let timestamp = parsed["timestamp"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string();

                                println!(
                                    "[publish-json] publisher_name={}, topic={}, payload={}, timestamp={}",
                                    publisher,
                                    topic,
                                    payload,
                                    timestamp
                                );

                                let json_payload =
                                    json!({
                                    "publisher_name": publisher,
                                    "topic": topic,
                                    "payload": payload,
                                    "timestamp": timestamp
                                }).to_string();

                                let mut subs = subscribers_inner.lock().unwrap();
                                for (topic, sinks) in subs.iter() {
                                    println!(
                                        "[DEBUG] Topic '{}' has {} subscribers",
                                        topic,
                                        sinks.len()
                                    );
                                }

                                if let Some(sinks) = subs.get_mut(&topic) {
                                    println!(
                                        "[publish-json] Subscribers for topic '{}': {}",
                                        topic,
                                        sinks.len()
                                    );

                                    let mut to_remove = Vec::new();
                                    for (i, s) in sinks.iter().enumerate() {
                                        if s.send(json_payload.clone()).is_err() {
                                            eprintln!(
                                                "[publish-json] Failed to send to subscriber."
                                            );
                                            to_remove.push(i);
                                        } else {
                                            println!("[publish-json] Sent to topic '{}'", topic);
                                        }
                                    }
                                    // Remove dead senders (in reverse order to avoid shifting indices)
                                    for i in to_remove.into_iter().rev() {
                                        sinks.remove(i);
                                    }
                                } else {
                                    println!("[publish-json] No subscribers for topic '{}'", topic);
                                }
                            }
                            Err(err) => {
                                eprintln!("[publish-json] Failed to parse JSON: {}", err);
                            }
                        }
                    }
                }
                Ok(_) => eprintln!("[run_connection] Received non-text message"),
                Err(e) => {
                    eprintln!("[run_connection] Error receiving: {:?}", e);
                    break;
                }
            }
        }
    });

    // Wait for both tasks to complete
    match tokio::try_join!(send_task, receive_task) {
        Ok(_) => println!("[run_connection] Connection closed cleanly."),
        Err(e) => {
            eprintln!("[run_connection] Task error: {:?}", e);
            return Err("WebSocket task crashed".into());
        }
    }

    // Cleanup subscriptions on client disconnect
    let mut subs = subscribers.lock().unwrap();
    for topic in my_topics.lock().unwrap().iter() {
        if let Some(vec) = subs.get_mut(topic) {
            vec.retain(|s| !same_channel(s, &tx_clone));
        }
    }

    println!("[run_connection] Cleanup complete.");
    Ok(())
}

/// Compares two channels to check if they are the same.
fn same_channel(a: &UnboundedSender<String>, b: &UnboundedSender<String>) -> bool {
    std::ptr::eq(a, b)
}