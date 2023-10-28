use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

use nom::AsBytes;

const OK_RESPONSE: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let request_buffer = BufReader::new(&stream);

                let http_request_lines: Vec<_> = request_buffer
                    .lines()
                    .map(|line| line.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect(); // we are not collecting body yet

                let http_request = parse_request(&http_request_lines).unwrap();

                if http_request.path == "/" {
                    stream.write(OK_RESPONSE).unwrap();
                } else if http_request.path.starts_with("/user-agent")
                    && http_request.headers.contains_key("User-Agent")
                {
                    let user_agent = http_request.headers.get("User-Agent").unwrap().as_str();
                    stream
                        .write_all(ok_response(user_agent).as_bytes())
                        .unwrap();
                } else if http_request.path.starts_with("/echo") {
                    stream
                        .write_all(ok_response(&&http_request.path[6..]).as_bytes())
                        .unwrap();
                } else {
                    stream.write_all(NOT_FOUND_RESPONSE).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

struct HttpRequest {
    path: String,
    headers: HashMap<String, String>,
}

fn parse_request(http_request_lines: &Vec<String>) -> Result<HttpRequest, &str> {
    let path = match parse_path(http_request_lines.get(0)) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let mut headers: HashMap<String, String> = HashMap::new();

    for line in http_request_lines.iter().skip(1) {
        let (name, value) = match line.split_once(":") {
            Some((name, value)) => (name, value),
            None => return Err("Wrongly formatted http request. Header is missing ':'"),
        };

        headers.insert(name.to_string(), value.to_string());
    }

    return Ok(HttpRequest { path, headers });
}

fn parse_path(first_line: Option<&String>) -> Result<String, Result<HttpRequest, &str>> {
    let path = match first_line {
        Some(first_line) => {
            let first_line_splitted = first_line.split_ascii_whitespace().collect::<Vec<_>>();

            let maybe_path = first_line_splitted.get(1);

            match maybe_path.map(|s| s.to_string()) {
                Some(path) => path,
                None => {
                    return Err(Err(
                        "Wrongly formatted http request. Failed to convert path to string.",
                    ))
                }
            }
        }
        None => return Err(Err("Wrongly formatted http request. Missing first line.")),
    };

    Ok(path)
}

fn ok_response(body: &str) -> Vec<u8> {
    let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n".to_vec();
    let content_length = body.len();

    let content_length_header = format!("Content-Length: {content_length}\r\n\r\n");
    response.extend_from_slice(content_length_header.as_bytes());

    response.extend_from_slice(body.as_bytes());

    response
}
