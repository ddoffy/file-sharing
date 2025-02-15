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
use nix::sys::stat::stat;
use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
                let filepath = format!("{}/{}", UPLOAD_DIR, sanitize_filename::sanitize(filename));
                // add timestamp to filename to avoid overwrite
                let filepath = format!("{}-{}", filepath, Utc::now().timestamp());
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


#[cfg(target_os = "linux")]
fn is_linux() -> bool {
    true
}

#[cfg(not(target_os = "linux"))]
fn is_linux() -> bool {
    false
}

#[cfg(target_arch = "aarch64")]
fn is_jetson() -> bool {
    true
}

#[cfg(not(target_arch = "aarch64"))]
fn is_jetson() -> bool {
    false
}

fn get_created_at(p: &str) -> SystemTime {
    let path = Path::new(p);
    let c_path = CStr::from_bytes_with_nul(path.as_os_str().as_bytes()).unwrap();

    let created_at = match stat(c_path) {
        Ok(metadata) => {
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metadata.st_mtime as u64)
        },
        Err(err) => {
            println!("Error: {:?}", err);
            SystemTime::now()
        }
    };

    created_at
}

async fn list_files() -> impl Responder {
    let paths = std::fs::read_dir(UPLOAD_DIR).unwrap();
    // return a list of files as json format for frontend NextJs
    let mut files = Vec::new();
    for path in paths {
        if let Ok(entry) = path {
            let file_name = entry.file_name().into_string().unwrap();
            let created_at = entry.metadata().unwrap().created();
            let mut final_created_at: SystemTime;
            if is_linux() && is_jetson() {
                // Jetson Nano has a bug in created time
                // so we use modified time instead
                let path_file = format!("{}/{}", UPLOAD_DIR, file_name);
                println!("Path: {}", path_file);
                final_created_at = get_created_at(&path_file);
            }            
            else {
                final_created_at = created_at.unwrap();
            }

            let created_at = DateTime::<Utc>::from(final_created_at);

            files.push(FileInfo {
                filename: file_name,
                size: entry.metadata().unwrap().len(),
                created_at: created_at.to_rfc3339(),
            });
        }
    }

    // sort files by created_at

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
