mod message_transformer;
mod morse_converter;
mod serial_send;

use chrono::{DateTime, Utc};
use clokwerk::{Scheduler, TimeUnits};
use futures_util::TryStreamExt;
use message_transformer::{convert_dash_message, convert_dot_message, convert_space_message};
use morse_converter::MorseConverter;
use parking_lot::RwLock;
use rand::prelude::*;
use rand::rng;
use serde::{Deserialize, Serialize};
use serial_send::SerialSender;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use warp::Buf;
use warp::Filter;
use warp::multipart::FormData;

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

#[derive(Debug, Serialize)]
struct BulkUploadResponse {
    success: bool,
    messages_added: usize,
    messages: Vec<Message>,
}

type TempoStore = Arc<RwLock<u64>>;

type MessageStore = Arc<RwLock<HashMap<String, Message>>>;

fn generate_random_tempo() -> u64 {
    rng().random_range(100..=1000)
}

#[tokio::main]
async fn main() {
    let message_store: MessageStore = Arc::new(RwLock::new(HashMap::new()));
    let morse_converter = Arc::new(MorseConverter {});
    let tempo_store: TempoStore = Arc::new(RwLock::new(generate_random_tempo()));

    println!("Initial tempo: {} ms", *tempo_store.read());

    let store_clone = message_store.clone();
    let converter_clone = morse_converter.clone();
    let tempo_clone = tempo_store.clone();
    thread::spawn(move || {
        start_message_scheduler(store_clone, converter_clone, tempo_clone);
    });

    let tempo_scheduler_clone = tempo_store.clone();
    thread::spawn(move || {
        start_tempo_scheduler(tempo_scheduler_clone);
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    let messages_store = message_store.clone();
    let morse_clone = morse_converter.clone();

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let index = warp::path::end().map(|| warp::reply::html(include_str!("../static/index.html")));

    let api = warp::path("api");

    let get_messages = api
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_store(messages_store.clone()))
        .and_then(get_all_messages);

    let create_message = api
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_store(messages_store.clone()))
        .and(with_morse_converter(morse_clone.clone()))
        .and_then(create_new_message);

    let upload_messages = api
        .and(warp::path("messages"))
        .and(warp::path("upload"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::multipart::form().max_length(1024 * 1024)) // 1MB max
        .and(with_store(messages_store.clone()))
        .and(with_morse_converter(morse_clone.clone()))
        .and_then(upload_messages_from_file);

    let update_message = api
        .and(warp::path("messages"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_store(messages_store.clone()))
        .and(with_morse_converter(morse_clone.clone()))
        .and_then(update_existing_message);

    let delete_message = api
        .and(warp::path("messages"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_store(messages_store.clone()))
        .and_then(delete_existing_message);

    let get_tempo = api
        .and(warp::path("tempo"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_tempo_store(tempo_store.clone()))
        .and_then(get_current_tempo);

    let routes = index
        .or(static_files)
        .or(get_messages)
        .or(create_message)
        .or(upload_messages)
        .or(update_message)
        .or(delete_message)
        .or(get_tempo)
        .with(cors);

    println!("Morse Code Web API running on http://localhost:3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
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

fn with_tempo_store(
    tempo: TempoStore,
) -> impl Filter<Extract = (TempoStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tempo.clone())
}

async fn get_all_messages(store: MessageStore) -> Result<impl warp::Reply, warp::Rejection> {
    let messages: Vec<Message> = store.read().values().cloned().collect();
    Ok(warp::reply::json(&messages))
}

async fn get_current_tempo(tempo_store: TempoStore) -> Result<impl warp::Reply, warp::Rejection> {
    let tempo = *tempo_store.read();
    let response = serde_json::json!({ "tempo_ms": tempo });
    Ok(warp::reply::json(&response))
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

async fn upload_messages_from_file(
    mut form: FormData,
    store: MessageStore,
    morse_converter: Arc<MorseConverter>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut file_content = String::new();
    let mut file_found = false;

    while let Ok(Some(part)) = form.try_next().await {
        if part.name() == "file" {
            file_found = true;
            let mut bytes = Vec::new();

            let mut stream = part.stream();
            while let Ok(Some(chunk)) = stream.try_next().await {
                chunk.reader().read_to_end(&mut bytes).unwrap();
            }

            file_content = String::from_utf8(bytes).map_err(|_| warp::reject::reject())?;
            break;
        }
    }

    if !file_found {
        return Err(warp::reject::reject());
    }

    let mut messages_added = Vec::new();
    let lines: Vec<&str> = file_content.lines().collect();

    for line in lines {
        let trimmed_line = line.trim();
        if !trimmed_line.is_empty() {
            let id = Uuid::new_v4().to_string();
            let morse_code = morse_converter.morse_converter(trimmed_line);

            let message = Message {
                id: id.clone(),
                text: trimmed_line.to_string(),
                morse_code,
                created_at: Utc::now(),
                last_sent: None,
                send_count: 0,
            };

            store.write().insert(id, message.clone());
            messages_added.push(message);
        }
    }

    let response = BulkUploadResponse {
        success: true,
        messages_added: messages_added.len(),
        messages: messages_added,
    };

    Ok(warp::reply::json(&response))
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

fn start_message_scheduler(
    store: MessageStore,
    morse_converter: Arc<MorseConverter>,
    tempo_store: TempoStore,
) {
    let mut scheduler = Scheduler::new();

    scheduler.every(20.seconds()).run(move || {
        send_random_message(&store, &morse_converter, &tempo_store);
    });

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}

fn start_tempo_scheduler(tempo_store: TempoStore) {
    let mut scheduler = Scheduler::new();

    scheduler.every(3.minutes()).run(move || {
        let new_tempo = generate_random_tempo();
        *tempo_store.write() = new_tempo;
        println!("Tempo changed to: {new_tempo} ms");
    });

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(5000)); // Check every 5 seconds
    }
}

fn send_random_message(
    store: &MessageStore,
    _morse_converter: &Arc<MorseConverter>,
    tempo_store: &TempoStore,
) {
    let selected_message_id = {
        let messages = store.read();
        if messages.is_empty() {
            println!("No messages in pool to send");
            return;
        }

        let unsent_message_ids: Vec<String> = messages
            .values()
            .filter(|m| m.last_sent.is_none())
            .map(|m| m.id.clone())
            .collect();

        if !unsent_message_ids.is_empty() {
            unsent_message_ids.choose(&mut rng()).unwrap().clone()
        } else {
            let all_ids: Vec<String> = messages.keys().cloned().collect();
            all_ids.choose(&mut rng()).unwrap().clone()
        }
    };

    let (message_text, morse_code) = {
        let messages = store.read();
        if let Some(message) = messages.get(&selected_message_id) {
            (message.text.clone(), message.morse_code.clone())
        } else {
            println!("Selected message no longer exists");
            return;
        }
    };

    let current_tempo = *tempo_store.read();

    println!("Sending message: {message_text}");
    println!("Morse code: {morse_code}");
    println!("Current tempo: {current_tempo} ms");

    send_morse_to_serial(&morse_code, current_tempo);

    let mut messages = store.write();
    if let Some(message) = messages.get_mut(&selected_message_id) {
        message.last_sent = Some(Utc::now());
        message.send_count += 1;
    }

    let post_message_delay = rng().random_range(10000..=20000);
    println!("Sleeping {post_message_delay} ms after message completion");
    thread::sleep(Duration::from_millis(post_message_delay));
}

fn send_morse_to_serial(morse_code: &str, tempo_ms: u64) {
    let mut serial_sender = SerialSender::new("/dev/serial0", 9600).unwrap();

    for char in morse_code.chars() {
        match char {
            '.' => {
                let dot_message = convert_dot_message();
                println!("Sending: {dot_message}");
                match serial_sender.send_raw(dot_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent dot via serial!"),
                    Err(e) => eprintln!("Failed to send dot via serial: {e}"),
                }
            }
            '-' => {
                let dash_message = convert_dash_message();
                println!("Sending: {dash_message}");
                match serial_sender.send_raw(dash_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent dash via serial!"),
                    Err(e) => eprintln!("Failed to send dash via serial: {e}"),
                }
            }
            ' ' => {
                let space_message = convert_space_message();
                println!("Sending: {space_message}");
                match serial_sender.send_raw(space_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent space via serial!"),
                    Err(e) => eprintln!("Failed to send space via serial: {e}"),
                }
            }
            _ => {
                continue;
            }
        }

        println!("Waiting {tempo_ms} ms (tempo)");
        thread::sleep(Duration::from_millis(tempo_ms));
    }
}
