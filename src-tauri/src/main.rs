// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::{process::{Command, exit}};
use tauri::{api::process::{Command as Command2}, Manager};
use fix_path_env::fix;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json;


// MACOS ONLY!
#[tauri::command]
fn open_file_macos(dir: &str) -> (){
    let mut child = Command::new("open")
        .arg("-R")
        .arg(dir)
        .spawn()
        .expect("Failed to start new process");
}

#[derive(Debug, Serialize, Deserialize)]
struct Caption {
    embeddings: Vec<f32>,
}


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn search(query: &str) -> Vec<String> {
    // Set-up sentence embeddings model
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()
        .expect("Failed to create embeddings model");

    let client = redis::Client::open("redis://127.0.0.1:1208/").expect("Failed to connect to Redis");
    let mut con = client.get_connection().expect("Failed to establish connection with Redis");

    let client_emb = redis::Client::open("redis://127.0.0.1:1209/").expect("Failed to connect to Redis");
    let mut con_emb = client_emb.get_connection().expect("Failed to establish connection with Redis");

    // Check if embeddings are missing for any keys in port 1208
    let keys: Vec<String> = con.keys("*").expect("Failed to get all keys from Redis");
    let keys_emb: Vec<String> = con_emb.keys("*").expect("Failed to get all keys from Redis");

    if keys.len() - 1 > keys_emb.len() {
        // Compute missing embeddings
        let mut missing_sentences: Vec<String> = Vec::new();
        for key in keys.iter() {
            if !keys_emb.contains(key) && key != "last_updated" {
                missing_sentences.push(key.to_owned());
            }
        }

        let embeddings = model.encode(&missing_sentences).expect("Failed to generate embeddings");

        for (key, embedding) in missing_sentences.iter().zip(embeddings.iter()) {
            let data = Caption {
                embeddings: embedding.to_vec(),
            };
            let json_data = serde_json::to_string(&data).expect("Failed to serialize data");
            con_emb.set::<String, String, ()>(key.to_string(), json_data).expect("Failed to set data in Redis");
        }
    }

    else if keys.len() - 1 < keys_emb.len() {
        // Remove extra embeddings
        for key in keys_emb.iter() {
            if !keys.contains(key) {
                con_emb.del::<String, ()>(key.to_string()).expect("Failed to delete key from Redis");
            }
        }
    }

    // Compute embeddings for the query
    let embedding_query = model.encode(&[query]).expect("Failed to generate embeddings");
    let emb_query = &embedding_query[0];

    let sentences: Vec<String> = con_emb.keys("*").expect("Failed to get all keys from Redis");
    
    // Retrieve sentences and embeddings from Redis
    let mut embeddings: Vec<Vec<f32>> = Vec::new();
    for sentence in sentences.iter() {
        let json_data: Option<String> = con_emb.get(sentence).expect("Failed to get keys from Redis");
        if let Some(json) = json_data {
            let data: Caption = serde_json::from_str(&json).expect("Failed to deserialize data");
            let field: Vec<f32> = data.embeddings;
            embeddings.push(field);
        }
    }

    // Compute cosine distances
    let mut distances: Vec<(f32, &str)> = Vec::new();
    for (embedding, sentence) in embeddings.iter().zip(sentences.iter()) {
        let distance = cosine_distance(emb_query, embedding);
        distances.push((distance, sentence.as_str()));
    }

    // Sort by cosine distances in descending order
    distances.sort_by(|(distance1, _), (distance2, _)| distance2.partial_cmp(distance1).unwrap());

    // Take the top 100 sentences based on distance in ascending order
    let top_sentences: Vec<String> = distances.iter()
        .take(100)
        .map(|(_, sentence)| con.get::<&str, String>(&sentence).expect("Failed to get keys from Redis").to_string())
        .collect();

    top_sentences
}


fn main() {
    fix_path_env::fix();

    let (rx_redis, child_redis) = Command2::new_sidecar("redis-server")
        .expect("failed to create redis-server binary command")
        .args(&["--port", "1208"])
        .spawn()
        .expect("Failed to spawn redis-server");

    let (rx_redis_emb, child_redis_emb) = Command2::new_sidecar("redis-server")
        .expect("failed to create redis-server binary command")
        .args(&["--port", "1209"])
        .spawn()
        .expect("Failed to spawn redis-server");


    let output = Command::new("pgrep")
    .arg("miniserve")
    .output()
    .expect("Failed to execute command");

    if output.stdout.is_empty() {
        // No existing process found, start a new one
        let child = Command::new("miniserve")
            .arg("--route-prefix=6a4e786120cb00c1a0f85dc5528f75debff6eec8")
            .arg("/")
            .spawn()
            .expect("Failed to start new process");
    
        println!("New process started.");
    } else {
        // Existing process found, kill it and start a new one
        if let Err(err) = Command::new("pkill")
            .arg("miniserve")
            .status()
        {
            eprintln!("Failed to kill existing process: {}", err);
            exit(1);
        }
    
        let child = Command::new("miniserve")
            .arg("--route-prefix=6a4e786120cb00c1a0f85dc5528f75debff6eec8")
            .arg("/")
            .spawn()
            .expect("Failed to start new process");
    
        println!("Existing process killed. New process started.");
    }

    tauri::Builder::default()
        .on_window_event(|event| match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
          event.window().app_handle().hide().unwrap();
          api.prevent_close();
        }
        _ => {}
         })
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![search, open_file_macos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


fn dot_product(vec1: &[f32], vec2: &[f32]) -> f32 {
    let delta = vec1.len() - vec2.len();
    let shortest_vec = match delta {
        d if d < 0 => vec1,
        d if d > 0 => vec2,
        _ => vec1,
    };
    let mut dot_product = 0.0;
    for i in 0..shortest_vec.len() {
        dot_product += vec1[i] * vec2[i];
    }
    dot_product
}

fn root_sum_square(vec: &[f32]) -> f32 {
    let mut sum_square = 0.0;
    for i in 0..vec.len() {
        sum_square += vec[i] * vec[i];
    }
    sum_square.sqrt()
}


  fn cosine_distance(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product = dot_product(vec1, vec2);
    let root_sum_square1 = root_sum_square(vec1);
    let root_sum_square2 = root_sum_square(vec2);
    dot_product / (root_sum_square1 * root_sum_square2)
}