use actix_web::{web, Responder};
use base64::engine::general_purpose;
use base64::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct UploadRequest {
    filename: String,
    chunk_id: usize,
    total_chunks: usize,
    data: String, // base64 encoded
}

pub type FilePartsMap = Arc<Mutex<HashMap<String, Vec<bool>>>>;

const UPLOAD_DIR: &str = "./uploads";

pub async fn upload_file(
    file_parts: web::Data<FilePartsMap>,
    payload: web::Json<UploadRequest>,
) -> impl Responder {
    let filename = &payload.filename.clone();
    let chunk_id = payload.chunk_id;
    let total_chunks = payload.total_chunks;
    let base64_data = &payload.data;

    // decode base64 data to bytes
    let data: Vec<u8> = match general_purpose::STANDARD.decode(base64_data.as_bytes()) {
        Ok(data) => data,
        Err(_) => return "Error decoding base64 data".to_string(),
    };
    // add timestamp to filename to prevent overwriting
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/{}", UPLOAD_DIR, filename))
        .await
        .unwrap();

    file.write_all(&data).await.unwrap();

    let mut map = file_parts.lock().await;
    let entry = map
        .entry(filename.clone())
        .or_insert(vec![false; total_chunks]);
    entry[chunk_id] = true;

    if entry.iter().all(|&received| received) {
        println!("All chunks received for {}", filename);
    }

    format!(
        "Chunk {}/{} received for {}",
        chunk_id, total_chunks, filename
    )
}
