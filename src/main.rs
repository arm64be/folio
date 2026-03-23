use async_tiny::{Server, Response, Header};
use tokio::fs;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct FileMetadata {
    upload_date: DateTime<Utc>,
    original_name: String,
    uploader: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = fs::create_dir_all("uploads").await;
    let _ = fs::create_dir_all("static").await;

    let mut server = Server::http("127.0.0.1:3000", false).await?;

    let home_page_content = fs::read_to_string("static/index.html").await.expect("no index.html");

    while let Some(request) = server.next().await {
        let path_str = request.url().to_string();
        let method = request.method().as_str();

        match (method, path_str.as_str()) {
            ("GET", "/") => {
                let _ = request.respond(Response::from_string(&home_page_content).with_header(html_header()));
            }

            ("GET", "/global_styles.css") => {
                if let Ok(css) = fs::read_to_string("static/global_styles.css").await {
                    let header = Header::new("Content-Type", "text/css").unwrap();
                    let _ = request.respond(Response::from_string(css).with_header(header));
                } else {
                    let _ = request.respond(Response::from_status_and_string(404, "Not Found"));
                }
            }

            ("GET", "/favicon.ico") => {
                if let Ok(data) = fs::read("static/favicon.ico").await {
                    let header = Header::new("Content-Type", "image/x-icon").unwrap();
                    let _ = request.respond(Response::from_data(data).with_header(header));
                } else {
                    let _ = request.respond(Response::from_status_and_string(404, "Not Found"));
                }
            }

            ("GET", p) if p.starts_with("/file/") => {
                let hash = p.trim_start_matches("/file/");
                if !is_safe_input(hash) {
                    let _ = request.respond(Response::from_status_and_string(400, "Bad Request"));
                    continue;
                }
                let meta_path = format!("uploads/{}/metadata.json", hash);
                if !path_is_safe(&meta_path) {
                    let _ = request.respond(Response::from_status_and_string(400, "Bad Request"));
                    continue;
                }

                if let Ok(meta_data) = fs::read_to_string(&meta_path).await {
                    let meta: FileMetadata = match serde_json::from_str(&meta_data) {
                        Ok(m) => m,
                        Err(_) => {
                            let _ = request.respond(Response::from_status_and_string(500, "Corrupt metadata"));
                            continue;
                        }
                    };
                    let name = html_escape(&meta.original_name);
                    let uploader = html_escape(&meta.uploader);
                    let html = format!(
                        "<!DOCTYPE html><html lang=\"en\"><head>\
                            <meta charset=\"UTF-8\">\
                            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
                            <title>{}</title>\
                            <link rel=\"icon\" type=\"image/x-icon\" href=\"/favicon.ico\">\
                            <link rel=\"stylesheet\" href=\"/global_styles.css\">\
                        </head><body>\
                            <div class=\"back-link\"><a href=\"/\">back</a></div>\
                            <div class=\"file-header\">\
                                <h1>{}</h1>\
                                <div class=\"file-meta\"><span>{}</span><span>{}</span></div>\
                            </div>\
                            <div class=\"file-actions\">\
                                <a href=\"/raw/{}\">download raw file</a>\
                            </div>\
                        </body></html>",
                        name, name, uploader, meta.upload_date, hash
                    );
                    let _ = request.respond(Response::from_string(html).with_header(html_header()));
                } else {
                    let _ = request.respond(Response::from_status_and_string(404, "File Not Found"));
                }
            }

            ("GET", p) if p.starts_with("/raw/") => {
                let hash = p.trim_start_matches("/raw/");
                if !is_safe_input(hash) {
                    let _ = request.respond(Response::from_status_and_string(400, "Bad Request"));
                    continue;
                }
                let file_path = format!("uploads/{}/raw", hash);
                if !path_is_safe(&file_path) {
                    let _ = request.respond(Response::from_status_and_string(400, "Bad Request"));
                    continue;
                }
                if let Ok(data) = fs::read(file_path).await {
                    let header = Header::new("Content-Disposition", "attachment").unwrap();
                    let _ = request.respond(Response::from_data(data).with_header(header));
                } else {
                    let _ = request.respond(Response::from_status_and_string(404, "Data missing"));
                }
            }

            _ => {
                let _ = request.respond(Response::from_status_and_string(404, "Route not found"));
            }
        }
    }
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn is_safe_input(input: &str) -> bool {
    if input.is_empty() || input.contains("..") || input.starts_with('/') {
        return false;
    }
    input.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

fn path_is_safe(path: &str) -> bool {
    let Ok(canon) = std::fs::canonicalize(path) else {
        return false;
    };
    let Ok(uploads_root) = std::fs::canonicalize("uploads") else {
        return false;
    };
    canon.starts_with(&uploads_root)
}

fn html_header() -> Header {
    Header::new("Content-Type", "text/html").unwrap()
}
