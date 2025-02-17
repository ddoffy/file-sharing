use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt as _;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::SystemTime;

const UPLOAD_DIR: &str = "./uploads";

#[derive(Debug, Serialize)]
struct FileInfo {
    filename: String,
    size: u64,
    created_at: String,
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
            limit: 100
        }
    }
}

async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    std::fs::create_dir_all(UPLOAD_DIR).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                let filename = format!("{}-{}", Utc::now().timestamp(), filename);
                let filepath = format!("{}/{}", UPLOAD_DIR, sanitize_filename::sanitize(filename));
                println!("Saving file to: {}", filepath);

                let mut f = File::create(filepath)?;
                while let Some(chunk) = field.try_next().await? {
                    f.write_all(&chunk)?;
                }
            }
        }
    }

    Ok(HttpResponse::Ok().body("Upload successful"))
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

//async fn list_files(request: web::Json<Pagination>) -> impl Responder {
//    let paths = std::fs::read_dir(UPLOAD_DIR).unwrap();
//    // return a list of files as json format for frontend NextJs
//    let mut files = Vec::new();
//    for path in paths {
//        if let Ok(entry) = path {
//            let file_name = entry.file_name().into_string().unwrap();
//            let created_at = entry.metadata().unwrap().created();
//
//            let created_at = match created_at {
//                Ok(time) => time,
//                Err(_) => SystemTime::now(),
//            };
//
//            let created_at = DateTime::<Utc>::from(created_at);
//
//            files.push(FileInfo {
//                filename: file_name,
//                size: entry.metadata().unwrap().len(),
//                created_at: created_at.to_rfc3339(),
//            });
//        }
//    }
//
//    // sort files by filename, cuz prefix of filename is timestamp
//    // descending order
//    files.sort_by(|a, b| b.filename.cmp(&a.filename));
//
//    HttpResponse::Ok().json(files.iter().skip(request.page * request.limit).take(request.limit).collect::<Vec<&FileInfo>>())
//}

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

    let output = Command::new("fd")
        .args(args)
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let files: Vec<&str> = stdout.split("\n").collect();
        let mut file_infos = Vec::new();

        for file in files {
            if file.len() > 0 {
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

        HttpResponse::Ok().json(file_infos.iter().skip(query.page * query.limit).take(query.limit).collect::<Vec<&FileInfo>>())
    } else {
        HttpResponse::NotFound().body("File not found.")
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to File Upload API")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .route("/", web::get().to(index))
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/download/{filename}", web::get().to(download_file))
            .route("/api/search", web::post().to(search_files))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
