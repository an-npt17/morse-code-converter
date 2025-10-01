mod message_transformer;
mod morse_converter;
mod serial_send;

use chrono::{DateTime, Utc};
use clokwerk::{Scheduler, TimeUnits};
use message_transformer::{
    TransformerConfig, convert_dash_message, convert_dot_message, convert_space_message,
};
use morse_converter::MorseConverter;
use parking_lot::RwLock;
use rand::prelude::*;
use rand::rng;
use serde::{Deserialize, Serialize};
use serial_send::SerialSender;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;
use warp::Filter;

static CONSECUTIVE_INSTRUMENT_COUNT: AtomicU32 = AtomicU32::new(0);
static CONFIG_CHANGED: AtomicBool = AtomicBool::new(false);
static LAMP_MODE_START_TIME: AtomicU64 = AtomicU64::new(0);

fn send_lamp() -> bool {
    let current_count = CONSECUTIVE_INSTRUMENT_COUNT.load(Ordering::SeqCst);

    if current_count >= 5 {
        let start_time = LAMP_MODE_START_TIME.load(Ordering::SeqCst);

        // If this is the first time entering lamp mode, record the start time
        if start_time == 0 {
            let now = Instant::now().elapsed().as_secs();
            LAMP_MODE_START_TIME.store(now, Ordering::SeqCst);
            return true;
        }

        // Check if 2 minutes (120 seconds) have passed
        let now = Instant::now().elapsed().as_secs();
        let elapsed = now.saturating_sub(start_time);

        if elapsed >= 120 {
            println!("Lamp mode timeout (2 minutes) - exiting lamp mode");
            LAMP_MODE_START_TIME.store(0, Ordering::SeqCst);
            return false;
        }

        return true;
    }

    false
}

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

type TempoStore = Arc<RwLock<u64>>;
type MessageStore = Arc<RwLock<HashMap<String, Message>>>;
type ConfigStore = Arc<RwLock<TransformerConfig>>;

const MESSAGES_FILE_PATH: &str = "messages.json";
const CONFIG_FILE_PATH: &str = "transformer_config.json";

fn generate_random_tempo(tempo_choices: &[u64]) -> u64 {
    if tempo_choices.is_empty() {
        return 700;
    }
    let mut rng = rand::rng();
    *tempo_choices.choose(&mut rng).unwrap()
}

fn load_messages_from_file(file_path: &str) -> HashMap<String, Message> {
    if !Path::new(file_path).exists() {
        println!(
            "Messages file {} not found, starting with empty message store",
            file_path
        );
        return HashMap::new();
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            if content.trim().is_empty() {
                println!(
                    "Messages file {} is empty, starting with empty message store",
                    file_path
                );
                return HashMap::new();
            }

            match serde_json::from_str::<HashMap<String, Message>>(&content) {
                Ok(messages) => {
                    println!("Loaded {} messages from {}", messages.len(), file_path);
                    messages
                }
                Err(e) => {
                    eprintln!("Failed to parse messages from {}: {}", file_path, e);
                    println!("Starting with empty message store");
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read messages file {}: {}", file_path, e);
            println!("Starting with empty message store");
            HashMap::new()
        }
    }
}

fn load_config_from_file(file_path: &str) -> TransformerConfig {
    if !Path::new(file_path).exists() {
        println!("Config file {} not found, using default config", file_path);
        return TransformerConfig::default();
    }

    match fs::read_to_string(file_path) {
        Ok(content) => match serde_json::from_str::<TransformerConfig>(&content) {
            Ok(config) => {
                println!("Loaded config from {}", file_path);
                config
            }
            Err(e) => {
                eprintln!("Failed to parse config from {}: {}", file_path, e);
                println!("Using default config");
                TransformerConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file {}: {}", file_path, e);
            println!("Using default config");
            TransformerConfig::default()
        }
    }
}

fn save_messages_to_file(messages: &HashMap<String, Message>, file_path: &str) {
    match serde_json::to_string_pretty(messages) {
        Ok(json_content) => match fs::write(file_path, json_content) {
            Ok(_) => println!("Messages saved to {}", file_path),
            Err(e) => eprintln!("Failed to write messages to {}: {}", file_path, e),
        },
        Err(e) => eprintln!("Failed to serialize messages: {}", e),
    }
}

fn save_config_to_file(config: &TransformerConfig, file_path: &str) {
    match serde_json::to_string_pretty(config) {
        Ok(json_content) => match fs::write(file_path, json_content) {
            Ok(_) => println!("Config saved to {}", file_path),
            Err(e) => eprintln!("Failed to write config to {}: {}", file_path, e),
        },
        Err(e) => eprintln!("Failed to serialize config: {}", e),
    }
}

fn start_auto_save_scheduler(message_store: MessageStore, config_store: ConfigStore) {
    thread::spawn(move || {
        let mut scheduler = Scheduler::new();

        scheduler.every(30.seconds()).run(move || {
            let messages = message_store.read();
            save_messages_to_file(&messages, MESSAGES_FILE_PATH);

            let config = config_store.read();
            save_config_to_file(&config, CONFIG_FILE_PATH);
        });

        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(5000));
        }
    });
}

