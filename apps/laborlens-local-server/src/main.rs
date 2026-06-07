use laborlens_local_server::{ApiResponse, GuideMessageRequest, LocalServer};
use std::collections::HashMap;
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
    let request = read_request(&mut stream)?;
    if request.raw.is_empty() {
        return Ok(());
    }

    let mut request_parts = request.first_line.split_whitespace();
    let method = request_parts.next().unwrap_or("");
    let path = request_parts.next().unwrap_or("/");

    let response = response_for_request(method, path, &request, server, ui_root);
    write_response(&mut stream, response)
}

struct HttpRequest {
    raw: Vec<u8>,
    first_line: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    let mut raw = Vec::new();
    let mut buffer = [0_u8; 16 * 1024];

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..bytes_read]);
        if header_end(&raw).is_some() {
            break;
        }
    }

    let Some(header_end) = header_end(&raw) else {
        return Ok(HttpRequest {
            raw,
            first_line: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        });
    };
    let header_text = String::from_utf8_lossy(&raw[..header_end]);
    let mut lines = header_text.lines();
    let first_line = lines.next().unwrap_or("").to_string();
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect::<HashMap<_, _>>();
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let body_start = header_end + 4;

    while raw.len().saturating_sub(body_start) < content_length {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..bytes_read]);
    }

    let body_end = body_start.saturating_add(content_length).min(raw.len());
    let body = raw[body_start..body_end].to_vec();

    Ok(HttpRequest {
        raw,
        first_line,
        headers,
        body,
    })
}

fn header_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|window| window == b"\r\n\r\n")
}

fn response_for_request(
    method: &str,
    path: &str,
    request: &HttpRequest,
    server: &LocalServer,
    ui_root: &Path,
) -> ApiResponse {
    if path.starts_with("/api/") {
        return match method {
            "GET" => server.api_get(path),
            "POST" if path == "/api/runs" => api_post_run(request, server),
            "POST" if path == "/api/guide/messages" => api_post_guide_message(request, server),
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

fn api_post_guide_message(request: &HttpRequest, server: &LocalServer) -> ApiResponse {
    match serde_json::from_slice::<GuideMessageRequest>(&request.body) {
        Ok(payload) => server.guide_message_response(payload),
        Err(_) => json_error(400, "invalid_guide_message_request"),
    }
}

fn api_post_run(request: &HttpRequest, server: &LocalServer) -> ApiResponse {
    let Some(content_type) = request.headers.get("content-type") else {
        return json_error(400, "missing_content_type");
    };
    let Some(boundary) = content_type
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("boundary="))
    else {
        return json_error(400, "missing_multipart_boundary");
    };
    let fields = parse_multipart_form_data(&request.body, boundary);
    let Some(employees_csv) = fields.get("employees_csv") else {
        return json_error(400, "missing_employees_csv");
    };
    let Some(attendance_csv) = fields.get("attendance_csv") else {
        return json_error(400, "missing_attendance_csv");
    };

    let response = server.start_run_from_csv_contents(employees_csv, attendance_csv);
    ApiResponse {
        status_code: 200,
        content_type: "application/json; charset=utf-8".to_string(),
        body: serde_json::to_string(&response).expect("run response should serialize"),
    }
}

fn parse_multipart_form_data(body: &[u8], boundary: &str) -> HashMap<String, String> {
    let marker = format!("--{boundary}");
    let text = String::from_utf8_lossy(body);
    let mut fields = HashMap::new();

    for part in text.split(&marker).skip(1) {
        if part.starts_with("--") {
            break;
        }
        let part = part.trim_start_matches("\r\n");
        let Some((headers, value)) = part.split_once("\r\n\r\n") else {
            continue;
        };
        let Some(name) = headers.lines().find_map(form_field_name) else {
            continue;
        };
        fields.insert(name, value.trim_end_matches("\r\n").to_string());
    }

    fields
}

fn form_field_name(header_line: &str) -> Option<String> {
    let (_, value) = header_line.split_once(':')?;
    if !header_line
        .to_ascii_lowercase()
        .starts_with("content-disposition:")
    {
        return None;
    }
    value.split(';').map(str::trim).find_map(|part| {
        part.strip_prefix("name=\"")
            .and_then(|name| name.strip_suffix('"'))
            .map(str::to_string)
    })
}

fn json_error(status_code: u16, error: &str) -> ApiResponse {
    ApiResponse {
        status_code,
        content_type: "application/json; charset=utf-8".to_string(),
        body: format!(r#"{{"error":"{error}"}}"#),
    }
}

fn serve_static(path: &str, ui_root: &Path) -> ApiResponse {
    let dist_root = ui_root.join("dist");
    if dist_root.exists() {
        let relative_path = match path {
            "/" | "/index.html" => "index.html",
            candidate => candidate.trim_start_matches('/'),
        };
        if relative_path.contains("..") {
            return ApiResponse {
                status_code: 404,
                content_type: "text/plain; charset=utf-8".to_string(),
                body: "not found".to_string(),
            };
        }
        return read_static_file(
            &dist_root.join(relative_path),
            static_content_type(relative_path),
        );
    }

    let (relative_path, content_type) = match path {
        "/" | "/index.html" => ("index.html", "text/html; charset=utf-8"),
        _ => {
            return ApiResponse {
                status_code: 404,
                content_type: "text/plain; charset=utf-8".to_string(),
                body: "not found".to_string(),
            };
        }
    };

    read_static_file(&ui_root.join(relative_path), content_type)
}

fn read_static_file(path: &Path, content_type: &str) -> ApiResponse {
    match fs::read_to_string(path) {
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

fn static_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        "text/plain; charset=utf-8"
    }
}

fn write_response(stream: &mut TcpStream, response: ApiResponse) -> io::Result<()> {
    let status_text = match response.status_code {
        200 => "OK",
        400 => "Bad Request",
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
