use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

use http_server_starter_rust::ThreadPool;
use nom::AsBytes;

const OK_RESPONSE: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const OK_RESPONSE_201: &[u8; 19] = b"HTTP/1.1 201 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let default_arg = String::from(".");
    let second_arg = args.get(2).unwrap_or(&default_arg);

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let base_dir = second_arg.clone();
        let stream = stream.unwrap();
        thread_pool
            .run(Box::new(move || {
                handle_connection(stream, base_dir.as_str())
            }))
            .unwrap();
        println!("dispatched request")
    }
}

fn handle_connection(mut stream: std::net::TcpStream, base_dir: &str) {
    println!("accepted new connection");

    let mut request_buffer = BufReader::new(&stream);

    let http_request_lines: Vec<_> = request_buffer
        .by_ref()
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let http_request = parse_request(&http_request_lines).unwrap();

    if http_request.path == "/" {
        stream.write(OK_RESPONSE).unwrap();
    } else if http_request.path.starts_with("/user-agent")
        && http_request.headers.contains_key("User-Agent")
    {
        let user_agent = http_request.headers.get("User-Agent").unwrap().as_str();
        stream
            .write_all(ok_text_response(user_agent).as_bytes())
            .unwrap();
    } else if http_request.path.starts_with("/echo") {
        stream
            .write_all(ok_text_response(&&http_request.path[6..]).as_bytes())
            .unwrap();
    } else if http_request.path.starts_with("/files/") && http_request.method == "GET" {
        let file = http_request.path.split("/").last().unwrap_or("");
        let mut path = PathBuf::new();
        path.push(base_dir);
        path.push(file);
        serve_file(stream, &path)
    } else if http_request.path.starts_with("/files/") && http_request.method == "POST" {
        let content_length: usize = http_request
            .headers
            .get("Content-Length")
            .unwrap_or(&String::from("10"))
            .parse()
            .unwrap();

        let mut body = vec![0; content_length];
        request_buffer.read_exact(&mut body).unwrap();

        let file = http_request.path.split("/").last().unwrap_or("");
        let mut path = PathBuf::new();
        path.push(base_dir);
        path.push(file);

        fs::write(path, body).unwrap();

        stream.write_all(OK_RESPONSE_201).unwrap();
    } else {
        stream.write_all(NOT_FOUND_RESPONSE).unwrap();
    }
}

#[derive(Debug)]
struct HttpRequest {
    path: String,
    headers: HashMap<String, String>,
    method: String,
}

fn parse_request(http_request_lines: &Vec<String>) -> Result<HttpRequest, &str> {
    let first_line = http_request_lines
        .get(0)
        .expect("Missing first line in http_request");
    let (method, path, _) = parse_first_line(first_line).unwrap();

    let mut headers: HashMap<String, String> = HashMap::new();

    for line in http_request_lines.iter().skip(1) {
        let (name, value) = match line.split_once(":") {
            Some((name, value)) => (name, value.trim()),
            None => return Err("Wrongly formatted http request. Header is missing ':'"),
        };

        headers.insert(name.to_string(), value.to_string());
    }

    return Ok(HttpRequest {
        path,
        headers,
        method,
    });
}

fn parse_first_line(first_line: &String) -> Result<(String, String, String), &str> {
    let first_line_splitted = first_line.split_ascii_whitespace().collect::<Vec<_>>();

    match (
        first_line_splitted.get(0),
        first_line_splitted.get(1),
        first_line_splitted.get(2),
    ) {
        (Some(method), Some(path), Some(protocol)) => {
            Ok((method.to_string(), path.to_string(), protocol.to_string()))
        }
        _ => Err("Wrongly formatted first line of http request"),
    }
}

fn ok_text_response(body: &str) -> Vec<u8> {
    let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n".to_vec();
    let content_length = body.len();

    let content_length_header = format!("Content-Length: {content_length}\r\n\r\n");
    response.extend_from_slice(content_length_header.as_bytes());

    response.extend_from_slice(body.as_bytes());

    response
}

fn serve_file(mut stream: TcpStream, file_path: &Path) {
    match fs::read(file_path) {
        Ok(mut file_content) => {
            let len = file_content.len();
            let mut response = Vec::with_capacity(len + 1024);
            response.extend(b"HTTP/1.1 200 OK\r\n");
            response.extend(format!("Content-Type: application/octet-stream\r\n").as_bytes());
            response.extend(format!("Content-Length: {len}\r\n\r\n").as_bytes());
            response.append(&mut file_content);
            stream.write_all(&response).unwrap();
        }
        Err(_) => {
            stream.write_all(NOT_FOUND_RESPONSE).unwrap();
        }
    }
}
