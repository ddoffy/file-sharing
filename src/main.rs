use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
use actix_web::{
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder
};
use actix_cors::Cors;
use futures_util::TryStreamExt as _;
use std::fs::File;
use std::io::Write;
use serde::Serialize;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

const UPLOAD_DIR: &str = "./uploads";

#[derive(Debug, Serialize)]
struct FileInfo {
    filename: String,
    size: u64,
    created_at: String,
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


async fn list_files() -> impl Responder {
    let paths = std::fs::read_dir(UPLOAD_DIR).unwrap();
    // return a list of files as json format for frontend NextJs
    let mut files = Vec::new();
    for path in paths {
        if let Ok(entry) = path {
            let file_name = entry.file_name().into_string().unwrap();
            let created_at = entry.metadata().unwrap().created();

            let created_at = match created_at {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            };

            let created_at = DateTime::<Utc>::from(created_at);

            files.push(FileInfo {
                filename: file_name,
                size: entry.metadata().unwrap().len(),
                created_at: created_at.to_rfc3339(),
            });
        }
    }

    // sort files by filename, cuz prefix of filename is timestamp
    // descending order
    files.sort_by(|a, b| a.filename.cmp(&b.filename));

    HttpResponse::Ok().json(files)
}

async fn download_file(req: HttpRequest,
    path: web::Path<String>) -> impl Responder {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::default().allow_any_origin().allow_any_header().allow_any_method())
            // home page as static page index html
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/files", web::get().to(list_files))
            .route("/api/download/{filename}", web::get().to(download_file))
            .service(Files::new("/", "./file-sharing-ui/out").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
