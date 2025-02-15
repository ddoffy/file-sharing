use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
use actix_web::{
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder
};
use futures_util::TryStreamExt as _;
use std::fs::File;
use std::io::Write;

const UPLOAD_DIR: &str = "./uploads";

/// Trang HTML hỗ trợ kéo-thả file
async fn index(_req: HttpRequest) -> impl Responder {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Kéo-thả file (Drag & Drop)</title>
    <style>
        body {
            font-family: sans-serif;
            margin: 2rem;
        }
        #drop_zone {
            width: 300px;
            height: 200px;
            border: 3px dashed #999;
            border-radius: 10px;
            text-align: center;
            line-height: 200px;
            color: #999;
            margin-bottom: 20px;
        }
        #drop_zone.hover {
            border-color: #666;
            color: #666;
        }
        #message {
            color: green;
        }
        .file-progress-container {
          margin-top: 1rem;
          border: 1px solid #ccc;
          padding: 5px;
          border-radius: 4px;
          margin-bottom: 1rem;
        }
        .progress-bar {
          width: 0%;
          height: 20px;
          background-color: #4caf50;
          border-radius: 3px;
          transition: width 0.2s;
        }
    </style>
</head>
<body>
    <h1>Chia sẻ file với Kéo-Thả (Drag & Drop)</h1>
    <div id="drop_zone">Kéo và thả file vào đây</div>
    
    <p id="message"></p>

    <!-- Container để hiển thị progress cho từng file -->
    <div id="progressContainer"></div>
    <div id="status"></div>

    <button onclick="window.location.href='/files'">Xem danh sách file</button>
    <script>
        const dropZone = document.getElementById('drop_zone');
        const message = document.getElementById('message');

        // Ngăn trình duyệt mở file khi kéo-thả
        function preventDefaults(e) {
            e.preventDefault();
            e.stopPropagation();
        }

        // Thêm bớt class khi kéo file vào/kéo ra
        function highlight(e) {
            dropZone.classList.add('hover');
        }

        function unhighlight(e) {
            dropZone.classList.remove('hover');
        }

        dropZone.addEventListener('dragenter', preventDefaults, false);
        dropZone.addEventListener('dragover', preventDefaults, false);
        dropZone.addEventListener('dragleave', preventDefaults, false);
        dropZone.addEventListener('drop', preventDefaults, false);

        dropZone.addEventListener('dragenter', highlight, false);
        dropZone.addEventListener('dragover', highlight, false);
        dropZone.addEventListener('dragleave', unhighlight, false);
        dropZone.addEventListener('drop', unhighlight, false);

        dropZone.addEventListener('drop', handleDrop, false);

        // Hàm xử lý khi "thả" file
        function handleDrop(e) {
            let dt = e.dataTransfer;
            let files = dt.files;

            // Xóa progress cũ nếu có
            document.getElementById('progressContainer').innerHTML = '';
            document.getElementById('status').innerText = '';

            //// Tạo form data
            //let formData = new FormData();
            //for (let i = 0; i < files.length; i++) {
            //    formData.append("file", files[i]);
            //}

            //// Gửi request upload
            //fetch("/upload", {
            //    method: "POST",
            //    body: formData
            //})
            //.then(response => response.text())
            //.then(data => {
            //    message.innerText = data; // Hiển thị thông báo
            //})
            //.catch(error => {
            //    message.innerText = "Lỗi khi upload: " + error;
            //});

              // Lặp qua từng file và gửi từng file một
            for (var i = 0; i < files.length; i++) {
              (function(file) {
                // Tạo container hiển thị progress cho file hiện tại
                var container = document.createElement('div');
                container.classList.add('file-progress-container');

                var fileName = document.createElement('div');
                fileName.innerText = file.name;
                container.appendChild(fileName);

                var progressBar = document.createElement('div');
                progressBar.classList.add('progress-bar');
                container.appendChild(progressBar);

                // Thêm container vào div chứa progress
                document.getElementById('progressContainer').appendChild(container);

                // Tạo FormData cho file hiện tại
                var formData = new FormData();
                formData.append("file", file);

                // Tạo XMLHttpRequest mới cho file này
                var xhr = new XMLHttpRequest();
                xhr.open("POST", "/upload", true);

                // Sự kiện progress cho upload
                xhr.upload.onprogress = function(e) {
                  if (e.lengthComputable) {
                    var percentComplete = (e.loaded / e.total) * 100;
                    progressBar.style.width = percentComplete + '%';
                  }
                };

                xhr.onload = function() {
                  if (xhr.status === 200) {
                    fileName.innerText += " - Upload thành công!";
                  } else {
                    fileName.innerText += " - Upload thất bại: " + xhr.status;
                  }
                };

                xhr.onerror = function() {
                  fileName.innerText += " - Lỗi khi upload.";
                };

                xhr.send(formData);
              })(files[i]);
            }
        }
    </script>
</body>
</html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// Hàm upload (xử lý multipart/form-data)
async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Tạo thư mục nếu chưa tồn tại
    std::fs::create_dir_all(UPLOAD_DIR).unwrap();

    // Lặp qua từng field trong multipart
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Lấy tên file từ header (content-disposition)
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                // sanitize để tránh ký tự đặc biệt
                let filepath = format!("{}/{}", UPLOAD_DIR, sanitize_filename::sanitize(filename));
                println!("Saving file to: {}", filepath);

                let mut f = File::create(filepath)?;
                // Ghi dữ liệu chunk
                while let Some(chunk) = field.try_next().await? {
                    f.write_all(&chunk)?;
                }
            }
        }
    }

    Ok(HttpResponse::Ok().body("Tải file thành công!"))
}

/// Liệt kê file đã upload
async fn list_files() -> impl Responder {
    let paths = std::fs::read_dir(UPLOAD_DIR).unwrap();
    let mut html_list = String::new();

    html_list.push_str("<h1>Danh sách file</h1><ul>");
    for path in paths {
        if let Ok(entry) = path {
            let file_name = entry.file_name().into_string().unwrap();
            html_list.push_str(&format!(
                "<li><a href=\"/download/{}\">{}</a></li>",
                file_name, file_name
            ));
        }
    }
    html_list.push_str("</ul>");
    html_list.push_str("<br><a href=\"/\">Quay lại trang Kéo-Thả</a>");

    HttpResponse::Ok().content_type("text/html").body(html_list)
}

/// Tải file xuống
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
    // Chạy server
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_file))
            .route("/files", web::get().to(list_files))
            .route("/download/{filename}", web::get().to(download_file))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
