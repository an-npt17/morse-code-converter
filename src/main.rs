mod message_transformer;
mod morse_converter;
mod serial_send;

use chrono::{DateTime, Utc};
use clokwerk::{Scheduler, TimeUnits};
use message_transformer::{convert_dash_message, convert_dot_message, convert_space_message};
use morse_converter::MorseConverter;
use parking_lot::RwLock;
use rand::prelude::*;
use rand::rng;
use serde::{Deserialize, Serialize};
use serial_send::SerialSender;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use warp::Filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: String,
    text: String,
    morse_code: String,
    created_at: DateTime<Utc>,
    last_sent: Option<DateTime<Utc>>,
    send_count: u32,
}

#[derive(Debug, Deserialize)]
struct CreateMessageRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct UpdateMessageRequest {
    text: String,
}

type MessageStore = Arc<RwLock<HashMap<String, Message>>>;

#[tokio::main]
async fn main() {
    let message_store: MessageStore = Arc::new(RwLock::new(HashMap::new()));
    let morse_converter = Arc::new(MorseConverter {});

    // Start the scheduler in a separate thread
    let store_clone = message_store.clone();
    let converter_clone = morse_converter.clone();
    thread::spawn(move || {
        start_scheduler(store_clone, converter_clone);
    });

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    // Routes
    let messages_store = message_store.clone();
    let morse_clone = morse_converter.clone();

    // Static files route
    let static_files = warp::path("static").and(warp::fs::dir("static"));

    // Root route serves the HTML
    let index = warp::path::end().map(|| warp::reply::html(include_str!("../static/index.html")));

    // API routes
    let api = warp::path("api");

    // GET /api/messages
    let get_messages = api
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_store(messages_store.clone()))
        .and_then(get_all_messages);

    // POST /api/messages
    let create_message = api
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_store(messages_store.clone()))
        .and(with_morse_converter(morse_clone.clone()))
        .and_then(create_new_message);

    // PUT /api/messages/:id
    let update_message = api
        .and(warp::path("messages"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_store(messages_store.clone()))
        .and(with_morse_converter(morse_clone.clone()))
        .and_then(update_existing_message);

    // DELETE /api/messages/:id
    let delete_message = api
        .and(warp::path("messages"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_store(messages_store.clone()))
        .and_then(delete_existing_message);

    let routes = index
        .or(static_files)
        .or(get_messages)
        .or(create_message)
        .or(update_message)
        .or(delete_message)
        .with(cors);

    println!("Morse Code Web API running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_store(
    store: MessageStore,
) -> impl Filter<Extract = (MessageStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

fn with_morse_converter(
    converter: Arc<MorseConverter>,
) -> impl Filter<Extract = (Arc<MorseConverter>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || converter.clone())
}

async fn get_all_messages(store: MessageStore) -> Result<impl warp::Reply, warp::Rejection> {
    let messages: Vec<Message> = store.read().values().cloned().collect();
    Ok(warp::reply::json(&messages))
}

async fn create_new_message(
    req: CreateMessageRequest,
    store: MessageStore,
    morse_converter: Arc<MorseConverter>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id = Uuid::new_v4().to_string();
    let morse_code = morse_converter.morse_converter(&req.text);

    let message = Message {
        id: id.clone(),
        text: req.text,
        morse_code,
        created_at: Utc::now(),
        last_sent: None,
        send_count: 0,
    };

    store.write().insert(id, message.clone());
    Ok(warp::reply::json(&message))
}

async fn update_existing_message(
    id: String,
    req: UpdateMessageRequest,
    store: MessageStore,
    morse_converter: Arc<MorseConverter>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut messages = store.write();

    if let Some(message) = messages.get_mut(&id) {
        message.text = req.text;
        message.morse_code = morse_converter.morse_converter(&message.text);
        Ok(warp::reply::json(message))
    } else {
        Err(warp::reject::not_found())
    }
}

async fn delete_existing_message(
    id: String,
    store: MessageStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut messages = store.write();

    if messages.remove(&id).is_some() {
        Ok(warp::reply::with_status(
            "",
            warp::http::StatusCode::NO_CONTENT,
        ))
    } else {
        Err(warp::reject::not_found())
    }
}

fn start_scheduler(store: MessageStore, morse_converter: Arc<MorseConverter>) {
    let mut scheduler = Scheduler::new();

    scheduler.every(5.minutes()).run(move || {
        send_random_message(&store, &morse_converter);
    });

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}

fn send_random_message(store: &MessageStore, _morse_converter: &Arc<MorseConverter>) {
    let selected_message_id = {
        let messages = store.read();
        if messages.is_empty() {
            println!("No messages in pool to send");
            return;
        }

        // Prefer messages that haven't been sent yet
        let unsent_message_ids: Vec<String> = messages
            .values()
            .filter(|m| m.last_sent.is_none())
            .map(|m| m.id.clone())
            .collect();

        if !unsent_message_ids.is_empty() {
            unsent_message_ids.choose(&mut rng()).unwrap().clone()
        } else {
            // If all messages have been sent, choose randomly from all
            let all_ids: Vec<String> = messages.keys().cloned().collect();
            all_ids.choose(&mut rng()).unwrap().clone()
        }
    }; // Read lock is dropped here

    // Get the message details for sending
    let (message_text, morse_code) = {
        let messages = store.read();
        if let Some(message) = messages.get(&selected_message_id) {
            (message.text.clone(), message.morse_code.clone())
        } else {
            println!("Selected message no longer exists");
            return;
        }
    }; // Read lock is dropped here

    println!("Sending message: {message_text}");
    println!("Morse code: {morse_code}");

    // Send morse code via serial
    send_morse_to_serial(&morse_code);

    // Update the message's last_sent time and send_count
    let mut messages = store.write();
    if let Some(message) = messages.get_mut(&selected_message_id) {
        message.last_sent = Some(Utc::now());
        message.send_count += 1;
    }
}

fn send_morse_to_serial(morse_code: &str) {
    let mut serial_sender = SerialSender::new("/dev/serial0", 9600).unwrap();

    for char in morse_code.chars() {
        match char {
            '.' => {
                let dot_message = convert_dot_message();
                println!("Sending: {dot_message}");
                match serial_sender.send_raw(dot_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent via serial!"),
                    Err(e) => eprintln!("Failed to send via serial: {e}"),
                }
            }
            '-' => {
                let dash_message = convert_dash_message();
                println!("Sending: {dash_message}");
                match serial_sender.send_raw(dash_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent via serial!"),
                    Err(e) => eprintln!("Failed to send via serial: {e}"),
                }
            }
            ' ' => {
                let space_message = convert_space_message();
                println!("Sending: {space_message}");
                match serial_sender.send_raw(space_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent via serial!"),
                    Err(e) => eprintln!("Failed to send via serial: {e}"),
                }
            }
            _ => {}
        }
    }
}