#[tokio::main]
async fn main() {
    let initial_messages = load_messages_from_file(MESSAGES_FILE_PATH);
    let message_store: MessageStore = Arc::new(RwLock::new(initial_messages));

    let initial_config = load_config_from_file(CONFIG_FILE_PATH);
    let config_store: ConfigStore = Arc::new(RwLock::new(initial_config.clone()));

    let morse_converter = Arc::new(MorseConverter {});
    let initial_tempo = generate_random_tempo(&initial_config.tempo_choices);
    let tempo_store: TempoStore = Arc::new(RwLock::new(initial_tempo));

    println!("Initial tempo: {} ms", initial_tempo);

    start_auto_save_scheduler(message_store.clone(), config_store.clone());

    let store_clone = message_store.clone();
    let converter_clone = morse_converter.clone();
    let tempo_clone = tempo_store.clone();
    let config_clone = config_store.clone();
    thread::spawn(move || {
        start_message_scheduler(store_clone, converter_clone, tempo_clone, config_clone);
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

    let save_messages = api
        .and(warp::path("messages"))
        .and(warp::path("save"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_store(messages_store.clone()))
        .and_then(save_messages_manually);

    let get_config = api
        .and(warp::path("config"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_config_store(config_store.clone()))
        .and_then(get_transformer_config);

    let update_config = api
        .and(warp::path("config"))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_config_store(config_store.clone()))
        .and_then(update_transformer_config);

    let routes = index
        .or(static_files)
        .or(get_messages)
        .or(create_message)
        .or(update_message)
        .or(delete_message)
        .or(get_tempo)
        .or(save_messages)
        .or(get_config)
        .or(update_config)
        .with(cors);

    let shutdown_store = message_store.clone();
    let shutdown_config = config_store.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, saving messages and config before shutdown...");
        let messages = shutdown_store.read();
        save_messages_to_file(&messages, MESSAGES_FILE_PATH);
        let config = shutdown_config.read();
        save_config_to_file(&config, CONFIG_FILE_PATH);
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

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

fn with_config_store(
    config: ConfigStore,
) -> impl Filter<Extract = (ConfigStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
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

async fn save_messages_manually(store: MessageStore) -> Result<impl warp::Reply, warp::Rejection> {
    let messages = store.read();
    save_messages_to_file(&messages, MESSAGES_FILE_PATH);

    let response = serde_json::json!({
        "success": true,
        "message": "Messages saved successfully",
        "count": messages.len()
    });
    Ok(warp::reply::json(&response))
}

async fn get_transformer_config(
    config_store: ConfigStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let config = config_store.read().clone();
    Ok(warp::reply::json(&config))
}

async fn update_transformer_config(
    new_config: TransformerConfig,
    config_store: ConfigStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    *config_store.write() = new_config.clone();
    save_config_to_file(&new_config, CONFIG_FILE_PATH);

    CONFIG_CHANGED.store(true, Ordering::SeqCst);
    println!("Config updated - current message sending will be interrupted");

    Ok(warp::reply::json(&new_config))
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
        message.text = req.text.clone();
        let normalized_text = req.text.replace('\n', " ").replace('\r', "");
        message.morse_code = morse_converter.morse_converter(&normalized_text);
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
    config_store: ConfigStore,
) {
    loop {
        let is_lamp_mode = send_lamp();

        if is_lamp_mode {
            let config = config_store.read();
            *tempo_store.write() = config.lamp_tempo_ms;
            println!(
                "Lamp mode active - using lamp tempo: {} ms",
                config.lamp_tempo_ms
            );
        }

        send_random_message(&store, &morse_converter, &tempo_store, &config_store);

        let config = config_store.read();
        let new_tempo = generate_random_tempo(&config.tempo_choices);
        drop(config);

        *tempo_store.write() = new_tempo;
        println!("New tempo: {} ms", new_tempo);

        if !is_lamp_mode {
            CONSECUTIVE_INSTRUMENT_COUNT.fetch_add(1, Ordering::SeqCst);
        } else {
            CONSECUTIVE_INSTRUMENT_COUNT.store(0, Ordering::SeqCst);
        }

        println!("Waiting 5 seconds before next message...");
        thread::sleep(Duration::from_secs(5));
    }
}

fn send_random_message(
    store: &MessageStore,
    morse_converter: &Arc<MorseConverter>,
    tempo_store: &TempoStore,
    config_store: &ConfigStore,
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

    send_morse_to_serial(&morse_code, current_tempo, config_store);

    let mut messages = store.write();
    if let Some(message) = messages.get_mut(&selected_message_id) {
        message.last_sent = Some(Utc::now());
        message.send_count += 1;
    }
}

fn send_morse_to_serial(morse_code: &str, tempo_ms: u64, config_store: &ConfigStore) {
    let mut serial_sender = SerialSender::new("/dev/serial0", 9600).unwrap();

    for char in morse_code.chars() {
        if CONFIG_CHANGED.load(Ordering::SeqCst) {
            println!("Config changed - interrupting current message");
            CONFIG_CHANGED.store(false, Ordering::SeqCst);
            return;
        }

        let config = config_store.read().clone();

        match char {
            '.' => {
                let dot_message = convert_dot_message(&config);
                println!("Sending: {dot_message}");
                match serial_sender.send_raw(dot_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent dot via serial!"),
                    Err(e) => eprintln!("Failed to send dot via serial: {e}"),
                }
                thread::sleep(Duration::from_millis(tempo_ms));
            }
            '-' => {
                let dash_message = convert_dash_message(&config);
                println!("Sending: {dash_message}");
                match serial_sender.send_raw(dash_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent dash via serial!"),
                    Err(e) => eprintln!("Failed to send dash via serial: {e}"),
                }
                thread::sleep(Duration::from_millis(tempo_ms * 4));
            }
            ' ' => {
                let space_message = convert_space_message(&config);
                println!("Sending: {space_message}");
                match serial_sender.send_raw(space_message.as_bytes()) {
                    Ok(_) => println!("Successfully sent space via serial!"),
                    Err(e) => eprintln!("Failed to send space via serial: {e}"),
                }
                thread::sleep(Duration::from_millis(tempo_ms * 4));
            }
            '\n' => {
                println!("New line character detected, sleep for 4*tempo");
                thread::sleep(Duration::from_millis(tempo_ms * 4));
            }
            _ => {
                continue;
            }
        }
    }
}
