use laborlens_local_server::{ApiResponse, LocalServer};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let (listener, address) = bind_listener()?;
    let ui_root = ui_root();
    let server = LocalServer::default();

    println!("LaborLens local UI: http://{address}/");
    println!("UI root: {}", ui_root.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_client(stream, &server, &ui_root) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }

    Ok(())
}

fn bind_listener() -> io::Result<(TcpListener, String)> {
    if let Ok(address) = env::var("LABORLENS_LOCAL_ADDR") {
        let listener = TcpListener::bind(&address)?;
        return Ok((listener, address));
    }

    for port in 5174..5181 {
        let address = format!("127.0.0.1:{port}");
        if let Ok(listener) = TcpListener::bind(&address) {
            return Ok((listener, address));
        }
    }

    TcpListener::bind("127.0.0.1:0").and_then(|listener| {
        let address = listener.local_addr()?.to_string();
        Ok((listener, address))
    })
}

fn ui_root() -> PathBuf {
    env::var("LABORLENS_UI_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("laborlens-local-ui")
        })
}

fn handle_client(mut stream: TcpStream, server: &LocalServer, ui_root: &Path) -> io::Result<()> {
    let mut buffer = [0_u8; 16 * 1024];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let Some(first_line) = request.lines().next() else {
        return Ok(());
    };
    let mut request_parts = first_line.split_whitespace();
    let method = request_parts.next().unwrap_or("");
    let path = request_parts.next().unwrap_or("/");
    let path = path.split('?').next().unwrap_or(path);

    let response = response_for_request(method, path, server, ui_root);
    write_response(&mut stream, response)
}

fn response_for_request(
    method: &str,
    path: &str,
    server: &LocalServer,
    ui_root: &Path,
) -> ApiResponse {
    if path.starts_with("/api/") {
        return match method {
            "GET" => server.api_get(path),
            "POST" if path == "/api/runs" => ApiResponse {
                status_code: 501,
                content_type: "application/json; charset=utf-8".to_string(),
                body: r#"{"error":"runs_endpoint_not_connected","message":"ユースケースボタンのDBサンプル読み込みを先に実装しています。"}"#.to_string(),
            },
            _ => ApiResponse {
                status_code: 405,
                content_type: "application/json; charset=utf-8".to_string(),
                body: r#"{"error":"method_not_allowed"}"#.to_string(),
            },
        };
    }

    if method != "GET" {
        return ApiResponse {
            status_code: 405,
            content_type: "text/plain; charset=utf-8".to_string(),
            body: "method not allowed".to_string(),
        };
    }

    serve_static(path, ui_root)
}

fn serve_static(path: &str, ui_root: &Path) -> ApiResponse {
    let (relative_path, content_type) = match path {
        "/" | "/index.html" => ("index.html", "text/html; charset=utf-8"),
        "/src/app.js" => ("src/app.js", "text/javascript; charset=utf-8"),
        "/src/styles.css" => ("src/styles.css", "text/css; charset=utf-8"),
        _ => {
            return ApiResponse {
                status_code: 404,
                content_type: "text/plain; charset=utf-8".to_string(),
                body: "not found".to_string(),
            };
        }
    };

    match fs::read_to_string(ui_root.join(relative_path)) {
        Ok(body) => ApiResponse {
            status_code: 200,
            content_type: content_type.to_string(),
            body,
        },
        Err(error) => ApiResponse {
            status_code: 500,
            content_type: "text/plain; charset=utf-8".to_string(),
            body: format!("failed to read static asset: {error}"),
        },
    }
}

fn write_response(stream: &mut TcpStream, response: ApiResponse) -> io::Result<()> {
    let status_text = match response.status_code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        501 => "Not Implemented",
        _ => "Internal Server Error",
    };
    let body = response.body.as_bytes();
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n",
        response.status_code,
        status_text,
        response.content_type,
        body.len()
    )?;
    stream.write_all(body)
}
