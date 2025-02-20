use crate::config::init_config;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::SystemTime;
use tokio::fs;
use tokio::sync::broadcast;

pub mod api;
pub mod config;
pub mod db;
pub mod ws;

const UPLOAD_DIR: &str = "./uploads";

#[derive(Debug, Serialize, Deserialize, Default)]
struct FileInfo {
    filename: String,
    size: u64,
    created_at: String,
}

impl Clone for FileInfo {
    fn clone(&self) -> Self {
        FileInfo {
            filename: self.filename.clone(),
            size: self.size,
            created_at: self.created_at.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SearchQuery {
    filename: String,
    extensions: Option<Vec<String>>,
    page: usize,
    limit: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        SearchQuery {
            filename: "".to_string(),
            extensions: None,
            page: 0,
            limit: 100,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SearchResult {
    files: Vec<FileInfo>,
    total: usize,
    page: usize,
    limit: usize,
}

async fn upload_file(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    std::fs::create_dir_all(UPLOAD_DIR).unwrap();

    let mut final_filename = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                let filename = format!("{}-{}", Utc::now().timestamp(), filename);
                final_filename = filename.clone();
                let filepath = format!("{}/{}", UPLOAD_DIR, sanitize_filename::sanitize(filename));
                println!("Saving file to: {}", filepath);

                let mut f = File::create(filepath)?;
                while let Some(chunk) = field.try_next().await? {
                    f.write_all(&chunk)?;
                }
            }
        }
    }

    let schema = req.connection_info().scheme().to_string();
    let host = req.connection_info().host().to_string();

    Ok(HttpResponse::Ok().body(format!("{}://{}/api/download/{}", schema, host, final_filename)))
}

// upload file to the server with binary data
async fn upload_file_binary(req: HttpRequest, body: web::Bytes) -> impl Responder {
    // Extract filename from "X-Filename" header
    let filename = req
        .headers()
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(format!("{}.unknown.bin", Utc::now().timestamp()).as_str())
        .to_string();

    // log out all headers
    println!("Headers: {:?}", req.headers());

    let filename = format!("{}-{}", Utc::now().timestamp(), filename);
    let filepath = format!("{}/{}", UPLOAD_DIR, sanitize_filename::sanitize(filename.clone()));
    println!("Saving file to: {}", filepath);

    // Create directory if not exists
    fs::create_dir_all(UPLOAD_DIR).await.unwrap();

    let mut f = File::create(filepath).unwrap();
    f.write_all(&body).unwrap();

    // return url to download the file
    let host = req.connection_info().host().to_string();
    HttpResponse::Ok().body(format!("{}/api/download/{}",host, filename))
}

fn get_created_at(file: &str) -> String {
    let created_at = std::fs::metadata(file).unwrap().created();

    let created_at = match created_at {
        Ok(time) => time,
        Err(_) => SystemTime::now(),
    };

    let created_at = DateTime::<Utc>::from(created_at);

    created_at.to_rfc3339()
}

async fn download_file(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let filepath = format!("{}/{}", UPLOAD_DIR, path.into_inner());
    if std::path::Path::new(&filepath).exists() {
        match NamedFile::open_async(filepath).await {
            Ok(named_file) => named_file.into_response(&req),
            Err(_) => HttpResponse::NotFound().body("File not found."),
        }
    } else {
        HttpResponse::NotFound().body("File not found.")
    }
}

async fn search_files(query: web::Json<SearchQuery>) -> impl Responder {
    // use find-fd to search files
    let mut args: Vec<String> = Vec::new();

    // build arguments for fd command
    if query.extensions.is_some() {
        for ext in query.extensions.iter().flatten() {
            args.push("-e".to_string());
            args.push(ext.to_string());
        }
    }

    args.push(query.filename.clone());
    args.push(UPLOAD_DIR.to_string());

    let output = Command::new("fd").args(args).output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let files: Vec<&str> = stdout.split("\n").collect();
        let mut file_infos = Vec::new();

        for file in files {
            if !file.is_empty() {
                let created_at = get_created_at(file);
                file_infos.push(FileInfo {
                    filename: file.replace(UPLOAD_DIR, "").replace("/", "").to_string(),
                    size: std::fs::metadata(file).unwrap().len(),
                    created_at,
                });
            }
        }

        // sort files by filename, cuz prefix of filename is timestamp
        // descending order
        file_infos.sort_by(|a, b| b.filename.cmp(&a.filename));

        let total = file_infos.len();
        let page = query.page;
        let limit = query.limit;


        HttpResponse::Ok().json(
            SearchResult {
                files: file_infos
                .iter()
                .skip(query.page * query.limit)
                .take(query.limit)
                .cloned()
                .collect(),
                total,
                page,
                limit,
            }
        )
    } else {
        HttpResponse::NotFound().body("File not found.")
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to File Upload API")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting server at http://localhost:8080");

    // set up services
    init_config();

    let (tx, _) = broadcast::channel::<web::Bytes>(128);

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .route("/", web::get().to(index))
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/upload-binary", web::post().to(upload_file_binary))
            .route("/api/download/{filename}", web::get().to(download_file))
            .route("/api/search", web::post().to(search_files))
            .route(
                "/api/v1/clipboard/{room_id}/store",
                web::post().to(api::v1::clipboard::store_clipboard),
            )
            .route(
                "/api/v1/clipboard/keys",
                web::get().to(api::v1::clipboard::get_clipboard_keys),
            )
            .route(
                "/api/v1/clipboard/{room_id}/get",
                web::get().to(api::v1::clipboard::get_clipboard),
            )
            .route("/ws", web::get().to(ws::ws_handler))
            .app_data(web::Data::new(tx.clone()))
            .service(
                web::resource("/ws-broadcast")
                    .route(web::get().to(ws::handshake_and_start_broadcast_ws)),
            )
            .service(web::resource("/send").route(web::post().to(ws::send_to_broadcast_ws)))
            // standard middleware
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
